use std::net::Ipv4Addr;

use pnet::packet::icmp::echo_request::MutableEchoRequestPacket;
use pnet::packet::icmp::IcmpTypes;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::MutablePacket;

use pnet::util;

use crate::cli::IcmpArgs;
use crate::flooder::IPV4_HEADER_LEN;

use super::{create_ipv4_packet, Packer};

const ICMP_HEADER_LEN: usize = 8;

#[derive(Clone)]
pub struct IcmpPacker {
  buffer_ip: Vec<u8>,
  buffer: Vec<u8>,
  args: IcmpArgs,
}

impl IcmpPacker {
  pub fn init(args: IcmpArgs) -> Self {
    let buffer = vec![0u8; args.common.size + ICMP_HEADER_LEN];
    let buffer_ip = vec![0u8; buffer.len() + IPV4_HEADER_LEN];

    Self {
      buffer_ip,
      buffer,
      args,
    }
  }
}

impl Packer for IcmpPacker {
  fn create_packet(&mut self) -> MutableIpv4Packet {
    let mut ipv4_packet = create_ipv4_packet(
      &mut self.buffer_ip,
      self.args.common.host,
      IpNextHeaderProtocols::Icmp,
      self.buffer.len(),
      self.args.common.random_source,
    );

    let mut icmp_packet = MutableEchoRequestPacket::new(&mut self.buffer).expect("Error creating icmp packet");
    icmp_packet.set_icmp_type(IcmpTypes::EchoRequest);
    let checksum = util::checksum(icmp_packet.packet_mut(), 1);
    icmp_packet.set_checksum(checksum);

    ipv4_packet.set_payload(icmp_packet.packet_mut());
    ipv4_packet
  }

  fn get_addr(&self) -> Ipv4Addr {
    self.args.common.host
  }
}
