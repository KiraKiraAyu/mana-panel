use bollard::Docker;
use bollard::container::{
    Config, CreateContainerOptions, InspectContainerOptions, ListContainersOptions, LogsOptions,
    RemoveContainerOptions, StatsOptions,
};
use bollard::image::{CreateImageOptions, ListImagesOptions, RemoveImageOptions};
use bollard::models::{ContainerInspectResponse, ContainerSummary, ImageSummary};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{AppError, AppResult};

/// High-level Docker service that wraps the bollard client.
#[derive(Clone)]
pub struct DockerService {
    client: Arc<Docker>,
}

// ---------------------------------------------------------------------------
// Public DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct ContainerInfo {
    pub id: String,
    pub names: Vec<String>,
    pub image: String,
    pub image_id: String,
    pub command: String,
    pub created: i64,
    pub state: String,
    pub status: String,
    pub ports: Vec<PortBinding>,
    pub labels: HashMap<String, String>,
    pub size_rw: Option<i64>,
    pub size_root_fs: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PortBinding {
    pub ip: String,
    pub private_port: u16,
    pub public_port: Option<u16>,
    pub port_type: String,
}

#[derive(Debug, Serialize)]
pub struct ContainerDetail {
    pub id: String,
    pub name: String,
    pub image: String,
    pub created: String,
    pub state: ContainerStateInfo,
    pub platform: String,
    pub driver: String,
    pub restart_count: i64,
    pub mounts: Vec<MountInfo>,
    pub network_mode: String,
    pub env: Vec<String>,
    pub cmd: Vec<String>,
    pub entrypoint: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ContainerStateInfo {
    pub status: String,
    pub running: bool,
    pub paused: bool,
    pub restarting: bool,
    pub oom_killed: bool,
    pub dead: bool,
    pub pid: i64,
    pub exit_code: i64,
    pub started_at: String,
    pub finished_at: String,
}

#[derive(Debug, Serialize)]
pub struct MountInfo {
    pub mount_type: String,
    pub source: String,
    pub destination: String,
    pub mode: String,
    pub rw: bool,
}

#[derive(Debug, Serialize)]
pub struct ContainerStats {
    pub cpu_percent: f64,
    pub memory_usage: u64,
    pub memory_limit: u64,
    pub memory_percent: f64,
    pub network_rx: u64,
    pub network_tx: u64,
    pub block_read: u64,
    pub block_write: u64,
    pub pids: u64,
}

#[derive(Debug, Serialize)]
pub struct ImageInfo {
    pub id: String,
    pub repo_tags: Vec<String>,
    pub repo_digests: Vec<String>,
    pub created: i64,
    pub size: i64,
    pub shared_size: i64,
    pub containers: i64,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct DockerActionResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateContainerRequest {
    pub name: Option<String>,
    pub image: String,
    pub cmd: Option<Vec<String>>,
    pub env: Option<Vec<String>>,
    pub ports: Option<HashMap<String, Vec<PortMap>>>,
    pub volumes: Option<HashMap<String, String>>,
    pub restart_policy: Option<String>,
    pub labels: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PortMap {
    pub host_ip: Option<String>,
    pub host_port: String,
}

#[derive(Debug, Serialize)]
pub struct PullProgress {
    pub status: String,
    pub id: Option<String>,
    pub progress: Option<String>,
}

// ---------------------------------------------------------------------------
// Implementation
// ---------------------------------------------------------------------------

impl DockerService {
    /// Create a new `DockerService`.  Tries the platform default connection
    /// (unix socket on Linux, named pipe on Windows).
    pub fn new() -> AppResult<Self> {
        let client = Docker::connect_with_local_defaults()
            .map_err(|e| AppError::System(format!("Failed to connect to Docker daemon: {}", e)))?;
        Ok(Self {
            client: Arc::new(client),
        })
    }

    // -- Containers ---------------------------------------------------------

    /// List containers.  When `all` is true stopped containers are included.
    pub async fn list_containers(&self, all: bool) -> AppResult<Vec<ContainerInfo>> {
        let mut filters = HashMap::new();
        if !all {
            filters.insert("status", vec!["running"]);
        }

        let opts = ListContainersOptions {
            all,
            filters,
            ..Default::default()
        };

        let containers = self
            .client
            .list_containers(Some(opts))
            .await
            .map_err(|e| AppError::System(format!("Docker list containers: {}", e)))?;

        Ok(containers.into_iter().map(Self::map_container).collect())
    }

    /// Inspect a single container by id/name.
    pub async fn inspect_container(&self, id: &str) -> AppResult<ContainerDetail> {
        let info = self
            .client
            .inspect_container(id, None::<InspectContainerOptions>)
            .await
            .map_err(|e| AppError::System(format!("Docker inspect container: {}", e)))?;

        Ok(Self::map_container_detail(info))
    }

    /// Start a container.
    pub async fn start_container(&self, id: &str) -> AppResult<DockerActionResponse> {
        self.client
            .start_container::<String>(id, None)
            .await
            .map_err(|e| AppError::System(format!("Docker start container: {}", e)))?;
        Ok(DockerActionResponse {
            success: true,
            message: format!("Container {} started", id),
        })
    }

    /// Stop a container.
    pub async fn stop_container(&self, id: &str) -> AppResult<DockerActionResponse> {
        self.client
            .stop_container(id, None)
            .await
            .map_err(|e| AppError::System(format!("Docker stop container: {}", e)))?;
        Ok(DockerActionResponse {
            success: true,
            message: format!("Container {} stopped", id),
        })
    }

    /// Restart a container.
    pub async fn restart_container(&self, id: &str) -> AppResult<DockerActionResponse> {
        self.client
            .restart_container(id, None)
            .await
            .map_err(|e| AppError::System(format!("Docker restart container: {}", e)))?;
        Ok(DockerActionResponse {
            success: true,
            message: format!("Container {} restarted", id),
        })
    }

    /// Remove a container.  `force` kills a running container before removal.
    pub async fn remove_container(&self, id: &str, force: bool) -> AppResult<DockerActionResponse> {
        let opts = RemoveContainerOptions {
            force,
            ..Default::default()
        };
        self.client
            .remove_container(id, Some(opts))
            .await
            .map_err(|e| AppError::System(format!("Docker remove container: {}", e)))?;
        Ok(DockerActionResponse {
            success: true,
            message: format!("Container {} removed", id),
        })
    }

    /// Fetch the last `tail` lines of logs for a container.
    pub async fn container_logs(&self, id: &str, tail: usize) -> AppResult<Vec<String>> {
        let opts = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            tail: tail.to_string(),
            ..Default::default()
        };

        let mut stream = self.client.logs(id, Some(opts));
        let mut lines: Vec<String> = Vec::new();

        while let Some(result) = stream.next().await {
            match result {
                Ok(output) => lines.push(output.to_string()),
                Err(e) => {
                    return Err(AppError::System(format!("Docker logs: {}", e)));
                }
            }
        }

        Ok(lines)
    }

    /// Get a one-shot stats snapshot for a container.
    pub async fn container_stats(&self, id: &str) -> AppResult<ContainerStats> {
        let opts = StatsOptions {
            stream: false,
            one_shot: true,
        };

        let mut stream = self.client.stats(id, Some(opts));

        if let Some(result) = stream.next().await {
            let stats = result.map_err(|e| AppError::System(format!("Docker stats: {}", e)))?;

            // CPU percentage
            let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64
                - stats.precpu_stats.cpu_usage.total_usage as f64;
            let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) as f64
                - stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;
            let online_cpus = stats.cpu_stats.online_cpus.unwrap_or(1) as f64;
            let cpu_percent = if system_delta > 0.0 {
                (cpu_delta / system_delta) * online_cpus * 100.0
            } else {
                0.0
            };

            // Memory
            let memory_usage = stats.memory_stats.usage.unwrap_or(0);
            let memory_limit = stats.memory_stats.limit.unwrap_or(1);
            let memory_percent = if memory_limit > 0 {
                (memory_usage as f64 / memory_limit as f64) * 100.0
            } else {
                0.0
            };

            // Network I/O
            let (mut network_rx, mut network_tx) = (0u64, 0u64);
            if let Some(networks) = &stats.networks {
                for net in networks.values() {
                    network_rx += net.rx_bytes;
                    network_tx += net.tx_bytes;
                }
            }

            // Block I/O
            let (mut block_read, mut block_write) = (0u64, 0u64);
            if let Some(ref io) = stats.blkio_stats.io_service_bytes_recursive {
                for entry in io {
                    match entry.op.as_str() {
                        "read" | "Read" => block_read += entry.value,
                        "write" | "Write" => block_write += entry.value,
                        _ => {}
                    }
                }
            }

            let pids = stats.pids_stats.current.unwrap_or(0);

            Ok(ContainerStats {
                cpu_percent,
                memory_usage,
                memory_limit,
                memory_percent,
                network_rx,
                network_tx,
                block_read,
                block_write,
                pids,
            })
        } else {
            Err(AppError::System("No stats received".into()))
        }
    }

