use std::net::{IpAddr, UdpSocket};

use clap::{Parser, Subcommand, ValueEnum};
use tsl_umd::v3_1::{Brightness as PBrightness, PACKET_LENGTH_31, TSL31Packet};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum TslVersion {
    V3,
    V4,
    V5,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Brightness {
    Off,
    Seventh,
    Half,
    Full,
}

impl Into<PBrightness> for Brightness {
    fn into(self) -> PBrightness {
        match self {
            Self::Off => PBrightness::Zero,
            Self::Seventh => PBrightness::OneSeventh,
            Self::Half => PBrightness::OneHalf,
            Self::Full => PBrightness::Full,
        }
    }
}

#[derive(Parser, Debug)]
#[command()]
struct Cli {
    #[arg(short, long, value_enum)]
    tsl_version: TslVersion,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Listen {
        #[arg(short, long)]
        bind: IpAddr,

        #[arg(short, long, default_value_t = 1234)]
        port: u16,
    },
    Send {
        #[arg(short, long)]
        ip: IpAddr,

        #[arg(short, long, default_value_t = 1234)]
        port: u16,

        #[arg(short, long)]
        addr: u8,

        #[arg(short, long)]
        tally: Vec<u8>,

        #[arg(short, long, value_enum, default_value_t=Brightness::Full)]
        brightness: Brightness,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    match args.cmd {
        Commands::Listen { bind, port } => {
            println!(
                "listening on {}:{} for tsl {:?} packets",
                bind, port, args.tsl_version
            );

            let sock = UdpSocket::bind((bind, port))?;
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
        Commands::Send {
            ip,
            port,
            addr,
            tally,
            brightness,
        } => {
            let sock = UdpSocket::bind("0.0.0.0:0")?;
            let buf = [0u8; PACKET_LENGTH_31];
            let mut p = TSL31Packet::new_unchecked(buf);
            p.set_address(addr).unwrap();
            let state = [
                tally.contains(&1),
                tally.contains(&2),
                tally.contains(&3),
                tally.contains(&4),
            ];

            p.set_tally(state);
            p.set_brightness(brightness.into());
            sock.send_to(&p.inner(), (ip, port))?;
        }
    }
    Ok(())
}
