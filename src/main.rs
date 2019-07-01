extern crate socket2;

use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::io;
use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
use std::process;
use std::str;
use std::str::FromStr;

use clap::{App, Arg, SubCommand};
use std::io::{Read, Write};

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
                .about(
                    "Listen on a particular network interface for datagrams from a multicast group",
                )
                .arg(
                    Arg::with_name("GROUP_IP")
                        .help("The multicast group to join")
                        .required(true),
                )
                .arg(
                    Arg::with_name("PORT")
                        .help("The port to bind on")
                        .required(true),
                )
                .arg(
                    Arg::with_name("NIC_IP")
                        .help("The network interface on which to send the join requests")
                        .required(true),
                )
                .arg(
                    Arg::with_name("PRINT_SRCADDR")
                        .long("printsrc")
                        .help("Print where the datagram came from")
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("send")
                .about("Send datagrams to a multicast group via a particular network interface")
                .arg(
                    Arg::with_name("GROUP_IP")
                        .help("The multicast group to send to")
                        .required(true),
                )
                .arg(
                    Arg::with_name("PORT")
                        .help("The destination port to send to")
                        .required(true),
                )
                .arg(
                    Arg::with_name("NIC_IP")
                        .help("The network interface on which to send the datagrams")
                        .required(true),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("listen", Some(subm)) => handle_listen(
            &subm.value_of("GROUP_IP").expect("a multicast group was expected"),
            &subm.value_of("PORT").expect("a port number was expected"),
            &subm.value_of("NIC_IP").expect("a nic was expected"),
            subm.is_present("PRINT_SRCADDR"),
        ),
        ("send", Some(subm)) => handle_send(
            &subm.value_of("GROUP_IP").expect("a multicast group was expected"),
            &subm.value_of("PORT").expect("a port number was expected"),
            &subm.value_of("NIC_IP").expect("a nic was expected"),
        ),
        (_, _) => Err("unsupported command, try --help option".to_string()),
    }
}

fn handle_listen(
    grp_str: &str,
    port_str: &str,
    nic_str: &str,
    printsrc: bool,
) -> Result<(), String> {
    let any = Ipv4Addr::new(0, 0, 0, 0);
    let port = u16::from_str(port_str).map_err(|e| format!("could not parse port number {}, {}", port_str, e))?;
    let grp = Ipv4Addr::from_str(grp_str).map_err(|e| format!("could not parse group address {}, {}", grp_str, e))?;
    let nic = Ipv4Addr::from_str(nic_str).map_err(|e| format!("could not parse nic address {}, {}", nic_str, e))?;
    let bind_sock_addr = SocketAddrV4::new(any, port);
    mcast_v4_readfrom(bind_sock_addr, grp, nic, printsrc).map_err(|e| format!("{}", e))
}

fn handle_send(grp_str: &str, port_str: &str, nic_str: &str) -> Result<(), String> {
    let port = u16::from_str(port_str).map_err(|e| format!("could not parse port number {}, {}", port_str, e))?;
    let grp = Ipv4Addr::from_str(grp_str).map_err(|e| format!("could not parse group address {}, {}", grp_str, e))?;
    let nic = Ipv4Addr::from_str(nic_str).map_err(|e| format!("could not parse nic address {}, {}", nic_str, e))?;
    mcast_v4_sendto(nic, SocketAddrV4::new(grp, port)).map_err(|e| format!("{}", e))
}

fn mcast_v4_sendto(nic: Ipv4Addr, group: SocketAddrV4) -> io::Result<()> {
    let snd_sock = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
    let bindaddr = SockAddr::from(SocketAddrV4::new(nic, 0));
    let dest = SockAddr::from(group);
    snd_sock.set_multicast_ttl_v4(1)?;
    snd_sock.bind(&bindaddr)?;
    let mut istream = std::io::stdin();

    let mut from_in: [u8; 65536] = [0; 65536];
    loop {
        match istream.read(&mut from_in) {
            Ok(0) => return Ok(()),
            Ok(n) => send_all_bytes(&from_in[0..n], &snd_sock, &dest)?,
            Err(e) => return Err(e),
        }
    }
}

fn send_all_bytes(bytes: &[u8], sock: &Socket, dest: &SockAddr) -> io::Result<()> {
    match sock.send_to(bytes, dest) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn mcast_v4_readfrom(
    bindaddr: SocketAddrV4,
    mcastgrp: Ipv4Addr,
    interface: Ipv4Addr,
    printsrc: bool,
) -> io::Result<()> {
    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
    let addr = SockAddr::from(bindaddr);
    socket.set_reuse_address(true)?;
    socket.set_reuse_port(true)?;
    socket.bind(&addr)?;

    socket.join_multicast_v4(&mcastgrp, &interface)?;
    binary_read_loop(&socket, printsrc)?;
    Ok(())
}

fn binary_read_loop(socket: &Socket, printsrc: bool) -> io::Result<()> {
    let mut rcv_buf: [u8; 65536] = [0; 65536];
    let mut stdout = std::io::stdout();
    loop {
        let (byte_count, sender) = socket.recv_from(&mut rcv_buf[..]).unwrap();
        if printsrc {
            writeln!(stdout, "from /{}", sender.as_inet().unwrap())?;
        }
        stdout.write_all(&rcv_buf[0..byte_count])?;
        stdout.flush()?;
    }
}
