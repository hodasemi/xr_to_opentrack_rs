mod viture_hotplug;

use std::{
    sync::mpsc::{channel, Receiver, Sender},
    time::Duration,
};

use anyhow::{bail, Result};
use rusb::{Context, HotplugBuilder, Registration, UsbContext};
use viture_hotplug::VitureHotPlugHandler;

use crate::{euler::EulerData, viture::viture_rs::Viture};

#[derive(Debug, Clone, Copy)]
enum HotPlugEvent {
    Arrived,
    Left,
}

pub struct VitureUsbController {
    debug: bool,
    sender: Sender<EulerData>,

    receiver: Receiver<HotPlugEvent>,

    context: Context,
    reg: Option<Registration<Context>>,

    viture: Option<Viture>,
}

impl VitureUsbController {
    pub fn new(debug: bool, imu_sender: Sender<EulerData>) -> Result<Self> {
        if !rusb::has_hotplug() {
            bail!("libusb misses hotplug capabilities! (probably update needed)");
        }

        let (sender, receiver) = channel();
        let context = Context::new()?;

        let reg = Some(
            HotplugBuilder::new()
                .enumerate(true)
                .vendor_id(VitureHotPlugHandler::VITURE_ID_VENDOR)
                .register(&context, Box::new(VitureHotPlugHandler::new(sender, debug)))?,
        );

        Ok(Self {
            debug,
            sender: imu_sender,

            receiver,

            context,
            reg,

            viture: None,
        })
    }

    pub fn check(&mut self) -> Result<()> {
        if self.debug {
            println!("usb controller: check")
        }

        loop {
            self.context
                .handle_events(Some(Duration::from_millis(20)))?;

            if let Ok(hotplug_event) = self.receiver.recv_timeout(Duration::from_millis(20)) {
                if self.debug {
                    println!("usb controller: hotplug event received: {hotplug_event:?}");
                }

                match hotplug_event {
                    HotPlugEvent::Arrived => {
                        if self.viture.is_none() {
                            println!("Add Viture Device");

                            self.viture = Some(Viture::new({
                                let sender = self.sender.clone();

                                move |euler| {
                                    let _ = sender.send(euler);
                                }
                            })?);
                        }
                    }
                    HotPlugEvent::Left => {
                        if self.viture.is_some() {
                            println!("Remove Viture Device");
                            self.viture = None;
                        }
                    }
                }
            }
        }
    }
}

impl Drop for VitureUsbController {
    fn drop(&mut self) {
        self.context.unregister_callback(self.reg.take().unwrap());
    }
}
