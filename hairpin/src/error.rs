use crate::commands;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Daemon(#[from] hairpin_daemon::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Clap(#[from] clap::Error),
    #[error(transparent)]
    InvalidCreateSourceArgs(#[from] commands::create::source::Error),
    #[error("Undefined")]
    Undefined,
}
