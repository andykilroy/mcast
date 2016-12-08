use std::net::UdpSocket;
use std::net::Ipv4Addr;

fn main() {
    mcast_reader(
        "0.0.0.0:9473",
        &Ipv4Addr::new(224, 0, 0, 1),
        &Ipv4Addr::new(127, 0, 0, 1)
    );
}

fn mcast_reader(bindaddr: &str,
                mcastgrp: &Ipv4Addr,
                interface: &Ipv4Addr) {
    let mut socket = UdpSocket::bind(bindaddr).unwrap();
    let mut rcv_buf: [u8; 65536] = [0; 65536];

    socket.join_multicast_v4(mcastgrp, interface).unwrap();
    loop {
        let (byte_count, sender) = socket.recv_from(&mut rcv_buf[..]).unwrap();
    }
}

