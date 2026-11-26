use crate::{
    api::{ApiCtx, ApiResult, utils},
    db::{CommunityContent, DbResult},
};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use chrono::NaiveDate;
use futures::{StreamExt, TryStreamExt, stream};
use serde::Serialize;
use url::Url;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CommunityContentResponse {
    id: String,
    link: String,
    author: String,
    label: String,
    description: String,
}

impl From<CommunityContent> for CommunityContentResponse {
    fn from(value: CommunityContent) -> Self {
        Self {
            id: value.name,
            link: value.link,
            author: value.author,
            label: value.label,
            description: value.description,
        }
    }
}

pub async fn get_gamejams(State(ctx): State<ApiCtx>) -> ApiResult {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct GamejamResponse {
        id: String,
        title: String,
        start_date: NaiveDate,
        end_date: NaiveDate,
        link: String,
        content: Vec<CommunityContentResponse>,
        cover_link: String,
        results: Option<()>,
    }

    let gamejams = ctx.db.get_gamejams().await?;
    let response: Vec<_> = stream::iter(gamejams)
        .then(async |base| -> DbResult<_> {
            let cover_link = format!("https://oknoweb.ru/api/gamejams/{}/cover", base.name);

            Ok(GamejamResponse {
                id: base.name,
                title: base.title,
                start_date: base.start_date,
                end_date: base.end_date,
                link: base.link,
                content: ctx
                    .db
                    .get_all_gamejam_content(base.id)
                    .await?
                    .into_iter()
                    .map(CommunityContentResponse::from)
                    .collect(),
                cover_link,
                results: None,
            })
        })
        .try_collect()
        .await?;

    Ok(Json(response).into_response())
}

pub async fn get_gamejam_cover(State(ctx): State<ApiCtx>, Path(id): Path<String>) -> ApiResult {
    let gamejam = ctx.db.get_gamejam_by_name(&id).await?;
    let gamejam_url = Url::parse(&gamejam.link)?;
    let cover_url = utils::get_cover_url_from_meta(&ctx.client, gamejam_url).await?;

    utils::proxy(&ctx.client, cover_url).await
}

pub async fn get_all_community_content(State(ctx): State<ApiCtx>) -> ApiResult {
    let content = ctx.db.get_all_community_content().await?;
    let response: Vec<_> = content
        .into_iter()
        .map(CommunityContentResponse::from)
        .collect();
    Ok(Json(response).into_response())
}

pub async fn get_community_content(State(ctx): State<ApiCtx>, Path(id): Path<String>) -> ApiResult {
    let content = ctx.db.get_community_content_by_name(&id).await?;
    let response = CommunityContentResponse::from(content);
    Ok(Json(response).into_response())
}

pub async fn get_community_content_cover(
    State(ctx): State<ApiCtx>,
    Path(id): Path<String>,
) -> ApiResult {
    let content = ctx.db.get_community_content_by_name(&id).await?;
    let url = content.link.parse()?;
    let url = utils::get_cover_url_from_meta(&ctx.client, url).await?;

    utils::proxy(&ctx.client, url).await
}
