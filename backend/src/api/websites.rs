use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, post, put},
};
use std::path::PathBuf;

use crate::{
    AppState,
    error::{AppError, AppResult},
    services::application::ApplicationManager,
    services::website::{CreateWebsiteRequest, UpdateWebsiteRequest, WebsiteInfo, WebsiteService},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_websites))
        .route("/", post(create_website))
        .route("/{id}", get(get_website))
        .route("/{id}", put(update_website))
        .route("/{id}", delete(delete_website))
}

async fn list_websites(State(state): State<AppState>) -> AppResult<Json<Vec<WebsiteInfo>>> {
    let websites = WebsiteService::list(&state.db).await?;
    Ok(Json(websites))
}

async fn get_website(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<WebsiteInfo>> {
    let website = WebsiteService::get_by_id(&state.db, id).await?;
    Ok(Json(website))
}

async fn create_website(
    State(state): State<AppState>,
    Json(payload): Json<CreateWebsiteRequest>,
) -> AppResult<Json<WebsiteInfo>> {
    let docker = state.docker.as_ref().ok_or_else(|| AppError::System("Docker service is not available".to_string()))?;
    let app_manager = app_manager_from_state(&state);

    let website = WebsiteService::create(&state.db, &app_manager, docker, payload).await?;

    Ok(Json(website))
}

async fn update_website(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateWebsiteRequest>,
) -> AppResult<Json<WebsiteInfo>> {
    let docker = state.docker.as_ref().ok_or_else(|| AppError::System("Docker service is not available".to_string()))?;
    let app_manager = app_manager_from_state(&state);

    let website = WebsiteService::update(&state.db, &app_manager, docker, id, payload).await?;

    Ok(Json(website))
}

async fn delete_website(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<serde_json::Value>> {
    let docker = state.docker.as_ref().ok_or_else(|| AppError::System("Docker service is not available".to_string()))?;
    let app_manager = app_manager_from_state(&state);

    WebsiteService::delete(&state.db, &app_manager, docker, id).await?;
    Ok(Json(
        serde_json::json!({ "success": true, "message": "Website deleted" }),
    ))
}

fn app_manager_from_state(state: &AppState) -> ApplicationManager {
    let app_root_dir = PathBuf::from(state.config.app_root_dir.clone());
    ApplicationManager::new(app_root_dir)
}
