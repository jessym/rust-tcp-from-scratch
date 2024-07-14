use etherparse::{Ipv4HeaderSlice, TcpHeaderSlice};

#[derive(Default)]
pub struct State {}

impl State {
    pub fn handle_packet(&mut self, ipv4_header: Ipv4HeaderSlice, tcp_header: TcpHeaderSlice, tcp_payload: &[u8]) {
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
    }
}
