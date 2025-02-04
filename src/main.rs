mod euler;
mod hotplug;
mod open_track_data;
mod viture;

use crate::euler::Euler;
use anyhow::Result;
use clap::Parser;
use hotplug::VitureUsbController;
use open_track_data::OpenTrackData;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::{
    io::{Read, Write},
    net::{Ipv4Addr, TcpListener, TcpStream, UdpSocket},
    sync::mpsc::{channel, Receiver},
    thread,
};

/// Tool to provide viture imu data to OpenTrack
#[derive(Debug, Parser)]
#[command(version = "0.1")]
#[command(about, long_about = None)]
struct Args {
    /// IP on which OpenTrack listens
    #[arg(short = 'i', long)]
    #[arg(default_value = "127.0.0.1")]
    open_track_ip: Ipv4Addr,

    /// Port on which OpenTrack listens
    #[arg(short = 'p', long, default_value_t = 4242)]
    open_track_port: u16,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Recenters to current position
    #[arg(short, long)]
    center: bool,
}

#[derive(Debug, Serialize, Deserialize)]
enum Command {
    Recenter,
}

const TCP_SOCKET: u16 = 4244;

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(commands) = check_cli_commands(&args) {
        let mut client = TcpStream::connect(("127.0.0.1", TCP_SOCKET))?;

        for command in commands {
            client.write_all(to_string(&command)?.as_bytes())?;
        }

        return Ok(());
    }

    if args.debug {
        println!("Starting program ...");
    }

    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.connect((args.open_track_ip, args.open_track_port))?;

    if args.debug {
        println!("Created udp socket");

        println!(
            "Connected udp socket to {:?}:{}",
            args.open_track_ip, args.open_track_port
        );
    }

    let (sender, receiver) = channel();
    let mut viture_usb_controller = VitureUsbController::new(args.debug, sender)?;

    if args.debug {
        println!("created everything: start loops");
    }

    thread::spawn(move || viture_usb_controller.check());
    send_to_opentrack(socket, receiver, args.debug)?;

    Ok(())
}

fn send_to_opentrack(socket: UdpSocket, receiver: Receiver<Euler>, debug: bool) -> Result<()> {
    if debug {
        println!("send to opentrack: start");
    }

    let mut server = TcpListener::bind(("127.0.0.1", TCP_SOCKET))?;

    let mut framenumber = 0;
    let mut reference_euler = None;

    loop {
        let euler_data = receiver.recv()?;
        let open_track_data = OpenTrackData::from_viture_sdk(euler_data, framenumber);

        if debug {
            println!(
                "yaw: {:.3}, pitch: {:.3}, roll: {:.3}",
                open_track_data.yaw, open_track_data.pitch, open_track_data.roll
            );
        }

        let _ = socket.send(&open_track_data.into_raw());

        framenumber += 1;
    }
}

fn check_tcp_command(server: &mut TcpListener) -> Option<Vec<Command>> {
    let mut commands = server
        .incoming()
        .map(|stream_res| {
            stream_res.map(|stream| {
                let tmp: Vec<Command> = Vec::new();
                let buf = String::new();

                loop {
                    let len = stream.read_to_string(&mut buf)?;

                    if len == 0 {
                        break;
                    }

                    tmp.push(from_str(&buf)?);
                }

                Ok(tmp)
            })
        })
        .collect::<Result<Result<Vec<Command>>>>();

    if commands.is_empty() {
        None
    } else {
        Some(commands)
    }
}

fn check_cli_commands(args: &Args) -> Option<Vec<Command>> {
    let mut commands = Vec::new();

    if args.center {
        commands.push(Command::Recenter);
    }

    if commands.is_empty() {
        None
    } else {
        Some(commands)
    }
}
