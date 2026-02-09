use secret_share_backend::{build_app, config::Config};

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    lambda_http::tracing::init_default_subscriber();

    tracing::info!("Starting Secret Share Lambda");

    let config = Config::from_env()
        .expect("Failed to load config");

    let (app, _state) = build_app(config).await
        .expect("Failed to build app");

    lambda_http::run(app).await
}
