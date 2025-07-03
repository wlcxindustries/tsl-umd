use std::net::{IpAddr, UdpSocket};

use clap::{Parser, ValueEnum};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum TslVersion {
    V3,
    V4,
    V5,
}
#[derive(Parser, Debug)]
#[command()]
struct Cli {
    #[arg(short, long, value_enum)]
    tsl_version: TslVersion,

    #[arg(short, long)]
    listen_addr: IpAddr,

    #[arg(short, long, default_value_t = 1234)]
    port: u16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    println!(
        "listening on {}:{} for tsl {:?} packets",
        args.listen_addr, args.port, args.tsl_version
    );

    let sock = UdpSocket::bind((args.listen_addr, args.port))?;
    loop {
        let mut buf = [0u8; 1024];
        let (count, remote) = sock.recv_from(&mut buf)?;
        println!("got {} bytes from {}", count, remote);
        println!("{:?}", &buf[0..count]);
        match args.tsl_version {
            TslVersion::V3 => {
                let packet = tsl_umd::v3_1::TSL31Packet::new_checked(&buf[0..count])?;
                println!("got packet {}", packet);
            }
            _ => unimplemented!(),
        }
    }
}
