use std::{
    alloc::Layout,
    borrow::Cow,
    collections::BTreeMap,
    ffi::{CStr, OsStr},
    i16,
    os::unix::ffi::OsStrExt,
    path::Path,
    ptr::null,
    time::Duration,
};

use libc::{c_char, pollfd};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    context::Context,
    error::{AllocationError, Error},
    event::{Event, MountEvent},
    libmount::root::{
        MNT_MONITOR_TYPE_KERNEL, MNT_MONITOR_TYPE_USERSPACE, libmnt_monitor, mnt_get_fstab_path,
        mnt_monitor_enable_kernel, mnt_monitor_enable_userspace, mnt_monitor_get_fd,
        mnt_monitor_next_change, mnt_monitor_wait, mnt_new_monitor, mnt_ref_monitor,
        mnt_unref_monitor,
    },
    table::Table,
    update::TableUpdate,
};
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MonitorType {
    Userspace = MNT_MONITOR_TYPE_USERSPACE,
    Kernel = MNT_MONITOR_TYPE_KERNEL,
}
impl TryFrom<i32> for MonitorType {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value as u32 {
            MNT_MONITOR_TYPE_USERSPACE => Ok(Self::Userspace),
            MNT_MONITOR_TYPE_KERNEL => Ok(Self::Kernel),
            err => Err(Error::UndefinedMonitorType(err)),
        }
    }
}

pub struct Monitor(*mut libmnt_monitor);
unsafe impl Send for Monitor {}
unsafe impl Sync for Monitor {}
impl Monitor {
    pub fn new() -> Result<Self, AllocationError<Monitor>> {
        unsafe {
            let value = mnt_new_monitor();
            if !value.is_null() {
                Ok(Self(value))
            } else {
                Err(AllocationError::default())
            }
        }
    }
    pub fn with_kernel(&mut self, enabled: bool) -> Result<(), Error> {
        unsafe {
            if mnt_monitor_enable_kernel(self.0, enabled as i32) != 0 {
                Err(Error::MonitorKernel)
            } else {
                Ok(())
            }
        }
    }
    pub fn with_userspace(&mut self, enabled: bool, filename: Option<&Path>) -> Result<(), Error> {
        unsafe {
            let filename = if let Some(path) = filename.map(|value| value.as_os_str()) {
                &raw const path as *const i8
            } else {
                null()
            };
            if mnt_monitor_enable_userspace(self.0, enabled as i32, filename) != 0 {
                Err(Error::MonitorUserspace)
            } else {
                Ok(())
            }
        }
    }
    pub async fn poll_until<'a, F>(
        self,
        rate: Duration,
        mut current_table: Option<Table>,
        fire_initial: bool,
        mut condition: impl FnMut() -> bool,
        handler: F,
    ) -> Result<(), Error>
    where
        F: Fn(Event<'a>) -> bool,
    {
        let millis = rate.as_millis() as std::ffi::c_int;
        unsafe {
            let output_ptr =
                std::alloc::alloc_zeroed(Layout::new::<*mut *const c_char>()) as *mut *const c_char;
            let mut ty = 0;
            let mut result;
            if fire_initial {
                if let Some(table) = &current_table {
                    for fs in table.iter()? {
                        if !handler(Event::MountEvent(MountEvent::Mount { filesystem: fs? })) {
                            return Ok(());
                        }
                    }
                }
            }

            while !condition() {
                result = mnt_monitor_wait(self.0, millis);
                match result {
                    0 => {
                        continue;
                    }
                    1 => {
                        result = mnt_monitor_next_change(self.0, output_ptr, &mut ty);
                        match result {
                            0 => {
                                let value = CStr::from_ptr(*output_ptr);
                                let path = Path::new(OsStr::from_bytes(value.to_bytes()));
                                if !handler(Event::MountEvent(MountEvent::MonitorUpdate {
                                    location: path,
                                    monitor_type: ty.try_into()?,
                                })) {
                                    return Ok(());
                                }

                                let new_table = Table::read(&path)?;
                                if let Some(table) = &current_table {
                                    let iter = table.diff(&new_table)?;
                                    for evt in iter {
                                        if !handler(Event::MountEvent(evt?)) {
                                            return Ok(());
                                        }
                                    }
                                } else {
                                    let iter = new_table.iter()?;
                                    for fs in iter {
                                        if !handler(Event::MountEvent(MountEvent::Mount {
                                            filesystem: fs?,
                                        })) {
                                            return Ok(());
                                        }
                                    }
                                }
                                current_table.replace(new_table);
                            }
                            1 => {
                                continue;
                            }

                            error => {
                                return Err(Error::MonitorNextChange(error));
                            }
                        }
                    }
                    error => return Err(Error::MonitorPoll(error)),
                }
            }
            std::alloc::dealloc(output_ptr as *mut u8, Layout::new::<*mut *const c_char>());
            Ok(())
        }
    }
}
impl Clone for Monitor {
    fn clone(&self) -> Self {
        unsafe {
            mnt_ref_monitor(self.0);
            Monitor(self.0)
        }
    }
}
impl Drop for Monitor {
    fn drop(&mut self) {
        unsafe {
            mnt_unref_monitor(self.0);
        }
    }
}
