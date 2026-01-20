use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use crate::models::{JiraServer, JiraTicket};
use crate::utils::encryption::decrypt;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Deserialize)]
pub struct CreateTicketRequest {
    pub server_id: Option<Uuid>,
    pub ticket_number: Option<String>,
}

pub async fn list_tickets(
    State(pool): State<MySqlPool>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Vec<JiraTicket>>, (StatusCode, String)> {
    let tickets = sqlx::query_as!(
        JiraTicket,
        "SELECT id as `id: Uuid`, user_id as `user_id: Uuid`, server_id as `server_id: Option<Uuid>`, ticket_number, ticket_summary, time_spent_seconds, saved_description, last_stopwatch_start as `last_stopwatch_start: Option<DateTime<Utc>>`, created_at as `created_at: DateTime<Utc>`, updated_at as `updated_at: DateTime<Utc>` FROM jira_tickets WHERE user_id = ?",
        user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(tickets))
}

pub async fn create_ticket(
    State(pool): State<MySqlPool>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<CreateTicketRequest>,
) -> Result<Json<JiraTicket>, (StatusCode, String)> {
    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO jira_tickets (id, user_id, server_id, ticket_number) VALUES (?, ?, ?, ?)",
        id,
        user_id,
        payload.server_id,
        payload.ticket_number
    )
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let ticket = sqlx::query_as!(
        JiraTicket,
        "SELECT id as `id: Uuid`, user_id as `user_id: Uuid`, server_id as `server_id: Option<Uuid>`, ticket_number, ticket_summary, time_spent_seconds, saved_description, last_stopwatch_start as `last_stopwatch_start: Option<DateTime<Utc>>`, created_at as `created_at: DateTime<Utc>`, updated_at as `updated_at: DateTime<Utc>` FROM jira_tickets WHERE id = ?",
        id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ticket))
}

#[derive(Deserialize)]
pub struct UpdateTicketRequest {
    pub server_id: Option<Uuid>,
    pub ticket_number: Option<String>,
    pub ticket_summary: Option<String>,
    pub time_spent_seconds: Option<i32>,
    pub saved_description: Option<String>,
    pub last_stopwatch_start: Option<DateTime<Utc>>,
}

pub async fn update_ticket(
    State(pool): State<MySqlPool>,
    Extension(user_id): Extension<Uuid>,
    Path(ticket_id): Path<Uuid>,
    Json(payload): Json<UpdateTicketRequest>,
) -> Result<Json<JiraTicket>, (StatusCode, String)> {
    sqlx::query!(
        "UPDATE jira_tickets SET server_id = ?, ticket_number = ?, ticket_summary = ?, time_spent_seconds = COALESCE(?, time_spent_seconds), saved_description = ?, last_stopwatch_start = ? WHERE id = ? AND user_id = ?",
        payload.server_id,
        payload.ticket_number,
        payload.ticket_summary,
        payload.time_spent_seconds,
        payload.saved_description,
        payload.last_stopwatch_start,
        ticket_id,
        user_id
    )
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let ticket = sqlx::query_as!(
        JiraTicket,
        "SELECT id as `id: Uuid`, user_id as `user_id: Uuid`, server_id as `server_id: Option<Uuid>`, ticket_number, ticket_summary, time_spent_seconds, saved_description, last_stopwatch_start as `last_stopwatch_start: Option<DateTime<Utc>>`, created_at as `created_at: DateTime<Utc>`, updated_at as `updated_at: DateTime<Utc>` FROM jira_tickets WHERE id = ?",
        ticket_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ticket))
}

