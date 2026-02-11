use axum::{
    extract::{Path, Query, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::{wrappers::IntervalStream, StreamExt};

use crate::{error::AppResult, AppState};

#[derive(Debug, Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cmd: Vec<String>,
    pub cpu_usage: f32,
    pub memory: u64,
    pub status: String,
    pub user: String,
    pub start_time: u64,
}

#[derive(Debug, Deserialize)]
pub struct ProcessQuery {
    pub sort_by: Option<String>,
    pub order: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ActionResponse {
    pub success: bool,
    pub message: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_processes))
        .route("/stream", get(processes_stream))
        .route("/{pid}/kill", post(kill_process))
        .route("/{pid}/stop", post(stop_process))
        .route("/{pid}/resume", post(resume_process))
}

async fn list_processes(
    State(state): State<AppState>,
    Query(query): Query<ProcessQuery>,
) -> AppResult<Json<Vec<ProcessInfo>>> {
    let processes = query_processes(&state, &query);
    Ok(Json(processes))
}

fn query_processes(state: &AppState, query: &ProcessQuery) -> Vec<ProcessInfo> {
    let mut processes = state.monitor.get_processes();

    // Filter by search term
    if let Some(ref search) = query.search {
        let search_lower = search.to_lowercase();
        processes.retain(|p| p.name.to_lowercase().contains(&search_lower));
    }

    // Sort processes
    if let Some(ref sort_by) = query.sort_by {
        let desc = query.order.as_deref() == Some("desc");
        match sort_by.as_str() {
            "cpu" => processes.sort_by(|a, b| {
                if desc {
                    b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap()
                } else {
                    a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap()
                }
            }),
            "memory" => processes.sort_by(|a, b| {
                if desc {
                    b.memory.cmp(&a.memory)
                } else {
                    a.memory.cmp(&b.memory)
                }
            }),
            "name" => processes.sort_by(|a, b| {
                if desc {
                    b.name.cmp(&a.name)
                } else {
                    a.name.cmp(&b.name)
                }
            }),
            "pid" => processes.sort_by(|a, b| {
                if desc {
                    b.pid.cmp(&a.pid)
                } else {
                    a.pid.cmp(&b.pid)
                }
            }),
            _ => {}
        }
    }

    processes
}

async fn kill_process(
    State(state): State<AppState>,
    Path(pid): Path<u32>,
) -> AppResult<Json<ActionResponse>> {
    state.monitor.kill_process(pid)?;
    Ok(Json(ActionResponse {
        success: true,
        message: format!("Process {} killed", pid),
    }))
}

async fn stop_process(
    State(state): State<AppState>,
    Path(pid): Path<u32>,
) -> AppResult<Json<ActionResponse>> {
    state.monitor.stop_process(pid)?;
    Ok(Json(ActionResponse {
        success: true,
        message: format!("Process {} stopped", pid),
    }))
}

async fn resume_process(
    State(state): State<AppState>,
    Path(pid): Path<u32>,
) -> AppResult<Json<ActionResponse>> {
    state.monitor.resume_process(pid)?;
    Ok(Json(ActionResponse {
        success: true,
        message: format!("Process {} resumed", pid),
    }))
}

async fn processes_stream(
    State(state): State<AppState>,
    Query(query): Query<ProcessQuery>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let interval = tokio::time::interval(Duration::from_secs(2));
    let stream = IntervalStream::new(interval).map(move |_| {
        let processes = query_processes(&state, &query);
        let json = serde_json::to_string(&processes).unwrap_or_default();
        Ok(Event::default().data(json))
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
