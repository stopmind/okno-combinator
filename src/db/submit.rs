use super::{Database, DbResult, convert_result};
use crate::{query_as_all, query_as_one, query_scalar_all, query_scalar_one};
use chrono::NaiveDate;
use serde::Serialize;
use sqlx::Type;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Type, Serialize)]
#[serde(rename_all = "camelCase")]
#[sqlx(rename_all = "camelCase", type_name = "submission_status")]
pub enum SubmissionStatus {
    Pending,
    Unverified,
    Reviewed,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Submission {
    pub id: i32,
    pub status: SubmissionStatus,
    pub name: String,
    pub link: String,
    pub date: NaiveDate,
    pub author: String,
    pub additional_info: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    pub id: i32,
    pub submission_id: i32,
    pub author: String,
    pub link: String,
    pub rating: i32,
}

#[derive(Debug, Serialize)]
pub struct Tag {
    pub tag: String,
    pub category: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagCategory {
    #[serde(rename = "category")]
    pub title: String,
    pub panel_only: bool,
    pub order: i32,
}

impl Database {
    pub async fn create_submission(
        &self,
        status: SubmissionStatus,
        name: &str,
        link: &str,
        author: &str,
        additional_info: Option<&str>,
    ) -> DbResult<(i32, NaiveDate)> {
        struct Return {
            id: i32,
            date: NaiveDate,
        }
        query_as_one!(
            self, Return,
            "INSERT INTO submissions (status, name, link, author, additional_info) VALUES ($1, $2, $3, $4, $5) RETURNING id, date",
            status, name, link, author, additional_info
        ).map(|r| (r.id, r.date))
    }

    pub async fn delete_submission(&self, sub_id: i32) -> DbResult<()> {
        convert_result(
            sqlx::query!("DELETE FROM submissions WHERE id = $1", sub_id)
                .execute(&self.pool)
                .await
                .map(|_| ()),
        )
    }

    pub async fn set_submission_status(
        &self,
        sub_id: i32,
        status: SubmissionStatus,
    ) -> DbResult<()> {
        convert_result(
            sqlx::query!(
                "UPDATE submissions SET status = $1 WHERE id = $2",
                status,
                sub_id
            )
            .execute(&self.pool)
            .await
            .map(|_| ()),
        )
    }

    pub async fn add_submission_tags(&self, sub_id: i32, tags: &[i32]) -> DbResult<()> {
        convert_result(
            sqlx::query!("INSERT INTO submission_tag (submission_id, tag_id) SELECT $1, x FROM UNNEST($2::INT[]) x", sub_id, tags)
                .execute(&self.pool)
                .await
                .map(|_| ())
        )
    }

    pub async fn edit_submission(
        &self,
        sub_id: i32,
        name: &str,
        link: &str,
        author: &str,
        additional_info: Option<&str>,
        date: NaiveDate,
    ) -> DbResult<()> {
        convert_result(
            sqlx::query!("UPDATE submissions SET name = $2, link = $3, author = $4, additional_info = $5, date = $6 WHERE id = $1",
                sub_id, name, link, author, additional_info, date
            )
                .execute(&self.pool)
                .await
                .map(|_| ())
        )
    }

    pub async fn remove_all_submission_tags(&self, sub_id: i32) -> DbResult<()> {
        convert_result(
            sqlx::query!(
                "DELETE FROM submission_tag WHERE submission_id = $1",
                sub_id
            )
            .execute(&self.pool)
            .await
            .map(|_| ()),
        )
    }

    pub async fn get_tag_id_by_title(&self, title: &str) -> DbResult<i32> {
        query_scalar_one!(self, "SELECT id FROM tags WHERE title = $1", title)
    }

    pub async fn get_all_submissions(&self) -> DbResult<Vec<Submission>> {
        query_as_all!(self, Submission, "SELECT * FROM submissions")
    }

    pub async fn get_submission(&self, sub_id: i32) -> DbResult<Submission> {
        query_as_one!(
            self,
            Submission,
            "SELECT * FROM submissions WHERE id = $1",
            sub_id
        )
    }

