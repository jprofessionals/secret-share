use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::error::AppError;

mod secrets;

pub struct PostgresRepository {
    pub(crate) pool: PgPool,
}

impl PostgresRepository {
    pub async fn new(database_url: &str) -> Result<Self, AppError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), AppError> {
        sqlx::migrate!()
            .run(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
