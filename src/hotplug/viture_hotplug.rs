use std::sync::mpsc::Sender;

use super::HotPlugEvent;
use rusb::{Device, Hotplug, UsbContext};

pub struct VitureHotPlugHandler {
    debug: bool,
    sender: Sender<HotPlugEvent>,
}

impl VitureHotPlugHandler {
    pub fn new(sender: Sender<HotPlugEvent>, debug: bool) -> Self {
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

    pub const VITURE_ID_VENDOR: u16 = 0x35ca;

    pub const VITURE_ID_PRODUCT: [u16; 7] = [
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
