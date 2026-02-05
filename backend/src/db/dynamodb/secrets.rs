use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::DynamoDbRepository;
use crate::db::SecretRepository;
use crate::error::AppError;
use crate::models::Secret;

#[async_trait]
impl SecretRepository for DynamoDbRepository {
    async fn create_secret(&self, _secret: &Secret) -> Result<(), AppError> {
        todo!("Implement create_secret for DynamoDB")
    }

    async fn get_secret(&self, _id: &Uuid) -> Result<Option<Secret>, AppError> {
        todo!("Implement get_secret for DynamoDB")
    }

    async fn update_secret(&self, _secret: &Secret) -> Result<(), AppError> {
        todo!("Implement update_secret for DynamoDB")
    }

    async fn extend_secret(
        &self,
        _id: &Uuid,
        _expires_at: DateTime<Utc>,
        _max_views: Option<i32>,
    ) -> Result<(), AppError> {
        todo!("Implement extend_secret for DynamoDB")
    }

    async fn delete_secret(&self, _id: &Uuid) -> Result<(), AppError> {
        todo!("Implement delete_secret for DynamoDB")
    }

    async fn cleanup_expired(&self) -> Result<u64, AppError> {
        // DynamoDB uses TTL for automatic cleanup
        tracing::info!("DynamoDB handles cleanup via TTL - no action needed");
        Ok(0)
    }
}
