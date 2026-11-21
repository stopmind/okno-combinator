use serde::Deserialize;
use std::{env, fs, io, path::PathBuf};
use thiserror::Error;

const CONFIG_PATH_VAR: &str = "OKNO_COMBINATOR_CONFIG";
const DEFAULT_CONFIG_PATH: &str = "/etc/okno-combinator.toml";

fn config_path() -> PathBuf {
    env::var(CONFIG_PATH_VAR)
        .map(PathBuf::from)
        .unwrap_or(DEFAULT_CONFIG_PATH.into())
}

#[derive(Error, Debug)]
#[error("failed to load config file: {0}")]
pub enum LoadError {
    Io(#[from] io::Error),
    Serde(#[from] toml::de::Error),
}

#[derive(Deserialize)]
pub struct Config {
    pub database_url: String,
    pub listen_addr: String,
    pub unix_path: PathBuf,
}

impl Config {
    pub fn load() -> Result<Config, LoadError> {
        let config_path = config_path();
        let data = fs::read(config_path)?;
        let config = toml::de::from_slice(&data)?;
        Ok(config)
    }
}
