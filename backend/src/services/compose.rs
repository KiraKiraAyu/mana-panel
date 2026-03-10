use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use crate::error::{AppError, AppResult, DockerErrorKind};
use crate::services::docker::DockerActionResponse;

#[derive(Debug, Clone)]
pub struct ComposeService {
    timeout_secs: u64,
}

impl Default for ComposeService {
    fn default() -> Self {
        Self { timeout_secs: 60 }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComposePortPublisher {
    #[serde(default, alias = "URL", alias = "Url", alias = "url")]
    pub url: Option<String>,
    #[serde(default, alias = "TargetPort", alias = "target_port")]
    pub target_port: Option<u16>,
    #[serde(default, alias = "PublishedPort", alias = "published_port")]
    pub published_port: Option<u16>,
    #[serde(default, alias = "Protocol", alias = "protocol")]
    pub protocol: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComposeContainerStatus {
    #[serde(default, alias = "ID", alias = "Id", alias = "id")]
    pub id: String,
    #[serde(default, alias = "Name", alias = "name")]
    pub name: String,
    #[serde(default, alias = "Service", alias = "service")]
    pub service: String,
    #[serde(default, alias = "State", alias = "state")]
    pub state: String,
    #[serde(default, alias = "Health", alias = "health")]
    pub health: Option<String>,
    #[serde(default, alias = "ExitCode", alias = "exit_code")]
    pub exit_code: Option<i32>,
    #[serde(default, alias = "Publishers", alias = "publishers")]
    pub publishers: Vec<ComposePortPublisher>,
}

#[derive(Debug, Clone)]
pub struct ComposeProject {
    pub compose_file: PathBuf,
    pub project_name: String,
}

impl ComposeService {
    pub fn new(timeout_secs: u64) -> Self {
        Self { timeout_secs }
    }

    pub async fn ensure_available(&self) -> AppResult<()> {
        let output = self
            .run_raw(&[], &["version"], None)
            .await
            .map_err(|e| AppError::docker("compose_version", DockerErrorKind::Unknown, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(AppError::docker(
                "compose_version",
                self.classify_error(&stderr),
                if stderr.trim().is_empty() {
                    "docker compose is not available".to_string()
                } else {
                    stderr
                },
            ));
        }

        Ok(())
    }

    pub async fn up(&self, project: &ComposeProject) -> AppResult<DockerActionResponse> {
        self.run_lifecycle(project, &["up", "-d", "--remove-orphans"], "compose_up")
            .await
    }

    pub async fn start(&self, project: &ComposeProject) -> AppResult<DockerActionResponse> {
        self.run_lifecycle(project, &["start"], "compose_start")
            .await
    }

    pub async fn stop(&self, project: &ComposeProject) -> AppResult<DockerActionResponse> {
        self.run_lifecycle(project, &["stop"], "compose_stop").await
    }

    pub async fn down(
        &self,
        project: &ComposeProject,
        remove_volumes: bool,
    ) -> AppResult<DockerActionResponse> {
        let mut args = vec!["down", "--remove-orphans"];
        if remove_volumes {
            args.push("-v");
        }

        self.run_lifecycle(project, &args, "compose_down").await
    }

    pub async fn logs(
        &self,
        project: &ComposeProject,
        tail: usize,
    ) -> AppResult<DockerActionResponse> {
        let tail_str = tail.to_string();
        self.run_lifecycle(project, &["logs", "--tail", &tail_str], "compose_logs")
            .await
    }

    pub async fn ps(&self, project: &ComposeProject) -> AppResult<Vec<ComposeContainerStatus>> {
        let output = self
            .run_compose(project, &["ps", "--format", "json"], "compose_ps")
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        self.parse_compose_ps_output(&stdout)
    }

    fn parse_compose_ps_output(&self, stdout: &str) -> AppResult<Vec<ComposeContainerStatus>> {
        let trimmed = stdout.trim();
        if trimmed.is_empty() {
            return Ok(vec![]);
        }

        if let Ok(items) = serde_json::from_str::<Vec<ComposeContainerStatus>>(trimmed) {
            return Ok(items);
        }

        if let Ok(item) = serde_json::from_str::<ComposeContainerStatus>(trimmed) {
            return Ok(vec![item]);
        }

        if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Ok(items) = Self::compose_statuses_from_json_value(value) {
                return Ok(items);
            }
        }

        let mut out = Vec::<ComposeContainerStatus>::new();
        for line in trimmed.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Ok(items) = serde_json::from_str::<Vec<ComposeContainerStatus>>(line) {
                out.extend(items);
                continue;
            }

            if let Ok(item) = serde_json::from_str::<ComposeContainerStatus>(line) {
                out.push(item);
                continue;
            }

            if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
                if let Ok(items) = Self::compose_statuses_from_json_value(value) {
                    out.extend(items);
                    continue;
                }
            }

            return Err(AppError::docker(
                "compose_ps_parse",
                DockerErrorKind::Unknown,
                format!("Failed to parse docker compose ps JSON line: {}", line),
            ));
        }

        if !out.is_empty() {
            return Ok(out);
        }

        Err(AppError::docker(
            "compose_ps_parse",
            DockerErrorKind::Unknown,
            format!(
                "Failed to parse docker compose ps JSON output: unsupported format (prefix: {})",
                &trimmed.chars().take(200).collect::<String>()
            ),
        ))
    }

    fn compose_statuses_from_json_value(
        value: serde_json::Value,
    ) -> Result<Vec<ComposeContainerStatus>, serde_json::Error> {
        match value {
            serde_json::Value::Array(arr) => arr
                .into_iter()
                .map(serde_json::from_value::<ComposeContainerStatus>)
                .collect(),
            serde_json::Value::Object(map) => {
                if let Some(v) = map.get("services").or_else(|| map.get("Services")) {
                    return match v {
                        serde_json::Value::Array(arr) => arr
                            .clone()
                            .into_iter()
                            .map(serde_json::from_value::<ComposeContainerStatus>)
                            .collect(),
                        serde_json::Value::Object(obj) => obj
                            .values()
                            .cloned()
                            .map(serde_json::from_value::<ComposeContainerStatus>)
                            .collect(),
                        _ => Ok(vec![]),
                    };
                }

                let all_values_are_objects = map.values().all(|v| v.is_object());
                if all_values_are_objects {
                    return map
                        .into_values()
                        .map(serde_json::from_value::<ComposeContainerStatus>)
                        .collect();
                }

                serde_json::from_value::<ComposeContainerStatus>(serde_json::Value::Object(map))
                    .map(|item| vec![item])
            }
            _ => Ok(vec![]),
        }
    }

    async fn run_lifecycle(
        &self,
        project: &ComposeProject,
        operation_args: &[&str],
        operation: &str,
    ) -> AppResult<DockerActionResponse> {
        let output = self.run_compose(project, operation_args, operation).await?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let message = if stdout.is_empty() {
            format!(
                "Compose operation '{}' succeeded for project '{}'",
                operation, project.project_name
            )
        } else {
            stdout
        };

        Ok(DockerActionResponse {
            success: true,
            message,
        })
    }

    async fn run_compose(
        &self,
        project: &ComposeProject,
        operation_args: &[&str],
        operation: &str,
    ) -> AppResult<std::process::Output> {
        let file = project.compose_file.to_string_lossy().to_string();
        let project_name = project.project_name.as_str();

        let mut compose_args = vec!["--project-name", project_name, "--file", file.as_str()];
        compose_args.extend(operation_args);

        let cwd = project.compose_file.parent().map(Path::to_path_buf);

        let output = self
            .run_raw(&compose_args, &[], cwd)
            .await
            .map_err(|e| AppError::docker(operation, DockerErrorKind::Unknown, e))?;

        if output.status.success() {
            return Ok(output);
        }

        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(AppError::docker(
            operation,
            self.classify_error(&stderr),
            if stderr.trim().is_empty() {
                format!(
                    "docker compose {} failed for project '{}'",
                    operation, project.project_name
                )
            } else {
                stderr
            },
        ))
    }

    async fn run_raw(
        &self,
        compose_args: &[&str],
        extra_args: &[&str],
        cwd: Option<PathBuf>,
    ) -> Result<std::process::Output, String> {
        let mut cmd = Command::new("docker");
        cmd.arg("compose");
        for arg in compose_args {
            cmd.arg(arg);
        }
        for arg in extra_args {
            cmd.arg(arg);
        }

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        let fut = cmd.output();

        match timeout(Duration::from_secs(self.timeout_secs), fut).await {
            Ok(Ok(output)) => Ok(output),
            Ok(Err(err)) => Err(format!("Failed to execute docker compose command: {}", err)),
            Err(_) => Err(format!(
                "docker compose command timed out after {} seconds",
                self.timeout_secs
            )),
        }
    }

    fn classify_error(&self, stderr: &str) -> DockerErrorKind {
        let msg = stderr.to_ascii_lowercase();

        if msg.contains("timed out") {
            return DockerErrorKind::Timeout;
        }
        if msg.contains("cannot connect to the docker daemon")
            || msg.contains("is the docker daemon running")
            || msg.contains("docker daemon")
        {
            return DockerErrorKind::DaemonUnavailable;
        }
        if msg.contains("not found")
            || msg.contains("no such service")
            || msg.contains("no such container")
            || msg.contains("no such project")
        {
            return DockerErrorKind::NotFound;
        }
        if msg.contains("permission denied") {
            return DockerErrorKind::PermissionDenied;
        }
        if msg.contains("port is already allocated")
            || msg.contains("address already in use")
            || msg.contains("bind for 0.0.0.0")
        {
            return DockerErrorKind::PortConflict;
        }
        if msg.contains("already in use") || msg.contains("conflict") {
            return DockerErrorKind::Conflict;
        }

        DockerErrorKind::Unknown
    }
}
