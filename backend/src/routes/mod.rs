mod secrets;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::models::{
    CreateSecretRequest, CreateSecretResponse, ExtendSecretRequest, ExtendSecretResponse,
    RetrieveSecretRequest, RetrieveSecretResponse, Secret,
};
use crate::AppState;

pub use secrets::{create_secret, extend_secret, health_check, retrieve_secret};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "SecretShare API",
        description = "Secure secret sharing with end-to-end encryption",
        version = "0.1.0"
    ),
    paths(
        secrets::health_check,
        secrets::create_secret,
        secrets::retrieve_secret,
        secrets::extend_secret
    ),
    components(schemas(
        CreateSecretRequest,
        CreateSecretResponse,
        RetrieveSecretRequest,
        RetrieveSecretResponse,
        ExtendSecretRequest,
        ExtendSecretResponse,
        Secret
    )),
    tags(
        (name = "secrets", description = "Secret management endpoints"),
        (name = "health", description = "Health check endpoints")
    )
)]
struct ApiDoc;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/secrets", post(create_secret))
        .route("/api/secrets/{id}", post(retrieve_secret))
        .route("/api/secrets/{id}/extend", post(extend_secret))
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi()),
        )
        .layer(CorsLayer::permissive())
        .with_state(state)
}
