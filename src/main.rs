mod imu_data;
mod imu_reader;
mod open_track_data;

use anyhow::Result;
use clap::Parser;
use imu_reader::XrImuReader;
use open_track_data::OpenTrackData;
use std::{
    net::{Ipv4Addr, UdpSocket},
    path::PathBuf,
    thread,
    time::Duration,
};

/// Connector tool between xr_driver and OpenTrack
#[derive(Debug, Parser)]
#[command(version = "0.1")]
#[command(about, long_about = None)]
struct Args {
    /// IP on which OpenTrack listens
    #[arg(short, long)]
    #[arg(default_value = "127.0.0.1")]
    open_track_ip: Ipv4Addr,

    /// Port on which OpenTrack listens
    #[arg(short, long, default_value_t = 4242)]
    open_track_port: u16,

    /// Path to the shared memory file to read the imu data from
    #[arg(short, long)]
    #[arg(default_value = "/tmp/shader_runtime_imu_quat_data")]
    imu_shm_file: PathBuf,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.connect((args.open_track_ip, args.open_track_port))?;

    if args.debug {
        println!(
            "Connected udp socket to {:?}:{}",
            args.open_track_ip, args.open_track_port
        );
    }

    let mut framenumber = 0;
    let imu_reader = XrImuReader::new(args.imu_shm_file)?;

    if args.debug {
        println!("Reading from shared memory file ...");
    }

    loop {
        let imu_data = imu_reader.read_imu_data()?;
        let open_track_data = OpenTrackData::new(imu_data, framenumber);

        if args.debug {
            println!("{open_track_data:?}");
        }

        socket.send(&open_track_data.into_raw())?;

        framenumber += 1;

        thread::sleep(Duration::from_millis(500));
    }
}
