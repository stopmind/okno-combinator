use axum::{http::StatusCode, response::IntoResponse};

pub async fn get_all_community_content() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}

pub async fn get_community_content() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}

pub async fn get_community_content_cover() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}
