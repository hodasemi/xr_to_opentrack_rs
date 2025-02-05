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

    pub fn invert_roll(&mut self) {
        self.roll = -self.roll;
    }

    pub fn invert_pitch(&mut self) {
        self.pitch = -self.pitch;
    }

    pub fn invert_yaw(&mut self) {
        self.yaw = -self.yaw;
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

    roll_invert: bool,
    pitch_invert: bool,
    yaw_invert: bool,
}

impl EulerHandler {
    pub fn new(debug: bool) -> Self {
        Self {
            debug,

            reference: None,

            roll_scale: 1.0,
            pitch_scale: 1.0,
            yaw_scale: 1.0,

            roll_invert: false,
            pitch_invert: false,
            yaw_invert: false,
        }
    }

    pub fn apply_commands(&mut self, commands: Vec<Command>, euler: Option<EulerData>) {
        if self.debug {
            println!("apply command: {commands:#?}");
        }

        for command in commands {
            match command {
                Command::Recenter => {
                    self.reference = euler;

                    if self.debug {
                        println!("new center: {:?}", self.reference);
                    }
                }

                Command::ScalePitch(f) => {
                    self.pitch_scale = f;

                    if self.debug {
                        println!("new pitch scale: {:?}", self.pitch_scale);
                    }
                }
                Command::ScaleRoll(f) => {
                    self.roll_scale = f;

                    if self.debug {
                        println!("new roll scale: {:?}", self.roll_scale);
                    }
                }
                Command::ScaleYaw(f) => {
                    self.yaw_scale = f;

                    if self.debug {
                        println!("new yaw scale: {:?}", self.yaw_scale);
                    }
                }

                Command::InvertPitch(i) => {
                    self.pitch_invert = i;

                    if self.debug {
                        println!("new pitch invert: {}", self.pitch_invert);
                    }
                }
                Command::InvertRoll(i) => {
                    self.roll_invert = i;

                    if self.debug {
                        println!("new roll invert: {}", self.roll_invert);
                    }
                }
                Command::InvertYaw(i) => {
                    self.yaw_invert = i;

                    if self.debug {
                        println!("new yaw invert: {}", self.yaw_invert);
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

        if self.pitch_invert {
            euler.invert_pitch();
        }

        if self.roll_invert {
            euler.invert_roll();
        }

        if self.yaw_invert {
            euler.invert_yaw();
        }

        euler
    }
}
