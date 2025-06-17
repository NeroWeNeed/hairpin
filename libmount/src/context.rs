use std::{
    ffi::{CStr, CString},
    os::unix::ffi::OsStrExt,
    path::Path,
    ptr::null,
    str::FromStr,
};

use crate::{
    error::{AllocationError, Error},
    libmount::root::{
        libmnt_context, mnt_free_context, mnt_new_context, mnt_table_parse_file,
        mnt_table_parse_mtab,
    },
    table::Table,
    util::get_utab_path,
};

#[derive(Debug)]
pub struct Context(*mut libmnt_context);
impl Context {
    pub fn new() -> Result<Self, AllocationError<Self>> {
        unsafe {
            let value = mnt_new_context();
            if !value.is_null() {
                Ok(Self(value))
            } else {
                Err(AllocationError::default())
            }
        }
    }
}
impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            mnt_free_context(self.0);
        }
    }
}
