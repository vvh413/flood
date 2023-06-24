mod cli;
mod flooder;

use std::io::Write;
use std::time::Duration;

use anyhow::Result;

use clap::Parser;
use cli::Command;
use flooder::icmp::IcmpFlooder;
use flooder::syn::SynFlooder;
use flooder::udp::UdpFlooder;
use flooder::Flooder;

const SYMBOLS: [char; 4] = ['/', '-', '\\', '|'];

fn main() -> Result<()> {
  let cli = cli::Cli::parse();

  let handles = match cli.command {
    Command::Icmp(args) => IcmpFlooder::init(args, cli.delay)?.start(cli.threads),
    Command::Udp(args) => UdpFlooder::init(args, cli.delay)?.start(cli.threads),
    Command::Syn(args) => SynFlooder::init(args, cli.delay)?.start(cli.threads),
  };
  let mut symbols = SYMBOLS.iter().cycle();
  while !handles.iter().all(|handle| handle.is_finished()) {
    print!("\rFlooding... {}", symbols.next().unwrap());
    std::io::stdout().flush().unwrap();
    std::thread::sleep(Duration::from_millis(100));
  }

  Ok(())
}
