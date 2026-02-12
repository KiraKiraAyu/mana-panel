use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::ErrorKind;
use std::net::TcpListener;

use crate::error::{AppError, AppResult};
use crate::services::docker::{
    CreateContainerRequest, DockerActionResponse, DockerService, PortMap,
};

const LABEL_MANAGED: &str = "mana-panel.managed";
const LABEL_APP_ROLE: &str = "mana-panel.app.role";
const LABEL_APP_CATEGORY: &str = "mana-panel.app.category";
const LABEL_APP_TEMPLATE: &str = "mana-panel.app.template";
const LABEL_APP_NAME: &str = "mana-panel.app.name";
const LABEL_PROXY_DOMAIN: &str = "mana-panel.proxy.site";
const LABEL_PROXY_UPSTREAM: &str = "mana-panel.proxy.upstream";

const APP_ROLE_VALUE: &str = "application";
const APP_CATEGORY_PROXY: &str = "proxy";

const TEMPLATE_NGINX_STANDARD: &str = "nginx-standard";
const TEMPLATE_CADDY_AUTOSSL: &str = "caddy-autossl";

#[derive(Debug, Clone, Serialize)]
pub struct ApplicationTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub image: String,
    pub default_ports: Vec<u16>,
    pub requires_domain: bool,
    pub requires_upstream: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApplicationPort {
    pub ip: String,
    pub private_port: u16,
    pub public_port: Option<u16>,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApplicationInstance {
    pub id: String,
    pub name: String,
    pub template_id: String,
    pub category: String,
    pub image: String,
    pub state: String,
    pub status: String,
    pub ports: Vec<ApplicationPort>,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InstallApplicationRequest {
    pub template_id: String,
    pub name: Option<String>,
    pub image_override: Option<String>,

    // For proxy-like templates
    pub upstream_host: Option<String>,
    pub upstream_port: Option<u16>,
    pub domain: Option<String>,

    // Host-published ports
    pub listen_http_port: Option<u16>,
    pub listen_https_port: Option<u16>,

    pub env: Option<Vec<String>>,
    pub replace_existing: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PortConflict {
    pub port: u16,
    pub occupied_by: String,
    pub occupant_type: String, // "container" | "host-process"
    pub occupant_container_id: Option<String>,
    pub occupant_image: Option<String>,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PreflightResult {
    pub ok: bool,
    pub requested_ports: Vec<u16>,
    pub conflicts: Vec<PortConflict>,
    pub warnings: Vec<String>,
}

#[derive(Clone)]
pub struct ApplicationManager {
    docker: DockerService,
}

impl ApplicationManager {
    pub fn new(docker: DockerService) -> Self {
        Self { docker }
    }

    pub fn list_templates(&self) -> Vec<ApplicationTemplate> {
        vec![
            ApplicationTemplate {
                id: TEMPLATE_NGINX_STANDARD.to_string(),
                name: "Nginx-Standard".to_string(),
                description: "Standard Nginx container template".to_string(),
                category: APP_CATEGORY_PROXY.to_string(),
                image: "nginx:stable-alpine".to_string(),
                default_ports: vec![80, 443],
                requires_domain: false,
                requires_upstream: false,
            },
            ApplicationTemplate {
                id: TEMPLATE_CADDY_AUTOSSL.to_string(),
                name: "Caddy-AutoSSL".to_string(),
                description: "Caddy reverse proxy template with automatic HTTPS using a domain"
                    .to_string(),
                category: APP_CATEGORY_PROXY.to_string(),
                image: "caddy:2-alpine".to_string(),
                default_ports: vec![80, 443],
                requires_domain: true,
                requires_upstream: true,
            },
        ]
    }

    pub async fn list_instances(&self) -> AppResult<Vec<ApplicationInstance>> {
        let containers = self.docker.list_containers(true).await?;

        let instances = containers
            .into_iter()
            .filter(|c| {
                c.labels
                    .get(LABEL_MANAGED)
                    .map(|v| v == "true")
                    .unwrap_or(false)
            })
            .filter(|c| {
                c.labels
                    .get(LABEL_APP_ROLE)
                    .map(|v| v == APP_ROLE_VALUE)
                    .unwrap_or(false)
            })
            .map(|c| ApplicationInstance {
                id: c.id,
                name: c
                    .labels
                    .get(LABEL_APP_NAME)
                    .cloned()
                    .or_else(|| c.names.first().cloned())
                    .unwrap_or_else(|| "unnamed".to_string()),
                template_id: c
                    .labels
                    .get(LABEL_APP_TEMPLATE)
                    .cloned()
                    .unwrap_or_default(),
                category: c
                    .labels
                    .get(LABEL_APP_CATEGORY)
                    .cloned()
                    .unwrap_or_default(),
                image: c.image,
                state: c.state,
                status: c.status,
                ports: c
                    .ports
                    .into_iter()
                    .map(|p| ApplicationPort {
                        ip: p.ip,
                        private_port: p.private_port,
                        public_port: p.public_port,
                        protocol: p.port_type,
                    })
                    .collect(),
                labels: c.labels,
            })
            .collect();

        Ok(instances)
    }

    pub async fn image_exists(&self, image: &str) -> AppResult<bool> {
        let target = image.trim();
        if target.is_empty() {
            return Ok(false);
        }

        let images = self.docker.list_images(true).await?;
        let normalized = if target.contains(':') {
            target.to_string()
        } else {
            format!("{}:latest", target)
        };

        for img in images {
            if img
                .repo_tags
                .iter()
                .any(|tag| tag == target || tag == &normalized)
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn install_application(
        &self,
        req: InstallApplicationRequest,
    ) -> AppResult<DockerActionResponse> {
        let template = self.find_template(&req.template_id).ok_or_else(|| {
            AppError::Validation(format!("Unknown template_id: {}", req.template_id))
        })?;

        if req.replace_existing.unwrap_or(false) && template.category == APP_CATEGORY_PROXY {
            let running = self.list_instances().await?;
            for inst in running
                .into_iter()
                .filter(|i| i.category == APP_CATEGORY_PROXY)
                .filter(|i| i.state == "running" || i.state == "created" || i.state == "restarting")
            {
                self.docker.remove_container(&inst.id, true).await?;
            }
        }

        let requested_ports = self.requested_ports_for_install(&req, &template);
        let preflight = self.preflight_ports(&requested_ports, None).await?;
        if !preflight.ok {
            return Err(AppError::Validation(
                self.format_conflict_message(&req.template_id, &preflight.conflicts),
            ));
        }

        let container_name = req
            .name
            .clone()
            .unwrap_or_else(|| format!("mana-{}", req.template_id));

        let image = req.image_override.clone().unwrap_or(template.image.clone());

        let mut labels = HashMap::new();
        labels.insert(LABEL_MANAGED.to_string(), "true".to_string());
        labels.insert(LABEL_APP_ROLE.to_string(), APP_ROLE_VALUE.to_string());
        labels.insert(LABEL_APP_CATEGORY.to_string(), template.category.clone());
        labels.insert(LABEL_APP_TEMPLATE.to_string(), template.id.clone());
        labels.insert(LABEL_APP_NAME.to_string(), container_name.clone());

        if let Some(domain) = req
            .domain
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            labels.insert(LABEL_PROXY_DOMAIN.to_string(), domain.to_string());
        }
        if let Some(upstream_host) = req
            .upstream_host
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            let upstream_port = req.upstream_port.unwrap_or(3000);
            labels.insert(
                LABEL_PROXY_UPSTREAM.to_string(),
                format!("{}:{}", upstream_host, upstream_port),
            );
        }

        let create_req = self.build_create_request(req, template, image, container_name, labels)?;
        let created = self.docker.create_container(create_req).await?;

        Ok(created)
    }

    pub async fn preflight_start(&self, container_id: &str) -> AppResult<PreflightResult> {
        let all = self.docker.list_containers(true).await?;
        let target = all
            .iter()
            .find(|c| c.id == container_id || c.id.starts_with(container_id))
            .ok_or_else(|| {
                AppError::NotFound(format!("Application container {} not found", container_id))
            })?;

        if !target
            .labels
            .get(LABEL_MANAGED)
            .map(|v| v == "true")
            .unwrap_or(false)
            || !target
                .labels
                .get(LABEL_APP_ROLE)
                .map(|v| v == APP_ROLE_VALUE)
                .unwrap_or(false)
        {
            return Err(AppError::Validation(
                "Target container is not a managed application instance".to_string(),
            ));
        }

        let mut requested_ports: Vec<u16> = target
            .ports
            .iter()
            .filter_map(|p| p.public_port)
            .collect::<HashSet<u16>>()
            .into_iter()
            .collect();

        if requested_ports.is_empty() {
            let template_id = target.labels.get(LABEL_APP_TEMPLATE).map(String::as_str);
            requested_ports = match template_id {
                Some(TEMPLATE_NGINX_STANDARD) | Some(TEMPLATE_CADDY_AUTOSSL) => vec![80, 443],
                _ => Vec::new(),
            };
        }

        self.preflight_ports(&requested_ports, Some(&target.id))
            .await
    }

    pub async fn start_application(&self, container_id: &str) -> AppResult<DockerActionResponse> {
        let preflight = self.preflight_start(container_id).await?;
        if !preflight.ok {
            return Err(AppError::Validation(
                self.format_conflict_message("application", &preflight.conflicts),
            ));
        }

        self.docker.start_container(container_id).await
    }

    pub async fn stop_application(&self, container_id: &str) -> AppResult<DockerActionResponse> {
        self.docker.stop_container(container_id).await
    }

    pub async fn remove_application(
        &self,
        container_id: &str,
        force: bool,
    ) -> AppResult<DockerActionResponse> {
        self.docker.remove_container(container_id, force).await
    }

    pub async fn preflight_ports(
        &self,
        requested_ports: &[u16],
        exclude_container_id: Option<&str>,
    ) -> AppResult<PreflightResult> {
        let requested_ports: Vec<u16> = requested_ports
            .iter()
            .copied()
            .filter(|p| *p > 0)
            .collect::<HashSet<u16>>()
            .into_iter()
            .collect();

        if requested_ports.is_empty() {
            return Ok(PreflightResult {
                ok: true,
                requested_ports: vec![],
                conflicts: vec![],
                warnings: vec![],
            });
        }

        let mut conflicts: Vec<PortConflict> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();

        // Docker-level conflict scan (primary path for your required behavior).
        let running = self.docker.list_containers(false).await?;
        for c in running {
            if let Some(exclude) = exclude_container_id {
                if c.id == exclude || c.id.starts_with(exclude) || exclude.starts_with(&c.id) {
                    continue;
                }
            }

            for p in &c.ports {
                if let Some(host_port) = p.public_port {
                    if requested_ports.contains(&host_port) {
                        let name = c
                            .names
                            .first()
                            .cloned()
                            .unwrap_or_else(|| c.id.chars().take(12).collect());

                        let suggestion = format!(
                            "Port {} is already used by container '{}'. Stop it first, or configure a different listening port.",
                            host_port, name
                        );

                        conflicts.push(PortConflict {
                            port: host_port,
                            occupied_by: name,
                            occupant_type: "container".to_string(),
                            occupant_container_id: Some(c.id.clone()),
                            occupant_image: Some(c.image.clone()),
                            suggestion,
                        });
                    }
                }
            }
        }

        // Host-level scan for non-Docker occupants.
        // We only add host-process conflicts for ports that do not already conflict with a Docker container.
        let conflicted_ports: HashSet<u16> = conflicts.iter().map(|c| c.port).collect();
        for port in requested_ports.iter().copied() {
            if conflicted_ports.contains(&port) {
                continue;
            }

            match TcpListener::bind(("0.0.0.0", port)) {
                Ok(listener) => {
                    drop(listener);
                }
                Err(err) if err.kind() == ErrorKind::AddrInUse => {
                    conflicts.push(PortConflict {
                        port,
                        occupied_by: "host process".to_string(),
                        occupant_type: "host-process".to_string(),
                        occupant_container_id: None,
                        occupant_image: None,
                        suggestion: format!(
                            "Port {} is occupied by a host process. Stop that process or choose a different port.",
                            port
                        ),
                    });
                }
                Err(err) if err.kind() == ErrorKind::PermissionDenied => {
                    warnings.push(format!(
                        "Permission denied while probing host port {}. Host-level occupancy could not be fully verified.",
                        port
                    ));
                }
                Err(err) => {
                    warnings.push(format!("Could not probe host port {}: {}", port, err));
                }
            }
        }

        Ok(PreflightResult {
            ok: conflicts.is_empty(),
            requested_ports,
            conflicts,
            warnings,
        })
    }

    fn build_create_request(
        &self,
        req: InstallApplicationRequest,
        template: ApplicationTemplate,
        image: String,
        container_name: String,
        labels: HashMap<String, String>,
    ) -> AppResult<CreateContainerRequest> {
        let http_host = req.listen_http_port.unwrap_or(80);
        let https_host = req.listen_https_port.unwrap_or(443);

        let mut ports: HashMap<String, Vec<PortMap>> = HashMap::new();
        ports.insert(
            "80/tcp".to_string(),
            vec![PortMap {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: http_host.to_string(),
            }],
        );
        ports.insert(
            "443/tcp".to_string(),
            vec![PortMap {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: https_host.to_string(),
            }],
        );

        let cmd = match template.id.as_str() {
            TEMPLATE_CADDY_AUTOSSL => {
                let domain = req
                    .domain
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| {
                        AppError::Validation(
                            "domain is required for template 'caddy-autossl'".to_string(),
                        )
                    })?;

                let upstream_host = req
                    .upstream_host
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| {
                        AppError::Validation(
                            "upstream_host is required for template 'caddy-autossl'".to_string(),
                        )
                    })?;

                let upstream_port = req.upstream_port.unwrap_or(3000);

                Some(vec![
                    "caddy".to_string(),
                    "reverse-proxy".to_string(),
                    "--from".to_string(),
                    domain.to_string(),
                    "--to".to_string(),
                    format!("{}:{}", upstream_host, upstream_port),
                ])
            }
            TEMPLATE_NGINX_STANDARD => None,
            _ => None,
        };

        Ok(CreateContainerRequest {
            name: Some(container_name),
            image,
            cmd,
            env: req.env,
            ports: Some(ports),
            volumes: None,
            restart_policy: Some("unless-stopped".to_string()),
            labels: Some(labels),
        })
    }

    fn requested_ports_for_install(
        &self,
        req: &InstallApplicationRequest,
        _template: &ApplicationTemplate,
    ) -> Vec<u16> {
        let mut ports = vec![
            req.listen_http_port.unwrap_or(80),
            req.listen_https_port.unwrap_or(443),
        ];
        ports.sort_unstable();
        ports.dedup();
        ports
    }

    fn find_template(&self, template_id: &str) -> Option<ApplicationTemplate> {
        self.list_templates()
            .into_iter()
            .find(|t| t.id == template_id)
    }

    fn format_conflict_message(&self, app_name: &str, conflicts: &[PortConflict]) -> String {
        if conflicts.is_empty() {
            return "No port conflicts detected".to_string();
        }

        let first = &conflicts[0];
        if first.occupant_type == "container" {
            format!(
                "Cannot start {}: port {} is already occupied by '{}'. Please change listening port(s) or stop the existing container.",
                app_name, first.port, first.occupied_by
            )
        } else {
            format!(
                "Cannot start {}: port {} is already occupied by a host process. Please change listening port(s) or stop the existing process.",
                app_name, first.port
            )
        }
    }
}
