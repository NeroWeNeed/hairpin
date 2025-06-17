use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use tokio::task::JoinError;

use crate::{
    context::Context,
    fs::FileSystem,
    iter::IterInternal,
    monitor::Monitor,
    table::{Table, TableDiff},
    update::TableUpdate,
};

#[derive(Clone, Copy)]
pub struct AllocationError<T>(PhantomData<T>);
unsafe impl<T> Send for AllocationError<T> {}
unsafe impl<T> Sync for AllocationError<T> {}
impl<T> Default for AllocationError<T> {
    fn default() -> Self {
        Self(PhantomData::default())
    }
}
impl<T> Debug for AllocationError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Error allocating {:?}", self.0))
    }
}
impl<T> Display for AllocationError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Error allocating {:?}", self.0))
    }
}
impl<T> std::error::Error for AllocationError<T> {}
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    AllocationMonitor(#[from] AllocationError<Monitor>),
    #[error(transparent)]
    AllocationTableDiff(#[from] AllocationError<TableDiff>),
    #[error(transparent)]
    AllocationTable(#[from] AllocationError<Table>),
    #[error(transparent)]
    AllocationIter(#[from] AllocationError<IterInternal>),
    #[error(transparent)]
    AllocationFileSystem(#[from] AllocationError<FileSystem>),
    #[error(transparent)]
    AllocationTableUpdate(#[from] AllocationError<TableUpdate>),
    #[error(transparent)]
    AllocationContext(#[from] AllocationError<Context>),

    #[error("Error getting Table Update filename")]
    TableUpdateFile,
    #[error("Error updating Table")]
    TableUpdate(i32),
    #[error("Error Parsing mtab")]
    ParsingMTab(i32),

    #[error("Error setting kernel monitoring")]
    MonitorKernel,
    #[error("Error setting userspace monitoring")]
    MonitorUserspace,
    #[error("Error polling for mounts: {0}")]
    MonitorPoll(i32),
    #[error("Error monitoring next change: {0}")]
    MonitorNextChange(i32),
    #[error("Undefined Monitor Type {0}")]
    UndefinedMonitorType(u32),
    #[error("Undefined Direction {0}")]
    UndefinedDirection(u32),
    #[error("Undefined Diff Operation: {0}")]
    UndefinedDiffOperation(u32),

    #[error("Error in iteration {0}")]
    Iter(i32),
}
#[derive(Debug, thiserror::Error)]
pub enum ServeError<E>
where
    E: std::error::Error,
{
    #[error(transparent)]
    LibMount(#[from] Error),
    #[error("Error in Handler: {0:?}")]
    Handler(E),
    #[error(transparent)]
    JoinError(#[from] JoinError),
    #[error(transparent)]
    IO(#[from] std::io::Error),
}
