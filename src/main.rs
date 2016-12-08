use std::net::UdpSocket;
use std::net::ToSocketAddrs;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::net::Ipv4Addr;
use std::thread;

fn main() {
    let send_only = false;
    let grp_addr = Ipv4Addr::new(224, 0, 0, 1);
    let grp_port = 9473u16;
    let nic = Ipv4Addr::new(127, 0, 0, 1);


    let grp_sock_addr = SocketAddrV4::new(grp_addr, grp_port);
    let any = Ipv4Addr::new(0, 0, 0, 0);

    let bind_sock_addr = SocketAddrV4::new(any, grp_port);
    if !send_only {
        thread::Builder::new().name("mcast_reader".to_string()).spawn(move || {
            mcast_reader(
                &(bind_sock_addr.to_string()),
                &grp_addr,
                &nic
            );
        });
    }


    read_from_stdin(nic, grp_sock_addr);
}

fn read_from_stdin(nic: Ipv4Addr, grp_sock_addr: SocketAddrV4) {
    
    let mut snd_sock = UdpSocket::bind(SocketAddrV4::new(nic, 0)).unwrap();
    let mut from_in = String::new();
    let istream = std::io::stdin();
    loop {
        istream.read_line(&mut from_in);
        snd_sock.send_to(from_in.as_bytes(), grp_sock_addr);
    }
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

