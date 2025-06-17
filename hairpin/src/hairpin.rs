use clap::Parser;
use hairpin::{Cli, Error, Resolver};

fn main() -> Result<(), Error> {
    Cli::try_parse()?.resolve(())
}
