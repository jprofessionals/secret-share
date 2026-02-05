use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{client::Waiters, config::Builder, Client};

use crate::error::AppError;

mod secrets;

pub struct DynamoDbRepository {
    pub(crate) client: Client,
    pub(crate) table_name: String,
}

impl DynamoDbRepository {
    pub async fn new(table_name: &str, endpoint: Option<&str>) -> Result<Self, AppError> {
        let client = if let Some(endpoint_url) = endpoint {
            // For local endpoint (DynamoDB Local), build config directly with dummy credentials
            // DynamoDB Local doesn't validate credentials but the SDK requires them
            let dynamo_config = Builder::new()
                .endpoint_url(endpoint_url)
                .region(aws_sdk_dynamodb::config::Region::new("us-east-1"))
                .credentials_provider(aws_sdk_dynamodb::config::Credentials::new(
                    "test",
                    "test",
                    None,
                    None,
                    "test",
                ))
                .behavior_version(aws_sdk_dynamodb::config::BehaviorVersion::latest())
                .build();
            Client::from_conf(dynamo_config)
        } else {
            // For production, use standard AWS credentials chain
            let config = aws_config::defaults(BehaviorVersion::latest()).load().await;
            Client::new(&config)
        };

        let repo = Self {
            client,
            table_name: table_name.to_string(),
        };

        // Create table if using local endpoint (development/testing)
        if endpoint.is_some() {
            repo.ensure_table_exists().await?;
        }

        Ok(repo)
    }

    async fn ensure_table_exists(&self) -> Result<(), AppError> {
        use aws_sdk_dynamodb::types::{
            AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType,
        };

        // Check if table exists
        let tables = self
            .client
            .list_tables()
            .send()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if tables
            .table_names()
            .iter()
            .any(|name| name == &self.table_name)
        {
            return Ok(());
        }

        // Create table
        self.client
            .create_table()
            .table_name(&self.table_name)
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("id")
                    .key_type(KeyType::Hash)
                    .build()
                    .map_err(|e| AppError::DatabaseError(e.to_string()))?,
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("id")
                    .attribute_type(ScalarAttributeType::S)
                    .build()
                    .map_err(|e| AppError::DatabaseError(e.to_string()))?,
            )
            .provisioned_throughput(
                ProvisionedThroughput::builder()
                    .read_capacity_units(5)
                    .write_capacity_units(5)
                    .build()
                    .map_err(|e| AppError::DatabaseError(e.to_string()))?,
            )
            .send()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Wait for table to be active
        self.client
            .wait_until_table_exists()
            .table_name(&self.table_name)
            .wait(std::time::Duration::from_secs(30))
            .await
            .map_err(|e| AppError::DatabaseError(format!("Table creation timeout: {}", e)))?;

        Ok(())
    }
}
