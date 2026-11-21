use crate::config::Config;
use axum::{Router, routing::get};
use log::{error, info};
use std::{io, path::Path};
use tokio::{
    net::{TcpListener, UnixListener},
    select,
};

mod submit;

fn scheme() -> Router {
    Router::new().route("/hello", get(|| async { "Hello, world!" }))
}

fn open_unix_socket(path: &Path) -> io::Result<UnixListener> {
    if path.exists() {
        std::fs::remove_file(path)?;
    }

    UnixListener::bind(path)
}

pub async fn serve(config: &Config) {
    let router = scheme();

    let tcp = match TcpListener::bind("0.0.0.0:3000").await {
        Ok(listener) => listener,
        Err(error) => {
            error!("failed to open tcp socket: {}", error);
            return;
        }
    };

    let unix = match open_unix_socket(&config.unix_path) {
        Ok(listener) => listener,
        Err(error) => {
            error!("failed to open unix socket: {}", error);
            return;
        }
    };

    let tcp_join = tokio::spawn(axum::serve(tcp, router.clone()).into_future());
    let unix_future = axum::serve(unix, router).into_future();

    info!("listening on addr: http://{}", config.listen_addr);
    info!(
        "listening unix domain socket: {}",
        config.unix_path.display()
    );

    select! {
        Err(err) = tcp_join => {
            error!("tcp serving error: {}", err);
        },
        Err(err) = unix_future => {
            error!("unix serving error: {}", err);
        }
    }
}
