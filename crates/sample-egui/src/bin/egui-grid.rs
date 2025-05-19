/*!
# sample egui grid
**/

use std::error::Error;

use tracing::{debug, info, level_filters::LevelFilter};
use tracing_subscriber::{Registry, fmt, prelude::*};

fn main() -> Result<(), Box<dyn Error>> {
        // ///////////////////////////////////////// [ tracing ] ///////////////////////////////////////// //
        let envfilter = tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy();
        let subscriber = Registry::default().with(fmt::Layer::default().with_filter(envfilter));
        tracing::subscriber::set_global_default(subscriber)?;

        // ///////////////////////////////////////// [ body ] ///////////////////////////////////////// //
        info!("oh my gosh so silly");
        println!("hello whilrd");
        debug!("oh my gosh so silly");

        // ///////////////////////////////////////// [ finish ] ///////////////////////////////////////// //
        Ok(())
}
