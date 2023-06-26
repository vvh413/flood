mod cli;
mod flooder;

use std::io::Write;
use std::time::Duration;

use anyhow::Result;

use clap::Parser;

use flooder::Flooder;

const SYMBOLS: [char; 4] = ['/', '-', '\\', '|'];

fn main() -> Result<()> {
  let cli = cli::Cli::parse();

  let flooder = Flooder::init(cli.global, cli.command)?;
  let handles = flooder.start();

  let mut symbols = SYMBOLS.iter().cycle();
  while !handles.iter().all(|handle| handle.is_finished()) {
    print!("\rFlooding... {}", symbols.next().unwrap());
    std::io::stdout().flush().unwrap();
    std::thread::sleep(Duration::from_millis(100));
  }

  Ok(())
}
