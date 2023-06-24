mod cli;
mod flood;
mod icmp;
mod ip;
mod udp;

use std::io::Write;

use std::time::Duration;

use anyhow::Result;

use clap::Parser;
use cli::Command;
use flood::Flood;

const SYMBOLS: [char; 4] = ['/', '-', '\\', '|'];

fn init(command: Command, delay: u64) -> Result<Box<dyn Flood>> {
  match command {
    Command::Icmp(args) => Ok(Box::new(icmp::IcmpFlood::init(args, delay)?)),
    Command::Udp(args) => Ok(Box::new(udp::UdpFlood::init(args, delay)?)),
  }
}

fn main() -> Result<()> {
  let cli = cli::Cli::parse();

  let handles = init(cli.command, cli.delay)?.start(cli.threads);

  let mut symbols = SYMBOLS.iter().cycle();
  while !handles.iter().all(|handle| handle.is_finished()) {
    print!("\rFlooding... {}", symbols.next().unwrap());
    std::io::stdout().flush().unwrap();
    std::thread::sleep(Duration::from_millis(100));
  }

  Ok(())
}
