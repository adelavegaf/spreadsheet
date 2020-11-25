//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use std::collections::{HashMap, HashSet};

#[derive(Message)]
#[rtype(result = "()")]
pub enum Event {}

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

impl Actor for WsServer {
  type Context = Context<Self>;
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

impl Handler<Connect> for WsServer {
  type Result = usize;

  fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
    println!("Someone connected to spreadsheet {}", msg.spreadsheet_id);

    // register session with random id
    let session_id = self.rng.gen::<usize>();
    self.sessions.insert(session_id, msg.addr);

    // add session to the list of subscribers to the specified spreadsheet
    self
      .spreadsheets
      .entry(msg.spreadsheet_id)
      .or_insert_with(HashSet::new)
      .insert(session_id);

    session_id
  }
}

impl Handler<Disconnect> for WsServer {
  type Result = ();

  fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
    println!("{} disconnected", msg.session_id);
    // Remove session id from sessions map
    self.sessions.remove(&msg.session_id);
    // Remove session id from all spreadsheets
    for (_, session_ids) in &mut self.spreadsheets {
      session_ids.remove(&msg.session_id);
    }
    // TODO(adelavega): advertise to other users in the same spreadsheet session
    // someone logged out.
  }
}
