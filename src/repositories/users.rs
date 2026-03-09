use crate::models::User;
use sqlx::MySqlPool;
use uuid::Uuid;

pub async fn create_user(
    pool: &MySqlPool,
    id: Uuid,
    email: &str,
    password_hash: &str,
    confirmation_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, confirmation_token) VALUES (?, ?, ?, ?)",
    )
    .bind(id)
    .bind(email)
    .bind(password_hash)
    .bind(confirmation_token)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn confirm_user(pool: &MySqlPool, token: &str) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE users SET is_confirmed = TRUE, confirmation_token = NULL WHERE confirmation_token = ?",
    )
    .bind(token)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn find_user_by_email(
    pool: &MySqlPool,
    email: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, is_confirmed, confirmation_token, created_at, updated_at FROM users WHERE email = ?",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
}

pub async fn update_confirmation_token(
    pool: &MySqlPool,
    email: &str,
    token: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("UPDATE users SET confirmation_token = ? WHERE email = ?")
        .bind(token)
        .bind(email)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

pub async fn update_password(
    pool: &MySqlPool,
    token: &str,
    password_hash: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE users SET password_hash = ?, is_confirmed = TRUE, confirmation_token = NULL WHERE confirmation_token = ?",
    )
    .bind(password_hash)
    .bind(token)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}
