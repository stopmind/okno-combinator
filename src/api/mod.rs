use crate::{
    config::Config,
    db::{Database, DbError},
    login::Login,
};
use axum::{
    Router,
    extract::Request,
    middleware,
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use log::{error, info};
use reqwest::StatusCode;
use std::{io, path::Path, sync::Arc};
use thiserror::Error;
use tokio::{
    net::{TcpListener, UnixListener},
    select,
};
use tower_http::cors::CorsLayer;

mod gamejams;
mod panel;
mod submit;
mod utils;

struct ApiContext {
    db: Arc<Database>,
    login: Arc<Login>,
    client: reqwest::Client,
}

type ApiCtx = Arc<ApiContext>;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("db error: {0}")]
    DbError(#[from] DbError),
    #[error("request to third-party api failed: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("failed to parse url: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("{0}")]
    Other(String),
}

impl From<&str> for ApiError {
    fn from(s: &str) -> Self {
        ApiError::Other(s.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        error!("error response: {}", self);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

type ApiResult<T = Response> = Result<T, ApiError>;

#[cfg(debug_assertions)]
async fn log_request(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().clone();

    let resp = next.run(req).await;
    info!("{} {} -> {}", method, path, resp.status());
    resp
}

#[rustfmt::skip]
fn scheme(ctx: Arc<ApiContext>) -> Router {
    let router = Router::new()
        .nest("/panel/", panel::scheme(ctx.clone()))

        .route("/submit", post(submit::submit))
        .route("/submissions", get(submit::get_submissions))
        .route("/submissions/verified/count", get(submit::get_verified_submissions_count))
        .route("/submissions/unverified/count",get(submit::get_unverified_submissions_count))
        .route("/submissions/{subid}", get(submit::get_submission))
        .route("/submissions/{subid}/reviews", get(submit::get_submission_reviews))
        .route("/submissions/{subid}/cover_horizontal", get(submit::get_submission_cover))
        .route("/submissions/{subid}/cover_vertical", get(submit::get_submission_cover))
        .route("/tags", get(submit::get_tags))
        .route("/tags/{tag}", get(submit::get_submissions_with_tag))
        .route("/tags/categories", get(submit::get_tags_categories))
        .route("/tags/categories/{category}", get(submit::get_tags_category))
        .route("/tags/categories/notPanelOnly", get(submit::get_tags_categories_not_panel_only))
        .route("/gamejams", get(gamejams::get_gamejams))
        .route("/gamejams/{id}/cover", get(gamejams::get_gamejam_cover))
        .route("/community/content", get(gamejams::get_all_community_content))
        .route("/community/content/{id}", get(gamejams::get_community_content))
        .route("/community/content/{id}/cover", get(gamejams::get_community_content_cover))

        .with_state(ctx)
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

pub async fn serve(db: Arc<Database>, login: Arc<Login>, config: &Config) {
    let ctx = Arc::new(ApiContext {
        db,
        login,
        client: reqwest::Client::new(),
    });
    let router = scheme(ctx);

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
