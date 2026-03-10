use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::{delete, get, post},
};
use serde::Deserialize;
use std::convert::Infallible;
use std::path::PathBuf;
use std::time::Duration;
use tokio_stream::{StreamExt, wrappers::IntervalStream};

use crate::{
    AppState,
    error::{AppError, AppResult},
    services::{
        application::{
            ApplicationInstance, ApplicationManager, ApplicationTemplate, InstallApplicationRequest,
        },
        application_task::{ApplicationTask, ApplicationTaskManager},
        docker::DockerActionResponse,
    },
};

#[derive(Debug, Deserialize)]
pub struct ForceQuery {
    pub force: Option<bool>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/templates", get(list_templates))
        .route("/instances", get(list_instances))
        .route("/install", post(install_application))
        .route("/tasks", get(list_application_tasks))
        .route("/tasks/stream", get(stream_application_tasks))
        .route("/tasks/{task_id}", get(get_application_task))
        .route("/tasks/{task_id}/stream", get(stream_application_task))
        .route("/{id}/start", post(start_application))
        .route("/{id}/stop", post(stop_application))
        .route("/{id}/update", post(update_application))
        .route("/{id}/logs", get(get_application_logs))
        .route("/{id}/env", get(get_application_env).post(update_application_env))
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
) -> AppResult<Json<ApplicationTask>> {
    let manager = manager_from_state(&state)?;
    let task = manager
        .enqueue_install_application(payload, state.docker.clone())
        .await;

    Ok(Json(task))
}

async fn list_application_tasks() -> AppResult<Json<Vec<ApplicationTask>>> {
    let task_manager = ApplicationTaskManager::global();
    Ok(Json(task_manager.list().await))
}

async fn stream_application_tasks()
-> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let task_manager = ApplicationTaskManager::global().clone();
    let interval = tokio::time::interval(Duration::from_millis(500));
    let stream = IntervalStream::new(interval).then(move |_| {
        let task_manager = task_manager.clone();
        async move {
            let tasks = task_manager.list().await;
            let payload = serde_json::to_string(&tasks).unwrap_or_else(|_| "[]".to_string());
            Ok(Event::default().data(payload))
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn get_application_task(Path(task_id): Path<String>) -> AppResult<Json<ApplicationTask>> {
    let task_manager = ApplicationTaskManager::global();
    let task = task_manager
        .get(&task_id)
        .await
        .ok_or_else(|| AppError::NotFound(format!("Application task '{}' not found", task_id)))?;
    Ok(Json(task))
}

async fn stream_application_task(
    Path(task_id): Path<String>,
) -> AppResult<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>> {
    let task_manager = ApplicationTaskManager::global().clone();
    task_manager
        .get(&task_id)
        .await
        .ok_or_else(|| AppError::NotFound(format!("Application task '{}' not found", task_id)))?;

    let interval = tokio::time::interval(Duration::from_millis(500));
    let stream_task_id = task_id.clone();

    let stream = IntervalStream::new(interval).then(move |_| {
        let task_manager = task_manager.clone();
        let task_id = stream_task_id.clone();
        async move {
            match task_manager.get(&task_id).await {
                Some(task) => {
                    let payload = serde_json::to_string(&task).unwrap_or_default();
                    Ok(Event::default().data(payload))
                }
                None => Ok(Event::default().data(
                    serde_json::json!({
                        "task_id": task_id,
                        "message": "Application task not found",
                    })
                    .to_string(),
                )),
            }
        }
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
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

async fn update_application(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<DockerActionResponse>> {
    let manager = manager_from_state(&state)?;
    let result = manager
        .update_application(&id, state.docker.clone())
        .await?;
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    pub tail: Option<usize>,
}

async fn get_application_logs(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<LogsQuery>,
) -> AppResult<Json<DockerActionResponse>> {
    let manager = manager_from_state(&state)?;
    let tail = query.tail.unwrap_or(500);
    let result = manager.get_application_logs(&id, tail).await?;
    Ok(Json(result))
}

async fn get_application_env(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<std::collections::HashMap<String, String>>> {
    let manager = manager_from_state(&state)?;
    let result = manager.get_application_env(&id)?;
    Ok(Json(result))
}

async fn update_application_env(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(env_map): Json<std::collections::HashMap<String, String>>,
) -> AppResult<Json<DockerActionResponse>> {
    let manager = manager_from_state(&state)?;
    let result = manager.update_application_env(&id, env_map, state.docker.clone()).await?;
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
    let app_root_dir = PathBuf::from(state.config.app_root_dir.clone());

    Ok(ApplicationManager::new(app_root_dir))
}
