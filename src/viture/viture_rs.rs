use anyhow::{bail, Result};
use std::{slice, sync::Mutex};

use super::viture_sys::*;
use crate::euler::EulerData;

static IMU_CALLBACK: Mutex<Option<Box<dyn FnMut(EulerData) -> () + Send + Sync + 'static>>> =
    Mutex::new(None);

#[derive(Clone)]
pub struct Viture {}

impl Viture {
    pub fn new(callback: impl FnMut(EulerData) -> () + Send + Sync + 'static) -> Result<Self> {
        *IMU_CALLBACK.lock().unwrap() = Some(Box::new(callback));

        let init = unsafe { init(Some(Self::imu_callback), None) };

        if !init {
            bail!("failed to initialize viture sdk");
        }

        let err = unsafe { set_imu(true) };

        match err {
            VitureResult::ERR_SUCCESS => (),

            _ => bail!("failed to enable imu"),
        }

        Ok(Self {})
    }

    extern "C" fn imu_callback(data: *mut u8, len: u16, _ts: u32) {
        if len < 12 {
            return;
        }

        if let Some(imu_callback) = &mut *IMU_CALLBACK.lock().unwrap() {
            let raw = unsafe { slice::from_raw_parts(data, 12) };

            imu_callback(EulerData {
                roll: f32::from_be_bytes(raw[0..4].try_into().unwrap()),
                pitch: f32::from_be_bytes(raw[4..8].try_into().unwrap()),
                yaw: f32::from_be_bytes(raw[8..12].try_into().unwrap()),
            })
        }
    }
}

impl Drop for Viture {
    fn drop(&mut self) {
        unsafe { deinit() }
    }
}
