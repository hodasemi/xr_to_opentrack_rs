use std::{
    sync::mpsc::{channel, Receiver, Sender},
    time::Duration,
};

use anyhow::{bail, Result};
use rusb::{Context, Device, Hotplug, HotplugBuilder, Registration, UsbContext};

use crate::viture::{Euler, Viture};

enum HotPlugEvent {
    Arrived,
    Left,
}

struct VitureHotPlugHandler {
    sender: Sender<HotPlugEvent>,
}

impl VitureHotPlugHandler {
    fn new(sender: Sender<HotPlugEvent>) -> Self {
        Self { sender }
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
        if Self::check_ids(&device) {
            let _ = self.sender.send(HotPlugEvent::Arrived);
        }
    }

    fn device_left(&mut self, device: Device<T>) {
        if Self::check_ids(&device) {
            let _ = self.sender.send(HotPlugEvent::Left);
        }
    }
}

pub struct VitureUsbController {
    sender: Sender<Euler>,

    receiver: Receiver<HotPlugEvent>,

    context: Context,
    reg: Option<Registration<Context>>,

    viture: Option<Viture>,
}

impl VitureUsbController {
    pub fn new(imu_sender: Sender<Euler>) -> Result<Self> {
        if !rusb::has_hotplug() {
            bail!("libusb misses hotplug capabilities! (probably update needed)");
        }

        let (sender, receiver) = channel();
        let context = Context::new()?;

        let reg = Some(
            HotplugBuilder::new()
                .enumerate(true)
                .register(&context, Box::new(VitureHotPlugHandler::new(sender)))?,
        );

        let viture = context
            .devices()?
            .iter()
            .find(|device| VitureHotPlugHandler::check_ids(device))
            .map(|_| {
                Viture::new({
                    let sender = imu_sender.clone();

                    move |euler| {
                        let _ = sender.send(euler);
                    }
                })
            })
            .transpose()?;

        Ok(Self {
            sender: imu_sender,

            receiver,

            context,
            reg,

            viture,
        })
    }

    pub async fn check(&mut self) -> Result<()> {
        loop {
            self.context.handle_events(None).unwrap();

            if let Ok(hotplug_event) = self.receiver.recv_timeout(Duration::from_millis(250)) {
                match hotplug_event {
                    HotPlugEvent::Arrived => {
                        if self.viture.is_none() {
                            self.viture = Some(Viture::new({
                                let sender = self.sender.clone();

                                move |euler| {
                                    let _ = sender.send(euler);
                                }
                            })?);
                        }
                    }
                    HotPlugEvent::Left => self.viture = None,
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

// fn main() -> rusb::Result<()> {
//     if rusb::has_hotplug() {
//         let context = Context::new()?;

//         let mut reg = Some(
//             HotplugBuilder::new()
//                 .enumerate(true)
//                 .register(&context, Box::new(HotPlugHandler {}))?,
//         );

//         loop {
//             context.handle_events(None).unwrap();
//             if let Some(reg) = reg.take() {
//                 context.unregister_callback(reg);
//                 break;
//             }
//         }
//         Ok(())
//     } else {
//         eprint!("libusb hotplug api unsupported");
//         Ok(())
//     }
// }
