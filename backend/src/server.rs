//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.

use super::models::*;
use super::schema::cells;
use actix::prelude::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// TODO: We should probably have a separate struct/enum for user requests and another
// one for responses
#[derive(Clone, Debug, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
#[serde(tag = "type")]
pub enum Event {
  Connected {
    user_id: usize,
    cells: Vec<Cell>,
  },
  Participants {
    ids: HashSet<usize>,
  },
  CellLocked {
    cell_idx: usize,
    user_id: usize,
  },
  CellUpdated {
    sheet_id: i32,
    row: i32,
    col: i32,
    raw: String,
  },
}

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
  pub sheet_id: usize,
  pub addr: Recipient<Event>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
  pub user_id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Text {
  pub user_id: usize,
  pub data: String,
}

// Create an individual message for cell update...

pub struct WsServer {
  db: PgConnection,
  // User ID -> WS connections
  user_to_addr: HashMap<usize, Recipient<Event>>,
  // Spreadsheet ID -> User IDs
  sheet_to_users: HashMap<usize, HashSet<usize>>,
  // User ID -> Spreadsheet ID
  user_to_sheet: HashMap<usize, usize>,
  rng: ThreadRng,
}

impl WsServer {
  // TODO(adelavega): proper error handling on all methods -- unwrapping is not sane
  pub fn new(db_url: &str) -> WsServer {
    WsServer {
      db: PgConnection::establish(db_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", db_url)),
      user_to_addr: HashMap::new(),
      sheet_to_users: HashMap::new(),
      user_to_sheet: HashMap::new(),
      rng: rand::thread_rng(),
    }
  }
  fn broadcast_participants(&self, sheet_id: usize) {
    if let Some(user_ids) = self.sheet_to_users.get(&sheet_id) {
      let event = Event::Participants {
        ids: user_ids.clone(),
      };
      self.broadcast(sheet_id, event);
    }
  }

  fn send(&self, user_id: usize, event: Event) {
    let addr = self.user_to_addr.get(&user_id).unwrap();
    let _ = addr.do_send(event);
  }

  fn broadcast(&self, sheet_id: usize, event: Event) {
    println!("broadcasting event {:?} to sheet {}", event, sheet_id);
    let user_ids = self.sheet_to_users.get(&sheet_id).unwrap();
    for id in user_ids {
      let addr = self.user_to_addr.get(id).unwrap();
      // Cloning on every iteration seems expensive
      let _ = addr.do_send(event.clone());
    }
  }
}

impl Actor for WsServer {
  type Context = Context<Self>;
}

impl Handler<Connect> for WsServer {
  type Result = usize;

  fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
    println!("Someone connected to sheet {}", msg.sheet_id);

    // register session with random id
    let new_user_id = self.rng.gen::<usize>();
    self.user_to_addr.insert(new_user_id, msg.addr);
    self.user_to_sheet.insert(new_user_id, msg.sheet_id);

    // add user to the list of subscribers of the sheet
    self
      .sheet_to_users
      .entry(msg.sheet_id)
      .or_insert_with(HashSet::new)
      .insert(new_user_id);

    // Get cells from spreadsheet in DB
    let results = cells::table
      .filter(cells::sheet_id.eq(msg.sheet_id as i32))
      .load::<Cell>(&self.db)
      .expect("Error loading posts");
    self.send(
      new_user_id,
      Event::Connected {
        user_id: new_user_id,
        cells: results,
      },
    );

    // Announce to other users that are connected to this spreadsheet someone else joined
    self.broadcast_participants(msg.sheet_id);

    new_user_id
  }
}

impl Handler<Disconnect> for WsServer {
  type Result = ();

  fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
    println!("{} disconnected", msg.user_id);

    self.user_to_addr.remove(&msg.user_id);

    let sheet_id = match self.user_to_sheet.remove(&msg.user_id) {
      Some(sheet_id) => sheet_id,
      None => {
        println!("{} has no sheet attached", msg.user_id);
        return;
      }
    };

    let sheet_users = match self.sheet_to_users.get_mut(&sheet_id) {
      Some(sheet_users) => sheet_users,
      None => {
        println!("{} has no users", sheet_id);
        return;
      }
    };
    sheet_users.remove(&msg.user_id);
    if sheet_users.is_empty() {
      // Prevent memory leak, remove entry once all sessions are closed
      self.sheet_to_users.remove(&sheet_id);
    }

    self.broadcast_participants(sheet_id);
  }
}

impl Handler<Text> for WsServer {
  type Result = ();

  fn handle(&mut self, msg: Text, _: &mut Context<Self>) {
    let sheet_id = self.user_to_sheet.get(&msg.user_id).unwrap();
    let event: Event = serde_json::from_str(&msg.data).unwrap();
    if let Event::CellUpdated {
      sheet_id,
      row,
      col,
      raw,
    } = &event
    {
      let new_cell = NewCell {
        sheet_id: *sheet_id as i32,
        row: *row as i32,
        col: *col as i32,
        raw: raw.clone(),
      };
      let cell: Cell = diesel::insert_into(cells::table)
        .values(&new_cell)
        .get_result(&self.db)
        .expect("Error saving cell");
      println!("inserted {:?}", cell);
    }
    self.broadcast(*sheet_id, event);
  }
}
