use std::thread;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use console::Term;

type QueueMutex = Arc<Mutex<VecDeque<char>>>;

fn spawn_producer(m: QueueMutex) {
  let t = Term::stdout();
  thread::spawn(move || loop {
    let c = t.read_char().unwrap();
    m.lock().unwrap().push_back(c);
  });
}

pub struct KeyEventQueue { m: QueueMutex }

impl KeyEventQueue {
  pub fn spawn() -> Self {
    let m = QueueMutex::default();
    spawn_producer(m.clone());
    Self { m }
  }

  pub fn is_empty(&self) -> bool {
    self.m.lock().unwrap().is_empty()
  }

  pub fn pop_blocking(&self) -> char {
    loop {
      let maybe = self.m.lock().unwrap().pop_front();
      if let Some(c) = maybe { return c; }
    }
  }
}
