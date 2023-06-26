use std::net::Ipv4Addr;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Command,

  #[command(flatten)]
  pub global: GlobalArgs,
}

#[derive(Args, Clone)]
pub struct GlobalArgs {
  /// Number of threads
  #[arg(short, long, default_value_t = 3)]
  pub threads: usize,

  /// Delay between packets in microseconds
  #[arg(short, long, default_value_t = 0)]
  pub delay: u64,
}

#[derive(Subcommand, Clone)]
pub enum Command {
  /// ICMP (ping) flood
  Icmp(IcmpArgs),
  /// UDP flood
  Udp(UdpArgs),
  /// SYN flood
  Syn(SynArgs),
}

#[derive(Args, Clone, Debug)]
pub struct CommonArgs {
  /// Destination address
  pub host: Ipv4Addr,

  /// Packet size in bytes
  #[arg(short, long, default_value_t = 1471)]
  pub size: usize,

  /// Random source addr
  #[arg(short, long)]
  pub random_source: bool,
}

#[derive(Args, Clone, Debug)]
pub struct IcmpArgs {
  #[command(flatten)]
  pub common: CommonArgs,
}

#[derive(Args, Clone, Debug)]
pub struct UdpArgs {
  #[command(flatten)]
  pub common: CommonArgs,

  /// Destination port to flood (optional)
  #[arg(short, long)]
  pub port: Option<u16>,

  /// Source port (optional)
  #[arg(long)]
  pub src_port: Option<u16>,
}

#[derive(Args, Clone, Debug)]
pub struct SynArgs {
  #[command(flatten)]
  pub common: CommonArgs,

  /// Destination port to flood (optional)
  #[arg(short, long)]
  pub port: Option<u16>,

  /// Source port (optional)
  #[arg(long)]
  pub src_port: Option<u16>,
}
