use std::{
    alloc::Layout,
    fmt::{Debug, Write},
    ops::Deref,
};

use crate::{
    error::Error,
    libusb::{
        libusb_close, libusb_device, libusb_device_descriptor, libusb_device_handle,
        libusb_get_device, libusb_get_device_address, libusb_get_device_descriptor,
        libusb_get_port_numbers, libusb_get_port_path, libusb_open,
    },
};

use super::{context::UsbContext, device_descriptor::UsbDeviceDescriptor};

#[derive(Debug)]
pub struct UsbDevice(pub(crate) *mut libusb_device);
impl From<*mut libusb_device> for UsbDevice {
    fn from(value: *mut libusb_device) -> Self {
        Self(value)
    }
}
impl UsbContext<UsbDevice> {
    pub fn device(&self) -> &UsbDevice {
        &self.1
    }
}
impl UsbDevice {
    pub fn addr(&self) -> u8 {
        unsafe { libusb_get_device_address(self.0) }
    }
    pub fn descriptor(&self) -> Result<UsbDeviceDescriptor, Error> {
        unsafe {
            let device_descriptor = std::alloc::alloc(Layout::new::<libusb_device_descriptor>())
                as *mut libusb_device_descriptor;

            let err = libusb_get_device_descriptor(self.0, device_descriptor);
            if err != 0 {
                Err(Error::DeviceDescriptor(err))
            } else {
                Ok(UsbDeviceDescriptor(device_descriptor))
            }
        }
    }
    pub fn open(&self) -> Result<UsbDeviceHandle, Error> {
        unsafe {
            let mut device_handle =
                std::ptr::null_mut::<libusb_device_handle>() as *mut libusb_device_handle;
            let err = libusb_open(self.0, std::ptr::addr_of_mut!(device_handle));
            if err != 0 {
                Err(Error::DeviceOpen(err))
            } else {
                Ok(UsbDeviceHandle(device_handle))
            }
        }
    }
    pub fn port_numbers<'a>(&self) -> PortNumberSet {
        let mut port_numbers: [u8; 8] = [0; 8];
        unsafe {
            let ptr = port_numbers.as_mut_ptr();
            let write_loc = ptr.offset(1);
            port_numbers[0] = libusb_get_port_numbers(self.0, write_loc, 7) as u8;
        };

        PortNumberSet(port_numbers)
    }
}
#[derive(Clone, Copy)]
pub struct PortNumberSet([u8; 8]);
impl PortNumberSet {
    pub fn len(&self) -> usize {
        self.0[0] as usize
    }
}
impl Debug for PortNumberSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('[')?;
        let mut first = true;
        for port in self.into_iter() {
            if !first {
                f.write_str(", ")?;
            }
            first = false;
            port.fmt(f)?;
        }
        f.write_char(']')
    }
}
impl IntoIterator for PortNumberSet {
    type Item = u8;

    type IntoIter = PortNumberIter;

    fn into_iter(self) -> Self::IntoIter {
        PortNumberIter(self, 0)
    }
}
pub struct PortNumberIter(PortNumberSet, usize);
impl Iterator for PortNumberIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 < self.0.len() {
            let current = self.0 .0[(self.1 + 1) as usize];
            self.1 += 1;
            Some(current)
        } else {
            None
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct UsbDeviceRef(pub(crate) *mut libusb_device);
unsafe impl Send for UsbDeviceRef {}
unsafe impl Sync for UsbDeviceRef {}

#[derive(Debug)]
pub struct UsbDeviceHandle(*mut libusb_device_handle);

impl UsbDeviceHandle {
    pub fn device(&self) -> UsbDevice {
        let device = unsafe { libusb_get_device(self.0) };
        UsbDevice(device)
    }
}
impl Deref for UsbDeviceHandle {
    type Target = *mut libusb_device_handle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Drop for UsbDeviceHandle {
    fn drop(&mut self) {
        unsafe {
            libusb_close(self.0);
        }
    }
}
