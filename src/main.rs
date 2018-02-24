extern crate socket2;

use std::net::UdpSocket;
use std::net::SocketAddrV4;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::str;
use std::clone::Clone;
use std::thread;
use std::io;
use socket2::{SockAddr, Socket, Domain, Type, Protocol};



#[derive(Copy)]
struct Params {
    send_only: bool,
    grp_sock_addr: SocketAddrV4,
    nic: Ipv4Addr,
}


impl Clone for Params {
    fn clone(&self) -> Params { *self }
}



fn main() {
    match read_params() {
        Ok(params) => use_parameters(params),
        Err(e) => eprintln!("could not parse params, {}", e)
    };
}

fn use_parameters(params: Params) {
    let any = Ipv4Addr::new(0, 0, 0, 0);
    let bind_sock_addr = SocketAddrV4::new(any, params.grp_sock_addr.port());

    if !params.send_only {
        thread::Builder::new().name("mcast_reader".to_string()).spawn(move || {
            mcast_reader_v4(
                &bind_sock_addr,
                &params.grp_sock_addr.ip(),
                &params.nic
            ).unwrap();
        }).unwrap();
    }

    read_from_stdin(params.nic, params.grp_sock_addr).unwrap();
}



fn read_params() -> std::result::Result<Params, String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 5 {
        return Err(format!("expected {} parameters, but there were {} in {:?}", 4, args.len() - 1, args))
    }
    let usage_string = "Usage: mcast <sendonly:bool> <group:ip> <port:int> <nic:ip>";
    let send_only = bool::from_str(&args[1]).map_err(|_| usage_string)?;
    let grp = Ipv4Addr::from_str(&args[2]).map_err(|_| usage_string)?;
    let port = u16::from_str(&args[3]).map_err(|_| usage_string)?;
    let nic = Ipv4Addr::from_str(&args[4]).map_err(|_| usage_string)?;

    Ok(Params {send_only: send_only, grp_sock_addr: SocketAddrV4::new(grp, port), nic: nic})
}



fn read_from_stdin(bindaddr: Ipv4Addr, dest: SocketAddrV4) -> io::Result<()> {
    let snd_sock = UdpSocket::bind(SocketAddrV4::new(bindaddr, 0)).unwrap();
    let istream = std::io::stdin();

    let mut from_in = String::new();
    loop {
        istream.read_line(&mut from_in)?;
        snd_sock.send_to(from_in.as_bytes(), dest)?;
        from_in.clear();
    }
}



fn mcast_reader_v4(bindaddr:  &SocketAddrV4,
                   mcastgrp:  &Ipv4Addr,
                   interface: &Ipv4Addr      ) -> io::Result<()> {
    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
    let addr = SockAddr::from(*bindaddr);
    socket.set_reuse_address(true)?;
    socket.set_ttl(1)?;
    socket.bind(&addr)?;

    socket.join_multicast_v4(mcastgrp, interface)?;
    udp_read_loop(&socket)?;
    Ok(())
}



fn udp_read_loop(socket: &Socket) -> io::Result<()> {
    let mut rcv_buf: [u8; 65536] = [0; 65536];
    loop {
        let (byte_count, sender) = socket.recv_from(&mut rcv_buf[..]).unwrap();
        let s = str::from_utf8(&rcv_buf[0..byte_count]).unwrap();
        println!("from {:?} rcvd '{}'", sender, s);
    }
}

