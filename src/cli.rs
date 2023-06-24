use std::net::Ipv4Addr;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Command,

  /// Number of threads
  #[arg(short, long, default_value_t = 3)]
  pub threads: usize,

  /// Delay between packets in microseconds
  #[arg(short, long, default_value_t = 0)]
  pub delay: u64,
}

#[derive(Subcommand)]
pub enum Command {
  /// ICMP (ping) flood
  Icmp(IcmpArgs),
  /// UDP flood
  Udp(UdpArgs),
  // /// SYN flood
  // SYN,
}

#[derive(Args)]
pub struct IcmpArgs {
  /// Destination address
  pub host: Ipv4Addr,

  /// Packet size in bytes
  #[arg(short, long, default_value_t = 1471)]
  pub size: usize,
}

#[derive(Args)]
pub struct UdpArgs {
  /// Destination address
  pub host: Ipv4Addr,

  /// Packet size in bytes
  #[arg(short, long, default_value_t = 0)]
  pub size: usize,

  /// Port to flood (optional)
  #[arg(short, long)]
  pub port: Option<u16>,
}
