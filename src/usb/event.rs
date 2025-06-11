use std::ops::BitOr;

use crate::{
    error::Error,
    libusb::{
        libusb_context, libusb_device, libusb_hotplug_event_LIBUSB_HOTPLUG_EVENT_DEVICE_ARRIVED,
        libusb_hotplug_event_LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT,
    },
};

use super::{
    context::{UsbContext, UsbContextRef},
    device::{UsbDevice, UsbDeviceRef},
};
pub type EventHandler<'a> = Box<
    dyn Fn(&UsbContext<UsbDevice>, UsbHotplugEventType) -> Result<(), Error> + 'a + Send + Sync,
>;

#[derive(Debug)]
pub(crate) struct UsbHotplugEventData(pub UsbContextRef, pub UsbDeviceRef, pub UsbHotplugEventType);
#[derive(Debug)]
pub enum UsbEvent {
    FileDescriptor(UsbFileDescriptorEventData),
    Hotplug(UsbHotplugEventData),
    Close,
}
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum UsbHotplugEventType {
    Undefined = 0,
    DeviceArrived = libusb_hotplug_event_LIBUSB_HOTPLUG_EVENT_DEVICE_ARRIVED,
    DeviceLeft = libusb_hotplug_event_LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT,
}
impl TryFrom<u32> for UsbHotplugEventType {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            #[allow(non_upper_case_globals)]
            libusb_hotplug_event_LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT => Ok(Self::DeviceLeft),
            #[allow(non_upper_case_globals)]
            libusb_hotplug_event_LIBUSB_HOTPLUG_EVENT_DEVICE_ARRIVED => Ok(Self::DeviceArrived),
            _ => Err(Error::UndefinedEvent(value)),
        }
    }
}
impl BitOr for UsbHotplugEventType {
    type Output = UsbHotplugEventMask;

    fn bitor(self, rhs: Self) -> Self::Output {
        UsbHotplugEventMask((self as u32) | (rhs as u32))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UsbHotplugEventMask(u32);
impl UsbHotplugEventMask {
    pub fn matches(&self, event: UsbHotplugEventType) -> bool {
        (self.0 & event as u32) != 0
    }
}
impl From<UsbHotplugEventType> for UsbHotplugEventMask {
    fn from(value: UsbHotplugEventType) -> Self {
        Self(value as u32)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UsbFileDescriptorEventData {
    Add(i32, i16),
    Remove(i32),
}
