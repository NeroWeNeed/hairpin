use clap::Parser;
mod commands;
mod error;
pub use error::*;

use crate::commands::Commands;

pub trait Resolver {
    type Context;
    type Error;
    fn resolve(self, context: Self::Context) -> Result<(), Self::Error>;
}
#[derive(Debug, Parser)]
#[command(name = "hairpin")]
#[command(about = "Last mile secret processor", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}
impl Resolver for Cli {
    type Context = ();

    type Error = crate::Error;

    fn resolve(self, _: Self::Context) -> Result<(), Self::Error> {
        self.command.resolve(())
    }
}
