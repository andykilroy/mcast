extern crate socket2;

use std::net::SocketAddrV4;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::str;
use std::io;
use std::process;
use socket2::{SockAddr, Socket, Domain, Type, Protocol};

use clap::{App, Arg, ArgMatches, SubCommand};
use std::io::ErrorKind;


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
    if let Err(e) = start_app() {
        eprintln!("{}", e);
        process::exit(1);
    };
}

fn start_app() -> std::result::Result<(), String> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("A tool for testing multicast UDP")
        .subcommand(
            SubCommand::with_name("listen")
                .about("Listen on a particular network interface for datagrams from a multicast group")
                .arg(
                    Arg::with_name("GROUP_IP")
                        .help("The multicast group to join")
                        .required(true)
                )
                .arg(
                    Arg::with_name("PORT")
                        .help("The port to bind on")
                        .required(true)
                )
                .arg(
                    Arg::with_name("NIC_IP")
                        .help("The network interface on which to send the join requests")
                        .required(true)
                )
        )
        .subcommand(
            SubCommand::with_name("send")
                .about("Send datagrams to a multicast group via a particular network interface")
                .arg(
                    Arg::with_name("GROUP_IP")
                        .help("The multicast group to send to")
                        .required(true)
                )
                .arg(
                    Arg::with_name("PORT")
                        .help("The destination port to send to")
                        .required(true)
                )
                .arg(
                    Arg::with_name("NIC_IP")
                        .help("The network interface on which to send the datagrams")
                        .required(true)
                )
        ).get_matches();

    match matches.subcommand() {
        ("listen", Some(subm)) => {
            handle_listen(
                &subm.value_of("GROUP_IP").expect("a multicast group was expected"),
                &subm.value_of("PORT").expect("a port number was expected"),
                &subm.value_of("NIC_IP").expect("a nic was expected"),
            )
        },
        ("send", Some(subm)) => {
            handle_send(
                &subm.value_of("GROUP_IP").expect("a multicast group was expected"),
                &subm.value_of("PORT").expect("a port number was expected"),
                &subm.value_of("NIC_IP").expect("a nic was expected"),
            )
        },
        (cmd, _) => Err("unsupported command, try --help option".to_string())
    }
}

fn handle_listen(grp_str: &str, port_str: &str, nic_str: &str) -> Result<(), String> {
    let any = Ipv4Addr::new(0, 0, 0, 0);
    let port = u16::from_str(port_str).map_err(|e| format!("could not parse port number {}, {}", port_str, e))?;
    let grp = Ipv4Addr::from_str(grp_str).map_err(|e| format!("could not parse group address {}, {}", grp_str, e))?;
    let nic = Ipv4Addr::from_str(nic_str).map_err(|e| format!("could not parse nic address {}, {}", nic_str, e))?;
    let bind_sock_addr = SocketAddrV4::new(any, port);
    mcast_reader_v4(&bind_sock_addr, &grp, &nic).map_err(|e| format!("{}", e))
}

fn handle_send(grp_str: &str, port_str: &str, nic_str: &str) -> Result<(), String> {
    let port = u16::from_str(port_str).map_err(|e| format!("could not parse port number {}, {}", port_str, e))?;
    let grp = Ipv4Addr::from_str(grp_str).map_err(|e| format!("could not parse group address {}, {}", grp_str, e))?;
    let nic = Ipv4Addr::from_str(nic_str).map_err(|e| format!("could not parse nic address {}, {}", nic_str, e))?;
    send_to_mcast_socket(nic, SocketAddrV4::new(grp, port)).map_err(|e| format!("{}", e))
}

fn use_parameters(params: Params) {
    let any = Ipv4Addr::new(0, 0, 0, 0);
    let bind_sock_addr = SocketAddrV4::new(any, params.grp_sock_addr.port());

    let res = match params.action {
        Command::SEND =>
            send_to_mcast_socket(
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


fn send_to_mcast_socket(nic: Ipv4Addr, group: SocketAddrV4) -> io::Result<()> {
    let snd_sock = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
    let bindaddr = SockAddr::from(SocketAddrV4::new(nic, 0));
    let dest = SockAddr::from(group);
    snd_sock.set_multicast_ttl_v4(1)?;
    snd_sock.bind(&bindaddr)?;
    let istream = std::io::stdin();

    let mut from_in = String::new();
    loop {
        match istream.read_line(&mut from_in) {
            Ok(0) => return Ok(()),
            Ok(_n) => send_all_bytes(from_in.trim_end().as_bytes(), &snd_sock, &dest)?,
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
    socket.set_reuse_port(true)?;
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

