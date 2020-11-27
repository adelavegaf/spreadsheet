//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
#[serde(tag = "type")]
pub enum Event {
  Participants { ids: HashSet<usize> },
  CellLock { id: usize, row: usize, col: usize },
}

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
  pub spreadsheet_id: usize,
  pub addr: Recipient<Event>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
  pub session_id: usize,
}

pub struct WsServer {
  // Session ID -> WS connections
  sessions: HashMap<usize, Recipient<Event>>,
  // Spreadsheet ID -> Session IDs
  spreadsheets: HashMap<usize, HashSet<usize>>,
  rng: ThreadRng,
}

impl Default for WsServer {
  fn default() -> WsServer {
    WsServer {
      sessions: HashMap::new(),
      spreadsheets: HashMap::new(),
      rng: rand::thread_rng(),
    }
  }
}

impl WsServer {
  fn broadcast_participants(&self, spreadsheet: usize) {
    // TODO(adelavega): add proper error handling -- no unwraps!
    let session_ids = self.spreadsheets.get(&spreadsheet).unwrap();
    for id in session_ids {
      let addr = self.sessions.get(id).unwrap();
      let msg = Event::Participants {
        // TODO(adelavega): cloning on every iteration seems expensive.
        ids: session_ids.clone(),
      };
      let _ = addr.do_send(msg);
    }
  }
}

impl Actor for WsServer {
  type Context = Context<Self>;
}

impl Handler<Connect> for WsServer {
  type Result = usize;

  fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
    println!("Someone connected to spreadsheet {}", msg.spreadsheet_id);

    // register session with random id
    let new_session_id = self.rng.gen::<usize>();
    self.sessions.insert(new_session_id, msg.addr);

    // add session to the list of subscribers to the specified spreadsheet
    self
      .spreadsheets
      .entry(msg.spreadsheet_id)
      .or_insert_with(HashSet::new)
      .insert(new_session_id);

    self.broadcast_participants(msg.spreadsheet_id);

    new_session_id
  }
}

impl Handler<Disconnect> for WsServer {
  type Result = ();

  fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
    println!("{} disconnected", msg.session_id);

    // Remove session id from sessions map
    self.sessions.remove(&msg.session_id);

    // Remove session id from all spreadsheets
    let mut updated_spreadsheets = vec![];
    for (spreadsheet, session_ids) in &mut self.spreadsheets {
      if session_ids.remove(&msg.session_id) {
        updated_spreadsheets.push(*spreadsheet);
      }
    }

    // Advertise changes to all clients
    for spreadsheet in updated_spreadsheets {
      self.broadcast_participants(spreadsheet);
    }
  }
}
