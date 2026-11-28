extern crate core;

pub mod api;
mod config;
mod db;
mod login;

use crate::{config::Config, db::Database, login::Login};
use log::LevelFilter;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    stop_log::init(None, LevelFilter::Info);

    let config = Config::load()?;
    let db = Arc::new(Database::connect(&config.database_url).await?);
    let login = Arc::new(Login::new());

    tokio::spawn(login.clone().worker());

    api::serve(db, login, &config).await;

    Ok(())
}
