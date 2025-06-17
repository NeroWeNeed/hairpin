use std::sync::Arc;

use model::{HairpinDaemon, HairpinDaemonOptions};
mod error;
pub mod model;
pub mod service;
pub use error::*;

impl HairpinDaemon {
    pub async fn start(options: HairpinDaemonOptions) -> Result<(), Error> {
        let daemon = Arc::new(HairpinDaemon::default());

        Ok(())
    }
}
