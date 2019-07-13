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
    /// Listen on a particular network interface for datagrams from one or more multicast groups
    Listen(ListenArgs),

    #[structopt(name = "send")]
    /// Send datagrams to a multicast group via a particular network interface
    Send(SendArgs),
}

#[derive(Debug, StructOpt)]
struct ListenArgs {
    /// The network interface on which to send the join requests
    nic: String,
    /// The port to bind on
    port: String,
    /// A multicast group to join
    group_ip: String,
    /// Additional multicast groups to join
    additional_grp_ips: Vec<String>,

    #[structopt(name = "printsrc", long)]
    /// Print the incoming datagram's source address
    print_src_addr: bool,
    #[structopt(name = "base64", long)]
    /// Encode incoming datagrams in base64
    base64_enc: bool,
}

#[derive(Debug, StructOpt)]
struct SendArgs {
    /// The network interface on which to send the datagrams
    nic: String,
    /// The destination port to send to
    port: String,
    /// The multicast group to send to
    group_ip: String,
}

fn main() -> Result<(), ExitFailure> {
    let args = CommandArgs::from_args();

    match args {
        CommandArgs::Listen(l) => handle_listen(l),
        CommandArgs::Send(s) => handle_send(s),
    }?;
    Ok(())
}

fn handle_listen(args: ListenArgs) -> Result<(), Error> {
    let nic = Ipv4Addr::from_str(&args.nic)
        .with_context(|_c| format!("could not parse nic address {}", args.nic))?;
    let port = u16::from_str(&args.port)
        .with_context(|_c| format!("could not parse port number {}", args.port))?;
    let grps_as_strings: Vec<String> = {
        let mut items = vec![args.group_ip.clone()];
        items.extend(args.additional_grp_ips);
        items
    };
    let grps = parse_ipv4_groups(&grps_as_strings)?;
    let bind_sock_addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port);

    let socket = create_server_socket(bind_sock_addr, &grps, nic)
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

fn create_server_socket(
    bindaddr: SocketAddrV4,
    groups: &[Ipv4Addr],
    interface: Ipv4Addr,
) -> Result<Socket, Error> {

    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
    let addr = SockAddr::from(bindaddr);
    socket.set_reuse_address(true)?;
    socket.set_reuse_port(true)?;
    socket
        .bind(&addr)
        .with_context(|_c| format!("could not bind on {}", bindaddr))?;
    for grp in groups {
        socket
            .join_multicast_v4(grp, &interface)
            .with_context(|_c| format!("could not join {} on interface {}", grp, interface))?;
    }
    Ok(socket)
}

fn handle_send(args: SendArgs) -> Result<(), Error> {
    let port = u16::from_str(&args.port)
        .with_context(|_c| format!("could not parse port number {}", args.port))?;
    let grp = Ipv4Addr::from_str(&args.group_ip)
        .with_context(|_c| format!("could not parse group address {}", args.group_ip))?;
    let nic = Ipv4Addr::from_str(&args.nic)
        .with_context(|_c| format!("could not parse nic address {}", args.nic))?;
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
