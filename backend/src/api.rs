pub mod applications;
pub mod auth;
pub mod docker;
pub mod files;
pub mod process;
pub mod services;
pub mod system;
pub mod terminal;

use axum::Router;

pub fn create_router() -> Router<crate::AppState> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/system", system::router())
        .nest("/processes", process::router())
        .nest("/files", files::router())
        .nest("/services", services::router())
        .nest("/terminal", terminal::router())
        .nest("/docker", docker::router())
        .nest("/applications", applications::router())
}
