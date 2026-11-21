extern crate core;

pub mod api;
mod config;

use crate::config::Config;
use log::LevelFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    stop_log::init(None, LevelFilter::Info);

    let config = Config::load()?;

    api::serve(&config).await;

    Ok(())
}
