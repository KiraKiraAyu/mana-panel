use serde::{Deserialize, Serialize};

pub const ROOT_AGENT_SOCKET_PATH: &str = "/run/mana-panel/root-agent.sock";
const ROOT_AGENT_SHARED_TOKEN: &str = "mana-panel-root-agent-fixed-token";
const ROOT_AGENT_TIMEOUT_MS: u64 = 5_000;
const PANEL_ROOT_DIR: &str = "/opt/mana-panel";

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemctlAction {
    Start,
    Stop,
    Restart,
}

impl SystemctlAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Restart => "restart",
        }
    }

    pub fn past_tense(self) -> &'static str {
        match self {
            Self::Start => "started",
            Self::Stop => "stopped",
            Self::Restart => "restarted",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessSignal {
    Kill,
    Stop,
    #[serde(rename = "continue")]
    Continue,
}

impl ProcessSignal {
    pub fn signal_arg(self) -> &'static str {
        match self {
            Self::Kill => "-KILL",
            Self::Stop => "-STOP",
            Self::Continue => "-CONT",
        }
    }

    pub fn past_tense(self) -> &'static str {
        match self {
            Self::Kill => "killed",
            Self::Stop => "stopped",
            Self::Continue => "resumed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootAgentResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<String>>,
}

impl RootAgentResponse {
    fn ok(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            logs: None,
        }
    }

    fn with_logs(logs: Vec<String>) -> Self {
        Self {
            success: true,
            message: "OK".to_string(),
            logs: Some(logs),
        }
    }

    fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            logs: None,
        }
    }
}

#[cfg(target_family = "unix")]
mod unix {
    use std::{
        path::{Path, PathBuf},
        process::Stdio,
    };

