use sea_orm::{entity::prelude::*, ActiveValue::Set};
use crate::db::entities::user;
use crate::error::{AppError, AppResult};
use crate::services::password;

/// User service for managing user accounts
pub struct UserService;

impl UserService {
    /// Create a new user with hashed password
    pub async fn create_user(
        db: &DatabaseConnection,
        username: &str,
        password: &str,
    ) -> AppResult<user::Model> {
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