    pub async fn get_submission_tags_titles(&self, sub_id: i32) -> DbResult<Vec<String>> {
        query_scalar_all!(
            self,
            "SELECT title FROM tags INNER JOIN submission_tag ON tags.id = tag_id WHERE submission_id = $1",
            sub_id
        )
    }

    pub async fn count_submissions_with_status(&self, status: SubmissionStatus) -> DbResult<u32> {
        query_scalar_one!(
            self,
            "SELECT COUNT(*) FROM submissions WHERE status = $1",
            status
        )
        .map(|count| count.unwrap_or(0) as u32)
    }

    pub async fn count_submissions_without_status(
        &self,
        status: SubmissionStatus,
    ) -> DbResult<u32> {
        query_scalar_one!(
            self,
            "SELECT COUNT(*) FROM submissions WHERE status != $1",
            status
        )
        .map(|count| count.unwrap_or(0) as u32)
    }

    pub async fn get_submissions_with_status(
        &self,
        status: SubmissionStatus,
    ) -> DbResult<Vec<Submission>> {
        query_as_all!(
            self,
            Submission,
            "SELECT * FROM submissions WHERE status = $1",
            status
        )
    }

    pub async fn get_submissions_without_status(
        &self,
        status: SubmissionStatus,
    ) -> DbResult<Vec<Submission>> {
        query_as_all!(
            self,
            Submission,
            "SELECT * FROM submissions WHERE status != $1",
            status
        )
    }

    pub async fn get_submission_reviews(&self, sub_id: i32) -> DbResult<Vec<Review>> {
        query_as_all!(
            self,
            Review,
            "SELECT * FROM reviews WHERE submission_id = $1",
            sub_id
        )
    }

    pub async fn submission_exists(&self, sub_id: i32) -> DbResult<bool> {
        query_scalar_one!(
            self,
            "SELECT EXISTS(SELECT 1 FROM submissions WHERE id = $1)",
            sub_id
        )
        .map(Option::unwrap_or_default)
    }

    pub async fn get_tags(&self) -> DbResult<Vec<Tag>> {
        query_as_all!(
            self,
            Tag,
            "SELECT tags.title AS tag, tag_categories.title AS category FROM tag_categories JOIN tags on tag_categories.id = tags.category_id"
        )
    }

    pub async fn get_submissions_with_tag(&self, tag_id: i32) -> DbResult<Vec<Submission>> {
        query_as_all!(
            self,
            Submission,
            "SELECT id, status, name, link, date, author, additional_info FROM submissions INNER JOIN submission_tag ON submission_id = submissions.id WHERE tag_id = $1",
            tag_id
        )
    }

    pub async fn get_tags_categories(&self) -> DbResult<Vec<TagCategory>> {
        query_as_all!(
            self,
            TagCategory,
            "SELECT title, panel_only, \"order\" FROM tag_categories"
        )
    }

    pub async fn get_tags_categories_no_panel(&self) -> DbResult<Vec<TagCategory>> {
        query_as_all!(
            self,
            TagCategory,
            "SELECT title, panel_only, \"order\" FROM tag_categories WHERE panel_only = false"
        )
    }

    pub async fn get_tags_category_by_title(&self, title: &str) -> DbResult<TagCategory> {
        query_as_one!(
            self,
            TagCategory,
            "SELECT title, panel_only, \"order\" FROM tag_categories WHERE title = $1",
            title
        )
    }

    pub async fn create_submission_review(
        &self,
        sub_id: i32,
        author: &str,
        link: &str,
        rating: i32,
    ) -> DbResult<i32> {
        query_scalar_one!(
            self,
            "INSERT INTO reviews (submission_id, author, link, rating) VALUES ($1, $2, $3, $4) RETURNING id",
            sub_id,
            author,
            link,
            rating
        )
    }

    pub async fn delete_submission_review(&self, review_id: i32) -> DbResult<()> {
        convert_result(
            sqlx::query!("DELETE FROM reviews WHERE id = $1", review_id)
                .execute(&self.pool)
                .await
                .map(|_| ()),
        )
    }
}
