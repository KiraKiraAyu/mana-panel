use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::{wrappers::IntervalStream, StreamExt};

use crate::AppState;

#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os: String,
    pub os_version: String,
    pub kernel: String,
    pub uptime: u64,
    pub cpu_count: usize,
    pub total_memory: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub memory_percent: f32,
    pub swap_used: u64,
    pub swap_total: u64,
    pub disks: Vec<DiskInfo>,
    pub networks: Vec<NetworkInfo>,
    pub load_avg: [f64; 3],
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub percent: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkInfo {
    pub name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/info", get(system_info))
        .route("/stats", get(system_stats))
        .route("/stats/stream", get(stats_stream))
}

async fn system_info(State(state): State<AppState>) -> Json<SystemInfo> {
    let info = state.monitor.get_system_info();
    Json(info)
}

async fn system_stats(State(state): State<AppState>) -> Json<SystemStats> {
    let stats = state.monitor.get_stats();
    Json(stats)
}

async fn stats_stream(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let interval = tokio::time::interval(Duration::from_secs(1));
    let stream = IntervalStream::new(interval).map(move |_| {
        let stats = state.monitor.get_stats();
        let json = serde_json::to_string(&stats).unwrap_or_default();
        Ok(Event::default().data(json))
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
