use std::thread;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use console::Term;
pub use console::Key;

type QueueMutex = Arc<Mutex<VecDeque<Key>>>;

fn spawn_producer(m: QueueMutex) {
  let t = Term::stdout();
  thread::spawn(move || loop {
    let c = t.read_key().unwrap();
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

  pub fn poll_key(&self) -> Option<Key> {
    match self.m.try_lock() {
      Ok(mut q) => q.pop_front(),
      Err(_)    => None,
    }
  }
}
