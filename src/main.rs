extern crate socket2;

use std::net::SocketAddrV4;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::str;
use std::io;
use socket2::{SockAddr, Socket, Domain, Type, Protocol};


struct Params {
    action: Command,
    grp_sock_addr: SocketAddrV4,
    nic: Ipv4Addr,
}

#[derive(Debug)]
enum Command {
    SEND,
    LISTEN,
//    JOIN,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmdParseError{

}

impl FromStr for Command {
    type Err = CmdParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "send" => Ok(Command::SEND),
            "listen" => Ok(Command::LISTEN),
            _ => Err(CmdParseError{})
        }
    }
}




fn main() {
    match read_params() {
        Ok(params) => use_parameters(params),
        Err(e) => eprintln!("Could not parse params, {}", e)
    };
}

fn read_params() -> std::result::Result<Params, String> {
    let usage_string = "Usage: mcast send|listen <group:ip> <port:int> <nic:ip>";
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 5 {
        return Err(format!("expected more\n{}", usage_string))
    }
    let act = Command::from_str(&args[1]).map_err(|_| usage_string)?;
    let grp = Ipv4Addr::from_str(&args[2]).map_err(|_| usage_string)?;
    let port = u16::from_str(&args[3]).map_err(|_| usage_string)?;
    let nic = Ipv4Addr::from_str(&args[4]).map_err(|_| usage_string)?;

    Ok(Params {action: act, grp_sock_addr: SocketAddrV4::new(grp, port), nic: nic})
}

fn use_parameters(params: Params) {
    let any = Ipv4Addr::new(0, 0, 0, 0);
    let bind_sock_addr = SocketAddrV4::new(any, params.grp_sock_addr.port());

    let res = match params.action {
        Command::SEND =>
            read_from_stdin(
                params.nic,
                params.grp_sock_addr
            ),
        Command::LISTEN =>
            mcast_reader_v4(
                &bind_sock_addr,
                &params.grp_sock_addr.ip(),
                &params.nic
            ),
    };

    match res {
        Err(e) => eprintln!("error while running: {}", e),
        _ => ()
    }
}


fn read_from_stdin(b: Ipv4Addr, d: SocketAddrV4) -> io::Result<()> {
    let snd_sock = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
    let bindaddr = SockAddr::from(SocketAddrV4::new(b, 0));
    let dest = SockAddr::from(d);
    snd_sock.set_ttl(1)?;
    snd_sock.bind(&bindaddr)?;
    let istream = std::io::stdin();

    let mut from_in = String::new();
    loop {
        match istream.read_line(&mut from_in) {
            Ok(0) => return Ok(()),
            Ok(_n) => send_all_bytes(from_in.trim_right().as_bytes(), &snd_sock, &dest)?,
            Err(e) => return Err(e)
        }
        from_in.clear();
    }
}

fn send_all_bytes(bytes: &[u8],
                  sock: &Socket,
                  dest: &SockAddr) -> io::Result<()> {
    match sock.send_to(bytes, dest) {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }
}


fn mcast_reader_v4(bindaddr:  &SocketAddrV4,
                   mcastgrp:  &Ipv4Addr,
                   interface: &Ipv4Addr      ) -> io::Result<()> {
    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
    let addr = SockAddr::from(*bindaddr);
    socket.set_reuse_address(true)?;
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
        println!("from /{}  {}", sender.as_inet().unwrap(), s);
    }
}

