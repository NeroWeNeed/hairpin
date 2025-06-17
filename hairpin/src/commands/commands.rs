use std::error::Error;

use clap::Subcommand;

use crate::Resolver;

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(subcommand)]
    Create(super::create::CreateCommands),
    Start(hairpin_daemon::model::HairpinDaemonOptions),
}
impl Resolver for Commands {
    type Context = ();

    type Error = crate::Error;

    fn resolve(self, context: Self::Context) -> Result<(), Self::Error> {
        match self {
            Commands::Create(value) => Ok(value.resolve(context)?),
            Commands::Start(value) => Ok(value.resolve(context)?),
        }
    }
}

pub fn parse_property<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}
