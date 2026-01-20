use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use crate::repositories::jira_servers as server_repo;
use crate::services::jira as jira_service;
use crate::utils::encryption::encrypt;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
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

pub async fn list_servers(
    State(pool): State<MySqlPool>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Vec<JiraServerResponse>>, (StatusCode, String)> {
    let servers = server_repo::list_servers_by_user(&pool, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

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

pub async fn create_server(
    State(pool): State<MySqlPool>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<CreateServerRequest>,
) -> Result<Json<JiraServerResponse>, (StatusCode, String)> {
    let id = Uuid::new_v4();
    let encrypted_password = encrypt(&payload.password);

    server_repo::create_server(
        &pool,
        id,
        user_id,
        &payload.name,
        &payload.url,
        &payload.username,
        &encrypted_password,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(JiraServerResponse {
        id,
        name: payload.name,
        url: payload.url,
        username: payload.username,
    }))
}

pub async fn delete_server(
    State(pool): State<MySqlPool>,
    Extension(user_id): Extension<Uuid>,
    Path(server_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let rows_affected = server_repo::delete_server(&pool, server_id, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if rows_affected == 0 {
        return Err((StatusCode::NOT_FOUND, "Server not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn test_credentials(
    State(pool): State<MySqlPool>,
    Extension(user_id): Extension<Uuid>,
    Path(server_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let server = server_repo::find_server_by_id(&pool, server_id, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Server not found".to_string()))?;

    jira_service::test_connection(&server)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e))?;

    Ok(StatusCode::OK)
}
