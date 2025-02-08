#![allow(unused)]

use std::{ffi::CString, marker::PhantomData, ptr};

use anyhow::{bail, Result};
use libc::*;

pub struct FtokIPC<T: Default, const N: usize> {
    mmap_type: PhantomData<[T; N]>,
    addr: *mut c_void,
}

impl<T: Default + Copy, const N: usize> FtokIPC<T, N> {
    pub fn new(path: &str) -> Result<Self> {
        let cpath = CString::new(path)?;

        let key = unsafe { ftok(cpath.as_ptr(), 0) };

        if key == -1 {
            bail!("failed to get ftok key");
        }

        let shmid = unsafe { shmget(key, size_of::<T>() * N, 0) };

        if shmid == -1 {
            bail!("failed to get shmid");
        }

        let addr = unsafe { shmat(shmid, ptr::null(), 0) };

        if addr == ptr::null_mut() {
            bail!("failed to attach to shared memory");
        }

        Ok(Self {
            mmap_type: PhantomData,
            addr,
        })
    }

    pub fn read(&mut self) -> [T; N] {
        let mut buffer = [T::default(); N];

        unsafe {
            memcpy(
                buffer.as_mut_ptr() as *mut c_void,
                self.addr,
                size_of::<T>() * N,
            )
        };

        buffer
    }
}

impl<T: Default, const N: usize> Drop for FtokIPC<T, N> {
    fn drop(&mut self) {
        let res = unsafe { shmdt(self.addr) };

        if res == -1 {
            panic!("failed to detach from shared memory");
        }
    }
}

#[cfg(test)]
mod test {
    use std::{f32::consts::PI, net::UdpSocket, thread, time::Duration};

    use crate::{euler::EulerData, open_track_data::OpenTrackData};

    use super::*;
    use anyhow::Result;

    use nalgebra::{Quaternion, UnitQuaternion};

    pub fn raw_quaternion_to_euler(w: f32, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        let unit_quat = UnitQuaternion::from_quaternion(Quaternion::new(w, x, y, z));
        unit_quat.euler_angles()
    }

    pub fn radian_to_degree(r: f32) -> f32 {
        r * 180.0 / PI
    }

    #[test]
    fn test_ipc() -> Result<()> {
        let path = "/tmp/shader_runtime_imu_quat_data";

        let mut ipc = FtokIPC::<f32, 4>::new(path)?;

        let socket = UdpSocket::bind("127.0.0.1:0")?;
        socket.connect("127.0.0.1:4242")?;

        let mut framenumber = 0;

        loop {
            let b = ipc.read();

            let (roll, pitch, yaw) = raw_quaternion_to_euler(b[3], b[0], b[1], b[2]);

            let ot_data = OpenTrackData::from_viture_sdk(
                EulerData {
                    roll: radian_to_degree(roll),
                    pitch: radian_to_degree(pitch),
                    yaw: radian_to_degree(yaw),
                },
                framenumber,
            );

            let _ = socket.send(&ot_data.into_raw());

            framenumber += 1;

            thread::sleep(Duration::from_millis(16));
        }
    }
}
