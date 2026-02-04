use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::repository::SecretRepository;
use super::Database;
use crate::error::AppError;
use crate::models::Secret;

#[async_trait]
impl SecretRepository for Database {
    async fn create_secret(&self, secret: &Secret) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO secrets (id, encrypted_data, created_at, expires_at, max_views, views, extendable, failed_attempts)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&secret.id)
        .bind(&secret.encrypted_data)
        .bind(&secret.created_at)
        .bind(&secret.expires_at)
        .bind(&secret.max_views)
        .bind(&secret.views)
        .bind(&secret.extendable)
        .bind(&secret.failed_attempts)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_secret(&self, id: &Uuid) -> Result<Option<Secret>, AppError> {
        let secret = sqlx::query_as::<_, Secret>(
            r#"
            SELECT id, encrypted_data, created_at, expires_at, max_views, views, extendable, failed_attempts
            FROM secrets
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(secret)
    }

    async fn update_secret(&self, secret: &Secret) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE secrets
            SET views = $2, failed_attempts = $3
            WHERE id = $1
            "#,
        )
        .bind(&secret.id)
        .bind(&secret.views)
        .bind(&secret.failed_attempts)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn extend_secret(
        &self,
        id: &Uuid,
        new_expires_at: DateTime<Utc>,
        new_max_views: Option<i32>,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE secrets
            SET expires_at = $2, max_views = $3
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(new_expires_at)
        .bind(new_max_views)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_secret(&self, id: &Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM secrets WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn cleanup_expired(&self) -> Result<u64, AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM secrets
            WHERE expires_at < NOW()
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected())
    }
}
