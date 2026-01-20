use sqlx::MySqlPool;
use uuid::Uuid;
use crate::models::JiraTicket;
use chrono::{DateTime, Utc};

pub async fn list_tickets_by_user(pool: &MySqlPool, user_id: Uuid) -> Result<Vec<JiraTicket>, sqlx::Error> {
    sqlx::query_as::<_, JiraTicket>(
        "SELECT * FROM jira_tickets WHERE user_id = ?"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn create_ticket(
    pool: &MySqlPool,
    id: Uuid,
    user_id: Uuid,
    server_id: Option<Uuid>,
    ticket_number: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO jira_tickets (id, user_id, server_id, ticket_number) VALUES (?, ?, ?, ?)"
    )
    .bind(id)
    .bind(user_id)
    .bind(server_id)
    .bind(ticket_number)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find_ticket_by_id(pool: &MySqlPool, id: Uuid) -> Result<Option<JiraTicket>, sqlx::Error> {
    sqlx::query_as::<_, JiraTicket>(
        "SELECT * FROM jira_tickets WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_ticket_by_id_and_user(pool: &MySqlPool, id: Uuid, user_id: Uuid) -> Result<Option<JiraTicket>, sqlx::Error> {
    sqlx::query_as::<_, JiraTicket>(
        "SELECT * FROM jira_tickets WHERE id = ? AND user_id = ?"
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn update_ticket(
    pool: &MySqlPool,
    id: Uuid,
    user_id: Uuid,
    server_id: Option<Uuid>,
    ticket_number: Option<&str>,
    ticket_summary: Option<&str>,
    time_spent_seconds: Option<i32>,
    saved_description: Option<&str>,
    last_stopwatch_start: Option<DateTime<Utc>>,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE jira_tickets SET server_id = ?, ticket_number = ?, ticket_summary = ?, time_spent_seconds = COALESCE(?, time_spent_seconds), saved_description = ?, last_stopwatch_start = ? WHERE id = ? AND user_id = ?"
    )
    .bind(server_id)
    .bind(ticket_number)
    .bind(ticket_summary)
    .bind(time_spent_seconds)
    .bind(saved_description)
    .bind(last_stopwatch_start)
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn update_ticket_summary(pool: &MySqlPool, id: Uuid, summary: &str) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE jira_tickets SET ticket_summary = ? WHERE id = ?"
    )
    .bind(summary)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn delete_ticket(pool: &MySqlPool, id: Uuid, user_id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM jira_tickets WHERE id = ? AND user_id = ?"
    )
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}
