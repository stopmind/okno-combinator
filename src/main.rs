extern crate core;

pub mod api;
mod config;
mod db;

use crate::{config::Config, db::Database};
use log::LevelFilter;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    stop_log::init(None, LevelFilter::Info);

    let config = Config::load()?;
    let db = Arc::new(Database::connect(&config.database_url).await?);

    api::serve(db, &config).await;

    Ok(())
}
