use std::{
    ffi::{CStr, OsStr},
    os::unix::ffi::OsStrExt,
    path::Path,
};

use crate::{
    error::{AllocationError, Error},
    libmount::root::{
        libmnt_fs, mnt_fs_get_bindsrc, mnt_fs_get_root, mnt_fs_get_target, mnt_new_fs, mnt_ref_fs,
        mnt_unref_fs,
    },
};

#[derive(Debug)]
pub struct FileSystem(pub(crate) *mut libmnt_fs);
unsafe impl Send for FileSystem {}
unsafe impl Sync for FileSystem {}
impl Clone for FileSystem {
    fn clone(&self) -> Self {
        unsafe {
            mnt_ref_fs(self.0);
            Self(self.0)
        }
    }
}
impl FileSystem {
    pub fn new() -> Result<Self, AllocationError<Self>> {
        unsafe {
            let result = mnt_new_fs();
            if !result.is_null() {
                Ok(Self(result))
            } else {
                Err(AllocationError::default())
            }
        }
    }
    pub fn root<'a>(&self) -> Option<&'a Path> {
        unsafe {
            let value = mnt_fs_get_root(self.0);
            if !value.is_null() {
                let value = CStr::from_ptr(value);
                let value = Path::new(OsStr::from_bytes(value.to_bytes()));
                Some(value)
            } else {
                None
            }
        }
    }
    pub fn bindsrc<'a>(&self) -> Option<&'a Path> {
        unsafe {
            let value = mnt_fs_get_bindsrc(self.0);
            if !value.is_null() {
                let value = CStr::from_ptr(value);
                let value = Path::new(OsStr::from_bytes(value.to_bytes()));
                Some(value)
            } else {
                None
            }
        }
    }
    pub fn target<'a>(&self) -> Option<&'a Path> {
        unsafe {
            let value = mnt_fs_get_target(self.0);
            if !value.is_null() {
                let value = CStr::from_ptr(value);
                let value = Path::new(OsStr::from_bytes(value.to_bytes()));
                Some(value)
            } else {
                None
            }
        }
    }
}
impl Drop for FileSystem {
    fn drop(&mut self) {
        unsafe {
            mnt_unref_fs(self.0);
        }
    }
}
