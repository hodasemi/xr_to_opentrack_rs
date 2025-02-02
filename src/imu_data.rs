use std::mem::transmute;

#[repr(C)]
pub struct ImuData {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,

    pub stage_1_quat_x: f32,
    pub stage_1_quat_y: f32,
    pub stage_1_quat_z: f32,
    pub stage_1_quat_w: f32,

    pub stage_2_quat_x: f32,
    pub stage_2_quat_y: f32,
    pub stage_2_quat_z: f32,
    pub stage_2_quat_w: f32,

    pub time_stamp_ms: f32,

    pub stage_1_ts: f32,
    pub stage_2_ts: f32,
}

impl From<[u8; size_of::<Self>()]> for ImuData {
    fn from(value: [u8; size_of::<Self>()]) -> Self {
        unsafe { transmute(value) }
    }
}
