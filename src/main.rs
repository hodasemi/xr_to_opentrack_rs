mod hotplug;
mod open_track_data;
mod viture;
mod viture_sys;

use anyhow::Result;
use clap::Parser;
use futures::future::try_join;
use hotplug::VitureUsbController;
use open_track_data::OpenTrackData;
use std::{
    net::{Ipv4Addr, UdpSocket},
    sync::mpsc::{channel, Receiver},
};
use viture::Euler;

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
}

#[async_std::main]
async fn main() -> Result<()> {
    let args = Args::parse();

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
    let mut viture_usb_controller = VitureUsbController::new(sender)?;

    try_join(
        send_to_opentrack(socket, receiver, args.debug),
        viture_usb_controller.check(),
    )
    .await?;

    Ok(())
}

async fn send_to_opentrack(
    socket: UdpSocket,
    receiver: Receiver<Euler>,
    debug: bool,
) -> Result<()> {
    let mut framenumber = 0;

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
