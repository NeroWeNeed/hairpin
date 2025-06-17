use std::{
    alloc::Layout, ffi::c_char, marker::PhantomData, os::unix::ffi::OsStrExt, path::Path, ptr::null,
};

use crate::{
    error::{AllocationError, Error},
    event::MountEvent,
    fs::FileSystem,
    iter::IterInternal,
    libmount::root::{
        MNT_ERR_EXEC, MNT_TABDIFF_MOUNT, MNT_TABDIFF_MOVE, MNT_TABDIFF_PROPAGATION,
        MNT_TABDIFF_REMOUNT, MNT_TABDIFF_UMOUNT, libmnt_fs, libmnt_tabdiff, libmnt_table,
        mnt_diff_tables, mnt_free_tabdiff, mnt_new_lock, mnt_new_tabdiff, mnt_new_table,
        mnt_new_table_from_file, mnt_ref_table, mnt_tabdiff_next_change, mnt_table_next_fs,
        mnt_table_parse_file, mnt_table_parse_fstab, mnt_table_parse_mtab, mnt_unref_table,
    },
    util::{get_fstab_path, get_mtab_path, get_utab_path},
};

pub struct Table(pub(crate) *mut libmnt_table);
impl Table {
    pub(crate) fn new() -> Result<Self, AllocationError<Self>> {
        unsafe {
            let value = mnt_new_table();
            if !value.is_null() {
                Ok(Self(value))
            } else {
                Err(AllocationError::default())
            }
        }
    }
    pub fn read(path: impl AsRef<Path>) -> Result<Self, AllocationError<Self>> {
        unsafe {
            let path = path.as_ref();
            println!("Path: {path:?}");

            let path = path.as_os_str().as_bytes().as_ptr() as *const c_char;
            let result = mnt_new_table_from_file(path);
            if !result.is_null() {
                Ok(Self(result))
            } else {
                Err(AllocationError::default())
            }
        }
    }
    pub fn parse_mtab(path: Option<&Path>) -> Result<Table, AllocationError<Self>> {
        unsafe {
            let path = path.map_or_else(|| null(), |path| path.as_os_str().as_bytes().as_ptr());
            let output = Table::new()?;
            let result = mnt_table_parse_mtab(output.0, path as *const i8);
            if result == 0 {
                Ok(output)
            } else {
                Err(AllocationError::default())
            }
        }
    }
    pub fn parse_fstab(path: Option<&Path>) -> Result<Table, AllocationError<Self>> {
        unsafe {
            let path = path.map_or_else(|| null(), |path| path.as_os_str().as_bytes().as_ptr());
            let output = Table::new()?;
            let result = mnt_table_parse_fstab(output.0, path as *const i8);
            if result == 0 {
                Ok(output)
            } else {
                Err(AllocationError::default())
            }
        }
    }
    pub fn diff<'a>(&self, other: &Table) -> Result<Iter<'a, TableDiff>, Error> {
        unsafe {
            let df = TableDiff::new()?;
            mnt_diff_tables(df.0, self.0, other.0);
            let iter = IterInternal::new(crate::iter::Direction::Forward)?;
            Ok(Iter(df, iter, PhantomData::default()))
        }
    }
    pub fn iter<'a>(&self) -> Result<Iter<'a, Table>, Error> {
        let iter = IterInternal::new(crate::iter::Direction::Forward)?;
        Ok(Iter(self.clone(), iter, PhantomData::default()))
    }
}

impl Clone for Table {
    fn clone(&self) -> Self {
        unsafe {
            mnt_ref_table(self.0);
            Self(self.0)
        }
    }
}
impl Drop for Table {
    fn drop(&mut self) {
        unsafe {
            mnt_unref_table(self.0);
        }
    }
}
pub struct Iter<'a, T>(T, IterInternal, PhantomData<&'a ()>);
impl<'a> Iterator for Iter<'a, Table> {
    type Item = Result<FileSystem, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let fs = std::alloc::alloc_zeroed(Layout::new::<*mut *mut libmnt_fs>())
                as *mut *mut libmnt_fs;
            let result = mnt_table_next_fs(self.0.0, self.1.0, fs);
            match result {
                0 => Some(Ok(FileSystem(*fs))),
                1 => None,
                err => Some(Err(Error::Iter(err))),
            }
        }
    }
}
impl<'a> Iterator for Iter<'a, TableDiff> {
    type Item = Result<MountEvent<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let old = std::alloc::alloc_zeroed(Layout::new::<*mut *mut libmnt_fs>())
                as *mut *mut libmnt_fs;
            let new = std::alloc::alloc_zeroed(Layout::new::<*mut *mut libmnt_fs>())
                as *mut *mut libmnt_fs;
            let mut operation = -1;
            let result = mnt_tabdiff_next_change(self.0.0, self.1.0, old, new, &mut operation);
            match result {
                0 => {
                    let operation: DiffOperation = match operation.try_into() {
                        Ok(operation) => operation,
                        Err(err) => {
                            return Some(Err(err));
                        }
                    };
                    match operation {
                        DiffOperation::Move => Some(Ok(MountEvent::Move {
                            from: FileSystem(*old),
                            to: FileSystem(*new),
                        })),
                        DiffOperation::UMount => Some(Ok(MountEvent::UMount {
                            filesystem: FileSystem(*old),
                        })),
                        DiffOperation::Remount => Some(Ok(MountEvent::Remount {
                            filesystem: FileSystem(*new),
                        })),
                        DiffOperation::Mount => Some(Ok(MountEvent::Mount {
                            filesystem: FileSystem(*new),
                        })),
                        DiffOperation::Propagation => Some(Ok(MountEvent::Propagate {
                            parent: FileSystem(*old),
                            child: FileSystem(*new),
                        })),
                    }
                }
                1 => None,
                err => Some(Err(Error::Iter(err))),
            }
        }
    }
}
#[repr(u32)]
enum DiffOperation {
    Move = MNT_TABDIFF_MOVE,
    UMount = MNT_TABDIFF_UMOUNT,
    Remount = MNT_TABDIFF_REMOUNT,
    Mount = MNT_TABDIFF_MOUNT,
    Propagation = MNT_TABDIFF_PROPAGATION,
}
impl TryFrom<i32> for DiffOperation {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value as u32 {
            MNT_TABDIFF_MOVE => Ok(Self::Move),
            MNT_TABDIFF_UMOUNT => Ok(Self::UMount),
            MNT_TABDIFF_REMOUNT => Ok(Self::Remount),
            MNT_TABDIFF_MOUNT => Ok(Self::Mount),
            MNT_TABDIFF_PROPAGATION => Ok(Self::Propagation),
            err => Err(Error::UndefinedDiffOperation(err)),
        }
    }
}

pub struct TableDiff(*mut libmnt_tabdiff);
impl TableDiff {
    fn new() -> Result<Self, AllocationError<Self>> {
        unsafe {
            let value = mnt_new_tabdiff();
            if !value.is_null() {
                Ok(Self(value))
            } else {
                Err(AllocationError::default())
            }
        }
    }
}
impl Drop for TableDiff {
    fn drop(&mut self) {
        unsafe {
            mnt_free_tabdiff(self.0);
        }
    }
}
