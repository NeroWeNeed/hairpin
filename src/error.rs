use tokio::task::JoinError;

use crate::libusb::libusb_device;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("Error opening device: {0}")]
    DeviceOpen(i32),

    #[error("Error registering callback: {0}")]
    RegisterCallback(i32),
    #[error("Error getting device descriptor: {0}")]
    DeviceDescriptor(i32),
    #[error("Undefined event: {0}")]
    UndefinedEvent(u32),
}
