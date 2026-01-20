use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use crate::models::JiraTicket;
use crate::repositories::jira_tickets as ticket_repo;
use crate::repositories::jira_servers as server_repo;
use crate::services::jira as jira_service;
use serde::Deserialize;
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
    let tickets = ticket_repo::list_tickets_by_user(&pool, user_id)
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

    ticket_repo::create_ticket(&pool, id, user_id, payload.server_id, payload.ticket_number.as_deref())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let ticket = ticket_repo::find_ticket_by_id(&pool, id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch created ticket".to_string()))?;

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
    ticket_repo::update_ticket(
        &pool,
        ticket_id,
        user_id,
        payload.server_id,
        payload.ticket_number.as_deref(),
        payload.ticket_summary.as_deref(),
        payload.time_spent_seconds,
        payload.saved_description.as_deref(),
        payload.last_stopwatch_start,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let ticket = ticket_repo::find_ticket_by_id(&pool, ticket_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Ticket not found".to_string()))?;

    Ok(Json(ticket))
}

pub async fn delete_ticket(
    State(pool): State<MySqlPool>,
    Extension(user_id): Extension<Uuid>,
    Path(ticket_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let rows_affected = ticket_repo::delete_ticket(&pool, ticket_id, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if rows_affected == 0 {
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
    let ticket = ticket_repo::find_ticket_by_id_and_user(&pool, ticket_id, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Ticket not found".to_string()))?;

    let server_id = ticket.server_id.ok_or((StatusCode::BAD_REQUEST, "No server selected for this ticket".to_string()))?;
    let ticket_number = ticket.ticket_number.as_ref().ok_or((StatusCode::BAD_REQUEST, "No ticket number for this ticket".to_string()))?;

    let server = server_repo::find_server_by_id_only(&pool, server_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Server not found".to_string()))?;

    jira_service::submit_worklog(&server, ticket_number, &payload.time_spent_formatted, &payload.description)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e))?;

    Ok(StatusCode::OK)
}

pub async fn get_ticket_summary(
    State(pool): State<MySqlPool>,
    Extension(user_id): Extension<Uuid>,
    Path(ticket_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let ticket = ticket_repo::find_ticket_by_id_and_user(&pool, ticket_id, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Ticket not found".to_string()))?;

    let server_id = ticket.server_id.ok_or((StatusCode::BAD_REQUEST, "No server selected".to_string()))?;
    let ticket_number = ticket.ticket_number.as_ref().ok_or((StatusCode::BAD_REQUEST, "No ticket number".to_string()))?;

    let server = server_repo::find_server_by_id_only(&pool, server_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Server not found".to_string()))?;

    let summary = jira_service::get_ticket_summary(&server, ticket_number)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e))?;

    ticket_repo::update_ticket_summary(&pool, ticket_id, &summary)
        .await
        .ok();

    Ok(Json(serde_json::json!({ "summary": summary })))
}
