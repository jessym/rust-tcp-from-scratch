use std::io;
use etherparse::Ipv4HeaderSlice;

use tun_tap::{Iface, Mode};

const PROTO_IPV4: u16 = 0x800;

fn main() -> io::Result<()> {
    let iface = Iface::new("mytun", Mode::Tun)?;
    let mut buf = [0u8; 1504];
    loop {
        let num_bytes = iface.recv(&mut buf[..])?;
        /*
        By default, tun/tap prepends packet information (4 bytes) for incoming data:
         - 2 bytes for flags
         - 2 bytes for protocol
        See https://docs.kernel.org/networking/tuntap.html
        */
        let flags = u16::from_be_bytes([buf[0], buf[1]]);
        let proto = u16::from_be_bytes([buf[2], buf[3]]);
        /*
        For the "proto", you might get the value 0x86DD
         - it's a bit difficult to find what this means
         - best thing to do, is Google: "protocol 86dd"
         - you'll find that this is an IPv6 packet: https://en.wikipedia.org/wiki/EtherType

         Let's ignore anything which isn't IPv4
         */
        if proto != PROTO_IPV4 {
            continue;
        }
        match Ipv4HeaderSlice::from_slice(&buf[4..num_bytes]) {
            Err(e) => {
                eprintln!("Skipping weird packet: {:?}", e)
            }
            Ok(header) => {
                eprintln!(
                    "[PI_FLAGS={}, PI_ETHER_TYPE=0x{:x}, num_bytes={}]: proto={:?}",
                    flags, proto, num_bytes - 4, header.protocol()
                );
            }
        }
    }
}
