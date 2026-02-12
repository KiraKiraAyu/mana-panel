use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::convert::Infallible;
use tokio_stream::{StreamExt, wrappers::ReceiverStream};

use crate::{
    AppState,
    error::{AppError, AppResult},
    services::{
        application::{
            ApplicationInstance, ApplicationManager, ApplicationTemplate,
            InstallApplicationRequest, PreflightResult,
        },
        docker::{DockerActionResponse, PullProgress},
    },
};

#[derive(Debug, Deserialize)]
pub struct ForceQuery {
    pub force: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct PullImageStreamQuery {
    pub image: String,
}

#[derive(Debug, Deserialize)]
pub struct ImageExistsQuery {
    pub image: String,
}

#[derive(Debug, Serialize)]
pub struct ImageExistsResponse {
    pub image: String,
    pub exists: bool,
}

#[derive(Debug, Serialize)]
pub struct InstallApplicationResponse {
    pub action: DockerActionResponse,
    pub image_pulled: bool,
    pub pull_progress: Vec<PullProgress>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/templates", get(list_templates))
        .route("/instances", get(list_instances))
        .route("/images/exists", get(image_exists))
        .route("/install", post(install_application))
        .route("/image-pull/stream", get(stream_image_pull))
        .route("/{id}/preflight", post(preflight_start))
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

async fn image_exists(
    State(state): State<AppState>,
    Query(query): Query<ImageExistsQuery>,
) -> AppResult<Json<ImageExistsResponse>> {
    let image = query.image.trim().to_string();
    if image.is_empty() {
        return Err(AppError::Validation("image is required".to_string()));
    }

    let docker = state
        .docker
        .clone()
        .ok_or_else(|| AppError::System("Docker is not available".to_string()))?;

    let images = docker.list_images(true).await?;
    let image_lower = image.to_lowercase();

    let normalized_candidates: HashSet<String> = {
        let mut set = HashSet::new();
        set.insert(image_lower.clone());
        if !image_lower.contains(':') {
            set.insert(format!("{}:latest", image_lower));
        }
        set
    };

    let exists = images.iter().any(|img| {
        img.repo_tags
            .iter()
            .map(|t| t.to_lowercase())
            .any(|tag| normalized_candidates.contains(&tag))
            || img.id.to_lowercase().contains(&image_lower)
    });

    Ok(Json(ImageExistsResponse { image, exists }))
}

async fn install_application(
    State(state): State<AppState>,
    Json(payload): Json<InstallApplicationRequest>,
) -> AppResult<Json<InstallApplicationResponse>> {
    let manager = manager_from_state(&state)?;

    match manager.install_application(payload.clone()).await {
        Ok(created) => Ok(Json(InstallApplicationResponse {
            action: created,
            image_pulled: false,
            pull_progress: vec![],
        })),
        Err(AppError::System(msg)) if msg.contains("No such image") => {
            let image = resolve_install_image(&manager, &payload).ok_or_else(|| {
                AppError::Validation(
                    "No such image and failed to resolve image from template/install payload"
                        .to_string(),
                )
            })?;

            let docker = state
                .docker
                .clone()
                .ok_or_else(|| AppError::System("Docker is not available".to_string()))?;

            let progress = docker.pull_image(&image).await?;
            let created = manager.install_application(payload).await?;

            Ok(Json(InstallApplicationResponse {
                action: created,
                image_pulled: true,
                pull_progress: progress,
            }))
        }
        Err(e) => Err(e),
    }
}

async fn stream_image_pull(
    State(state): State<AppState>,
    Query(query): Query<PullImageStreamQuery>,
) -> AppResult<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>> {
    let image = query.image.trim().to_string();
    if image.is_empty() {
        return Err(AppError::Validation("image is required".to_string()));
    }

    let docker = state
        .docker
        .clone()
        .ok_or_else(|| AppError::System("Docker is not available".to_string()))?;

    let (event_tx, event_rx) = tokio::sync::mpsc::channel::<Event>(128);
    let docker_for_pull = docker.clone();
    let image_for_pull = image.clone();

    tokio::spawn(async move {
        let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel::<PullProgress>(128);

        let pull_handle = tokio::spawn(async move {
            docker_for_pull
                .pull_image_stream(&image_for_pull, progress_tx)
                .await
        });

        while let Some(item) = progress_rx.recv().await {
            let payload = serde_json::to_string(&item).unwrap_or_else(|_| "{}".to_string());
            if event_tx
                .send(Event::default().event("progress").data(payload))
                .await
                .is_err()
            {
                return;
            }
        }

        let terminal_event = match pull_handle.await {
            Ok(Ok(())) => {
                let payload = serde_json::json!({
                    "status": "done",
                    "image": image
                })
                .to_string();
                Event::default().event("done").data(payload)
            }
            Ok(Err(err)) => {
                let payload = serde_json::json!({
                    "status": "error",
                    "message": err.to_string()
                })
                .to_string();
                Event::default().event("error").data(payload)
            }
            Err(join_err) => {
                let payload = serde_json::json!({
                    "status": "error",
                    "message": format!("Pull task join error: {}", join_err)
                })
                .to_string();
                Event::default().event("error").data(payload)
            }
        };

        let _ = event_tx.send(terminal_event).await;
    });

    let stream = ReceiverStream::new(event_rx).map(|event| Ok(event));

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

async fn preflight_start(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<PreflightResult>> {
    let manager = manager_from_state(&state)?;
    let result = manager.preflight_start(&id).await?;
    Ok(Json(result))
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

fn resolve_install_image(
    manager: &ApplicationManager,
    payload: &InstallApplicationRequest,
) -> Option<String> {
    if let Some(image) = payload
        .image_override
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        return Some(image.to_string());
    }

    manager
        .list_templates()
        .into_iter()
        .find(|t| t.id == payload.template_id)
        .map(|t| t.image)
}

fn manager_from_state(state: &AppState) -> AppResult<ApplicationManager> {
    let docker = state
        .docker
        .clone()
        .ok_or_else(|| AppError::System("Docker is not available".to_string()))?;
    Ok(ApplicationManager::new(docker))
}
