use std::net::Ipv4Addr;

use pnet::packet::ip::IpNextHeaderProtocol;
use pnet::packet::ipv4::MutableIpv4Packet;

pub const IPV4_HEADER_LEN: usize = 21;
const TTL: u8 = 64;

pub fn create_ipv4_packet(
  buffer_ip: &mut [u8],
  dest: Ipv4Addr,
  next_level_protocol: IpNextHeaderProtocol,
  payload_size: usize,
) -> MutableIpv4Packet {
  let mut ipv4_packet = MutableIpv4Packet::new(buffer_ip).expect("Error creating ipv4 packet");
  ipv4_packet.set_version(4);
  ipv4_packet.set_header_length(IPV4_HEADER_LEN as u8);
  ipv4_packet.set_total_length((IPV4_HEADER_LEN + payload_size) as u16);
  ipv4_packet.set_ttl(TTL);
  ipv4_packet.set_next_level_protocol(next_level_protocol);
  ipv4_packet.set_destination(dest);
  ipv4_packet
}
