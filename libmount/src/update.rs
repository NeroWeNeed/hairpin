use std::{
    ffi::{CStr, OsStr},
    os::unix::ffi::OsStrExt,
    path::Path,
    ptr::null_mut,
};

use crate::{
    error::{AllocationError, Error},
    libmount::root::{
        libmnt_update, mnt_free_update, mnt_new_update, mnt_update_get_filename, mnt_update_table,
    },
};

#[derive(Debug)]
pub struct TableUpdate(*mut libmnt_update);
impl TableUpdate {
    pub fn new() -> Result<Self, AllocationError<Self>> {
        unsafe {
            let value = mnt_new_update();
            if !value.is_null() {
                Ok(TableUpdate(value))
            } else {
                Err(AllocationError::default())
            }
        }
    }
    pub fn update(&self) -> Result<(), Error> {
        unsafe {
            let result = mnt_update_table(self.0, null_mut());
            if result != 0 {
                Err(Error::TableUpdate(result))
            } else {
                Ok(())
            }
        }
    }
    pub fn file<'a>(&self) -> Result<&'a Path, Error> {
        unsafe {
            let output_ptr = mnt_update_get_filename(self.0);
            if !output_ptr.is_null() {
                let value = CStr::from_ptr(output_ptr);
                let path = Path::new(OsStr::from_bytes(value.to_bytes()));
                Ok(path)
            } else {
                Err(Error::TableUpdateFile)
            }
        }
    }
}
impl Drop for TableUpdate {
    fn drop(&mut self) {
        unsafe {
            mnt_free_update(self.0);
        }
    }
}
