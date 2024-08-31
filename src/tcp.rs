use etherparse::{IpNumber, Ipv4Header, Ipv4HeaderSlice, TcpHeader, TcpHeaderSlice};
use std::io;
use tun_tap::Iface;

#[derive(Debug)]
enum State {
    Closed,
    Listen,
    SynRcvd,
    Estab,
}

#[derive(Debug)]
pub struct Connection {
    state: State,
}

impl Default for Connection {
    fn default() -> Self {
        Self {
            state: State::Listen,
        }
    }
}

impl Connection {
    pub fn handle_packet(
        &mut self,
        interface: &Iface,
        ipv4_header: Ipv4HeaderSlice,
        tcp_header: TcpHeaderSlice,
        tcp_payload: &[u8],
    ) -> io::Result<usize> {
        eprintln!(
            "   > IP [Address {} ==> {}]: received {} payload bytes of protocol {:?}",
            ipv4_header.source_addr(),
            ipv4_header.destination_addr(),
            ipv4_header.payload_len().unwrap(),
            ipv4_header.protocol(),
        );
        eprintln!(
            "      > TCP [Port {} ==> {}]: received {} payload bytes",
            tcp_header.source_port(),
            tcp_header.destination_port(),
            tcp_payload.len(),
        );

        let mut buffer = [0u8; 1500];
        match self.state {
            State::Closed => {
                eprintln!(
                    "HANDSHAKE for state {:?} - not accepting any segments",
                    self
                );
                Ok(0)
            }
            State::Listen => {
                if !tcp_header.syn() {
                    eprintln!(
                        "HANDSHAKE for state {:?} - only expecting SYN segments",
                        self
                    );
                }

                // Received SYN, sending SYN,ACK
                let mut tcp_response = TcpHeader::new(
                    tcp_header.destination_port(),
                    tcp_header.source_port(),
                    todo!(),
                    todo!(),
                );
                tcp_response.syn = true;
                tcp_response.ack = true;

                let mut ip_response = Ipv4Header::new(
                    tcp_response.header_len_u16(),
                    todo!(),
                    IpNumber::TCP,
                    ipv4_header.destination(),
                    ipv4_header.source(),
                )
                .unwrap();

                let mut unwritten = &mut buffer[..];
                ip_response.write(&mut unwritten)?;
                tcp_response.write(&mut unwritten)?;
                // ip_response.write()

                Ok(0)
            }
            State::SynRcvd => Ok(0),
            State::Estab => Ok(0),
        }
    }
}
