mod euler;
mod ftok_ipc;
mod hotplug;
mod open_track_data;
mod viture;

use crate::euler::EulerData;
use anyhow::Result;
use clap::Parser;
use euler::EulerHandler;
use hotplug::VitureUsbController;
use open_track_data::OpenTrackData;
use ring_channel::ring_channel;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::{
    io::{Read, Write},
    net::{Ipv4Addr, TcpListener, TcpStream, UdpSocket},
    num::NonZeroUsize,
    sync::{
        mpsc::{channel, Receiver},
        Arc, Mutex,
    },
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

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Recenters to current position
    #[arg(long)]
    center: bool,

    /// Scale yaw output
    #[arg(long = "sy")]
    scale_yaw: Option<f32>,

    /// Scale pitch output
    #[arg(long = "sp")]
    scale_pitch: Option<f32>,

    /// Scale roll output
    #[arg(long = "sr")]
    scale_roll: Option<f32>,

    /// Invert yaw output
    #[arg(long = "iy")]
    invert_yaw: Option<bool>,

    /// Invert pitch output
    #[arg(long = "ip")]
    invert_pitch: Option<bool>,

    /// Invert roll output
    #[arg(long = "ir")]
    invert_roll: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
enum Command {
    Recenter,

    ScaleYaw(f32),
    ScalePitch(f32),
    ScaleRoll(f32),

    InvertYaw(bool),
    InvertPitch(bool),
    InvertRoll(bool),
}

const TCP_SOCKET: u16 = 4244;

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(commands) = check_cli_commands(&args) {
        let mut client = TcpStream::connect(("127.0.0.1", TCP_SOCKET))?;

        for command in commands {
            client.write_all(to_string(&command)?.as_bytes())?;
            client.write(";".as_bytes())?;
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
    send_to_opentrack(socket, receiver, args.debug, args.verbose)?;

    Ok(())
}

fn send_to_opentrack(
    socket: UdpSocket,
    receiver: Receiver<EulerData>,
    debug: bool,
    verbose: bool,
) -> Result<()> {
    if debug {
        println!("send to opentrack: start");
    }

    let mut framenumber = 0;
    let euler_handler: Arc<Mutex<EulerHandler>> = Arc::new(Mutex::new(EulerHandler::new(debug)));

    let mut server = TcpListener::bind(("127.0.0.1", TCP_SOCKET))?;
    let (euler_sender, euler_receiver) = ring_channel(NonZeroUsize::new(1).unwrap());

    thread::spawn({
        let euler_handler = euler_handler.clone();

        move || loop {
            match check_tcp_command(&mut server, debug) {
                Ok(commands) => {
                    if let Some(commands) = commands {
                        let last_euler = euler_receiver.try_recv().ok();

                        if debug {
                            println!("received command: {commands:#?}");
                        }

                        euler_handler
                            .lock()
                            .unwrap()
                            .apply_commands(commands, last_euler);
                    }
                }
                Err(err) => {
                    if debug {
                        println!("tcp error: {err:?}");
                    }
                }
            }
        }
    });

    loop {
        let mut euler_data = receiver.recv()?;

        euler_sender.send(euler_data)?;
        euler_data = euler_handler.lock().unwrap().apply_config(euler_data);

        let open_track_data = OpenTrackData::from_viture_sdk(euler_data, framenumber);

        if debug && verbose {
            println!(
                "yaw: {:.3}, pitch: {:.3}, roll: {:.3}",
                open_track_data.yaw, open_track_data.pitch, open_track_data.roll
            );
        }

        let _ = socket.send(&open_track_data.into_raw());

        framenumber += 1;
    }
}

fn check_tcp_command(server: &mut TcpListener, debug: bool) -> Result<Option<Vec<Command>>> {
    let mut commands = Vec::new();

    let (mut stream, _) = server.accept()?;

    if debug {
        println!("incoming stream")
    }

    loop {
        let mut buf = String::new();
        let len = stream.read_to_string(&mut buf)?;

        if len == 0 {
            if debug {
                println!("received empty message");
            }

            break;
        }

        if debug {
            println!("received message: {buf}");
        }

        for split in buf.split(";") {
            if let Ok(cmd) = from_str(split) {
                commands.push(cmd);
            }
        }
    }

    if debug {
        println!("received commands: {commands:#?}");
    }

    Ok(if commands.is_empty() {
        None
    } else {
        Some(commands)
    })
}

fn check_cli_commands(args: &Args) -> Option<Vec<Command>> {
    let mut commands = Vec::new();

    if args.center {
        commands.push(Command::Recenter);
    }

    if let Some(f) = args.scale_pitch {
        commands.push(Command::ScalePitch(f));
    }

    if let Some(f) = args.scale_roll {
        commands.push(Command::ScaleRoll(f));
    }

    if let Some(f) = args.scale_yaw {
        commands.push(Command::ScaleYaw(f));
    }

    if let Some(i) = args.invert_pitch {
        commands.push(Command::InvertPitch(i));
    }

    if let Some(i) = args.invert_roll {
        commands.push(Command::InvertRoll(i));
    }

    if let Some(i) = args.invert_yaw {
        commands.push(Command::InvertYaw(i));
    }

    if args.debug {
        println!("{commands:#?}");
    }

    if commands.is_empty() {
        None
    } else {
        Some(commands)
    }
}
