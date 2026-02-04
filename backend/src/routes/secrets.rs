use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{
    CreateSecretRequest, CreateSecretResponse, ExtendSecretRequest, ExtendSecretResponse,
    RetrieveSecretRequest, RetrieveSecretResponse,
};
use crate::services::secrets as service;
use crate::AppState;

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = String)
    ),
    tag = "health"
)]
pub async fn health_check() -> &'static str {
    "OK"
}

#[utoipa::path(
    post,
    path = "/api/secrets",
    request_body = CreateSecretRequest,
    responses(
        (status = 200, description = "Secret created successfully", body = CreateSecretResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "secrets"
)]
pub async fn create_secret(
    State(state): State<AppState>,
    Json(payload): Json<CreateSecretRequest>,
) -> Result<Json<CreateSecretResponse>, AppError> {
    tracing::info!("Creating new secret");
    let response = service::create(&state.db, &state.config, payload).await?;
    tracing::info!("Secret created with id: {}", response.id);
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/secrets/{id}",
    params(
        ("id" = Uuid, Path, description = "Secret ID")
    ),
    request_body = RetrieveSecretRequest,
    responses(
        (status = 200, description = "Secret retrieved successfully", body = RetrieveSecretResponse),
        (status = 401, description = "Invalid passphrase"),
        (status = 404, description = "Secret not found"),
        (status = 410, description = "Secret expired or max views reached")
    ),
    tag = "secrets"
)]
pub async fn retrieve_secret(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RetrieveSecretRequest>,
) -> Result<Json<RetrieveSecretResponse>, AppError> {
    tracing::info!("Retrieving secret: {}", id);
    let response = service::retrieve(&state.db, &state.config, id, &payload.passphrase).await?;
    tracing::info!("Secret retrieved successfully");
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/secrets/{id}/extend",
    params(
        ("id" = Uuid, Path, description = "Secret ID")
    ),
    request_body = ExtendSecretRequest,
    responses(
        (status = 200, description = "Secret extended successfully", body = ExtendSecretResponse),
        (status = 400, description = "Invalid request or exceeds limits"),
        (status = 401, description = "Invalid passphrase"),
        (status = 403, description = "Secret is not extendable"),
        (status = 404, description = "Secret not found")
    ),
    tag = "secrets"
)]
pub async fn extend_secret(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ExtendSecretRequest>,
) -> Result<Json<ExtendSecretResponse>, AppError> {
    tracing::info!("Extending secret: {}", id);
    let response = service::extend(&state.db, &state.config, id, payload).await?;
    tracing::info!("Secret extended successfully");
    Ok(Json(response))
}
