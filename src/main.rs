mod key_event_queue;
use key_event_queue::KeyEventQueue;

fn main() {
  let event_queue = KeyEventQueue::spawn();
  loop {
    match event_queue.poll_key() {
      Some(k) => println!("{:?}", k),
      None    => {}
    }
  }
}
