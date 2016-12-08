use std::net::UdpSocket;
use std::net::AddrParseError;
use std::net::ToSocketAddrs;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::net::Ipv4Addr;
use std::error::Error;
use std::str::FromStr;
use std::clone::Clone;
use std::marker::Copy;
use std::thread;

#[derive(Copy)]
struct Params {
    send_only : bool,
    grp_addr : Ipv4Addr,
    grp_port : u16,
    nic : Ipv4Addr,
}

impl Clone for Params {
    fn clone(&self) -> Params { *self }
}


fn main() {
    let params = read_params().unwrap();

    let grp_sock_addr = SocketAddrV4::new(params.grp_addr, params.grp_port);
    let any = Ipv4Addr::new(0, 0, 0, 0);

    let bind_sock_addr = SocketAddrV4::new(any, params.grp_port);
    if !params.send_only {
        thread::Builder::new().name("mcast_reader".to_string()).spawn(move || {
            mcast_reader(
                &(bind_sock_addr.to_string()),
                &params.grp_addr,
                &params.nic
            );
        });
    }

    read_from_stdin(params.nic, grp_sock_addr);
}

fn read_params() -> Result<Params, AddrParseError> {
    let args: Vec<String> = std::env::args().collect();
    let send_only = bool::from_str(&args[1]).unwrap();
    let grp = Ipv4Addr::from_str(&args[2]).unwrap();
    let port = u16::from_str(&args[3]).unwrap();
    let nic = Ipv4Addr::from_str(&args[4]).unwrap();

    Ok(Params {send_only: send_only, grp_addr: grp, grp_port: port, nic: nic})
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

