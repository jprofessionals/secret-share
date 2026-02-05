use secret_share_backend::{
    config::{Config, DatabaseConfig},
    create_router,
    db::{DynamoDbRepository, SecretRepository},
    AppState,
};
use std::sync::Arc;
use testcontainers::{runners::AsyncRunner, ContainerAsync, GenericImage};
use tokio::net::TcpListener;

#[allow(dead_code)]
pub struct DynamoDbTestContext {
    pub base_url: String,
    pub client: reqwest::Client,
    pub db: Arc<dyn SecretRepository>,
    _container: ContainerAsync<GenericImage>,
}

#[allow(dead_code)]
impl DynamoDbTestContext {
    pub async fn new() -> Self {
        let container = GenericImage::new("amazon/dynamodb-local", "latest")
            .with_exposed_port(8000.into())
            .start()
            .await
            .expect("Failed to start DynamoDB Local container");

        let host = container.get_host().await.expect("Failed to get host");
        let port = container
            .get_host_port_ipv4(8000)
            .await
            .expect("Failed to get port");

        let endpoint = format!("http://{}:{}", host, port);
        let table_name = "secrets-test";

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let dynamo = DynamoDbRepository::new(table_name, Some(&endpoint))
            .await
            .expect("Failed to connect to DynamoDB");

        let config = Config {
            database: DatabaseConfig::DynamoDB {
                table: table_name.to_string(),
                endpoint: Some(endpoint),
            },
            base_url: "http://localhost".to_string(),
            port: 3000,
            max_secret_days: 30,
            max_secret_views: 100,
            max_failed_attempts: 10,
        };

        let db: Arc<dyn SecretRepository> = Arc::new(dynamo);
        let state = AppState {
            db: db.clone(),
            config: Arc::new(config),
        };

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to random port");
        let addr = listener.local_addr().expect("Failed to get local address");
        let base_url = format!("http://{}", addr);

        let app = create_router(state);

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        DynamoDbTestContext {
            base_url,
            client: reqwest::Client::new(),
            db,
            _container: container,
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}
