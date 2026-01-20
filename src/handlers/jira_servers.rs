use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use crate::models::JiraServer;
use crate::utils::encryption::{decrypt, encrypt};
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
    let servers = sqlx::query_as::<_, JiraServer>(
        "SELECT id, user_id, name, url, username, encrypted_password, created_at, updated_at FROM jira_servers WHERE user_id = ?"
    )
    .bind(user_id)
    .fetch_all(&pool)
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

    sqlx::query!(
        "INSERT INTO jira_servers (id, user_id, name, url, username, encrypted_password) VALUES (?, ?, ?, ?, ?, ?)",
        id,
        user_id,
        payload.name,
        payload.url,
        payload.username,
        encrypted_password
    )
    .execute(&pool)
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
    // Also deletes tickets due to CASCADE in DB, but the issue says "remove that server record... along with any ticket in the Jira Tickets List"
    // CASCADE handles this.
    let result = sqlx::query!(
        "DELETE FROM jira_servers WHERE id = ? AND user_id = ?",
        server_id,
        user_id
    )
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Server not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn test_credentials(
    State(pool): State<MySqlPool>,
    Extension(user_id): Extension<Uuid>,
    Path(server_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let server = sqlx::query_as!(
        JiraServer,
        "SELECT id as `id: Uuid`, user_id as `user_id: Uuid`, name, url, username, encrypted_password, created_at as `created_at: DateTime<Utc>`, updated_at as `updated_at: DateTime<Utc>` FROM jira_servers WHERE id = ? AND user_id = ?",
        server_id,
        user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Server not found".to_string()))?;

    let password = decrypt(&server.encrypted_password);

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/rest/api/2/myself", server.url))
        .basic_auth(server.username, Some(password))
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to connect to Jira: {}", e)))?;

    if resp.status().is_success() {
        Ok(StatusCode::OK)
    } else {
        Err((StatusCode::UNAUTHORIZED, "Failed to authenticate with Jira".to_string()))
    }
}
