use std::{ops::BitOr, path::Path};

use crate::{error::Error, fs::FileSystem, monitor::MonitorType};
#[derive(Debug)]
pub enum Event<'a> {
    MountEvent(MountEvent<'a>),
    Close,
    Error(Error),
}
#[derive(Debug, Clone)]
pub enum MountEvent<'a> {
    MonitorUpdate {
        location: &'a Path,
        monitor_type: MonitorType,
    },

    Mount {
        filesystem: FileSystem,
    },
    UMount {
        filesystem: FileSystem,
    },
    Remount {
        filesystem: FileSystem,
    },
    Move {
        from: FileSystem,
        to: FileSystem,
    },
    Propagate {
        parent: FileSystem,
        child: FileSystem,
    },
}
#[derive(Debug, Clone, Copy)]
pub struct MountEventMask(u32);
impl MountEventMask {
    pub const MONITOR_UPDATE: MountEventMask = MountEventMask(0b1);
    pub const MOUNT: MountEventMask = MountEventMask(0b10);
    pub const UMOUNT: MountEventMask = MountEventMask(0b100);
    pub const REMOUNT: MountEventMask = MountEventMask(0b1000);
    pub const MOVE: MountEventMask = MountEventMask(0b10000);
    pub const PROPAGATE: MountEventMask = MountEventMask(0b100000);

    pub fn matches<'a>(&self, event: &MountEvent<'a>) -> bool {
        (self.0 & event.mask().0) != 0
    }
}
impl BitOr for MountEventMask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl<'a> MountEvent<'a> {
    pub fn mask(&self) -> MountEventMask {
        match self {
            MountEvent::MonitorUpdate {
                location: _,
                monitor_type: _,
            } => MountEventMask::MONITOR_UPDATE,
            MountEvent::Mount { filesystem: _ } => MountEventMask::MOUNT,
            MountEvent::UMount { filesystem: _ } => MountEventMask::UMOUNT,
            MountEvent::Remount { filesystem: _ } => MountEventMask::REMOUNT,
            MountEvent::Move { from: _, to: _ } => MountEventMask::MOVE,
            MountEvent::Propagate {
                parent: _,
                child: _,
            } => MountEventMask::PROPAGATE,
        }
    }
}
impl<'a> From<MountEvent<'a>> for MountEventMask {
    fn from(value: MountEvent<'a>) -> Self {
        value.mask()
    }
}
impl<'a> BitOr for MountEvent<'a> {
    type Output = MountEventMask;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.mask() | rhs.mask()
    }
}
