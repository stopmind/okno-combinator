use crate::{
    db::{Database, DbResult, convert_result},
    query_as_all, query_as_one, query_scalar_one,
};
use sqlx::query;

pub type UserId = i32;

pub struct User {
    pub id: UserId,
    pub username: String,
    pub password: String,
    pub locked: bool,
    pub admin: bool,
}

impl Database {
    pub async fn create_user(
        &self,
        username: &str,
        password: &str,
        admin: bool,
    ) -> DbResult<UserId> {
        query_scalar_one!(
            self,
            "INSERT INTO users (username, password, admin) VALUES ($1, $2, $3) RETURNING id",
            username,
            password,
            admin
        )
    }

    pub async fn get_user(&self, id: UserId) -> DbResult<User> {
        query_as_one!(self, User, "SELECT * FROM users WHERE id = $1", id)
    }

    pub async fn get_user_by_name(&self, name: &str) -> DbResult<User> {
        query_as_one!(self, User, "SELECT * FROM users WHERE username = $1", name,)
    }

    pub async fn get_all_users(&self) -> DbResult<Vec<User>> {
        query_as_all!(self, User, "SELECT * FROM users")
    }

    pub async fn delete_user(&self, id: UserId) -> DbResult<()> {
        convert_result(
            query!("DELETE FROM users WHERE id = $1", id)
                .execute(&self.pool)
                .await,
        )
        .map(|_| ())
    }

    pub async fn set_user_lock(&self, id: UserId, locked: bool) -> DbResult<()> {
        convert_result(
            query!("UPDATE users SET locked = $2 WHERE id = $1", id, locked)
                .execute(&self.pool)
                .await,
        )
        .map(|_| ())
    }

    pub async fn set_user_password(&self, id: UserId, password: &str) -> DbResult<()> {
        convert_result(
            query!("UPDATE users SET password = $2 WHERE id = $1", id, password)
                .execute(&self.pool)
                .await,
        )
        .map(|_| ())
    }
}
