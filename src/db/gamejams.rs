use super::{Database, DbResult};
use crate::{query_as_all, query_as_one};
use chrono::NaiveDate;
use serde::Serialize;

#[derive(Debug)]
pub struct Gamejam {
    pub id: i32,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub link: String,
    pub title: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunityContent {
    pub id: i32,
    pub name: String,
    pub link: String,
    pub author: String,
    pub label: String,
    pub description: String,
}

impl Database {
    pub async fn get_gamejams(&self) -> DbResult<Vec<Gamejam>> {
        query_as_all!(self, Gamejam, "SELECT * FROM gamejams",)
    }

    pub async fn get_all_gamejam_content(
        &self,
        gamejam_id: i32,
    ) -> DbResult<Vec<CommunityContent>> {
        query_as_all!(
            self,
            CommunityContent,
            "SELECT community_content.id, name, link, author, label, description FROM community_content INNER JOIN gamejam_content ON community_content.id = content_id WHERE gamejam_id = $1",
            gamejam_id
        )
    }

    pub async fn get_all_community_content(&self) -> DbResult<Vec<CommunityContent>> {
        query_as_all!(self, CommunityContent, "SELECT * FROM community_content")
    }

    pub async fn get_gamejam_by_name(&self, name: &str) -> DbResult<Gamejam> {
        query_as_one!(
            self,
            Gamejam,
            "SELECT * FROM gamejams WHERE name = $1",
            name
        )
    }

    pub async fn get_community_content_by_name(&self, name: &str) -> DbResult<CommunityContent> {
        query_as_one!(
            self,
            CommunityContent,
            "SELECT * FROM community_content WHERE name = $1",
            name,
        )
    }
}
