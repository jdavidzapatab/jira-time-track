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
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::AppState;
use tracing::{error, info, debug, instrument};
use validator::Validate;

#[derive(Deserialize)]
pub struct CreateTicketRequest {
    pub server_id: Option<Uuid>,
    pub ticket_number: Option<String>,
    pub at_top: Option<bool>,
}

#[instrument(skip(state))]
pub async fn list_tickets(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Vec<JiraTicket>>, (StatusCode, String)> {
    let tickets = ticket_repo::list_tickets_by_user(&state.pool, user_id)
        .await
        .map_err(|e| {
            error!(error = ?e, user_id = %user_id, "Failed to list tickets from database");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    Ok(Json(tickets))
}

#[instrument(skip(state, payload))]
pub async fn create_ticket(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<CreateTicketRequest>,
) -> Result<Json<JiraTicket>, (StatusCode, String)> {
    let id = Uuid::new_v4();
    let at_top = payload.at_top.unwrap_or(false);

    ticket_repo::create_ticket(&state.pool, id, user_id, payload.server_id, payload.ticket_number.as_deref(), at_top)
        .await
        .map_err(|e| {
            error!(error = ?e, user_id = %user_id, "Failed to create ticket in database");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    let ticket = ticket_repo::find_ticket_by_id(&state.pool, id)
        .await
        .map_err(|e| {
            error!(error = ?e, ticket_id = %id, "Failed to fetch created ticket from database");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?
        .ok_or_else(|| {
            error!(ticket_id = %id, "Failed to fetch created ticket: not found");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch created ticket".to_string())
        })?;

    info!(user_id = %user_id, ticket_id = %id, "Jira ticket line created successfully");
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

#[instrument(skip(state, payload))]
pub async fn update_ticket(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(ticket_id): Path<Uuid>,
    Json(payload): Json<UpdateTicketRequest>,
) -> Result<Json<JiraTicket>, (StatusCode, String)> {
    ticket_repo::update_ticket(
        &state.pool,
        ticket_repo::UpdateTicketParams {
            id: ticket_id,
            user_id,
            server_id: payload.server_id,
            ticket_number: payload.ticket_number.as_deref(),
            ticket_summary: payload.ticket_summary.as_deref(),
            time_spent_seconds: payload.time_spent_seconds,
            saved_description: payload.saved_description.as_deref(),
            last_stopwatch_start: payload.last_stopwatch_start,
        },
    )
    .await
    .map_err(|e| {
        error!(error = ?e, user_id = %user_id, ticket_id = %ticket_id, "Failed to update ticket in database");
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    let ticket = ticket_repo::find_ticket_by_id(&state.pool, ticket_id)
        .await
        .map_err(|e| {
            error!(error = ?e, ticket_id = %ticket_id, "Failed to fetch updated ticket from database");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?
        .ok_or_else(|| {
            debug!(ticket_id = %ticket_id, "Updated ticket not found");
            (StatusCode::NOT_FOUND, "Ticket not found".to_string())
        })?;

    Ok(Json(ticket))
}

#[instrument(skip(state))]
pub async fn delete_ticket(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(ticket_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let rows_affected = ticket_repo::delete_ticket(&state.pool, ticket_id, user_id)
        .await
        .map_err(|e| {
            error!(error = ?e, user_id = %user_id, ticket_id = %ticket_id, "Failed to delete ticket from database");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    if rows_affected == 0 {
        debug!(user_id = %user_id, ticket_id = %ticket_id, "Attempted to delete non-existent ticket");
        return Err((StatusCode::NOT_FOUND, "Ticket not found".to_string()));
    }

    info!(user_id = %user_id, ticket_id = %ticket_id, "Jira ticket line deleted successfully");
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, Validate)]
pub struct SubmitWorklogRequest {
    pub time_spent_formatted: String,
    #[validate(length(min = 1, message = "Worklog description is required"))]
    pub description: String,
}

#[instrument(skip(state, payload))]
pub async fn submit_worklog(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(ticket_id): Path<Uuid>,
    Json(payload): Json<SubmitWorklogRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    if let Err(e) = payload.validate() {
        debug!(error = ?e, "Worklog validation failed");
        return Err((StatusCode::BAD_REQUEST, format!("Validation error: {}", e)));
    }

    let ticket = ticket_repo::find_ticket_by_id_and_user(&state.pool, ticket_id, user_id)
        .await
        .map_err(|e| {
            error!(error = ?e, user_id = %user_id, ticket_id = %ticket_id, "Failed to fetch ticket for worklog submission");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?
        .ok_or_else(|| {
            debug!(user_id = %user_id, ticket_id = %ticket_id, "Worklog submission attempted for non-existent ticket");
            (StatusCode::NOT_FOUND, "Ticket not found".to_string())
        })?;

    let server_id = ticket.server_id.ok_or_else(|| {
        debug!(ticket_id = %ticket_id, "Worklog submission failed: no server selected");
        (StatusCode::BAD_REQUEST, "No server selected for this ticket".to_string())
    })?;
    let ticket_number = ticket.ticket_number.as_ref().ok_or_else(|| {
        debug!(ticket_id = %ticket_id, "Worklog submission failed: no ticket number");
        (StatusCode::BAD_REQUEST, "No ticket number for this ticket".to_string())
    })?;

    let server = server_repo::find_server_by_id_only(&state.pool, server_id)
        .await
        .map_err(|e| {
            error!(error = ?e, server_id = %server_id, "Failed to fetch server for worklog submission");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?
        .ok_or_else(|| {
            error!(server_id = %server_id, "Server not found during worklog submission");
            (StatusCode::INTERNAL_SERVER_ERROR, "Server not found".to_string())
        })?;

    jira_service::submit_worklog(&server, ticket_number, &payload.time_spent_formatted, &payload.description)
        .await
        .map_err(|e| {
            error!(error = %e, ticket_number = %ticket_number, "Failed to submit worklog to Jira");
            (StatusCode::BAD_GATEWAY, e)
        })?;

    info!(user_id = %user_id, ticket_number = %ticket_number, "Worklog submitted to Jira successfully");
    Ok(StatusCode::OK)
}

#[instrument(skip(state))]
pub async fn get_ticket_summary(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(ticket_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let ticket = ticket_repo::find_ticket_by_id_and_user(&state.pool, ticket_id, user_id)
        .await
        .map_err(|e| {
            error!(error = ?e, user_id = %user_id, ticket_id = %ticket_id, "Failed to fetch ticket for summary update");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?
        .ok_or_else(|| {
            debug!(user_id = %user_id, ticket_id = %ticket_id, "Summary fetch attempted for non-existent ticket");
            (StatusCode::NOT_FOUND, "Ticket not found".to_string())
        })?;

    let server_id = ticket.server_id.ok_or_else(|| {
        debug!(ticket_id = %ticket_id, "Summary fetch failed: no server selected");
        (StatusCode::BAD_REQUEST, "No server selected".to_string())
    })?;
    let ticket_number = ticket.ticket_number.as_ref().ok_or_else(|| {
        debug!(ticket_id = %ticket_id, "Summary fetch failed: no ticket number");
        (StatusCode::BAD_REQUEST, "No ticket number".to_string())
    })?;

    let server = server_repo::find_server_by_id_only(&state.pool, server_id)
        .await
        .map_err(|e| {
            error!(error = ?e, server_id = %server_id, "Failed to fetch server for summary update");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?
        .ok_or_else(|| {
            error!(server_id = %server_id, "Server not found during summary update");
            (StatusCode::INTERNAL_SERVER_ERROR, "Server not found".to_string())
        })?;

    let summary = jira_service::get_ticket_summary(&server, ticket_number)
        .await
        .map_err(|e| {
            debug!(error = %e, ticket_number = %ticket_number, "Failed to fetch ticket summary from Jira");
            (StatusCode::BAD_GATEWAY, e)
        })?;

    ticket_repo::update_ticket_summary(&state.pool, ticket_id, &summary)
        .await
        .ok();

    info!(ticket_number = %ticket_number, "Ticket summary fetched and updated from Jira");
    Ok(Json(serde_json::json!({ "summary": summary })))
}

#[derive(Deserialize)]
pub struct ReorderTicketsRequest {
    pub ticket_ids: Vec<Uuid>,
}

#[instrument(skip(state, payload))]
pub async fn reorder_tickets(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<ReorderTicketsRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    ticket_repo::update_tickets_order(&state.pool, user_id, &payload.ticket_ids)
        .await
        .map_err(|e| {
            error!(error = ?e, user_id = %user_id, "Failed to reorder tickets in database");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    info!(user_id = %user_id, "Tickets reordered successfully");
    Ok(StatusCode::OK)
}
