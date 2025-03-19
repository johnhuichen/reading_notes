use log::info;
use snafu::{ResultExt, Whatever};

pub fn init() -> Result<(), Whatever> {
    log4rs::init_file("config/log4rs.yml", Default::default())
        .whatever_context("Cannot initialize logger")?;
    info!("Starting Stock Data Ingestion App");

    Ok(())
}