    /// Create (but do not start) a container.
    pub async fn create_container(
        &self,
        req: CreateContainerRequest,
    ) -> AppResult<DockerActionResponse> {
        // Build port bindings for HostConfig
        let mut port_bindings: HashMap<String, Option<Vec<bollard::models::PortBinding>>> =
            HashMap::new();
        let mut exposed_ports: HashMap<String, HashMap<(), ()>> = HashMap::new();

        if let Some(ref ports) = req.ports {
            for (container_port, host_maps) in ports {
                exposed_ports.insert(container_port.clone(), HashMap::new());
                let bindings: Vec<bollard::models::PortBinding> = host_maps
                    .iter()
                    .map(|pm| bollard::models::PortBinding {
                        host_ip: pm.host_ip.clone(),
                        host_port: Some(pm.host_port.clone()),
                    })
                    .collect();
                port_bindings.insert(container_port.clone(), Some(bindings));
            }
        }

        // Build bind mounts
        let binds: Option<Vec<String>> = req.volumes.as_ref().map(|v| {
            v.iter()
                .map(|(host, container)| format!("{}:{}", host, container))
                .collect()
        });

        // Restart policy
        let restart_policy =
            req.restart_policy
                .as_deref()
                .map(|p| bollard::models::RestartPolicy {
                    name: Some(match p {
                        "always" => bollard::models::RestartPolicyNameEnum::ALWAYS,
                        "unless-stopped" => bollard::models::RestartPolicyNameEnum::UNLESS_STOPPED,
                        "on-failure" => bollard::models::RestartPolicyNameEnum::ON_FAILURE,
                        _ => bollard::models::RestartPolicyNameEnum::NO,
                    }),
                    maximum_retry_count: None,
                });

        let host_config = bollard::models::HostConfig {
            port_bindings: if port_bindings.is_empty() {
                None
            } else {
                Some(port_bindings)
            },
            binds,
            restart_policy,
            ..Default::default()
        };

        let config = Config {
            image: Some(req.image.clone()),
            cmd: req.cmd,
            env: req.env,
            labels: req.labels,
            exposed_ports: if exposed_ports.is_empty() {
                None
            } else {
                Some(exposed_ports)
            },
            host_config: Some(host_config),
            ..Default::default()
        };

        let opts = req.name.as_ref().map(|n| CreateContainerOptions {
            name: n.as_str(),
            platform: None,
        });

        let response = self
            .client
            .create_container(opts, config)
            .await
            .map_err(|e| AppError::System(format!("Docker create container: {}", e)))?;

        Ok(DockerActionResponse {
            success: true,
            message: format!("Container created with id {}", response.id),
        })
    }

