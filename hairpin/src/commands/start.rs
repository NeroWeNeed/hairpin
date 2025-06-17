use hairpin_daemon::model::{HairpinDaemon, HairpinDaemonOptions};

use crate::Resolver;

impl Resolver for HairpinDaemonOptions {
    type Context = ();

    type Error = crate::Error;

    fn resolve(self, _: Self::Context) -> Result<(), Self::Error> {
        Ok(tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(HairpinDaemon::start(self))?)
    }
}
