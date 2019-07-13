extern crate socket2;

use socket2::{Domain, Protocol, SockAddr, Socket, Type};

use std::io;
use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
use std::process;
use std::str;
use std::str::FromStr;
use std::io::{Read, Write, Stdout};

use clap::{App, Arg, SubCommand};

use failure::ResultExt;
use failure::Error;
use exitfailure::ExitFailure;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "A tool for testing multicast UDP", rename_all = "kebab-case")]
enum CommandArgs {
    #[structopt(name = "listen")]
    /// Listen on a particular network interface for datagrams from a multicast group
    Listen {
        /// The multicast group to join
        group_ip: String,
        /// The port to bind on
        port: String,
        /// The network interface on which to send the join requests
        nic: String,
        #[structopt(name = "printsrc", long)]
        /// Print where the datagram came from
        print_src_addr: bool,
        #[structopt(name = "base64", long)]
        /// Encode incoming datagrams in base64
        base64_enc: bool,
    },
    #[structopt(name = "send")]
    /// Send datagrams to a multicast group via a particular network interface
    Send {
        /// The multicast group to send to
        group_ip: String,
        /// The destination port to send to
        port: String,
        /// The network interface on which to send the datagrams
        nic: String,
    }
}

fn main() -> Result<(), ExitFailure> {
    let args = CommandArgs::from_args();

    match args {
        CommandArgs::Listen {
            group_ip: g,
            port: p,
            nic: n,
            print_src_addr: ps,
            base64_enc: b
        } => handle_listen(&g, &p, &n, ps, b),
        CommandArgs::Send {
            group_ip: g,
            port: p,
            nic: n,
        } => handle_send(&g, &p, &n),
    }?;
    Ok(())
}

fn handle_listen(
    grp_str: &str,
    port_str: &str,
    nic_str: &str,
    printsrc: bool,
    base64enc: bool
) -> Result<(), Error> {
    let any = Ipv4Addr::new(0, 0, 0, 0);
    let port = u16::from_str(port_str).with_context(|_c| format!("Could not parse port number {}", port_str))?;
    let grp = Ipv4Addr::from_str(grp_str).with_context(|_c| format!("Could not parse group address {}", grp_str))?;
    let nic = Ipv4Addr::from_str(nic_str).with_context(|_c| format!("Could not parse nic address {}", nic_str))?;
    let bind_sock_addr = SocketAddrV4::new(any, port);
    mcast_v4_readfrom(bind_sock_addr, grp, nic, printsrc, base64enc)?;
    Ok(())
}

fn handle_send(grp_str: &str, port_str: &str, nic_str: &str) -> Result<(), Error> {
    let port = u16::from_str(port_str).with_context(|_c| format!("Could not parse port number {}", port_str))?;
    let grp = Ipv4Addr::from_str(grp_str).with_context(|_c| format!("Could not parse group address {}", grp_str))?;
    let nic = Ipv4Addr::from_str(nic_str).with_context(|_c| format!("Could not parse nic address {}", nic_str))?;
    mcast_v4_sendto(nic, SocketAddrV4::new(grp, port))?;
    Ok(())
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
    sock.send_to(bytes, dest)?;
    Ok(())
}

fn mcast_v4_readfrom(
    bindaddr: SocketAddrV4,
    mcastgrp: Ipv4Addr,
    interface: Ipv4Addr,
    printsrc: bool,
    base64enc: bool,
) -> io::Result<()> {
    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
    let addr = SockAddr::from(bindaddr);
    socket.set_reuse_address(true)?;
    socket.set_reuse_port(true)?;
    socket.bind(&addr)?;

    socket.join_multicast_v4(&mcastgrp, &interface)?;
    read_loop(&socket, printsrc, base64enc)?;
    Ok(())
}

fn read_loop(socket: &Socket, printsrc: bool, base64enc: bool) -> io::Result<()> {
    let mut rcv_buf: [u8; 65536] = [0; 65536];
    let mut stdout = std::io::stdout();
    loop {
        let (byte_count, sender) = socket.recv_from(&mut rcv_buf[..]).unwrap();
        if printsrc {
            writeln!(stdout, "### from /{}", sender.as_inet().unwrap())?;
        }
        if base64enc {
            write_base64(&mut stdout, &rcv_buf[0..byte_count])?;
        } else {
            stdout.write_all(&rcv_buf[0..byte_count])?;
        }
        stdout.flush()?;
    }
}

fn write_base64(stdout: &mut Stdout, input: &[u8]) -> io::Result<()> {
    // a length that will produce a base64 encoded line of 64 chars.
    let piece_length = 48;
    let limit = input.len() / piece_length;
    for i in 0..limit {
        let start = i * piece_length;
        let end = (i + 1) * piece_length;
        let encoded = base64::encode(&input[start..end]);
        stdout.write_all(encoded.as_bytes())?;
        stdout.write_all("\n".as_bytes())?;
    }

    let encoded = base64::encode(&input[(limit * piece_length)..]);
    stdout.write_all(encoded.as_bytes())?;
    stdout.write_all("\n".as_bytes())
}
