use crate::AppState;
use crate::repositories::jira_servers as server_repo;
use crate::services::jira as jira_service;
use crate::utils::encryption::encrypt;
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct JiraServerResponse {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub username: String,
}

#[instrument(skip(state))]
pub async fn list_servers(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Vec<JiraServerResponse>>, (StatusCode, String)> {
    let servers = server_repo::list_servers_by_user(&state.pool, user_id)
        .await
        .map_err(|e| {
            error!(error = ?e, user_id = %user_id, "Failed to list servers from database");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    let response = servers
        .into_iter()
        .map(|s| JiraServerResponse {
            id: s.id,
            name: s.name,
            url: s.url,
            username: s.username,
        })
        .collect();

    Ok(Json(response))
}

#[instrument(skip(state, payload))]
pub async fn create_server(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<CreateServerRequest>,
) -> Result<Json<JiraServerResponse>, (StatusCode, String)> {
    let id = Uuid::new_v4();
    let encrypted_password = encrypt(&payload.password);

    server_repo::create_server(
        &state.pool,
        id,
        user_id,
        &payload.name,
        &payload.url,
        &payload.username,
        &encrypted_password,
    )
    .await
    .map_err(|e| {
        error!(error = ?e, user_id = %user_id, server_name = %payload.name, "Failed to create server in database");
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    info!(user_id = %user_id, server_id = %id, server_name = %payload.name, "Jira server created successfully");
    Ok(Json(JiraServerResponse {
        id,
        name: payload.name,
        url: payload.url,
        username: payload.username,
    }))
}

#[instrument(skip(state))]
pub async fn delete_server(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(server_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let rows_affected = server_repo::delete_server(&state.pool, server_id, user_id)
        .await
        .map_err(|e| {
            error!(error = ?e, user_id = %user_id, server_id = %server_id, "Failed to delete server from database");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    if rows_affected == 0 {
        debug!(user_id = %user_id, server_id = %server_id, "Attempted to delete non-existent server");
        return Err((StatusCode::NOT_FOUND, "Server not found".to_string()));
    }

    info!(user_id = %user_id, server_id = %server_id, "Jira server deleted successfully");
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct TestServerRequest {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[instrument(skip(payload))]
pub async fn test_new_server_credentials(
    Json(payload): Json<TestServerRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    jira_service::test_connection_params(&payload.url, &payload.username, &payload.password)
        .await
        .map_err(|e| {
            debug!(error = %e, url = %payload.url, "Jira connection test failed for new server");
            (StatusCode::BAD_GATEWAY, e)
        })?;

    info!(url = %payload.url, "Jira connection test successful for new server");
    Ok(StatusCode::OK)
}

#[instrument(skip(state))]
pub async fn test_credentials(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(server_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let server = server_repo::find_server_by_id(&state.pool, server_id, user_id)
        .await
        .map_err(|e| {
            error!(error = ?e, user_id = %user_id, server_id = %server_id, "Failed to fetch server for credential test");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?
        .ok_or_else(|| {
            debug!(user_id = %user_id, server_id = %server_id, "Credential test attempted for non-existent server");
            (StatusCode::NOT_FOUND, "Server not found".to_string())
        })?;

    jira_service::test_connection(&server)
        .await
        .map_err(|e| {
            debug!(error = %e, user_id = %user_id, server_id = %server_id, "Jira connection test failed for existing server");
            (StatusCode::BAD_GATEWAY, e)
        })?;

    info!(user_id = %user_id, server_id = %server_id, "Jira connection test successful for existing server");
    Ok(StatusCode::OK)
}
