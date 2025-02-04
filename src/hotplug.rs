use std::{
    sync::mpsc::{channel, Receiver, Sender},
    time::Duration,
};

use anyhow::{bail, Result};
use rusb::{Context, Device, Hotplug, HotplugBuilder, Registration, UsbContext};

use crate::viture::{Euler, Viture};

#[derive(Debug, Clone, Copy)]
enum HotPlugEvent {
    Arrived,
    Left,
}

struct VitureHotPlugHandler {
    debug: bool,
    sender: Sender<HotPlugEvent>,
}

impl VitureHotPlugHandler {
    fn new(sender: Sender<HotPlugEvent>, debug: bool) -> Self {
        Self { debug, sender }
    }

    fn check_ids<T: UsbContext>(device: &Device<T>) -> bool {
        if let Ok(descriptor) = device.device_descriptor() {
            if descriptor.vendor_id() == Self::VITURE_ID_VENDOR {
                if Self::VITURE_ID_PRODUCT
                    .iter()
                    .any(|&product_id| product_id == descriptor.product_id())
                {
                    return true;
                }
            }
        }

        false
    }

    const VITURE_ID_VENDOR: u16 = 0x35ca;

    const VITURE_ID_PRODUCT: [u16; 7] = [
        0x1011, // One
        0x1013, // One
        0x1017, // One
        0x1015, // One Lite
        0x101b, // One Lite
        0x1019, // Pro
        0x101d, // Pro
    ];
}

impl<T: UsbContext> Hotplug<T> for VitureHotPlugHandler {
    fn device_arrived(&mut self, device: Device<T>) {
        if self.debug {
            println!(
                "hotplug event arrived received ({:04X}:{:04X})",
                device.device_descriptor().unwrap().vendor_id(),
                device.device_descriptor().unwrap().product_id(),
            );
        }

        if Self::check_ids(&device) {
            if self.debug {
                println!("hotplug event arrived sent to channel");
            }

            let _ = self.sender.send(HotPlugEvent::Arrived);
        }
    }

    fn device_left(&mut self, device: Device<T>) {
        if self.debug {
            println!("hotplug event left received");
        }

        if Self::check_ids(&device) {
            if self.debug {
                println!("hotplug event left sent to channel");
            }

            let _ = self.sender.send(HotPlugEvent::Left);
        }
    }
}

pub struct VitureUsbController {
    debug: bool,
    sender: Sender<Euler>,

    receiver: Receiver<HotPlugEvent>,

    context: Context,
    reg: Option<Registration<Context>>,

    viture: Option<Viture>,
}

impl VitureUsbController {
    pub fn new(debug: bool, imu_sender: Sender<Euler>) -> Result<Self> {
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
