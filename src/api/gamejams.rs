use crate::api::ApiCtx;
use axum::{extract::State, http::StatusCode, response::IntoResponse};

pub async fn get_gamejams(State(ctx): State<ApiCtx>) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}

pub async fn get_gamejam_cover(State(ctx): State<ApiCtx>) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}
