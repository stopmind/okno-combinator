use crate::{
    api::{
        ApiContext, ApiCtx, ApiResult,
        submit::{SubmissionResponse, format_cover_image},
    },
    db::{DbResult, SubmissionStatus, User, UserId},
    login::Token,
};
use axum::{
    Json, Router,
    extract::{Path, Request, State},
    http::{StatusCode, header},
    middleware,
    middleware::Next,
    response::IntoResponse,
    routing::{get, post},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use chrono::NaiveDate;
use futures::{StreamExt, TryStreamExt, stream};
use serde::{Deserialize, Serialize};

const TOKEN_COOKIE: &str = "oknoweb_panel_token";

pub async fn auth(
    State(ctx): State<ApiCtx>,
    cookies: CookieJar,
    req: Request,
    next: Next,
) -> ApiResult {
    auth_base(&ctx, &cookies)?;
    Ok(next.run(req).await)
}

pub async fn admin_auth(
    State(ctx): State<ApiCtx>,
    cookies: CookieJar,
    req: Request,
    next: Next,
) -> ApiResult {
    let user = auth_base(&ctx, &cookies)?;
    let user = ctx.db.get_user(user).await?;
    if user.admin {
        Ok(next.run(req).await)
    } else {
        Ok(StatusCode::FORBIDDEN.into_response())
    }
}

fn auth_base(ctx: &ApiContext, cookies: &CookieJar) -> ApiResult<UserId> {
    let cookie = cookies.get(TOKEN_COOKIE).ok_or("no auth cookie set")?;
    let token = Token::try_from_base64(cookie.value()).ok_or("invalid token")?;

    let user_id = ctx.login.check_token(&token).ok_or("unknown token")?;

    Ok(user_id)
}

#[rustfmt::skip]
pub fn scheme(ctx: ApiCtx) -> Router<ApiCtx> {
    Router::new()
        .route("/login", post(login))
        .merge(auth_scheme(ctx.clone()))
        .merge(admin_scheme(ctx))
}

#[rustfmt::skip]
fn auth_scheme(ctx: ApiCtx) -> Router<ApiCtx> {
    Router::new()
        .route("/check_login", get(async || StatusCode::OK))
        .route("/submissions", get(get_submissions))
        .route("/submissions/{id}/reject", post(reject_submission))
        .route("/submissions/{id}/pend", post(pend_submission))
        .route("/submissions/{id}/edit", post(edit_submission))
        .route("/submissions/{id}/reviews", post(review_submission))
        .route("/submissions/{id}/reviews/{review_id}/delete", post(delete_submission_review))


        .layer(middleware::from_fn_with_state(ctx, auth))
}

#[rustfmt::skip]
fn admin_scheme(ctx: ApiCtx) -> Router<ApiCtx> {
    Router::new()
        .route("/users", get(get_users))
        .route("/users/add", post(add_user))
        .route("/users/{id}",
            get(get_user).delete(delete_user)
        )
        .route("/users/{id}/kick", post(kick_user))
        .route("/users/{id}/locked", post(lock_user))
        .route("/users/{id}/password", post(set_user_password))
        .layer(middleware::from_fn_with_state(ctx, admin_auth))
}

#[derive(Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct AddUserPayload {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    id: UserId,
    token: String,
    admin: bool,
}

#[derive(Serialize)]
struct UserResponse {
    id: UserId,
    username: String,
    locked: bool,
    admin: bool,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            username: user.username,
            locked: user.locked,
            admin: user.admin,
        }
    }
}

