use std::io;
use etherparse::{Ipv4HeaderSlice, TcpHeaderSlice};

use tun_tap::{Iface, Mode};

const PROTO_IPV4: u16 = 0x800;

fn main() -> io::Result<()> {
    /*
    The TUN mode means: you'll be supplied with raw IP packets
    The TAP mode means: you'll be supplied with raw Ethernet frames
     */
    let iface = Iface::new("mytun", Mode::Tun)?;
    let mut buf = [0u8; 1504];
    loop {
        let num_bytes = iface.recv(&mut buf[..])?;
        /*
        By default, tun/tap prepends packet information (4 bytes) for incoming data:
         - 2 bytes for flags
         - 2 bytes for protocol (= always the protocol number of the ETHERNET FRAME, whether you're using TUN or TAP)
         - Also: see the documentation of Mode::Tun
        See https://docs.kernel.org/networking/tuntap.html
        */
        let packet_info_length = 4;
        let packet_info_flags = u16::from_be_bytes([buf[0], buf[1]]);
        let packet_info_proto = u16::from_be_bytes([buf[2], buf[3]]);
        let packet = &buf[packet_info_length..];
        let packet_size = num_bytes - packet_info_length;
        /*
        For the "proto", you might get the value 0x86DD
         - this refers to the protocol number of the parent Ethernet frame
         - in case of 0x86DD, it's an IPv6 packet: https://en.wikipedia.org/wiki/EtherType
         - For convenience, let's ignore anything which isn't IPv4 on the next line
         */
        if packet_info_proto != PROTO_IPV4 {
            continue;
        }
        match Ipv4HeaderSlice::from_slice(&buf[4..num_bytes]) {
            Err(e) => {
                eprintln!("Skipping weird IPv4 packet: {:?}", e)
            }
            Ok(ipv4_header) => {
                let protocol = ipv4_header.protocol();
                let ip_src = ipv4_header.source_addr();
                let ip_dest = ipv4_header.destination_addr();
                let ip_payload_size = ipv4_header.payload_len().unwrap();
                match TcpHeaderSlice::from_slice(&packet[(packet_size - ip_payload_size as usize)..]) {
                    // Alternative: match TcpHeaderSlice::from_slice(&packet[ipv4_header.slice().len()..]) {
                    Err(e) => {
                        eprintln!("Skipping weird TCP packet: {:?}", e);
                    }
                    Ok(tcp_header) => {
                        let port_src = tcp_header.source_port();
                        let port_dest = tcp_header.destination_port();
                        eprintln!(
                            "🎭 [EFRAME_FLAGS={}, EFRAME_PROTO=0x{:x}, IP_PACKET_SIZE={}]",
                            packet_info_flags, packet_info_proto, packet_size);
                        eprintln!("   IP [Address {} -> {}] Got {} payload bytes of protocol {:?}",
                                  ip_src, ip_dest, ip_payload_size, protocol);
                        eprintln!("      TCP [Port {} -> {}]",
                                  port_src, port_dest);
                    }
                }
            }
        }
    }
}
