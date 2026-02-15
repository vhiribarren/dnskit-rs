pub mod message;
pub mod parser;

pub const UDP_RECV_BUFFER_SIZE: usize = 512;
pub type UdpRecvBuffer = [u8; UDP_RECV_BUFFER_SIZE];

pub fn allocate_udp_recv_buffer() -> UdpRecvBuffer {
    [0; UDP_RECV_BUFFER_SIZE]
}
