use crate::api::{ApiError, ApiResult};
use axum::{body::Body, http::Response};
use reqwest::Client;
use scraper::{Html, Selector};
use url::Url;

pub async fn get_cover_url_from_meta(client: &Client, url: Url) -> ApiResult<Url> {
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

pub async fn proxy(client: &Client, url: Url) -> ApiResult {
    let origin = client.get(url).send().await?;

    let mut response_builder = Response::builder().status(origin.status());
    *response_builder.headers_mut().unwrap() = origin.headers().clone();
    let response = response_builder
        .body(Body::from_stream(origin.bytes_stream()))
        .unwrap();

    Ok(response)
}
