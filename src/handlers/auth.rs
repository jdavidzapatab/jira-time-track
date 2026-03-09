use crate::AppState;
use crate::repositories::users as user_repo;
use crate::services::auth as auth_service;
use crate::utils::generate_jwt;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{debug, error, info, instrument};
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate, Debug)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub password_confirmation: String,
}

#[instrument(skip(state, payload))]
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    if let Err(e) = payload.validate() {
        debug!(error = ?e, "Registration validation failed");
        return Err((StatusCode::BAD_REQUEST, format!("Validation error: {}", e)));
    }

    if payload.password != payload.password_confirmation {
        debug!("Passwords do not match during registration");
        return Err((
            StatusCode::BAD_REQUEST,
            "Passwords do not match".to_string(),
        ));
    }

    let password_hash = auth_service::hash_password(&payload.password).map_err(|e| {
        error!(error = ?e, "Failed to hash password");
        (StatusCode::INTERNAL_SERVER_ERROR, e)
    })?;

    let id = Uuid::new_v4();
    let confirmation_token = Uuid::new_v4().to_string();

    user_repo::create_user(
        &state.pool,
        id,
        &payload.email,
        &password_hash,
        &confirmation_token,
    )
    .await
    .map_err(|e| {
        error!(error = ?e, email = %payload.email, "Failed to create user in database");
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    let base_url = env::var("APP_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let confirmation_link = format!("{}/confirm?token={}", base_url, confirmation_token);

    // Send email
    state
        .mail_service
        .send_email(
            &payload.email,
            "Confirm your JiraTrack account",
            format!(
                "Please confirm your account by clicking this link: {}",
                confirmation_link
            ),
        )
        .await
        .map_err(|e| {
            error!(error = ?e, email = %payload.email, "Failed to send confirmation email");
            (StatusCode::INTERNAL_SERVER_ERROR, e)
        })?;

    info!(email = %payload.email, "User registered successfully");
    debug!(confirmation_link = %confirmation_link, "Confirmation link generated");

    Ok(StatusCode::CREATED)
}

#[derive(Deserialize, Debug)]
pub struct ConfirmRequest {
    pub token: String,
}

#[instrument(skip(state, query, payload))]
pub async fn confirm(
    State(state): State<AppState>,
    query: Option<Query<ConfirmRequest>>,
    payload: Option<Json<ConfirmRequest>>,
) -> Result<StatusCode, (StatusCode, String)> {
    let token = if let Some(Query(q)) = query {
        if !q.token.is_empty() {
            q.token
        } else if let Some(Json(p)) = payload {
            p.token
        } else {
            debug!("Missing token in confirm request");
            return Err((StatusCode::BAD_REQUEST, "Missing token".to_string()));
        }
    } else if let Some(Json(p)) = payload {
        p.token
    } else {
        debug!("Missing token in confirm request");
        return Err((StatusCode::BAD_REQUEST, "Missing token".to_string()));
    };

    let rows_affected = user_repo::confirm_user(&state.pool, &token)
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to confirm user in database");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    if rows_affected == 0 {
        debug!("Invalid token provided for confirmation");
        return Err((StatusCode::BAD_REQUEST, "Invalid token".to_string()));
    }

    info!("User account confirmed successfully");
    Ok(StatusCode::OK)
}

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Debug)]
pub struct LoginResponse {
    pub token: String,
}

#[instrument(skip(state, payload))]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    let user = user_repo::find_user_by_email(&state.pool, &payload.email)
        .await
        .map_err(|e| {
            error!(error = ?e, email = %payload.email, "Failed to find user by email during login");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?
        .ok_or_else(|| {
            debug!(email = %payload.email, "Login attempt with non-existent email");
            (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string())
        })?;

    if !user.is_confirmed {
        debug!(email = %payload.email, "Login attempt for unconfirmed account");
        return Err((StatusCode::FORBIDDEN, "Account not confirmed".to_string()));
    }

    auth_service::verify_password(&payload.password, &user.password_hash).map_err(|e| {
        debug!(email = %payload.email, "Invalid password for login");
        (StatusCode::UNAUTHORIZED, e)
    })?;

    let token = generate_jwt(user.id).map_err(|e| {
        error!(error = ?e, user_id = %user.id, "Failed to generate JWT");
        e
    })?;

    info!(email = %payload.email, user_id = %user.id, "User logged in successfully");
    Ok(Json(LoginResponse { token }))
}

#[derive(Deserialize, Debug)]
pub struct PasswordChangeRequest {
    pub email: String,
}

#[instrument(skip(state, payload))]
pub async fn request_password_change(
    State(state): State<AppState>,
    Json(payload): Json<PasswordChangeRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let token = Uuid::new_v4().to_string();

    let rows_affected = user_repo::update_confirmation_token(&state.pool, &payload.email, &token)
        .await
        .map_err(|e| {
            error!(error = ?e, email = %payload.email, "Failed to update confirmation token for password reset");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    if rows_affected > 0 {
        let base_url =
            env::var("APP_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
        let change_link = format!("{}/change-password?token={}", base_url, token);

        // Send email
        state.mail_service.send_email(
            &payload.email,
            "Reset your JiraTrack password",
            format!("You requested a password reset. Please click this link to set a new password: {}", change_link)
        ).await.map_err(|e| {
            error!(error = ?e, email = %payload.email, "Failed to send password reset email");
            (StatusCode::INTERNAL_SERVER_ERROR, e)
        })?;

        info!(email = %payload.email, "Password reset link requested and sent");
        debug!(change_link = %change_link, "Password change link generated");
    } else {
        debug!(email = %payload.email, "Password reset requested for unknown email");
    }

    Ok(StatusCode::OK)
}

#[derive(Deserialize, Validate, Debug)]
pub struct PasswordChangeExecuteRequest {
    pub token: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub password_confirmation: String,
}

#[instrument(skip(state, payload))]
pub async fn change_password(
    State(state): State<AppState>,
    Json(payload): Json<PasswordChangeExecuteRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    if let Err(e) = payload.validate() {
        debug!(error = ?e, "Password change validation failed");
        return Err((StatusCode::BAD_REQUEST, format!("Validation error: {}", e)));
    }

    if payload.password != payload.password_confirmation {
        debug!("Passwords do not match during password change");
        return Err((
            StatusCode::BAD_REQUEST,
            "Passwords do not match".to_string(),
        ));
    }

    let password_hash = auth_service::hash_password(&payload.password).map_err(|e| {
        error!(error = ?e, "Failed to hash new password");
        (StatusCode::INTERNAL_SERVER_ERROR, e)
    })?;

    let rows_affected = user_repo::update_password(&state.pool, &payload.token, &password_hash)
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to update password in database");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    if rows_affected == 0 {
        debug!("Invalid or expired token for password change");
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid or expired token".to_string(),
        ));
    }

    info!("Password updated successfully");
    Ok(StatusCode::OK)
}
