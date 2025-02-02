use std::mem::transmute;

use nalgebra::{Quaternion, UnitQuaternion};

use crate::{imu_data::ImuData, viture::Euler};

#[repr(C)]
#[derive(Debug)]
pub struct OpenTrackData {
    pub x: f64,
    pub y: f64,
    pub z: f64,

    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,

    pub frame_number: u32,

    _padding: u32,
}

impl OpenTrackData {
    pub fn from_shm_data(imu_data: ImuData, frame_number: u32) -> Self {
        let (roll, pitch, yaw) =
            Self::raw_quaternion_to_euler(imu_data.w, imu_data.x, imu_data.y, imu_data.z);

        Self {
            x: imu_data.x as f64 * 10.0,
            y: imu_data.y as f64 * 10.0,
            z: imu_data.z as f64 * 10.0,

            yaw: yaw as f64,
            pitch: pitch as f64,
            roll: roll as f64,

            frame_number,

            _padding: 0,
        }
    }

    pub fn from_viture_sdk(euler: Euler, frame_number: u32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,

            yaw: euler.yaw as f64,
            pitch: euler.pitch as f64,
            roll: euler.roll as f64,

            frame_number,

            _padding: 0,
        }
    }

    fn raw_quaternion_to_euler(w: f32, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        let unit_quat = UnitQuaternion::from_quaternion(Quaternion::new(w, x, y, z));
        unit_quat.euler_angles()
    }

    pub fn into_raw(self) -> [u8; 52] {
        let tmp: [u8; 56] = unsafe { transmute(self) };
        tmp[..52].try_into().unwrap()
    }
}
