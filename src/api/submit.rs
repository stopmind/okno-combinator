use crate::{
    api::{ApiContext, ApiCtx, ApiError, ApiResult},
    db::{DbResult, Submission, SubmissionStatus},
};
use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::{Response, StatusCode},
    response::IntoResponse,
};
use futures::{StreamExt, TryStreamExt, stream};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubmissionResponse {
    #[serde(flatten)]
    base: Submission,
    tags: Vec<String>,
    cover_image: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitRequest {
    name: String,
    link: String,
    additional_info: Option<String>,
    author: String,
    tags: Vec<String>,
}

fn format_cover_image(id: i32) -> String {
    format!("https://oknoweb.ru/submissions/{id}/cover")
}

pub async fn get_submissions(State(ctx): State<ApiCtx>) -> ApiResult {
    let submissions = ctx.db.get_all_submissions().await?;
    let model: Vec<_> = stream::iter(submissions)
        .then(async |base| -> DbResult<SubmissionResponse> {
            Ok(SubmissionResponse {
                tags: ctx.db.get_submission_tags_titles(base.id).await?,
                cover_image: format_cover_image(base.id),
                base,
            })
        })
        .try_collect()
        .await?;

    Ok(Json(model).into_response())
}

pub async fn submit(State(ctx): State<ApiCtx>, Json(submit_req): Json<SubmitRequest>) -> ApiResult {
    let tags_id: Vec<_> = match stream::iter(&submit_req.tags)
        .then(|title| ctx.db.get_tag_id_by_title(title))
        .try_collect()
        .await
    {
        Ok(ids) => ids,
        Err(err) if err.is_not_found() => return Ok("Invalid tags".into_response()),
        Err(err) => return Err(err.into()),
    };

    let (id, date) = ctx
        .db
        .create_submission(
            SubmissionStatus::Unverified,
            &submit_req.name,
            &submit_req.link,
            &submit_req.author,
            submit_req.additional_info.as_deref(),
        )
        .await?;

    ctx.db.add_submission_tags(id, &tags_id).await?;

    let response = SubmissionResponse {
        base: Submission {
            id,
            status: SubmissionStatus::Pending,
            name: submit_req.name,
            link: submit_req.link,
            date,
            author: submit_req.author,
            additional_info: submit_req.additional_info,
        },
        tags: submit_req.tags,
        cover_image: format_cover_image(id),
    };

    Ok(Json(response).into_response())
}

pub async fn get_verified_submissions_count(State(ctx): State<ApiCtx>) -> ApiResult {
    let count = ctx
        .db
        .count_submissions_without_status(SubmissionStatus::Unverified)
        .await?;
    Ok(Json(count).into_response())
}

pub async fn get_unverified_submissions_count(State(ctx): State<ApiCtx>) -> ApiResult {
    let count = ctx
        .db
        .count_submissions_with_status(SubmissionStatus::Unverified)
        .await?;
    Ok(Json(count).into_response())
}

pub async fn get_submission(State(ctx): State<ApiCtx>, Path(sub_id): Path<i32>) -> ApiResult {
    let res = ctx.db.get_submission(sub_id).await;
    match res {
        Err(err) if err.is_not_found() => Ok(StatusCode::NOT_FOUND.into_response()),
        Err(err) => Err(err.into()),
        Ok(submission) => Ok(Json(submission).into_response()),
    }
}

pub async fn get_submission_reviews(
    State(ctx): State<ApiCtx>,
    Path(sub_id): Path<i32>,
) -> ApiResult {
    if ctx.db.submission_exists(sub_id).await? {
        let reviews = ctx.db.get_submission_reviews(sub_id).await?;
        Ok(Json(reviews).into_response())
    } else {
        Ok(StatusCode::NOT_FOUND.into_response())
    }
}

pub async fn get_submission_cover(State(ctx): State<ApiCtx>, Path(sub_id): Path<i32>) -> ApiResult {
    get_cover_for_submission(&ctx, sub_id).await
}

pub async fn get_tags(State(ctx): State<ApiCtx>) -> ApiResult {
    let tags = ctx.db.get_tags().await?;
    Ok(Json(tags).into_response())
}

pub async fn get_submissions_with_tag(
    State(ctx): State<ApiCtx>,
    Path(tag): Path<String>,
) -> ApiResult {
    let tag_id = match ctx.db.get_tag_id_by_title(&tag).await {
        Err(err) if err.is_not_found() => return Ok(StatusCode::NOT_FOUND.into_response()),
        Err(err) => return Err(err.into()),
        Ok(tag_id) => tag_id,
    };

    let result = ctx.db.get_submissions_with_tag(tag_id).await?;
    Ok(Json(result).into_response())
}

pub async fn get_tags_categories(State(ctx): State<ApiCtx>) -> ApiResult {
    let result = ctx.db.get_tags_categories().await?;
    Ok(Json(result).into_response())
}

pub async fn get_tags_categories_not_panel_only(State(ctx): State<ApiCtx>) -> ApiResult {
    let result = ctx.db.get_tags_categories_no_panel().await?;
    Ok(Json(result).into_response())
}

pub async fn get_tags_category(
    State(ctx): State<ApiCtx>,
    Path(category): Path<String>,
) -> ApiResult {
    let result = match ctx.db.get_tags_category_by_title(&category).await {
        Err(err) if err.is_not_found() => return Ok(StatusCode::NOT_FOUND.into_response()),
        Err(err) => return Err(err.into()),
        Ok(cat) => cat,
    };

    Ok(Json(result).into_response())
}

async fn get_cover_url_from_url(client: &Client, url: Url) -> ApiResult<Url> {
    let Some(domain) = url.domain() else {
        return Err(ApiError::Other(format!("can't get cover for url: {url}")));
    };

    if domain.ends_with(".itch.io") {
        let mut data_url = url;
        data_url.path_segments_mut().unwrap().push("data.json");

        #[derive(Deserialize)]
        struct ItchGameData {
            cover_image: Url,
        }

        let data: ItchGameData = client
            .get(data_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        return Ok(data.cover_image);
    }

    let content = client
        .get(url.clone())
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let dom = Html::parse_document(content.as_str());
    let selector = Selector::parse("meta[property=\"og:image\"]").unwrap();

    let options = Url::options().base_url(Some(&url));

    let cover_url = dom
        .select(&selector)
        .filter_map(|element| options.parse(element.attr("content")?).ok())
        .next();

    cover_url.ok_or_else(|| ApiError::Other(format!("can't get cover for url: {url}")))
}

async fn get_cover_for_submission(ctx: &ApiContext, sub_id: i32) -> ApiResult {
    let sub = ctx.db.get_submission(sub_id).await?;
    let url = Url::parse(&sub.link)?;
    let cover_url = get_cover_url_from_url(&ctx.client, url).await?;

    let cover_response = ctx.client.get(cover_url).send().await?.error_for_status()?;

    let mut response_builder = Response::builder().status(cover_response.status());
    *response_builder.headers_mut().unwrap() = cover_response.headers().clone();
    let response = response_builder
        .body(Body::from_stream(cover_response.bytes_stream()))
        .unwrap();

    Ok(response)
}
