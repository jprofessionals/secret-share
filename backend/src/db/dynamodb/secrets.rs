use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use super::DynamoDbRepository;
use crate::db::SecretRepository;
use crate::error::AppError;
use crate::models::Secret;

impl DynamoDbRepository {
    fn item_to_secret(
        &self,
        item: &HashMap<String, AttributeValue>,
    ) -> Result<Secret, AppError> {
        let get_s = |key: &str| -> Result<String, AppError> {
            item.get(key)
                .and_then(|v| v.as_s().ok())
                .map(|s| s.to_string())
                .ok_or_else(|| AppError::DatabaseError(format!("Missing or invalid field: {}", key)))
        };

        let get_n = |key: &str| -> Result<i64, AppError> {
            item.get(key)
                .and_then(|v| v.as_n().ok())
                .and_then(|n| n.parse().ok())
                .ok_or_else(|| AppError::DatabaseError(format!("Missing or invalid field: {}", key)))
        };

        let get_bool = |key: &str| -> Result<bool, AppError> {
            item.get(key)
                .and_then(|v| v.as_bool().ok())
                .copied()
                .ok_or_else(|| AppError::DatabaseError(format!("Missing or invalid field: {}", key)))
        };

        let id = Uuid::parse_str(&get_s("id")?)
            .map_err(|e| AppError::DatabaseError(format!("Invalid UUID: {}", e)))?;

        let created_at = DateTime::parse_from_rfc3339(&get_s("created_at")?)
            .map_err(|e| AppError::DatabaseError(format!("Invalid created_at: {}", e)))?
            .with_timezone(&Utc);

        let expires_at_timestamp = get_n("expires_at")?;
        let expires_at = DateTime::from_timestamp(expires_at_timestamp, 0)
            .ok_or_else(|| AppError::DatabaseError("Invalid expires_at timestamp".to_string()))?;

        let max_views = item
            .get("max_views")
            .and_then(|v| v.as_n().ok())
            .and_then(|n| n.parse().ok());

        Ok(Secret {
            id,
            encrypted_data: get_s("encrypted_data")?,
            created_at,
            expires_at,
            max_views,
            views: get_n("views")? as i32,
            extendable: get_bool("extendable")?,
            failed_attempts: get_n("failed_attempts")? as i32,
        })
    }
}

#[async_trait]
impl SecretRepository for DynamoDbRepository {
    async fn create_secret(&self, secret: &Secret) -> Result<(), AppError> {
        let mut item = vec![
            ("id".to_string(), AttributeValue::S(secret.id.to_string())),
            (
                "encrypted_data".to_string(),
                AttributeValue::S(secret.encrypted_data.clone()),
            ),
            (
                "created_at".to_string(),
                AttributeValue::S(secret.created_at.to_rfc3339()),
            ),
            (
                "expires_at".to_string(),
                AttributeValue::N(secret.expires_at.timestamp().to_string()),
            ),
            ("views".to_string(), AttributeValue::N(secret.views.to_string())),
            (
                "extendable".to_string(),
                AttributeValue::Bool(secret.extendable),
            ),
            (
                "failed_attempts".to_string(),
                AttributeValue::N(secret.failed_attempts.to_string()),
            ),
        ];

        if let Some(max_views) = secret.max_views {
            item.push(("max_views".to_string(), AttributeValue::N(max_views.to_string())));
        }

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item.into_iter().collect()))
            .send()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_secret(&self, id: &Uuid) -> Result<Option<Secret>, AppError> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let Some(item) = result.item else {
            return Ok(None);
        };

        let secret = self.item_to_secret(&item)?;
        Ok(Some(secret))
    }

    async fn update_secret(&self, secret: &Secret) -> Result<(), AppError> {
        self.client
            .update_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(secret.id.to_string()))
            .update_expression("SET #views = :views, #failed_attempts = :failed_attempts")
            .expression_attribute_names("#views", "views")
            .expression_attribute_names("#failed_attempts", "failed_attempts")
            .expression_attribute_values(":views", AttributeValue::N(secret.views.to_string()))
            .expression_attribute_values(
                ":failed_attempts",
                AttributeValue::N(secret.failed_attempts.to_string()),
            )
            .send()
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
        let mut update_expr = "SET #expires_at = :expires_at".to_string();
        let mut expr_names = vec![("#expires_at".to_string(), "expires_at".to_string())];
        let mut expr_values = vec![(
            ":expires_at".to_string(),
            AttributeValue::N(new_expires_at.timestamp().to_string()),
        )];

        if let Some(max_views) = new_max_views {
            update_expr.push_str(", #max_views = :max_views");
            expr_names.push(("#max_views".to_string(), "max_views".to_string()));
            expr_values.push((":max_views".to_string(), AttributeValue::N(max_views.to_string())));
        }

        let mut req = self
            .client
            .update_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .update_expression(update_expr);

        for (name, value) in expr_names {
            req = req.expression_attribute_names(name, value);
        }

        for (name, value) in expr_values {
            req = req.expression_attribute_values(name, value);
        }

        req.send()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_secret(&self, id: &Uuid) -> Result<(), AppError> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn cleanup_expired(&self) -> Result<u64, AppError> {
        // DynamoDB uses TTL for automatic cleanup
        tracing::info!("DynamoDB handles cleanup via TTL - no action needed");
        Ok(0)
    }
}
