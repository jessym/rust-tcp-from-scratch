use std::collections::HashMap;
use std::hash::Hash;
use std::io;
use std::net::Ipv4Addr;

use etherparse::{Ipv4HeaderSlice, TcpHeaderSlice};
use tun_tap::{Iface, Mode};

mod tcp;

const PROTO_IPV4: u16 = 0x0800;
// const PROTO_IPV6: u16 = 0x86dd;
// const PROTO_ARP: u16 = 0x0806;

#[derive(Hash, PartialEq, Eq)]
struct Socket {
    ip: Ipv4Addr,
    port: u16,
}

impl Default for Socket {
    fn default() -> Self {
        Self {
            ip: Ipv4Addr::new(0, 0, 0, 0),
            port: 0,
        }
    }
}

#[derive(Default, Hash, PartialEq, Eq)]
struct SocketPair {
    src: Socket,
    dest: Socket,
}

fn main() -> io::Result<()> {
    let _sp = SocketPair::default();

    let _connections = HashMap::<SocketPair, tcp::State>::default();
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

        let ip_packet = &buffer[incoming_frame_tuntap_prepended_info_length..];
        let ip_packet_size =
            incoming_frame_total_size - incoming_frame_tuntap_prepended_info_length;
        // Note: you cannot use `let ip_packet_size = ip_packet.len();` here...
        // ...that's because `ip_packet.len()` will always return the number 1500 (buffer size minus 4), NOT how many bytes were actually copied into it from the interface

        /*
        For the `incoming_frame_tuntap_proto`, you might get the value 0x86DD
         - this refers to the protocol number of the parent Ethernet frame
         - in case of 0x86DD, it's an IPv6 packet: https://en.wikipedia.org/wiki/EtherType
        For convenience, let's ignore anything which isn't IPv4 on the next line

        Quick side note, we would have only gotten IP packets (ipv4 or ipv6) at this point anyway...
        ...because our interface operates in TUN mode, which filters out ARP packets and anything part of an ENET frame which doesn't declare its payload to be an IP packet
         */
        if incoming_frame_tuntap_proto != PROTO_IPV4 {
            continue;
        }
        match Ipv4HeaderSlice::from_slice(ip_packet) {
            Err(e) => {
                eprintln!("Skipping weird IPv4 packet: {:?}", e)
            }
            Ok(ipv4_header) => {
                let ip_packet_proto = ipv4_header.protocol();
                let ip_packet_src = ipv4_header.source_addr();
                let ip_packet_dest = ipv4_header.destination_addr();
                let ip_packet_payload_size = ipv4_header.payload_len().unwrap();
                let tcp_segment = &ip_packet[ipv4_header.slice().len()..];
                // Alternative: let tcp_segment = &ip_packet[(ip_packet_size - ip_packet_payload_size as usize)..];
                match TcpHeaderSlice::from_slice(tcp_segment) {
                    // Alternative: match TcpHeaderSlice::from_slice(&packet[ipv4_header.slice().len()..]) {
                    Err(e) => {
                        eprintln!("Skipping weird TCP packet: {:?}", e);
                    }
                    Ok(tcp_header) => {
                        let tcp_segment_src = tcp_header.source_port();
                        let tcp_segment_dest = tcp_header.destination_port();
                        eprintln!(
                            "🎭 [EFRAME_FLAGS={}, EFRAME_PROTO=0x{:x}, IP_PACKET_SIZE={}]",
                            incoming_frame_tuntap_flags,
                            incoming_frame_tuntap_proto,
                            ip_packet_size
                        );
                        eprintln!(
                            "   > IP [Address {} ==> {}]: received {} bytes of protocol {:?}",
                            ip_packet_src, ip_packet_dest, ip_packet_payload_size, ip_packet_proto
                        );
                        eprintln!(
                            "      > TCP [Port {} ==> {}]",
                            tcp_segment_src, tcp_segment_dest
                        );
                    }
                }
            }
        }
    }
}
