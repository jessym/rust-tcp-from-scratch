use std::io;

use tun_tap::{Iface, Mode};

fn main() -> io::Result<()> {
    let nic = Iface::new("mytun", Mode::Tun)?;
    let mut buf = [0u8; 1504];
    loop {
        let num_bytes = nic.recv(&mut buf[..])?;
        eprintln!("read {} bytes: {:x?}", num_bytes, &buf[..num_bytes]);
    }
}
