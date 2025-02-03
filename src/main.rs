mod imu_data;
mod imu_reader;
mod open_track_data;
mod viture;
mod viture_sys;

use anyhow::Result;
use clap::Parser;
use open_track_data::OpenTrackData;
use std::{
    net::{Ipv4Addr, UdpSocket},
    path::PathBuf,
    sync::mpsc::{channel, Sender},
    thread,
    time::Duration,
};
use viture::{Euler, Viture};

/// Connector tool between xr_driver and OpenTrack
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

    /// Path to the shared memory file to read the imu data from
    #[arg(short = 's', long)]
    #[arg(default_value = "/tmp/shader_runtime_imu_quat_data")]
    imu_shm_file: PathBuf,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Starting program ...");

    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.connect((args.open_track_ip, args.open_track_port))?;

    println!("Created udp socket");

    if args.debug {
        println!(
            "Connected udp socket to {:?}:{}",
            args.open_track_ip, args.open_track_port
        );
    }

    let (sender, receiver) = channel();

    let mut _viture_sdk = Some(create_viture_sdk(sender.clone(), args.debug));
    let mut framenumber = 0;

    loop {
        match receiver.recv_timeout(Duration::from_secs(2)) {
            Ok(euler_data) => {
                let open_track_data = OpenTrackData::from_viture_sdk(euler_data, framenumber);

                if args.debug {
                    println!(
                        "yaw: {:.3}, pitch: {:.3}, roll: {:.3}",
                        open_track_data.yaw, open_track_data.pitch, open_track_data.roll
                    );
                }

                let _ = socket.send(&open_track_data.into_raw());

                framenumber += 1;
            }
            Err(_err) => {
                _viture_sdk = None;
                _viture_sdk = Some(create_viture_sdk(sender.clone(), args.debug));
            }
        }
    }
}

fn create_viture_sdk(sender: Sender<Euler>, enable_debug: bool) -> Viture {
    loop {
        if enable_debug {
            println!("Trying to initialize viture sdk ...");
        }

        match Viture::new({
            let sender = sender.clone();

            move |euler| {
                let _ = sender.send(euler);
            }
        }) {
            Ok(viture) => return viture,
            Err(err) => {
                println!("ERR: {err:?}");
            }
        }

        thread::sleep(Duration::from_secs(1));
    }
}
