use sqlx::MySqlPool;
use uuid::Uuid;
use crate::models::JiraTicket;
use chrono::{DateTime, Utc};

pub async fn list_tickets_by_user(pool: &MySqlPool, user_id: Uuid) -> Result<Vec<JiraTicket>, sqlx::Error> {
    sqlx::query_as::<_, JiraTicket>(
        "SELECT * FROM jira_tickets WHERE user_id = ? ORDER BY sort_order, created_at"
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
    at_top: bool,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    if at_top {
        // Shift existing ones up
        sqlx::query("UPDATE jira_tickets SET sort_order = sort_order + 1 WHERE user_id = ?")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        
        sqlx::query(
            "INSERT INTO jira_tickets (id, user_id, server_id, ticket_number, sort_order) VALUES (?, ?, ?, ?, 0)"
        )
        .bind(id)
        .bind(user_id)
        .bind(server_id)
        .bind(ticket_number)
        .execute(&mut *tx)
        .await?;
    } else {
        sqlx::query(
            "INSERT INTO jira_tickets (id, user_id, server_id, ticket_number, sort_order) 
             SELECT ?, ?, ?, ?, COALESCE(MAX(sort_order), -1) + 1 FROM jira_tickets WHERE user_id = ?"
        )
        .bind(id)
        .bind(user_id)
        .bind(server_id)
        .bind(ticket_number)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
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

pub struct UpdateTicketParams<'a> {
    pub id: Uuid,
    pub user_id: Uuid,
    pub server_id: Option<Uuid>,
    pub ticket_number: Option<&'a str>,
    pub ticket_summary: Option<&'a str>,
    pub time_spent_seconds: Option<i32>,
    pub saved_description: Option<&'a str>,
    pub last_stopwatch_start: Option<DateTime<Utc>>,
}

pub async fn update_ticket(
    pool: &MySqlPool,
    params: UpdateTicketParams<'_>,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE jira_tickets SET server_id = ?, ticket_number = ?, ticket_summary = ?, time_spent_seconds = COALESCE(?, time_spent_seconds), saved_description = ?, last_stopwatch_start = ? WHERE id = ? AND user_id = ?"
    )
    .bind(params.server_id)
    .bind(params.ticket_number)
    .bind(params.ticket_summary)
    .bind(params.time_spent_seconds)
    .bind(params.saved_description)
    .bind(params.last_stopwatch_start)
    .bind(params.id)
    .bind(params.user_id)
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

pub async fn update_tickets_order(pool: &MySqlPool, user_id: Uuid, ticket_ids: &[Uuid]) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    for (index, id) in ticket_ids.iter().enumerate() {
        sqlx::query(
            "UPDATE jira_tickets SET sort_order = ? WHERE id = ? AND user_id = ?"
        )
        .bind(index as i32)
        .bind(id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}