    use serde::{Deserialize, Serialize};
    use tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        net::{UnixListener, UnixStream},
        time::{timeout, Duration},
    };

    use crate::error::{AppError, AppResult};

    use super::{ProcessSignal, RootAgentResponse, SystemctlAction};

    const MAX_REQUEST_BYTES: usize = 16 * 1024;
    const COMMAND_TIMEOUT: Duration = Duration::from_secs(15);
    const IO_TIMEOUT: Duration = Duration::from_secs(5);

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct RootAgentRequest {
        token: String,
        command: RootAgentCommand,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    enum RootAgentCommand {
        Ping,
        EnsurePanelDirectories {
            uid: u32,
            gid: u32,
        },
        Systemctl {
            action: SystemctlAction,
            unit: String,
        },
        ServiceLogs {
            unit: String,
            lines: u32,
        },
        ProcessSignal {
            pid: u32,
            signal: ProcessSignal,
        },
    }

    #[derive(Debug, Clone)]
    pub struct RootAgentClient {
        socket_path: PathBuf,
        token: String,
        request_timeout: Duration,
    }

    impl RootAgentClient {
        pub fn new() -> AppResult<Self> {
            Ok(Self {
                socket_path: PathBuf::from(super::ROOT_AGENT_SOCKET_PATH),
                token: super::ROOT_AGENT_SHARED_TOKEN.to_string(),
                request_timeout: Duration::from_millis(super::ROOT_AGENT_TIMEOUT_MS),
            })
        }

        pub async fn ping(&self) -> AppResult<()> {
            self.send(RootAgentCommand::Ping).await?;
            Ok(())
        }

        pub async fn ensure_panel_directories(&self, uid: u32, gid: u32) -> AppResult<()> {
            self.send(RootAgentCommand::EnsurePanelDirectories { uid, gid })
                .await?;
            Ok(())
        }

        pub async fn systemctl(
            &self,
            action: SystemctlAction,
            unit: &str,
        ) -> AppResult<RootAgentResponse> {
            self.send(RootAgentCommand::Systemctl {
                action,
                unit: unit.to_string(),
            })
            .await
        }

        pub async fn service_logs(&self, unit: &str, lines: u32) -> AppResult<RootAgentResponse> {
            self.send(RootAgentCommand::ServiceLogs {
                unit: unit.to_string(),
                lines,
            })
            .await
        }

        pub async fn process_signal(
            &self,
            pid: u32,
            signal: ProcessSignal,
        ) -> AppResult<RootAgentResponse> {
            self.send(RootAgentCommand::ProcessSignal { pid, signal })
                .await
        }

        async fn send(&self, command: RootAgentCommand) -> AppResult<RootAgentResponse> {
            let request = RootAgentRequest {
                token: self.token.clone(),
                command,
            };
            let payload = serde_json::to_string(&request).map_err(|e| {
                AppError::Internal(anyhow::anyhow!("Encode root agent request: {e}"))
            })?;

            let mut stream = timeout(self.request_timeout, UnixStream::connect(&self.socket_path))
                .await
                .map_err(|_| {
                    AppError::System("Timed out while connecting to root agent socket".to_string())
                })?
                .map_err(|e| {
                    AppError::System(format!(
                        "Failed to connect to root agent socket ({}): {}",
                        self.socket_path.display(),
                        e
                    ))
                })?;

            timeout(self.request_timeout, stream.write_all(payload.as_bytes()))
                .await
                .map_err(|_| {
                    AppError::System("Timed out while sending root agent request".to_string())
                })?
                .map_err(|e| {
                    AppError::System(format!("Failed to write root agent request: {}", e))
                })?;

            timeout(self.request_timeout, stream.write_all(b"\n"))
                .await
                .map_err(|_| {
                    AppError::System("Timed out while finalizing root agent request".to_string())
                })?
                .map_err(|e| {
                    AppError::System(format!("Failed to finalize root agent request: {}", e))
                })?;

            timeout(self.request_timeout, stream.flush())
                .await
                .map_err(|_| {
                    AppError::System("Timed out while flushing root agent request".to_string())
                })?
                .map_err(|e| {
                    AppError::System(format!("Failed to flush root agent request: {}", e))
                })?;

            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            let read = timeout(self.request_timeout, reader.read_line(&mut line))
                .await
                .map_err(|_| {
                    AppError::System("Timed out while reading root agent response".to_string())
                })?
                .map_err(|e| {
                    AppError::System(format!("Failed to read root agent response: {}", e))
                })?;

            if read == 0 {
                return Err(AppError::System(
                    "Root agent closed connection without response".to_string(),
                ));
            }

            let response: RootAgentResponse =
                serde_json::from_str(line.trim_end()).map_err(|e| {
                    AppError::System(format!("Invalid root agent response payload: {}", e))
                })?;

            if response.success {
                Ok(response)
            } else {
                Err(AppError::System(response.message))
            }
        }
    }

    pub async fn run_root_agent() -> anyhow::Result<()> {
        let token = super::ROOT_AGENT_SHARED_TOKEN.to_string();
        let socket_path = PathBuf::from(super::ROOT_AGENT_SOCKET_PATH);
        ensure_socket_parent(&socket_path)?;

        if socket_path.exists() {
            std::fs::remove_file(&socket_path).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to remove stale root agent socket ({}): {}",
                    socket_path.display(),
                    e
                )
            })?;
        }

        let listener = UnixListener::bind(&socket_path).map_err(|e| {
            anyhow::anyhow!(
                "Failed to bind root agent socket ({}): {}",
                socket_path.display(),
                e
            )
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            // Backend runs as non-root user; keep socket world-readable/writable for local dev.
            let perms = std::fs::Permissions::from_mode(0o666);
            std::fs::set_permissions(&socket_path, perms).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to set root agent socket permissions ({}): {}",
                    socket_path.display(),
                    e
                )
            })?;
        }

        tracing::info!("Root agent listening on {}", socket_path.display());

        loop {
            let (stream, _) = listener
                .accept()
                .await
                .map_err(|e| anyhow::anyhow!("Accept root agent connection failed: {}", e))?;
            let expected_token = token.clone();

            tokio::spawn(async move {
                if let Err(e) = handle_client(stream, &expected_token).await {
                    tracing::warn!("Root agent request failed: {}", e);
                }
            });
        }
    }

    async fn handle_client(mut stream: UnixStream, expected_token: &str) -> anyhow::Result<()> {
        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();

        let bytes_read = timeout(IO_TIMEOUT, reader.read_line(&mut line))
            .await
            .map_err(|_| anyhow::anyhow!("Timed out while reading agent request"))??;

        if bytes_read == 0 {
            return Ok(());
        }

        if line.len() > MAX_REQUEST_BYTES {
            write_response(
                &mut stream,
                RootAgentResponse::error("Request payload too large"),
            )
            .await?;
            return Ok(());
        }

        let request = match serde_json::from_str::<RootAgentRequest>(line.trim_end()) {
            Ok(req) => req,
            Err(_) => {
                write_response(
                    &mut stream,
                    RootAgentResponse::error("Invalid request payload"),
                )
                .await?;
                return Ok(());
            }
        };

        if request.token != expected_token {
            write_response(
                &mut stream,
                RootAgentResponse::error("Unauthorized request"),
            )
            .await?;
            return Ok(());
        }

        let response = match dispatch_command(request.command).await {
            Ok(resp) => resp,
            Err(err) => RootAgentResponse::error(err),
        };

        write_response(&mut stream, response).await?;
        Ok(())
    }

    async fn write_response(
        stream: &mut UnixStream,
        response: RootAgentResponse,
    ) -> anyhow::Result<()> {
        let payload = serde_json::to_vec(&response)
            .map_err(|e| anyhow::anyhow!("Failed to encode response: {}", e))?;

        timeout(IO_TIMEOUT, stream.write_all(&payload))
            .await
            .map_err(|_| anyhow::anyhow!("Timed out writing response"))??;

        timeout(IO_TIMEOUT, stream.write_all(b"\n"))
            .await
            .map_err(|_| anyhow::anyhow!("Timed out finalizing response"))??;

        timeout(IO_TIMEOUT, stream.flush())
            .await
            .map_err(|_| anyhow::anyhow!("Timed out flushing response"))??;

        Ok(())
    }

    async fn dispatch_command(command: RootAgentCommand) -> Result<RootAgentResponse, String> {
        match command {
            RootAgentCommand::Ping => Ok(RootAgentResponse::ok("pong")),
            RootAgentCommand::EnsurePanelDirectories { uid, gid } => {
                run_ensure_panel_directories(uid, gid).await
            }
            RootAgentCommand::Systemctl { action, unit } => {
                if !is_valid_unit_name(&unit) {
                    return Err("Invalid systemd unit name".to_string());
                }

                run_systemctl(action, &unit).await
            }
            RootAgentCommand::ServiceLogs { unit, lines } => {
                if !is_valid_unit_name(&unit) {
                    return Err("Invalid systemd unit name".to_string());
                }

                run_service_logs(&unit, lines.min(1000)).await
            }
            RootAgentCommand::ProcessSignal { pid, signal } => {
                if pid <= 1 {
                    return Err("PID must be greater than 1".to_string());
                }

                run_process_signal(pid, signal).await
            }
        }
    }

    async fn run_systemctl(
        action: SystemctlAction,
        unit: &str,
    ) -> Result<RootAgentResponse, String> {
        let args = vec![action.as_str().to_string(), unit.to_string()];
        run_command("systemctl", &args).await?;

        Ok(RootAgentResponse::ok(format!(
            "Service {} {} successfully",
            unit,
            action.past_tense()
        )))
    }

    async fn run_service_logs(unit: &str, lines: u32) -> Result<RootAgentResponse, String> {
        let args = vec![
            "-u".to_string(),
            unit.to_string(),
            "-n".to_string(),
            lines.to_string(),
            "--no-pager".to_string(),
        ];
        let output = run_command("journalctl", &args).await?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let logs = stdout.lines().map(|line| line.to_string()).collect();

        Ok(RootAgentResponse::with_logs(logs))
    }

    async fn run_process_signal(
        pid: u32,
        signal: ProcessSignal,
    ) -> Result<RootAgentResponse, String> {
        let args = vec![signal.signal_arg().to_string(), pid.to_string()];
        run_command("kill", &args).await?;

        Ok(RootAgentResponse::ok(format!(
            "Process {} {}",
            pid,
            signal.past_tense()
        )))
    }

    async fn run_ensure_panel_directories(uid: u32, gid: u32) -> Result<RootAgentResponse, String> {
        use std::os::unix::fs::PermissionsExt;

        std::fs::create_dir_all(super::PANEL_ROOT_DIR)
            .map_err(|e| format!("Failed to create '{}': {}", super::PANEL_ROOT_DIR, e))?;

        let dir_perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(super::PANEL_ROOT_DIR, dir_perms).map_err(|e| {
            format!(
                "Failed to set permissions on '{}': {}",
                super::PANEL_ROOT_DIR,
                e
            )
        })?;

        let args = vec![
            "-R".to_string(),
            format!("{}:{}", uid, gid),
            super::PANEL_ROOT_DIR.to_string(),
        ];
        run_command("chown", &args).await?;

        Ok(RootAgentResponse::ok(format!(
            "Ensured '{}' is owned by {}:{}",
            super::PANEL_ROOT_DIR,
            uid,
            gid
        )))
    }

    async fn run_command(program: &str, args: &[String]) -> Result<std::process::Output, String> {
        let mut command = tokio::process::Command::new(program);
        command
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let child = command
            .spawn()
            .map_err(|e| format!("Failed to execute {}: {}", program, e))?;

        let output = timeout(COMMAND_TIMEOUT, child.wait_with_output())
            .await
            .map_err(|_| format!("{} timed out", program))
            .and_then(|res| res.map_err(|e| format!("{} failed: {}", program, e)))?;

        if output.status.success() {
            Ok(output)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let detail = if !stderr.is_empty() {
                stderr
            } else if !stdout.is_empty() {
                stdout
            } else {
                format!("exit status {}", output.status)
            };

            Err(format!("{} {}", program, detail))
        }
    }

    fn is_valid_unit_name(name: &str) -> bool {
        !name.is_empty()
            && name.len() <= 256
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '@')
    }

    fn ensure_socket_parent(socket_path: &Path) -> anyhow::Result<()> {
        let parent = socket_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid root agent socket path"))?;

        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to create socket directory ({}): {}",
                    parent.display(),
                    e
                )
            })?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            // Allow non-root backend process to traverse to the socket.
            let dir_perms = std::fs::Permissions::from_mode(0o755);
            std::fs::set_permissions(parent, dir_perms).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to set socket directory permissions ({}): {}",
                    parent.display(),
                    e
                )
            })?;
        }

        Ok(())
    }

    pub use RootAgentClient as PublicRootAgentClient;
}

