use std::thread;
use std::sync::{Arc, Condvar, Mutex};
use std::collections::VecDeque;
use console::Term;

type QueueMutex = Arc<Mutex<VecDeque<char>>>;

fn spawn_listener(m: QueueMutex, c: Arc<Condvar>) {
  let t = Term::stdout();
  thread::spawn(move || loop {
    let e = t.read_char().unwrap();
    m.lock().unwrap().push_back(e);
    c.notify_all();
  });
}

pub struct KeyEventQueue {
  m: QueueMutex,
  c: Arc<Condvar>,
}

impl KeyEventQueue {
  pub fn spawn() -> Self {
    let m = QueueMutex::default();
    let c = Arc::new(Condvar::new());
    spawn_listener(m.clone(), c.clone());
    Self { m, c }
  }

  pub fn is_empty(&self) -> bool {
    self.m.lock().unwrap().is_empty()
  }

  pub fn pop_blocking(&self) -> char {
    let mut q = self.m.lock().unwrap();
    loop {
      if let Some(e) = q.pop_front() { return e; }
      q = self.c.wait(q).unwrap();
    }
  }
}
