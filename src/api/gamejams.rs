use axum::{http::StatusCode, response::IntoResponse};

pub async fn get_gamejams() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}

pub async fn get_gamejam_cover() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}