#[cfg(target_family = "unix")]
pub use unix::{run_root_agent, PublicRootAgentClient as RootAgentClient};

#[cfg(not(target_family = "unix"))]
#[derive(Debug, Clone)]
pub struct RootAgentClient;

#[cfg(not(target_family = "unix"))]
impl RootAgentClient {
    pub fn new() -> crate::error::AppResult<Self> {
        Err(crate::error::AppError::System(
            "Root agent is only supported on Unix platforms".to_string(),
        ))
    }

    pub async fn ping(&self) -> crate::error::AppResult<()> {
        Err(crate::error::AppError::System(
            "Root agent is only supported on Unix platforms".to_string(),
        ))
    }

    pub async fn ensure_panel_directories(
        &self,
        _uid: u32,
        _gid: u32,
    ) -> crate::error::AppResult<()> {
        Err(crate::error::AppError::System(
            "Root agent is only supported on Unix platforms".to_string(),
        ))
    }

    pub async fn systemctl(
        &self,
        _action: SystemctlAction,
        _unit: &str,
    ) -> crate::error::AppResult<RootAgentResponse> {
        Err(crate::error::AppError::System(
            "Root agent is only supported on Unix platforms".to_string(),
        ))
    }

    pub async fn service_logs(
        &self,
        _unit: &str,
        _lines: u32,
    ) -> crate::error::AppResult<RootAgentResponse> {
        Err(crate::error::AppError::System(
            "Root agent is only supported on Unix platforms".to_string(),
        ))
    }

    pub async fn process_signal(
        &self,
        _pid: u32,
        _signal: ProcessSignal,
    ) -> crate::error::AppResult<RootAgentResponse> {
        Err(crate::error::AppError::System(
            "Root agent is only supported on Unix platforms".to_string(),
        ))
    }
}

#[cfg(not(target_family = "unix"))]
pub async fn run_root_agent() -> anyhow::Result<()> {
    anyhow::bail!("Root agent is only supported on Unix platforms")
}
