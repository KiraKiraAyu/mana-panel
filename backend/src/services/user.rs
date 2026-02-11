use sea_orm::{entity::prelude::*, ActiveValue::Set};
use crate::db::entities::user;
use crate::error::{AppError, AppResult};
use crate::services::password;

/// User service for managing user accounts
pub struct UserService;

impl UserService {
    /// Create a new user with hashed password
    /// Fails if any user exists
    pub async fn create_user(
        db: &DatabaseConnection,
        username: &str,
        password: &str,
    ) -> AppResult<user::Model> {
        // Enforce single user policy
        let count = Self::count_users(db).await?;
        if count > 0 {
            return Err(AppError::Validation("User already exists. Only one user is allowed.".to_string()));
        }

        // Check if username already exists
        let existing = user::Entity::find()
            .filter(user::Column::Username.eq(username))
            .one(db)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        
        if existing.is_some() {
            return Err(AppError::Validation("Username already exists".to_string()));
        }
        
        // Hash the password
        let password_hash = password::hash_password(password)?;
        
        let now = chrono::Utc::now();
        
        let new_user = user::ActiveModel {
            id: Default::default(),
            username: Set(username.to_string()),
            password_hash: Set(password_hash),
            created_at: Set(now),
            updated_at: Set(now),
        };
        
        let user = new_user
            .insert(db)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        
        Ok(user)
    }

    /// Create a new refresh token for a user
    pub async fn create_refresh_token(
        db: &DatabaseConnection,
        user_id: i32,
    ) -> AppResult<String> {
        use crate::db::entities::refresh_token;
        use rand::Rng;

        // Generate random token
        let token: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();
        
        let now = chrono::Utc::now();
        let expires_at = now + chrono::Duration::days(30); // 30 days validity

        let new_token = refresh_token::ActiveModel {
            token_hash: Set(token.clone()),
            user_id: Set(user_id),
            expires_at: Set(expires_at),
            created_at: Set(now),
            used: Set(false),
        };

        new_token
            .insert(db)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        
        Ok(token)
    }

    /// Verify and rotate refresh token
    /// Returns (User, NewRefreshToken)
    pub async fn rotate_refresh_token(
        db: &DatabaseConnection,
        token: &str,
    ) -> AppResult<(user::Model, String)> {
        use crate::db::entities::refresh_token;

        let token_model = refresh_token::Entity::find_by_id(token)
            .one(db)
            .await
            .map_err(|e| AppError::Internal(e.into()))?
            .ok_or_else(|| AppError::Auth("Invalid refresh token".to_string()))?;

        // Check if used (replay attack prevention)
        if token_model.used {
            // Security: Invalidate ALL tokens for this user if a reused token is detected
            tracing::warn!("Reused refresh token detected for user {}. Revoking all sessions.", token_model.user_id);
            Self::revoke_all_user_tokens(db, token_model.user_id).await?;
            return Err(AppError::Auth("Invalid refresh token (reused)".to_string()));
        }

        // Check expiry
        if token_model.expires_at < chrono::Utc::now() {
            // Delete expired token
            let _ = refresh_token::Entity::delete_by_id(token)
                .exec(db)
                .await;
            return Err(AppError::Auth("Refresh token expired".to_string()));
        }

        // Mark current token as used
        let mut active_token: refresh_token::ActiveModel = token_model.clone().into();
        active_token.used = Set(true);
        active_token.update(db).await.map_err(|e| AppError::Internal(e.into()))?;

        // Get user
        let user = user::Entity::find_by_id(token_model.user_id)
            .one(db)
            .await
            .map_err(|e| AppError::Internal(e.into()))?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Issue new token
        let new_token = Self::create_refresh_token(db, user.id).await?;

        Ok((user, new_token))
    }

    /// Revoke a specific refresh token (logout)
    pub async fn revoke_refresh_token(
        db: &DatabaseConnection,
        token: &str,
    ) -> AppResult<()> {
        use crate::db::entities::refresh_token;
        
        refresh_token::Entity::delete_by_id(token)
            .exec(db)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        
        Ok(())
    }

    /// Revoke all tokens for a user
    pub async fn revoke_all_user_tokens(
        db: &DatabaseConnection,
        user_id: i32,
    ) -> AppResult<()> {
        use crate::db::entities::refresh_token;

        refresh_token::Entity::delete_many()
            .filter(refresh_token::Column::UserId.eq(user_id))
            .exec(db)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        
        Ok(())
    }
    
    /// Find a user by username
    pub async fn find_by_username(
        db: &DatabaseConnection,
        username: &str,
    ) -> AppResult<Option<user::Model>> {
        let user = user::Entity::find()
            .filter(user::Column::Username.eq(username))
            .one(db)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        
        Ok(user)
    }
    
    /// Verify user credentials
    pub async fn verify_credentials(
        db: &DatabaseConnection,
        username: &str,
        password: &str,
    ) -> AppResult<Option<user::Model>> {
        let user = Self::find_by_username(db, username).await?;
        
        match user {
            Some(u) => {
                if password::verify_password(password, &u.password_hash)? {
                    Ok(Some(u))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
    
    /// Update user password
    pub async fn update_password(
        db: &DatabaseConnection,
        user_id: i32,
        new_password: &str,
    ) -> AppResult<()> {
        let password_hash = password::hash_password(new_password)?;
        let now = chrono::Utc::now();
        
        user::Entity::update_many()
            .col_expr(user::Column::PasswordHash, Expr::value(password_hash))
            .col_expr(user::Column::UpdatedAt, Expr::value(now))
            .filter(user::Column::Id.eq(user_id))
            .exec(db)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        
        Ok(())
    }
    
    /// Get user count (for initial setup check)
    pub async fn count_users(db: &DatabaseConnection) -> AppResult<u64> {
        let count = user::Entity::find()
            .count(db)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        
        Ok(count)
    }
    
    /// Initialize default admin user if no users exist
    pub async fn init_default_admin(db: &DatabaseConnection) -> AppResult<()> {
        let count = Self::count_users(db).await?;
        
        if count == 0 {
            tracing::info!("Creating default admin user");
            Self::create_user(db, "admin", "admin").await?;
        }
        
        Ok(())
    }
}
