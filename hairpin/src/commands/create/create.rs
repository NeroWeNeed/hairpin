use clap::Subcommand;

use crate::Resolver;

use super::source::CreateSourceArgs;

#[derive(Debug, Subcommand)]
pub enum CreateCommands {
    #[command(arg_required_else_help = true)]
    Source(CreateSourceArgs),
}
impl Resolver for CreateCommands {
    type Context = ();

    type Error = crate::Error;

    fn resolve(self, context: Self::Context) -> Result<(), Self::Error> {
        match self {
            CreateCommands::Source(value) => Ok(value.resolve(context)?),
        }
    }
}
