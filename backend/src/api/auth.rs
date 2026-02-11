use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{error::{AppError, AppResult}, services::user::UserService, AppState};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub success: bool,
    pub message: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/me", get(current_user))
        .route("/password", post(change_password))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use crate::middleware::auth::Claims;

    // Validate against database
    let user = UserService::verify_credentials(&state.db, &payload.username, &payload.password)
        .await?
        .ok_or_else(|| AppError::Auth("Invalid credentials".to_string()))?;

    let expiry = chrono::Utc::now() + chrono::Duration::hours(state.config.jwt_expiry_hours);
    
    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        exp: expiry.timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(LoginResponse {
        token,
        expires_at: expiry,
    }))
}

async fn current_user(
    claims: crate::middleware::auth::Claims,
) -> AppResult<Json<UserInfo>> {
    Ok(Json(UserInfo {
        id: claims.sub.parse().unwrap_or(0),
        username: claims.username,
    }))
}

async fn change_password(
    State(state): State<AppState>,
    claims: crate::middleware::auth::Claims,
    Json(payload): Json<ChangePasswordRequest>,
) -> AppResult<Json<MessageResponse>> {
    let user_id: i32 = claims.sub.parse()
        .map_err(|_| AppError::Auth("Invalid user ID".to_string()))?;
    
    // Verify current password
    let user = UserService::find_by_username(&state.db, &claims.username)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    
    if !crate::services::password::verify_password(&payload.current_password, &user.password_hash)? {
        return Err(AppError::Auth("Current password is incorrect".to_string()));
    }
    
    // Update password
    UserService::update_password(&state.db, user_id, &payload.new_password).await?;
    
    Ok(Json(MessageResponse {
        success: true,
        message: "Password changed successfully".to_string(),
    }))
}
