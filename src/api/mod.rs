use crate::config::Config;
use axum::{
    Router,
    extract::Request,
    middleware,
    middleware::Next,
    response::Response,
    routing::{get, post},
};
use log::{error, info};
use std::{io, path::Path};
use tokio::{
    net::{TcpListener, UnixListener},
    select,
};
use tower_http::cors::CorsLayer;

mod community;
mod gamejams;
mod submit;

#[cfg(debug_assertions)]
async fn log_request(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().clone();

    let resp = next.run(req).await;
    info!("{} {} -> {}", method, path, resp.status());
    resp
}

#[rustfmt::skip]
fn scheme() -> Router {
    let router = Router::new()
        .route("/submit", post(submit::submit))
        .route("/submissions", get(submit::get_submissions))
        .route("/submissions/verified/count", get(submit::get_verified_submissions_count))
        .route("/submissions/unverified/count",get(submit::get_unverified_submissions_count))
        .route("/submissions/{subid}", get(submit::get_submission))
        .route("/submissions/{subid}/reviews", get(submit::get_submission_reviews))
        .route("/submissions/{subid}/cover_horizontal", get(submit::get_submission_cover_horizontal))
        .route("/submissions/{subid}/cover_vertical", get(submit::get_submission_cover_vertical))
        .route("/tags", get(submit::get_tags))
        .route("/tags/{tag}", get(submit::get_tag))
        .route("/tags/categories", get(submit::get_tags_categories))
        .route("/tags/categories/{category}", get(submit::get_tags_category))
        .route("/tags/categories/notPanelOnly", get(submit::get_tags_categories_not_panel_only))
        .route("/gamejams", get(gamejams::get_gamejams))
        .route("/gamejams/{id}/cover", get(gamejams::get_gamejam_cover))
        .route("/community/content", get(community::get_all_community_content))
        .route("/community/content/{id}", get(community::get_community_content))
        .route("/community/content/{id}/cover", get(community::get_community_content_cover))
        .layer(CorsLayer::permissive());

    #[cfg(debug_assertions)]
    let router = router.layer(middleware::from_fn(log_request));

    router
}

fn open_unix_socket(path: &Path) -> io::Result<UnixListener> {
    if path.exists() {
        std::fs::remove_file(path)?;
    }

    UnixListener::bind(path)
}

pub async fn serve(config: &Config) {
    let router = scheme();

    let tcp = match TcpListener::bind(&config.listen_addr).await {
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
