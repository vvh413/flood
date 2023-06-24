use std::thread::JoinHandle;

pub trait Flood {
  fn start(&self, threads: usize) -> Vec<JoinHandle<()>>;
}
