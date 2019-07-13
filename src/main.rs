extern crate socket2;

use socket2::{Domain, Protocol, SockAddr, Socket, Type};

use std::io;
use std::io::{Read, Stdout, Write};
use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
use std::str;
use std::str::FromStr;

use exitfailure::ExitFailure;

#[macro_use]
extern crate failure;
use failure::Error;
use failure::ResultExt;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "A tool for testing multicast UDP", rename_all = "kebab-case")]
enum CommandArgs {
    #[structopt(name = "listen")]
    /// Listen on a particular network interface for datagrams
    /// from one or more multicast groups
    ListenV4(ListenV4Args),

    #[structopt(name = "send")]
    /// Send datagrams to a multicast group via a particular
    /// network interface
    SendV4(SendV4Args),
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct ListenV4Args {
    /// The network interface on which to send the join requests
    nic: Ipv4Addr,
    /// The port to bind on
    port: u16,
    /// A multicast group to join
    group_ip: Ipv4Addr,
    /// Additional multicast groups to join
    additional_grp_ips: Vec<Ipv4Addr>,

    #[structopt(name = "printsrc", long)]
    /// Print the incoming datagram's source address
    print_src_addr: bool,
    #[structopt(name = "base64", long)]
    /// Encode incoming datagrams in base64
    base64_enc: bool,
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct SendV4Args {
    #[structopt(long, default_value = "1")]
    /// Instructs routers to discard the datagram if it traverses
    /// more than this number of hops
    hops: u32,
    /// The network interface on which to send the datagrams
    nic: Ipv4Addr,
    /// The destination port to send to
    port: u16,
    /// The multicast group to send to
    group_ip: Ipv4Addr,
}

fn main() -> Result<(), ExitFailure> {
    let args = CommandArgs::from_args();

    match args {
        CommandArgs::ListenV4(l) => handle_listen(l),
        CommandArgs::SendV4(s) => handle_send(s),
    }?;
    Ok(())
}

fn handle_listen(args: ListenV4Args) -> Result<(), Error> {
    let socket = ipv4_server_socket(&args)
        .with_context(|_c| format!("could not create socket"))?;
    read_loop(&socket, args.print_src_addr, args.base64_enc)?;

    Ok(())
}

fn parse_ipv4_groups(groups_str: &[String]) -> Result<Vec<Ipv4Addr>, Error> {
    let mut grps: Vec<Ipv4Addr> = vec![];
    for addr in groups_str.iter() {
        let grp = Ipv4Addr::from_str(&addr)
            .with_context(|_c| format!("could not parse group address {}", addr))?;
        grps.push(grp);
    }
    Ok(grps)
}

fn ipv4_server_socket(args: &ListenV4Args) -> Result<Socket, Error> {
    let bindaddr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), args.port);
    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
    let addr = SockAddr::from(bindaddr);

    socket.set_reuse_address(true)?;
    socket.set_reuse_port(true)?;
    socket
        .bind(&addr)
        .with_context(|_c| format!("could not bind on {}", bindaddr))?;

    let groups: Vec<Ipv4Addr> = {
        let mut items = vec![args.group_ip];
        items.extend(args.additional_grp_ips.clone());
        items
    };
    for grp in groups {
        socket
            .join_multicast_v4(&grp, &args.nic)
            .with_context(|_c| format!("could not join {} on interface {}", grp, args.nic))?;
    }
    Ok(socket)
}

fn handle_send(args: SendV4Args) -> Result<(), Error> {
    mcast_v4_sendto(args.nic, SocketAddrV4::new(args.group_ip, args.port), args.hops)?;
    Ok(())
}

fn mcast_v4_sendto(nic: Ipv4Addr, group: SocketAddrV4, hop_count: u32) -> io::Result<()> {
    let snd_sock = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
    let bindaddr = SockAddr::from(SocketAddrV4::new(nic, 0));
    let dest = SockAddr::from(group);
    snd_sock.set_multicast_ttl_v4(hop_count)?;
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
