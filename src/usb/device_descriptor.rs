use std::alloc::Layout;

use crate::{
    error::Error,
    libusb::{libusb_device_descriptor, libusb_get_device_descriptor},
};

use super::device::{UsbDevice, UsbDeviceRef};

#[derive(Debug)]
pub struct UsbDeviceDescriptor(pub(crate) *mut libusb_device_descriptor);
impl UsbDeviceDescriptor {
    pub fn usb(&self) -> u16 {
        unsafe { (*self.0).bcdUSB }
    }
    pub fn device_class(&self) -> u8 {
        unsafe { (*self.0).bDeviceClass }
    }
    pub fn device_sub_class(&self) -> u8 {
        unsafe { (*self.0).bDeviceSubClass }
    }
    pub fn device_protocol(&self) -> u8 {
        unsafe { (*self.0).bDeviceProtocol }
    }
    pub fn max_packet_size_0(&self) -> u8 {
        unsafe { (*self.0).bMaxPacketSize0 }
    }
    pub fn vendor_id(&self) -> u16 {
        unsafe { (*self.0).idVendor }
    }
    pub fn product_id(&self) -> u16 {
        unsafe { (*self.0).idProduct }
    }
    pub fn device(&self) -> u16 {
        unsafe { (*self.0).bcdDevice }
    }
    pub fn manufacturer_index(&self) -> u8 {
        unsafe { (*self.0).iManufacturer }
    }
    pub fn product_index(&self) -> u8 {
        unsafe { (*self.0).iProduct }
    }
    pub fn serial_number_index(&self) -> u8 {
        unsafe { (*self.0).iSerialNumber }
    }
    pub fn number_of_configurations(&self) -> u8 {
        unsafe { (*self.0).bNumConfigurations }
    }
    pub fn descriptor_type(&self) -> u8 {
        unsafe { (*self.0).bDescriptorType }
    }
}
impl Drop for UsbDeviceDescriptor {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.0 as *mut u8, Layout::new::<libusb_device_descriptor>());
        }
    }
}
