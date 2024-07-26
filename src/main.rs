mod tcp;

use std::collections::HashMap;
use std::hash::Hash;
use std::io;
use std::net::Ipv4Addr;

use etherparse::{Ipv4HeaderSlice, TcpHeaderSlice};
use tun_tap::{Iface, Mode};

use crate::tcp::State;

// Proto field in Ethernet frame header
const ETHERNET_FRAME_PROTO_IPV4: u16 = 0x0800;

// Proto field in the IP packet header
const IP_PACKET_PROTO_TCP: u8 = 0x06;

#[derive(Hash, PartialEq, Eq)]
struct Socket {
    ip: Ipv4Addr,
    port: u16,
}

#[derive(Hash, PartialEq, Eq)]
struct SocketPair {
    src: Socket,
    dest: Socket,
}

fn main() -> io::Result<()> {
    let mut connections = HashMap::<SocketPair, State>::new();

    /*
    The TUN mode means: you'll be supplied with raw IP packets
    The TAP mode means: you'll be supplied with raw Ethernet frames
     */
    let iface = Iface::new("mytun", Mode::Tun)?;
    let mut buffer = [0u8; 1504];
    loop {
        let incoming_frame_total_size = iface.recv(&mut buffer[..])?;

        /*
        By default, tun/tap prepends packet information (4 bytes) for incoming data:
         - 2 bytes for flags
         - 2 bytes for protocol (= always the protocol number of the ETHERNET FRAME, whether you're using TUN or TAP)
        For more information:
         - See the Rust library documentation of the Mode enum (either in RustRover or via this link): https://docs.rs/tun-tap/latest/tun_tap/enum.Mode.html
         - See the Kernel's TUN/TAP documentation: https://docs.kernel.org/networking/tuntap.html
        */
        let incoming_frame_tuntap_prepended_info_length = 4;
        let incoming_frame_tuntap_flags = u16::from_be_bytes([buffer[0], buffer[1]]);
        let incoming_frame_tuntap_proto = u16::from_be_bytes([buffer[2], buffer[3]]);
        if incoming_frame_tuntap_proto != ETHERNET_FRAME_PROTO_IPV4 {
            /*
            You might get some frames with EtherType 0x86DD here...
             - this refers to the protocol number of the parent Ethernet frame
             - in case of 0x86DD, it's an IPv6 packet: https://en.wikipedia.org/wiki/EtherType
            For convenience, we ignore anything which isn't IPv4
             */
            eprintln!("Skipping unsupported Ethernet frame: 0x{:x}", incoming_frame_tuntap_proto);
            continue;
            /*
            Quick side note, we would have only gotten IP packets (ipv4 or ipv6) at this point anyway...
            ...because our interface operates in TUN mode, which filters out ARP packets and any other types of Ethernet frames payloads
             */
        }

        // We know that the payload "must" (should) be an IP packet at this point
        let ip_packet = &buffer[incoming_frame_tuntap_prepended_info_length..incoming_frame_total_size];
        match Ipv4HeaderSlice::from_slice(ip_packet) {
            Err(e) => {
                eprintln!("Skipping invalid IPv4 packet: {:?}", e)
            }
            Ok(ipv4_header) => {
                let ip_packet_src_addr = ipv4_header.source_addr();
                let ip_packet_dest_addr = ipv4_header.destination_addr();
                let ip_packet_proto = ipv4_header.protocol();
                if ip_packet_proto.0 != IP_PACKET_PROTO_TCP {
                    eprintln!("Skipping unsupported IPv4 packet: {:?}", ip_packet_proto);
                    continue;
                }

                // We know that the payload "must" (should) be a TCP segment at this point
                let tcp_segment = &ip_packet[ipv4_header.slice().len()..];
                match TcpHeaderSlice::from_slice(tcp_segment) {
                    Err(e) => {
                        eprintln!("Skipping invalid TCP packet: {:?}", e);
                    }
                    Ok(tcp_header) => {
                        let tcp_segment_src_port = tcp_header.source_port();
                        let tcp_segment_dest_port = tcp_header.destination_port();
                        let tcp_payload = &tcp_segment[tcp_header.slice().len()..];
                        eprintln!(
                            "🎭 [EFRAME_FLAGS={}, EFRAME_PROTO=0x{:x}, IP_PACKET_SIZE={}]",
                            incoming_frame_tuntap_flags,
                            incoming_frame_tuntap_proto,
                            ip_packet.len()
                        );

                        let socket_pair = SocketPair {
                            src: Socket {
                                ip: ip_packet_src_addr,
                                port: tcp_segment_src_port,
                            },
                            dest: Socket {
                                ip: ip_packet_dest_addr,
                                port: tcp_segment_dest_port,
                            },
                        };
                        connections
                            .entry(socket_pair)
                            .or_default()
                            .handle_packet(ipv4_header, tcp_header, tcp_payload);
                    }
                }
            }
        }
    }
}
