use axum::{
    extract::{State, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use uuid::Uuid;
use validator::Validate;
use crate::repositories::users as user_repo;
use crate::services::auth as auth_service;
use crate::utils::generate_jwt;

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

    let password_hash = auth_service::hash_password(&payload.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let id = Uuid::new_v4();
    let confirmation_token = Uuid::new_v4().to_string();

    user_repo::create_user(&pool, id, &payload.email, &password_hash, &confirmation_token)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // In a real app, send email here.
    println!("Confirmation link: http://localhost:3000/confirm?token={}", confirmation_token);

    Ok(StatusCode::CREATED)
}

#[derive(Deserialize)]
pub struct ConfirmRequest {
    pub token: String,
}

pub async fn confirm(
    State(pool): State<MySqlPool>,
    query: Option<Query<ConfirmRequest>>,
    payload: Option<Json<ConfirmRequest>>,
) -> Result<StatusCode, (StatusCode, String)> {
    let token = if let Some(Query(q)) = query {
        if !q.token.is_empty() {
            q.token
        } else if let Some(Json(p)) = payload {
            p.token
        } else {
            return Err((StatusCode::BAD_REQUEST, "Missing token".to_string()));
        }
    } else if let Some(Json(p)) = payload {
        p.token
    } else {
        return Err((StatusCode::BAD_REQUEST, "Missing token".to_string()));
    };

    let rows_affected = user_repo::confirm_user(&pool, &token)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if rows_affected == 0 {
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
    let user = user_repo::find_user_by_email(&pool, &payload.email)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    if !user.is_confirmed {
        return Err((StatusCode::FORBIDDEN, "Account not confirmed".to_string()));
    }

    auth_service::verify_password(&payload.password, &user.password_hash)
        .map_err(|e| (StatusCode::UNAUTHORIZED, e))?;

    let token = generate_jwt(user.id)?;

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
    
    let rows_affected = user_repo::update_confirmation_token(&pool, &payload.email, &token)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if rows_affected > 0 {
        println!("Password change link: http://localhost:3000/change-password?token={}", token);
    }

    Ok(StatusCode::OK)
}
