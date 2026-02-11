use axum::{
    extract::Path,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[cfg(target_os = "linux")]
use std::process::Command;

#[derive(Debug, Serialize)]
pub struct ServiceInfo {
    pub name: String,
    pub description: String,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
}

#[derive(Debug, Serialize)]
pub struct ServiceLogs {
    pub name: String,
    pub logs: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ServiceActionResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    pub lines: Option<usize>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_services))
        .route("/{name}/start", post(start_service))
        .route("/{name}/stop", post(stop_service))
        .route("/{name}/restart", post(restart_service))
        .route("/{name}/logs", get(get_logs))
}

fn validate_service_name(name: &str) -> Result<(), crate::error::AppError> {
    if name.is_empty() || name.len() > 256 {
        return Err(crate::error::AppError::Validation("Invalid service name length".to_string()));
    }

    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '@') {
        return Err(crate::error::AppError::Validation("Invalid service name characters".to_string()));
    }

    Ok(())
}

async fn list_services() -> crate::error::AppResult<Json<Vec<ServiceInfo>>> {
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("systemctl")
            .args(["list-units", "--type=service", "--all", "--no-pager", "--plain"])
            .output()
            .map_err(|e| crate::error::AppError::System(format!("Failed to run systemctl: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut services = Vec::new();

        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                services.push(ServiceInfo {
                    name: parts[0].trim_end_matches(".service").to_string(),
                    load_state: parts[1].to_string(),
                    active_state: parts[2].to_string(),
                    sub_state: parts[3].to_string(),
                    description: parts[4..].join(" "),
                });
            }
        }

        Ok(Json(services))
    }

    #[cfg(not(target_os = "linux"))]
    {
        Ok(Json(vec![
            ServiceInfo {
                name: "ssh".to_string(),
                description: "OpenSSH Server".to_string(),
                load_state: "loaded".to_string(),
                active_state: "active".to_string(),
                sub_state: "running".to_string(),
            },
            ServiceInfo {
                name: "nginx".to_string(),
                description: "A high performance web server".to_string(),
                load_state: "loaded".to_string(),
                active_state: "active".to_string(),
                sub_state: "running".to_string(),
            },
        ]))
    }
}

async fn start_service(Path(name): Path<String>) -> crate::error::AppResult<Json<ServiceActionResponse>> {
    validate_service_name(&name)?;
    run_systemctl_action(&name, "start")
}

async fn stop_service(Path(name): Path<String>) -> crate::error::AppResult<Json<ServiceActionResponse>> {
    validate_service_name(&name)?;
    run_systemctl_action(&name, "stop")
}

async fn restart_service(Path(name): Path<String>) -> crate::error::AppResult<Json<ServiceActionResponse>> {
    validate_service_name(&name)?;
    run_systemctl_action(&name, "restart")
}

fn run_systemctl_action(name: &str, action: &str) -> crate::error::AppResult<Json<ServiceActionResponse>> {
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("systemctl")
            .args([action, name])
            .output()
            .map_err(|e| crate::error::AppError::System(format!("Failed to run systemctl: {}", e)))?;

        if output.status.success() {
            Ok(Json(ServiceActionResponse {
                success: true,
                message: format!("Service {} {}ed successfully", name, action),
            }))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(crate::error::AppError::System(format!("Failed to {} service: {}", action, stderr)))
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        Ok(Json(ServiceActionResponse {
            success: true,
            message: format!("Service {} {}ed successfully (mock)", name, action),
        }))
    }
}

async fn get_logs(
    Path(name): Path<String>,
    axum::extract::Query(query): axum::extract::Query<LogsQuery>,
) -> crate::error::AppResult<Json<ServiceLogs>> {
    validate_service_name(&name)?;

    let lines = query.lines.unwrap_or(100).min(1000);

    #[cfg(target_os = "linux")]
    {
        let output = Command::new("journalctl")
            .args(["-u", &name, "-n", &lines.to_string(), "--no-pager"])
            .output()
            .map_err(|e| crate::error::AppError::System(format!("Failed to run journalctl: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let logs: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();

        Ok(Json(ServiceLogs { name, logs }))
    }

    #[cfg(not(target_os = "linux"))]
    {
        Ok(Json(ServiceLogs {
            name,
            logs: vec![
                "Mock log entry 1".to_string(),
                "Mock log entry 2".to_string(),
            ],
        }))
    }
}
