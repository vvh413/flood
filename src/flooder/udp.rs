use std::net::Ipv4Addr;

use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::udp::MutableUdpPacket;
use pnet::packet::MutablePacket;

use pnet::util;
use rand::Rng;

use crate::cli::UdpArgs;
use crate::flooder::IPV4_HEADER_LEN;

use super::{create_ipv4_packet, Packer};

const UDP_HEADER_LEN: usize = 8;

#[derive(Clone)]
pub struct UdpPacker {
  buffer_ip: Vec<u8>,
  buffer: Vec<u8>,
  args: UdpArgs,
}

impl UdpPacker {
  pub fn init(args: UdpArgs) -> Self {
    let buffer = vec![0u8; args.common.size + UDP_HEADER_LEN];
    let buffer_ip = vec![0u8; buffer.len() + IPV4_HEADER_LEN];

    let mut args = args;
    args.src_port = Some(args.src_port.unwrap_or(rand::thread_rng().gen_range(49152..=65535)));

    Self {
      buffer_ip,
      buffer,
      args,
    }
  }
}

impl Packer for UdpPacker {
  fn create_packet(&mut self) -> MutableIpv4Packet {
    let udp_size = self.buffer.len();
    let mut ipv4_packet = create_ipv4_packet(
      &mut self.buffer_ip,
      self.args.common.host,
      IpNextHeaderProtocols::Udp,
      udp_size,
      self.args.common.random_source,
    );

    let mut udp_packet = MutableUdpPacket::new(&mut self.buffer).expect("Error creating udp packet");
    udp_packet.set_source(self.args.src_port.unwrap());
    udp_packet.set_destination(self.args.port.unwrap_or(rand::thread_rng().gen_range(1..49152)));
    udp_packet.set_length(udp_size as u16);
    let checksum = util::checksum(udp_packet.packet_mut(), 1);
    udp_packet.set_checksum(checksum);

    ipv4_packet.set_payload(udp_packet.packet_mut());
    ipv4_packet
  }

  fn get_addr(&self) -> Ipv4Addr {
    self.args.common.host
  }
}
