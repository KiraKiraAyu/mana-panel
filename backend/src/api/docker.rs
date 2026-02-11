use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::{
    error::AppResult,
    services::docker::{
        ContainerDetail, ContainerInfo, ContainerStats, CreateContainerRequest,
        DockerActionResponse, ImageInfo, PullProgress,
    },
    AppState,
};

// ---------------------------------------------------------------------------
// Query helpers
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AllQuery {
    pub all: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ForceQuery {
    pub force: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    pub tail: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct PullImageRequest {
    pub image: String,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new()
        // Containers
        .route("/containers", get(list_containers))
        .route("/containers", post(create_container))
        .route("/containers/{id}", get(inspect_container))
        .route("/containers/{id}", delete(remove_container))
        .route("/containers/{id}/start", post(start_container))
        .route("/containers/{id}/stop", post(stop_container))
        .route("/containers/{id}/restart", post(restart_container))
        .route("/containers/{id}/logs", get(container_logs))
        .route("/containers/{id}/stats", get(container_stats))
        // Images
        .route("/images", get(list_images))
        .route("/images/pull", post(pull_image))
        .route("/images/{id}", delete(remove_image))
}

// ---------------------------------------------------------------------------
// Container handlers
// ---------------------------------------------------------------------------

async fn list_containers(
    State(state): State<AppState>,
    Query(query): Query<AllQuery>,
) -> AppResult<Json<Vec<ContainerInfo>>> {
    let docker = state.docker.as_ref().ok_or_else(|| {
        crate::error::AppError::System("Docker is not available".into())
    })?;
    let all = query.all.unwrap_or(true);
    let containers = docker.list_containers(all).await?;
    Ok(Json(containers))
}

async fn inspect_container(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ContainerDetail>> {
    let docker = state.docker.as_ref().ok_or_else(|| {
        crate::error::AppError::System("Docker is not available".into())
    })?;
    let detail = docker.inspect_container(&id).await?;
    Ok(Json(detail))
}

async fn start_container(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<DockerActionResponse>> {
    let docker = state.docker.as_ref().ok_or_else(|| {
        crate::error::AppError::System("Docker is not available".into())
    })?;
    let resp = docker.start_container(&id).await?;
    Ok(Json(resp))
}

async fn stop_container(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<DockerActionResponse>> {
    let docker = state.docker.as_ref().ok_or_else(|| {
        crate::error::AppError::System("Docker is not available".into())
    })?;
    let resp = docker.stop_container(&id).await?;
    Ok(Json(resp))
}

async fn restart_container(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<DockerActionResponse>> {
    let docker = state.docker.as_ref().ok_or_else(|| {
        crate::error::AppError::System("Docker is not available".into())
    })?;
    let resp = docker.restart_container(&id).await?;
    Ok(Json(resp))
}

async fn remove_container(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ForceQuery>,
) -> AppResult<Json<DockerActionResponse>> {
    let docker = state.docker.as_ref().ok_or_else(|| {
        crate::error::AppError::System("Docker is not available".into())
    })?;
    let force = query.force.unwrap_or(false);
    let resp = docker.remove_container(&id, force).await?;
    Ok(Json(resp))
}

async fn container_logs(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<LogsQuery>,
) -> AppResult<Json<Vec<String>>> {
    let docker = state.docker.as_ref().ok_or_else(|| {
        crate::error::AppError::System("Docker is not available".into())
    })?;
    let tail = query.tail.unwrap_or(200).min(5000);
    let logs = docker.container_logs(&id, tail).await?;
    Ok(Json(logs))
}

async fn container_stats(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ContainerStats>> {
    let docker = state.docker.as_ref().ok_or_else(|| {
        crate::error::AppError::System("Docker is not available".into())
    })?;
    let stats = docker.container_stats(&id).await?;
    Ok(Json(stats))
}

// ---------------------------------------------------------------------------
// Container creation
// ---------------------------------------------------------------------------

async fn create_container(
    State(state): State<AppState>,
    Json(payload): Json<CreateContainerRequest>,
) -> AppResult<Json<DockerActionResponse>> {
    let docker = state.docker.as_ref().ok_or_else(|| {
        crate::error::AppError::System("Docker is not available".into())
    })?;
    let resp = docker.create_container(payload).await?;
    Ok(Json(resp))
}

// ---------------------------------------------------------------------------
// Image handlers
// ---------------------------------------------------------------------------

async fn list_images(
    State(state): State<AppState>,
    Query(query): Query<AllQuery>,
) -> AppResult<Json<Vec<ImageInfo>>> {
    let docker = state.docker.as_ref().ok_or_else(|| {
        crate::error::AppError::System("Docker is not available".into())
    })?;
    let all = query.all.unwrap_or(false);
    let images = docker.list_images(all).await?;
    Ok(Json(images))
}

async fn pull_image(
    State(state): State<AppState>,
    Json(payload): Json<PullImageRequest>,
) -> AppResult<Json<Vec<PullProgress>>> {
    let docker = state.docker.as_ref().ok_or_else(|| {
        crate::error::AppError::System("Docker is not available".into())
    })?;
    let progress = docker.pull_image(&payload.image).await?;
    Ok(Json(progress))
}

async fn remove_image(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ForceQuery>,
) -> AppResult<Json<DockerActionResponse>> {
    let docker = state.docker.as_ref().ok_or_else(|| {
        crate::error::AppError::System("Docker is not available".into())
    })?;
    let force = query.force.unwrap_or(false);
    let resp = docker.remove_image(&id, force).await?;
    Ok(Json(resp))
}
