mod gamejams;
mod submit;

use sqlx::{PgPool, migrate::MigrateError};
use thiserror::Error;

pub use gamejams::*;
pub use submit::*;

#[derive(Debug, Error)]
#[error("failed to connect to database: {0}")]
pub enum DbConnectionError {
    ConnectionError(#[from] sqlx::Error),
    MigrationError(#[from] MigrateError),
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),
}

impl DbError {
    pub fn is_not_found(&self) -> bool {
        matches!(self, DbError::DatabaseError(sqlx::Error::RowNotFound))
    }
}

pub type DbResult<T> = Result<T, DbError>;

pub fn convert_result<T>(base: sqlx::Result<T>) -> DbResult<T> {
    match base {
        Ok(v) => Ok(v),
        Err(e) => Err(e.into()),
    }
}

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self, DbConnectionError> {
        let pool = PgPool::connect(url).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Database { pool })
    }
}

#[macro_export]
macro_rules! query_as_one (
    ($db:expr, $out_struct:path, $query:expr) => (
        $crate::db::convert_result(
            sqlx::query_as!($out_struct, $query)
                .fetch_one(&$db.pool)
                .await
        )
    );
    ($db:expr, $out_struct:path, $query:expr, $($args:tt)*) => (
        $crate::db::convert_result(
            sqlx::query_as!($out_struct, $query, $($args)*)
                .fetch_one(&$db.pool)
                .await
        )
    )
);

#[macro_export]
macro_rules! query_as_all (
    ($db:expr, $out_struct:path, $query:expr) => (
        $crate::db::convert_result(
            sqlx::query_as!($out_struct, $query)
                .fetch_all(&$db.pool)
                .await
        )
    );
    ($db:expr, $out_struct:path, $query:expr, $($args:tt)*) => (
        $crate::db::convert_result(
            sqlx::query_as!($out_struct, $query, $($args)*)
                .fetch_all(&$db.pool)
                .await
        )
    )
);

#[macro_export]
macro_rules! query_scalar_one (
    ($db:expr, $query:expr) => (
        $crate::db::convert_result(
            sqlx::query_scalar!($query)
                .fetch_one(&$db.pool)
                .await
        )
    );
    ($db:expr, $query:expr, $($args:tt)*) => (
        $crate::db::convert_result(
            sqlx::query_scalar!($query, $($args)*)
                .fetch_one(&$db.pool)
                .await
        )
    )
);

#[macro_export]
macro_rules! query_scalar_all (
    ($db:expr, $query:expr) => (
        $crate::db::convert_result(
            sqlx::query_scalar!($query)
                .fetch_all(&$db.pool)
                .await
        )
    );
    ($db:expr, $query:expr, $($args:tt)*) => (
        $crate::db::convert_result(
            sqlx::query_scalar!($query, $($args)*)
                .fetch_all(&$db.pool)
                .await
        )
    )
);