    // -- Images -------------------------------------------------------------

    /// List local images.
    pub async fn list_images(&self, all: bool) -> AppResult<Vec<ImageInfo>> {
        let opts = ListImagesOptions::<String> {
            all,
            ..Default::default()
        };

        let images = self
            .client
            .list_images(Some(opts))
            .await
            .map_err(|e| AppError::System(format!("Docker list images: {}", e)))?;

        Ok(images.into_iter().map(Self::map_image).collect())
    }

    /// Check whether an image exists locally by tag or id.
    pub async fn image_exists(&self, image: &str) -> AppResult<bool> {
        let target = image.trim();
        if target.is_empty() {
            return Ok(false);
        }

        let normalized_target = target.trim_start_matches("sha256:");
        let images = self.list_images(true).await?;

        Ok(images.iter().any(|img| {
            img.id == target
                || img.id.trim_start_matches("sha256:") == normalized_target
                || img.repo_tags.iter().any(|tag| tag == target)
        }))
    }

    /// Remove a local image.
    pub async fn remove_image(&self, id: &str, force: bool) -> AppResult<DockerActionResponse> {
        let opts = RemoveImageOptions {
            force,
            ..Default::default()
        };

        self.client
            .remove_image(id, Some(opts), None)
            .await
            .map_err(|e| AppError::System(format!("Docker remove image: {}", e)))?;

        Ok(DockerActionResponse {
            success: true,
            message: format!("Image {} removed", id),
        })
    }

    /// Pull an image and stream progress events via channel in real time.
    pub async fn pull_image_stream(
        &self,
        image: &str,
        tx: tokio::sync::mpsc::Sender<PullProgress>,
    ) -> AppResult<()> {
        let (repo, tag) = if let Some(pos) = image.rfind(':') {
            (&image[..pos], &image[pos + 1..])
        } else {
            (image, "latest")
        };

        let opts = CreateImageOptions {
            from_image: repo,
            tag,
            ..Default::default()
        };

        let mut stream = self.client.create_image(Some(opts), None, None);

        while let Some(result) = stream.next().await {
            match result {
                Ok(info) => {
                    let item = PullProgress {
                        status: info.status.unwrap_or_default(),
                        id: info.id,
                        progress: info.progress,
                    };

                    if tx.send(item).await.is_err() {
                        // Receiver dropped (client disconnected), stop producing events.
                        break;
                    }
                }
                Err(e) => {
                    return Err(AppError::System(format!("Docker pull image: {}", e)));
                }
            }
        }

        Ok(())
    }

