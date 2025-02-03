use std::mem::transmute;

use crate::viture::Euler;

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

    pub fn into_raw(self) -> [u8; 52] {
        let tmp: [u8; 56] = unsafe { transmute(self) };
        tmp[..52].try_into().unwrap()
    }
}
