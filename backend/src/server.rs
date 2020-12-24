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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Request {
  UpdateCell {
    user_id: i32,
    sheet_id: i32,
    row: i32,
    col: i32,
    raw: String,
  },
}

#[derive(Clone, Debug, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
#[serde(tag = "type")]
pub enum Response {
  Connected { user_id: i32, cells: Vec<Cell> },
  Participants { ids: HashSet<i32> },
  CellLocked { user_id: i32, cell: Cell },
  CellUpdated { user_id: i32, cell: Cell },
  Error { message: String },
}

#[derive(Message)]
#[rtype(i32)]
pub struct Connect {
  pub sheet_id: i32,
  pub addr: Recipient<Response>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
  pub user_id: i32,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Text {
  pub user_id: i32,
  pub data: String,
}

// Create an individual message for cell update...

pub struct WsServer {
  db: PgConnection,
  // User ID -> WebSocket Actor
  user_to_addr: HashMap<i32, Recipient<Response>>,
  // Spreadsheet ID -> User IDs
  sheet_to_users: HashMap<i32, HashSet<i32>>,
  // User ID -> Spreadsheet ID
  user_to_sheet: HashMap<i32, i32>,
  rng: ThreadRng,
}

impl WsServer {
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

  fn handle_req(&self, req: Request) {
    match req {
      Request::UpdateCell {
        user_id,
        sheet_id,
        row,
        col,
        raw,
      } => {
        self.update_cell(user_id, sheet_id, row, col, raw);
      }
    };
  }

  fn update_cell(&self, user_id: i32, sheet_id: i32, row: i32, col: i32, raw: String) {
    let new_cell = NewCell {
      sheet_id,
      row,
      col,
      raw,
    };
    match diesel::insert_into(cells::table)
      .values(&new_cell)
      .on_conflict((cells::sheet_id, cells::row, cells::col))
      .do_update()
      .set(&new_cell)
      .get_result(&self.db)
    {
      Ok(cell) => {
        let resp = Response::CellUpdated { user_id, cell };
        self.broadcast(sheet_id, resp);
      }
      Err(_) => {
        let resp = Response::Error {
          message: format!("failed to update cell {:?}", new_cell),
        };
        self.send(user_id, resp);
      }
    };
  }

  fn broadcast_participants(&self, sheet_id: i32) {
    if let Some(user_ids) = self.sheet_to_users.get(&sheet_id) {
      let resp = Response::Participants {
        ids: user_ids.clone(),
      };
      self.broadcast(sheet_id, resp);
    }
  }

  fn send(&self, user_id: i32, response: Response) {
    println!("sending response {:?} to user {}", response, user_id);
    match self.user_to_addr.get(&user_id) {
      Some(addr) => {
        let _ = addr.do_send(response);
      }
      None => println!("no address found for user {}", user_id),
    };
  }

  fn broadcast(&self, sheet_id: i32, response: Response) {
    println!("broadcasting response {:?} to sheet {}", response, sheet_id);
    let user_ids = match self.sheet_to_users.get(&sheet_id) {
      Some(ids) => ids,
      None => {
        println!("no users found in sheet {}", sheet_id);
        return;
      }
    };
    for id in user_ids {
      let addr = match self.user_to_addr.get(id) {
        Some(addr) => addr,
        None => {
          println!("no address found for user {}", id);
          continue;
        }
      };
      // Cloning on every iteration seems expensive
      let _ = addr.do_send(response.clone());
    }
  }
}

impl Actor for WsServer {
  type Context = Context<Self>;
}

impl Handler<Connect> for WsServer {
  type Result = i32;

  fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
    println!("Someone connected to sheet {}", msg.sheet_id);

    // register session with random id
    let new_user_id = self.rng.gen::<i32>();
    self.user_to_addr.insert(new_user_id, msg.addr);
    self.user_to_sheet.insert(new_user_id, msg.sheet_id);

    // add user to the list of subscribers of the sheet
    self
      .sheet_to_users
      .entry(msg.sheet_id)
      .or_insert_with(HashSet::new)
      .insert(new_user_id);

    // Get cells from spreadsheet in DB
    match cells::table
      .filter(cells::sheet_id.eq(msg.sheet_id as i32))
      .load::<Cell>(&self.db)
    {
      Ok(cells) => {
        self.send(
          new_user_id,
          Response::Connected {
            user_id: new_user_id,
            cells,
          },
        );
      }
      Err(_) => {
        // TODO: we will advertise this user joined regardless of whether he actually loaded the cells
        // which seems off.
        self.send(
          new_user_id,
          Response::Error {
            message: "Error loading cells".to_string(),
          },
        )
      }
    };

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
    let req: Request = match serde_json::from_str(&msg.data) {
      Ok(req) => req,
      Err(_) => {
        let resp = Response::Error {
          message: format!("unable to parse request {:?}", msg.data),
        };
        self.send(msg.user_id, resp);
        return;
      }
    };
    self.handle_req(req);
  }
}
