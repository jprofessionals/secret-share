use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::Secret;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SecretRepository: Send + Sync {
    async fn create_secret(&self, secret: &Secret) -> Result<(), AppError>;
    async fn get_secret(&self, id: &Uuid) -> Result<Option<Secret>, AppError>;
    async fn update_secret(&self, secret: &Secret) -> Result<(), AppError>;
    async fn extend_secret(
        &self,
        id: &Uuid,
        expires_at: DateTime<Utc>,
        max_views: Option<i32>,
    ) -> Result<(), AppError>;
    async fn delete_secret(&self, id: &Uuid) -> Result<(), AppError>;
    async fn cleanup_expired(&self) -> Result<u64, AppError>;
}

#[async_trait]
impl<T: SecretRepository + ?Sized> SecretRepository for Arc<T> {
    async fn create_secret(&self, secret: &Secret) -> Result<(), AppError> {
        (**self).create_secret(secret).await
    }

    async fn get_secret(&self, id: &Uuid) -> Result<Option<Secret>, AppError> {
        (**self).get_secret(id).await
    }

    async fn update_secret(&self, secret: &Secret) -> Result<(), AppError> {
        (**self).update_secret(secret).await
    }

    async fn extend_secret(
        &self,
        id: &Uuid,
        expires_at: DateTime<Utc>,
        max_views: Option<i32>,
    ) -> Result<(), AppError> {
        (**self).extend_secret(id, expires_at, max_views).await
    }

    async fn delete_secret(&self, id: &Uuid) -> Result<(), AppError> {
        (**self).delete_secret(id).await
    }

    async fn cleanup_expired(&self) -> Result<u64, AppError> {
        (**self).cleanup_expired().await
    }
}
