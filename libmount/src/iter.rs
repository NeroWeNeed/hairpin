use crate::{
    error::{AllocationError, Error},
    libmount::root::{
        MNT_ITER_BACKWARD, MNT_ITER_FORWARD, libmnt_iter, mnt_free_iter, mnt_iter_get_direction,
        mnt_new_iter, mnt_reset_iter,
    },
};

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    Forward = MNT_ITER_FORWARD,
    Backward = MNT_ITER_BACKWARD,
}
impl TryFrom<i32> for Direction {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value as u32 {
            MNT_ITER_FORWARD => Ok(Self::Forward),
            MNT_ITER_BACKWARD => Ok(Self::Backward),
            err => Err(Error::UndefinedDirection(err)),
        }
    }
}
#[derive(Debug)]
pub struct IterInternal(pub(crate) *mut libmnt_iter);
impl IterInternal {
    pub fn new(direction: Direction) -> Result<Self, AllocationError<Self>> {
        unsafe {
            let result = mnt_new_iter(direction as i32);
            if !result.is_null() {
                Ok(Self(result))
            } else {
                Err(AllocationError::default())
            }
        }
    }
    pub fn direction(&self) -> Result<Direction, Error> {
        unsafe { (mnt_iter_get_direction(self.0) as i32).try_into() }
    }
    pub fn reset(&self, directon: Option<Direction>) {
        unsafe {
            let direction = directon.map_or(-1, |value| value as i32);
            mnt_reset_iter(self.0, direction);
        }
    }
}

impl Drop for IterInternal {
    fn drop(&mut self) {
        unsafe {
            mnt_free_iter(self.0);
        }
    }
}
