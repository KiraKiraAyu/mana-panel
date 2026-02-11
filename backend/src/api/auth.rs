use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
    response::{IntoResponse, Response},
    http::{header, HeaderMap},
};
use axum_extra::extract::cookie::{Cookie, SameSite};
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
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/me", get(current_user))
        .route("/password", post(change_password))
}

const REFRESH_TOKEN_COOKIE: &str = "refresh_token";

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<impl IntoResponse> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use crate::middleware::auth::Claims;

    // Validate against database
    let user = UserService::verify_credentials(&state.db, &payload.username, &payload.password)
        .await?
        .ok_or_else(|| AppError::Auth("Invalid credentials".to_string()))?;

    // Create Refresh Token
    let refresh_token = UserService::create_refresh_token(&state.db, user.id).await?;

    // Create Access Token
    let expiry = chrono::Utc::now() + chrono::Duration::hours(state.config.jwt_expiry_hours);
    
    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        exp: expiry.timestamp() as usize,
    };

    let access_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.into()))?;

    let expires_at = expiry;

    // Set Cookie
    let cookie = Cookie::build((REFRESH_TOKEN_COOKIE, refresh_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        // .secure(true) // TODO: Enable in production with HTTPS
        .build();
    
    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());

    Ok((headers, Json(LoginResponse {
        token: access_token,
        expires_at,
    })))
}

async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<impl IntoResponse> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use crate::middleware::auth::Claims;

    // Get refresh token from cookie
    let cookie = headers
        .get(header::COOKIE)
        .and_then(|val| val.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|c| {
                let c = c.trim();
                if c.starts_with(&format!("{}=", REFRESH_TOKEN_COOKIE)) {
                    Some(c.split_once('=').unwrap().1)
                } else {
                    None
                }
            })
        })
        .ok_or_else(|| AppError::Auth("Missing refresh token".to_string()))?;

    // Verify/Rotate
    let (user, new_refresh_token) = UserService::rotate_refresh_token(&state.db, cookie).await?;

    // Create new Access Token
    let expiry = chrono::Utc::now() + chrono::Duration::hours(state.config.jwt_expiry_hours);
    
    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        exp: expiry.timestamp() as usize,
    };

    let access_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.into()))?;

    // Set new Cookie
    let cookie = Cookie::build((REFRESH_TOKEN_COOKIE, new_refresh_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .build();
    
    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());

    Ok((headers, Json(LoginResponse {
        token: access_token,
        expires_at: expiry,
    })))
}

async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<impl IntoResponse> {
    // Attempt to get token to revoke it in DB
    if let Some(cookie_val) = headers
        .get(header::COOKIE)
        .and_then(|val| val.to_str().ok()) {
            if let Some(token) = cookie_val.split(';').find_map(|c| {
                let c = c.trim();
                if c.starts_with(&format!("{}=", REFRESH_TOKEN_COOKIE)) {
                    Some(c.split_once('=').unwrap().1)
                } else {
                    None
                }
            }) {
                let _ = UserService::revoke_refresh_token(&state.db, token).await;
            }
    }

    // Clear Cookie
    let cookie = Cookie::build((REFRESH_TOKEN_COOKIE, ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(time::Duration::seconds(0))
        .build();
    
    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());

    Ok((headers, Json(MessageResponse {
        success: true,
        message: "Logged out successfully".to_string(),
    })))
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
