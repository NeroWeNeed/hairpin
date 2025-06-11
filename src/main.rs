#![allow(unused)]
use std::{
    alloc::Layout,
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::BTreeMap,
    fmt::Debug,
    ptr::{addr_of_mut, null, null_mut},
    rc::Rc,
    sync::{Arc, RwLock},
    time::Duration,
};

use error::Error;
use libc::{nfds_t, poll, pollfd};
use libusb::{
    libusb_context, libusb_device, libusb_error_LIBUSB_SUCCESS, libusb_event_handler_active,
    libusb_event_handling_ok, libusb_exit, libusb_get_next_timeout, libusb_get_pollfds,
    libusb_handle_events_timeout, libusb_hotplug_callback_handle,
    libusb_hotplug_deregister_callback, libusb_hotplug_event,
    libusb_hotplug_event_LIBUSB_HOTPLUG_EVENT_DEVICE_ARRIVED,
    libusb_hotplug_event_LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT,
    libusb_hotplug_flag_LIBUSB_HOTPLUG_ENUMERATE, libusb_hotplug_register_callback,
    libusb_init_context, libusb_lock_event_waiters, libusb_lock_events, libusb_pollfd,
    libusb_pollfds_handle_timeouts, libusb_set_pollfd_notifiers, libusb_try_lock_events,
    libusb_unlock_event_waiters, libusb_unlock_events, libusb_wait_for_event, timeval,
    LIBUSB_HOTPLUG_MATCH_ANY,
};

use tokio::{
    sync::Mutex,
    task::{JoinError, JoinHandle, JoinSet},
};
use tokio_util::sync::CancellationToken;
use usb::{event::UsbHotplugEventType, guard::Guard, listener::UsbListener};

pub(crate) mod libusb {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
mod error;
mod usb;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = UsbListener::builder();
    builder.on_event(
        UsbHotplugEventType::DeviceArrived,
        Box::new(|source, event| {
            let device = source.device();
            println!("Device Arrived");
            let descriptor = device.descriptor()?;

            let address = device.addr();
            println!("Addr: {:?}", address);
            println!(
                "Vendor Id: {}, Product Id: {}, Protocol: {}, Class: {}, SubClass: {}, Ports: {:?}",
                &descriptor.vendor_id(),
                &descriptor.product_id(),
                &descriptor.device_protocol(),
                &descriptor.device_class(),
                &descriptor.device_sub_class(),
                device.port_numbers()
            );
            Ok(())
        }),
    );
    builder.on_event(
        UsbHotplugEventType::DeviceLeft,
        Box::new(|_, _| {
            println!("Device: Removed");
            Ok(())
        }),
    );
    println!("Running...");
    builder.serve().await?;
    println!("Shutting Down...");

    Ok(())
}
