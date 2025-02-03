#![allow(unused)]

pub type CallbackIMU = extern "C" fn(data: *mut u8, len: u16, ts: u32);
pub type CallbackMCU = extern "C" fn(msgid: u16, data: *mut u8, len: u16, ts: u32);

#[repr(i32)]
#[allow(non_camel_case_types)]
pub enum VitureResult {
    ERR_TIMEOUT = -3,
    ERR_RSP_ERROR = -2,
    ERR_WRITE_FAIL = -1,

    ERR_SUCCESS = 0,
    ERR_FAILURE = 1,
    ERR_INVALID_ARGUMENT = 2,
    ERR_NOT_ENOUGH_MEMORY = 3,
    ERR_UNSUPPORTED_CMD = 4,
    ERR_CRC_MISMATCH = 5,
    ERR_VER_MISMATCH = 6,
    ERR_MSG_ID_MISMATCH = 7,
    ERR_MSG_STX_MISMATCH = 8,
    ERR_CODE_NOT_WRITTEN = 9,
}

#[repr(i32)]
pub enum VitureState {
    Off = 0,
    On = 1,
}

#[repr(i32)]
pub enum VitureImuFrequency {
    Frequency60 = 0,
    Frequency90 = 1,
    Frequency120 = 2,
    Frequency240 = 3,
}

#[link(name = "viture_one_sdk_static", kind = "static")]
unsafe extern "C" {
    pub unsafe fn init(
        imu_callback: Option<CallbackIMU>,
        mcu_callback: Option<CallbackMCU>,
    ) -> bool;
    pub unsafe fn deinit();

    pub unsafe fn set_imu(on_off: bool) -> VitureResult;
    pub unsafe fn get_imu_state() -> VitureState;

    pub unsafe fn set_3d(on_off: bool) -> VitureResult;
    pub unsafe fn get_3d_state() -> VitureState;

    pub unsafe fn set_imu_fq(frequency: VitureImuFrequency) -> VitureResult;
    pub unsafe fn get_imu_fq() -> VitureImuFrequency;

    pub unsafe fn open_log(value: i32) -> i32;
}
