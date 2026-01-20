use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use uuid::Uuid;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub password_confirmation: String,
}

pub async fn register(
    State(pool): State<MySqlPool>,
    Json(payload): Json<RegisterRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    if let Err(e) = payload.validate() {
        return Err((StatusCode::BAD_REQUEST, format!("Validation error: {}", e)));
    }

    if payload.password != payload.password_confirmation {
        return Err((StatusCode::BAD_REQUEST, "Passwords do not match".to_string()));
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .to_string();

    let id = Uuid::new_v4();
    let confirmation_token = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, confirmation_token) VALUES (?, ?, ?, ?)",
    )
    .bind(id)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&confirmation_token)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // In a real app, send email here. For now, we just log it or provide the token in a way "visible in email inbox"
    println!("Confirmation link: http://localhost:3000/api/auth/confirm?token={}", confirmation_token);

    Ok(StatusCode::CREATED)
}

#[derive(Deserialize)]
pub struct ConfirmRequest {
    pub token: String,
}

pub async fn confirm(
    State(pool): State<MySqlPool>,
    Json(payload): Json<ConfirmRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query(
        "UPDATE users SET is_confirmed = TRUE, confirmation_token = NULL WHERE confirmation_token = ?",
    )
    .bind(&payload.token)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::BAD_REQUEST, "Invalid token".to_string()));
    }

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

pub async fn login(
    State(pool): State<MySqlPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    let user = sqlx::query_as::<_, crate::models::User>(
        "SELECT id, email, password_hash, is_confirmed, confirmation_token, created_at, updated_at FROM users WHERE email = ?",
    )
    .bind(&payload.email)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    if !user.is_confirmed {
        return Err((StatusCode::FORBIDDEN, "Account not confirmed".to_string()));
    }

    use argon2::PasswordVerifier;
    let argon2 = Argon2::default();
    let parsed_hash = argon2::PasswordHash::new(&user.password_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if argon2.verify_password(payload.password.as_bytes(), &parsed_hash).is_err() {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
    }

    // Generate JWT
    let token = crate::utils::generate_jwt(user.id)?;

    Ok(Json(LoginResponse { token }))
}

#[derive(Deserialize)]
pub struct PasswordChangeRequest {
    pub email: String,
}

pub async fn request_password_change(
    State(pool): State<MySqlPool>,
    Json(payload): Json<PasswordChangeRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let token = Uuid::new_v4().to_string();
    
    // In a real app, update user with reset token and send email
    sqlx::query(
        "UPDATE users SET confirmation_token = ? WHERE email = ?",
    )
    .bind(&token)
    .bind(&payload.email)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    println!("Password change link: http://localhost:3000/api/auth/reset-password?token={}", token);

    Ok(StatusCode::OK)
}