async fn login(State(ctx): State<ApiCtx>, Json(payload): Json<LoginPayload>) -> ApiResult {
    let user = ctx.db.get_user_by_name(&payload.username).await?;
    if user.password != payload.password {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    if user.locked {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    let token = ctx.login.new_token(user.id).base64();

    let cookie = Cookie::build((TOKEN_COOKIE, token.clone()))
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(true)
        .to_string();
    let response = LoginResponse {
        id: user.id,
        admin: user.admin,
        token,
    };

    Ok(([(header::SET_COOKIE, cookie)], Json(response)).into_response())
}

async fn get_users(State(ctx): State<ApiCtx>) -> ApiResult {
    let users = ctx.db.get_all_users().await?;
    let response: Vec<_> = users.into_iter().map(UserResponse::from).collect();

    Ok(Json(response).into_response())
}

async fn get_user(State(ctx): State<ApiCtx>, Path(id): Path<UserId>) -> ApiResult {
    let user = ctx.db.get_user(id).await?;
    Ok(Json(UserResponse::from(user)).into_response())
}

async fn delete_user(State(ctx): State<ApiCtx>, Path(id): Path<UserId>) -> ApiResult {
    ctx.db.delete_user(id).await?;
    Ok(StatusCode::OK.into_response())
}

async fn add_user(State(ctx): State<ApiCtx>, Json(payload): Json<AddUserPayload>) -> ApiResult {
    ctx.db
        .create_user(&payload.username, &payload.password, false)
        .await?;
    Ok(StatusCode::OK.into_response())
}

async fn kick_user(State(ctx): State<ApiCtx>, Path(id): Path<UserId>) -> ApiResult {
    ctx.login.revoke_token(id);
    Ok(StatusCode::OK.into_response())
}

async fn lock_user(
    State(ctx): State<ApiCtx>,
    Path(id): Path<UserId>,
    Json(lock): Json<bool>,
) -> ApiResult {
    if lock {
        ctx.login.revoke_token(id);
    }
    ctx.db.set_user_lock(id, lock).await?;
    Ok(StatusCode::OK.into_response())
}

async fn set_user_password(
    State(ctx): State<ApiCtx>,
    Path(id): Path<UserId>,
    Json(password): Json<String>,
) -> ApiResult {
    ctx.db.set_user_password(id, &password).await?;
    Ok(StatusCode::OK.into_response())
}

async fn get_submissions(State(ctx): State<ApiCtx>) -> ApiResult {
    let submissions = ctx.db.get_all_submissions().await?;
    let model: Vec<_> = stream::iter(submissions)
        .then(
            async |base| -> DbResult<crate::api::submit::SubmissionResponse> {
                Ok(SubmissionResponse {
                    tags: ctx.db.get_submission_tags_titles(base.id).await?,
                    cover_image: format_cover_image(base.id),
                    base,
                })
            },
        )
        .try_collect()
        .await?;

    Ok(Json(model).into_response())
}

async fn reject_submission(State(ctx): State<ApiCtx>, Path(id): Path<i32>) -> ApiResult {
    ctx.db.delete_submission(id).await?;
    Ok(StatusCode::OK.into_response())
}

async fn pend_submission(State(ctx): State<ApiCtx>, Path(id): Path<i32>) -> ApiResult {
    ctx.db
        .set_submission_status(id, SubmissionStatus::Pending)
        .await?;
    Ok(StatusCode::OK.into_response())
}

#[derive(Deserialize)]
struct EditSubmissionRequest {
    pub name: String,
    pub link: String,
    #[serde(rename = "additionalInfo")]
    pub additional_info: Option<String>,
    pub author: String,
    pub tags: Vec<String>,
    pub date: NaiveDate,
}

async fn edit_submission(
    State(ctx): State<ApiCtx>,
    Path(id): Path<i32>,
    Json(request): Json<EditSubmissionRequest>,
) -> ApiResult {
    let tags_id: Vec<_> = stream::iter(&request.tags)
        .then(|title| ctx.db.get_tag_id_by_title(title))
        .try_collect()
        .await?;

    ctx.db
        .edit_submission(
            id,
            &request.name,
            &request.link,
            &request.author,
            request.additional_info.as_deref(),
            request.date,
        )
        .await?;

    ctx.db.remove_all_submission_tags(id).await?;
    ctx.db.add_submission_tags(id, &tags_id).await?;

    Ok(StatusCode::OK.into_response())
}

#[derive(Deserialize)]
struct ReviewRequest {
    author: String,
    link: String,
    rating: i32,
}

async fn review_submission(
    State(ctx): State<ApiCtx>,
    Path(id): Path<i32>,
    Json(request): Json<ReviewRequest>,
) -> ApiResult {
    ctx.db
        .create_submission_review(id, &request.author, &request.link, request.rating)
        .await?;
    Ok(StatusCode::OK.into_response())
}

async fn delete_submission_review(
    State(ctx): State<ApiCtx>,
    Path((_, id)): Path<(i32, i32)>,
) -> ApiResult {
    ctx.db.delete_submission_review(id).await?;
    Ok(StatusCode::OK.into_response())
}