    /// Pull an image.  Returns collected progress messages.
    pub async fn pull_image(&self, image: &str) -> AppResult<Vec<PullProgress>> {
        let (repo, tag) = if let Some(pos) = image.rfind(':') {
            (&image[..pos], &image[pos + 1..])
        } else {
            (image, "latest")
        };

        let opts = CreateImageOptions {
            from_image: repo,
            tag,
            ..Default::default()
        };

        let mut stream = self.client.create_image(Some(opts), None, None);
        let mut progress_items: Vec<PullProgress> = Vec::new();

        while let Some(result) = stream.next().await {
            match result {
                Ok(info) => {
                    progress_items.push(PullProgress {
                        status: info.status.unwrap_or_default(),
                        id: info.id,
                        progress: info.progress,
                    });
                }
                Err(e) => {
                    return Err(AppError::System(format!("Docker pull image: {}", e)));
                }
            }
        }

        Ok(progress_items)
    }

    // -- Helpers (private) --------------------------------------------------

    fn map_container(c: ContainerSummary) -> ContainerInfo {
        let ports = c
            .ports
            .unwrap_or_default()
            .into_iter()
            .map(|p| PortBinding {
                ip: p.ip.unwrap_or_default(),
                private_port: p.private_port,
                public_port: p.public_port,
                port_type: p.typ.map(|t| format!("{:?}", t)).unwrap_or_default(),
            })
            .collect();

        ContainerInfo {
            id: c.id.unwrap_or_default(),
            names: c
                .names
                .unwrap_or_default()
                .into_iter()
                .map(|n| n.trim_start_matches('/').to_string())
                .collect(),
            image: c.image.unwrap_or_default(),
            image_id: c.image_id.unwrap_or_default(),
            command: c.command.unwrap_or_default(),
            created: c.created.unwrap_or(0),
            state: c.state.unwrap_or_default(),
            status: c.status.unwrap_or_default(),
            ports,
            labels: c.labels.unwrap_or_default(),
            size_rw: c.size_rw,
            size_root_fs: c.size_root_fs,
        }
    }

    fn map_container_detail(info: ContainerInspectResponse) -> ContainerDetail {
        let state = info.state.as_ref();
        let config = info.config.as_ref();
        let host_config = info.host_config.as_ref();

        let mounts = info
            .mounts
            .unwrap_or_default()
            .into_iter()
            .map(|m| MountInfo {
                mount_type: m.typ.map(|t| format!("{:?}", t)).unwrap_or_default(),
                source: m.source.unwrap_or_default(),
                destination: m.destination.unwrap_or_default(),
                mode: m.mode.unwrap_or_default(),
                rw: m.rw.unwrap_or(false),
            })
            .collect();

        ContainerDetail {
            id: info.id.unwrap_or_default(),
            name: info
                .name
                .unwrap_or_default()
                .trim_start_matches('/')
                .to_string(),
            image: config.and_then(|c| c.image.clone()).unwrap_or_default(),
            created: info.created.unwrap_or_default(),
            state: ContainerStateInfo {
                status: state
                    .and_then(|s| s.status.as_ref())
                    .map(|s| format!("{:?}", s))
                    .unwrap_or_default(),
                running: state.and_then(|s| s.running).unwrap_or(false),
                paused: state.and_then(|s| s.paused).unwrap_or(false),
                restarting: state.and_then(|s| s.restarting).unwrap_or(false),
                oom_killed: state.and_then(|s| s.oom_killed).unwrap_or(false),
                dead: state.and_then(|s| s.dead).unwrap_or(false),
                pid: state.and_then(|s| s.pid).unwrap_or(0),
                exit_code: state.and_then(|s| s.exit_code).unwrap_or(0),
                started_at: state.and_then(|s| s.started_at.clone()).unwrap_or_default(),
                finished_at: state
                    .and_then(|s| s.finished_at.clone())
                    .unwrap_or_default(),
            },
            platform: info.platform.unwrap_or_default(),
            driver: info.driver.unwrap_or_default(),
            restart_count: info.restart_count.unwrap_or(0),
            mounts,
            network_mode: host_config
                .and_then(|h| h.network_mode.clone())
                .unwrap_or_default(),
            env: config.and_then(|c| c.env.clone()).unwrap_or_default(),
            cmd: config.and_then(|c| c.cmd.clone()).unwrap_or_default(),
            entrypoint: config
                .and_then(|c| c.entrypoint.clone())
                .unwrap_or_default(),
        }
    }

    fn map_image(img: ImageSummary) -> ImageInfo {
        ImageInfo {
            id: img.id,
            repo_tags: img.repo_tags,
            repo_digests: img.repo_digests,
            created: img.created,
            size: img.size,
            shared_size: img.shared_size,
            containers: img.containers,
            labels: img.labels,
        }
    }
}
