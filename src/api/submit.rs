use axum::{http::StatusCode, response::IntoResponse};

pub async fn submit() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}

pub async fn get_submissions() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}

pub async fn get_verified_submissions_count() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}

pub async fn get_unverified_submissions_count() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}

pub async fn get_submission() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}

pub async fn get_submission_reviews() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}

pub async fn get_submission_cover_horizontal() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}

pub async fn get_submission_cover_vertical() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}

pub async fn get_tags() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}

pub async fn get_tag() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}

pub async fn get_tags_categories() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}

pub async fn get_tags_categories_not_panel_only() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}

pub async fn get_tags_category() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "PLACEHOLDER")
}
