use secret_share_backend::{
    config::{Config, DatabaseConfig},
    create_router,
    db::{PostgresRepository, SecretRepository},
    AppState,
};
use std::sync::Arc;
use testcontainers::{runners::AsyncRunner, ContainerAsync};
use testcontainers_modules::postgres::Postgres;
use tokio::net::TcpListener;

#[allow(dead_code)]
pub struct PostgresTestContext {
    pub base_url: String,
    pub client: reqwest::Client,
    pub db: Arc<dyn SecretRepository>,
    _container: ContainerAsync<Postgres>,
}

#[allow(dead_code)]
impl PostgresTestContext {
    pub async fn new() -> Self {
        let container = Postgres::default()
            .start()
            .await
            .expect("Failed to start PostgreSQL container");

        let host = container.get_host().await.expect("Failed to get host");
        let port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("Failed to get port");

        let database_url = format!("postgres://postgres:postgres@{}:{}/postgres", host, port);
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let pg = PostgresRepository::new(&database_url)
            .await
            .expect("Failed to connect to database");
        pg.migrate().await.expect("Failed to run migrations");

        let config = Config {
            database: DatabaseConfig::Postgres {
                url: database_url.clone(),
            },
            base_url: "http://localhost".to_string(),
            port: 3000,
            max_secret_days: 30,
            max_secret_views: 100,
            max_failed_attempts: 10,
        };

        let db: Arc<dyn SecretRepository> = Arc::new(pg);
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

        PostgresTestContext {
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
