use crate::api::ApiCtx;
use axum::{extract::State, http::StatusCode, response::IntoResponse};

pub async fn get_all_community_content(State(ctx): State<ApiCtx>) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}

pub async fn get_community_content(State(ctx): State<ApiCtx>) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}

pub async fn get_community_content_cover(State(ctx): State<ApiCtx>) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}
