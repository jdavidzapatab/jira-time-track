use sqlx::MySqlPool;
use uuid::Uuid;
use crate::models::JiraServer;

pub async fn list_servers_by_user(pool: &MySqlPool, user_id: Uuid) -> Result<Vec<JiraServer>, sqlx::Error> {
    sqlx::query_as::<_, JiraServer>(
        "SELECT id, user_id, name, url, username, encrypted_password, created_at, updated_at FROM jira_servers WHERE user_id = ?"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn create_server(
    pool: &MySqlPool,
    id: Uuid,
    user_id: Uuid,
    name: &str,
    url: &str,
    username: &str,
    encrypted_password: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO jira_servers (id, user_id, name, url, username, encrypted_password) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(id)
    .bind(user_id)
    .bind(name)
    .bind(url)
    .bind(username)
    .bind(encrypted_password)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_server(pool: &MySqlPool, server_id: Uuid, user_id: Uuid) -> Result<u64, sqlx::Error> {
    // Manually delete tickets first to ensure they are removed regardless of DB constraint settings
    sqlx::query(
        "DELETE FROM jira_tickets WHERE server_id = ? AND user_id = ?"
    )
    .bind(server_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    let result = sqlx::query(
        "DELETE FROM jira_servers WHERE id = ? AND user_id = ?"
    )
    .bind(server_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn find_server_by_id(pool: &MySqlPool, server_id: Uuid, user_id: Uuid) -> Result<Option<JiraServer>, sqlx::Error> {
    sqlx::query_as::<_, JiraServer>(
        "SELECT * FROM jira_servers WHERE id = ? AND user_id = ?"
    )
    .bind(server_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn find_server_by_id_only(pool: &MySqlPool, server_id: Uuid) -> Result<Option<JiraServer>, sqlx::Error> {
    sqlx::query_as::<_, JiraServer>(
        "SELECT * FROM jira_servers WHERE id = ?"
    )
    .bind(server_id)
    .fetch_optional(pool)
    .await
}
