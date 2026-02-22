use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{
    AppState,
    error::{AppError, AppResult},
    services::{
        application::{
            ApplicationInstance, ApplicationManager, ApplicationTemplate, InstallApplicationRequest,
        },
        docker::DockerActionResponse,
    },
};

#[derive(Debug, Deserialize)]
pub struct ForceQuery {
    pub force: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct InstallApplicationResponse {
    pub action: DockerActionResponse,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/templates", get(list_templates))
        .route("/instances", get(list_instances))
        .route("/install", post(install_application))
        .route("/{id}/start", post(start_application))
        .route("/{id}/stop", post(stop_application))
        .route("/{id}", delete(remove_application))
}

async fn list_templates(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<ApplicationTemplate>>> {
    let manager = manager_from_state(&state)?;
    Ok(Json(manager.list_templates()))
}

async fn list_instances(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<ApplicationInstance>>> {
    let manager = manager_from_state(&state)?;
    let instances = manager.list_instances().await?;
    Ok(Json(instances))
}

async fn install_application(
    State(state): State<AppState>,
    Json(payload): Json<InstallApplicationRequest>,
) -> AppResult<Json<InstallApplicationResponse>> {
    let manager = manager_from_state(&state)?;
    let created = manager.install_application(payload).await?;
    Ok(Json(InstallApplicationResponse { action: created }))
}

async fn start_application(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<DockerActionResponse>> {
    let manager = manager_from_state(&state)?;
    let result = manager.start_application(&id).await?;
    Ok(Json(result))
}

async fn stop_application(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<DockerActionResponse>> {
    let manager = manager_from_state(&state)?;
    let result = manager.stop_application(&id).await?;
    Ok(Json(result))
}

async fn remove_application(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ForceQuery>,
) -> AppResult<Json<DockerActionResponse>> {
    let manager = manager_from_state(&state)?;
    let force = query.force.unwrap_or(true);
    let result = manager.remove_application(&id, force).await?;
    Ok(Json(result))
}

fn manager_from_state(state: &AppState) -> AppResult<ApplicationManager> {
    let docker = state
        .docker
        .clone()
        .ok_or_else(|| AppError::System("Docker is not available".to_string()))?;

    let app_root_dir = PathBuf::from(state.config.app_root_dir.clone());

    Ok(ApplicationManager::new(docker, app_root_dir))
}
