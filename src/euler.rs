use std::ops::Sub;

use crate::Command;

#[derive(Debug, Clone, Copy)]
pub struct EulerData {
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
}

impl EulerData {
    pub fn scale_roll(&mut self, scale: f32) {
        self.roll *= scale;
    }

    pub fn scale_pitch(&mut self, scale: f32) {
        self.pitch *= scale;
    }

    pub fn scale_yaw(&mut self, scale: f32) {
        self.yaw *= scale;
    }
}

impl Sub for EulerData {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            roll: self.roll - rhs.roll,
            pitch: self.pitch - rhs.pitch,
            yaw: self.yaw - rhs.yaw,
        }
    }
}

pub struct EulerHandler {
    debug: bool,

    reference: Option<EulerData>,

    roll_scale: f32,
    pitch_scale: f32,
    yaw_scale: f32,
}

impl EulerHandler {
    pub fn new(debug: bool) -> Self {
        Self {
            debug,

            reference: None,

            roll_scale: 1.0,
            pitch_scale: 1.0,
            yaw_scale: 1.0,
        }
    }

    pub fn apply_commands(&mut self, commands: Vec<Command>, euler: EulerData) {
        for command in commands {
            match command {
                Command::Recenter => {
                    self.reference = Some(euler);

                    if self.debug {
                        println!("new center: {:?}", self.reference);
                    }
                }
                Command::ScalePitch(f) => {
                    self.pitch_scale = f;

                    if self.debug {
                        println!("mew pitch scale: {:?}", self.pitch_scale);
                    }
                }
                Command::ScaleRoll(f) => {
                    self.roll_scale = f;

                    if self.debug {
                        println!("mew roll scale: {:?}", self.roll_scale);
                    }
                }
                Command::ScaleYaw(f) => {
                    self.yaw_scale = f;

                    if self.debug {
                        println!("mew yaw scale: {:?}", self.yaw_scale);
                    }
                }
            }
        }
    }

    pub fn apply_config(&self, mut euler: EulerData) -> EulerData {
        if let Some(reference) = self.reference {
            euler = euler - reference;
        }

        euler.scale_pitch(self.pitch_scale);
        euler.scale_roll(self.roll_scale);
        euler.scale_yaw(self.yaw_scale);

        euler
    }
}