pub async fn delete_ticket(
    State(pool): State<MySqlPool>,
    Extension(user_id): Extension<Uuid>,
    Path(ticket_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query!(
        "DELETE FROM jira_tickets WHERE id = ? AND user_id = ?",
        ticket_id,
        user_id
    )
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Ticket not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct SubmitWorklogRequest {
    pub time_spent_formatted: String,
    pub description: String,
}

pub async fn submit_worklog(
    State(pool): State<MySqlPool>,
    Extension(user_id): Extension<Uuid>,
    Path(ticket_id): Path<Uuid>,
    Json(payload): Json<SubmitWorklogRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let ticket = sqlx::query_as!(
        JiraTicket,
        "SELECT id as `id: Uuid`, user_id as `user_id: Uuid`, server_id as `server_id: Option<Uuid>`, ticket_number, ticket_summary, time_spent_seconds, saved_description, last_stopwatch_start as `last_stopwatch_start: Option<DateTime<Utc>>`, created_at as `created_at: DateTime<Utc>`, updated_at as `updated_at: DateTime<Utc>` FROM jira_tickets WHERE id = ? AND user_id = ?",
        ticket_id,
        user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let server_id = ticket.server_id.ok_or((StatusCode::BAD_REQUEST, "No server selected for this ticket".to_string()))?;
    let ticket_number = ticket.ticket_number.ok_or((StatusCode::BAD_REQUEST, "No ticket number for this ticket".to_string()))?;

    let server = sqlx::query_as!(
        JiraServer,
        "SELECT id as `id: Uuid`, user_id as `user_id: Uuid`, name, url, username, encrypted_password, created_at as `created_at: DateTime<Utc>`, updated_at as `updated_at: DateTime<Utc>` FROM jira_servers WHERE id = ?",
        server_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let password = decrypt(&server.encrypted_password);
    let client = reqwest::Client::new();

    // Add Worklog
    let worklog_url = format!("{}/rest/api/2/issue/{}/worklog", server.url, ticket_number);
    let worklog_resp = client
        .post(&worklog_url)
        .basic_auth(&server.username, Some(&password))
        .json(&serde_json::json!({
            "comment": payload.description,
            "timeSpent": payload.time_spent_formatted
        }))
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to send worklog to Jira: {}", e)))?;

    if !worklog_resp.status().is_success() {
        let err_text = worklog_resp.text().await.unwrap_or_default();
        return Err((StatusCode::BAD_GATEWAY, format!("Jira worklog error: {}", err_text)));
    }

    // Add Comment (Requirement: "as a comment and as worklog")
    let comment_url = format!("{}/rest/api/2/issue/{}/comment", server.url, ticket_number);
    let _comment_resp = client
        .post(&comment_url)
        .basic_auth(&server.username, Some(&password))
        .json(&serde_json::json!({
            "body": format!("Time spent: {}\n\n{}", payload.time_spent_formatted, payload.description)
        }))
        .send()
        .await;

    // Clear local time and description if successfully submitted? 
    // The requirement for "Clear" button handles manual clearing. 
    // Usually submission would also clear or reset.

    Ok(StatusCode::OK)
}

pub async fn get_ticket_summary(
    State(pool): State<MySqlPool>,
    Extension(user_id): Extension<Uuid>,
    Path(ticket_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let ticket = sqlx::query_as!(
        JiraTicket,
        "SELECT id as `id: Uuid`, user_id as `user_id: Uuid`, server_id as `server_id: Option<Uuid>`, ticket_number, ticket_summary, time_spent_seconds, saved_description, last_stopwatch_start as `last_stopwatch_start: Option<DateTime<Utc>>`, created_at as `created_at: DateTime<Utc>`, updated_at as `updated_at: DateTime<Utc>` FROM jira_tickets WHERE id = ? AND user_id = ?",
        ticket_id,
        user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let server_id = ticket.server_id.ok_or((StatusCode::BAD_REQUEST, "No server selected".to_string()))?;
    let ticket_number = ticket.ticket_number.ok_or((StatusCode::BAD_REQUEST, "No ticket number".to_string()))?;

    let server = sqlx::query_as!(
        JiraServer,
        "SELECT id as `id: Uuid`, user_id as `user_id: Uuid`, name, url, username, encrypted_password, created_at as `created_at: DateTime<Utc>`, updated_at as `updated_at: DateTime<Utc>` FROM jira_servers WHERE id = ?",
        server_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let password = decrypt(&server.encrypted_password);
    let client = reqwest::Client::new();

    let url = format!("{}/rest/api/2/issue/{}", server.url, ticket_number);
    let resp = client
        .get(&url)
        .basic_auth(&server.username, Some(&password))
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to fetch ticket from Jira: {}", e)))?;

    if !resp.status().is_success() {
         return Err((StatusCode::BAD_GATEWAY, "Jira fetch error".to_string()));
    }

    let data: serde_json::Value = resp.json().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let summary = data["fields"]["summary"].as_str().unwrap_or("No summary found");

    sqlx::query!(
        "UPDATE jira_tickets SET ticket_summary = ? WHERE id = ?",
        summary,
        ticket_id
    )
    .execute(&pool)
    .await
    .ok();

    Ok(Json(serde_json::json!({ "summary": summary })))
}
