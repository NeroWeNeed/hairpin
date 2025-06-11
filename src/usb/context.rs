use std::{
    alloc::{alloc, Layout},
    ops::Deref,
    ptr::{addr_of_mut, null_mut},
    sync::Arc,
};

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    error::Error,
    libusb::{
        libusb_context, libusb_device, libusb_error_LIBUSB_SUCCESS, libusb_exit,
        libusb_hotplug_callback_handle, libusb_hotplug_deregister_callback, libusb_hotplug_event,
        libusb_hotplug_event_LIBUSB_HOTPLUG_EVENT_DEVICE_ARRIVED,
        libusb_hotplug_event_LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT,
        libusb_hotplug_flag_LIBUSB_HOTPLUG_ENUMERATE, libusb_hotplug_register_callback,
        libusb_init_context, libusb_set_pollfd_notifiers, LIBUSB_HOTPLUG_MATCH_ANY,
    },
};

use super::{device::UsbDevice, guard::Guard};
#[derive(Debug)]
pub struct UsbContext<T>(pub(crate) *mut libusb_context, pub(crate) T);

unsafe impl<T> Send for UsbContext<T> {}
unsafe impl<T> Sync for UsbContext<T> {}
impl<T> Deref for UsbContext<T> {
    type Target = *mut libusb_context;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
/// Reference to a [UsbContextHandle]. This version doesn't drop the pointer, and is used for
/// wrapping around passed references from callbacks
#[derive(Debug, Clone, Copy)]
pub struct UsbContextRef(pub(crate) *mut libusb_context);
unsafe impl Send for UsbContextRef {}
unsafe impl Sync for UsbContextRef {}
