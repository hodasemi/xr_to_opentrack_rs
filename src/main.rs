mod imu_data;
mod imu_reader;
mod open_track_data;

use anyhow::Result;
use imu_reader::XrImuReader;
use open_track_data::OpenTrackData;
use std::net::UdpSocket;

const OPEN_TRACK_IP: &'static str = "127.0.0.1";
const OPEN_TRACK_PORT: u16 = 4242;

const SHM_FILE: &'static str = "/tmp/shader_runtime_imu_quat_data";

fn main() -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.connect((OPEN_TRACK_IP, OPEN_TRACK_PORT))?;

    let mut framenumber = 0;
    let imu_reader = XrImuReader::new(SHM_FILE)?;

    loop {
        let imu_data = imu_reader.read_imu_data()?;
        let open_track_data = OpenTrackData::new(imu_data, framenumber);

        socket.send(&open_track_data.into_raw())?;

        framenumber += 1;
    }
}
