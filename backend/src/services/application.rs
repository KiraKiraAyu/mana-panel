use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tera::{Context, Tera};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::services::application_task::{
    ApplicationTask, ApplicationTaskLogLevel, ApplicationTaskManager,
};
use crate::services::compose::{ComposeContainerStatus, ComposeProject, ComposeService};
use crate::services::docker::{DockerActionResponse, DockerService, PullProgress};
use crate::services::fs_utils;

const APP_COMPOSE_BASE_DIR: &str = ".mana-panel/applications";

const META_FILE_NAME: &str = "meta.json";
const RESOLVED_CONFIG_FILE_NAME: &str = "resolved_config.json";
const DEFAULT_APP_TOML: &str = "app.toml";
const DEFAULT_COMPOSE_FILE: &str = "docker-compose.yml";

static APP_OPERATION_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn app_operation_lock() -> &'static Mutex<()> {
    APP_OPERATION_LOCK.get_or_init(|| Mutex::new(()))
}

/// Defines supported UI/input schema types for template parameters.
/// These values are serialized to the frontend and used for validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InputType {
    Text,
    Password,
    Textarea,
    Number,
    Boolean,
    Select,
}

/// Describes one user-facing parameter declared by an application template.
/// A parameter can map to compose variables, env output keys, and validation rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationTemplateParam {
    pub key: String,
    pub label: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
    pub input: InputType,
    pub placeholder: Option<String>,
    pub env: Option<String>,
    /// If set, the value will be written to .env with this key.
    pub env_key: Option<String>,
    /// Validation rule (regex or keyword).
    pub rule: Option<String>,
    /// For `input = "select"`: the list of allowed values.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<String>,
}

/// Describes one logical port entry in a template.
/// The `key` is user-facing while `container_port/protocol` define runtime mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationTemplatePort {
    pub key: String,
    pub container_port: u16,
    pub protocol: String, // tcp | udp
    pub default_host_port: Option<u16>,
    pub required: bool,
}

/// Declares an environment variable rule in a template.
/// Values may be static (`value`) or derived from another parameter (`from`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationTemplateEnv {
    pub key: String,
    pub value: Option<String>,
    pub from: Option<String>,
    pub required: bool,
}

/// Compose service definition extracted from a template compose file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationTemplateService {
    pub name: String,
    pub image: Option<String>,
}

/// Canonical application template returned by the backend.
/// This is the aggregate model used by the UI to render forms and install options.
#[derive(Debug, Clone, Serialize)]
pub struct ApplicationTemplate {
    pub id: String,
    pub name: String,
    pub version: String,
    pub category: String,
    pub description: String,
    pub icon_path: Option<String>,
    pub readme_path: Option<String>,

    // Compose template metadata
    pub compose_file: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub services: Vec<ApplicationTemplateService>,

    // Dynamic form + runtime mappings
    pub params: Vec<ApplicationTemplateParam>,
    pub ports: Vec<ApplicationTemplatePort>,
    pub env: Vec<ApplicationTemplateEnv>,

    /// Explicit configuration file declarations from `[[config_file]]`.
    pub config_files: Vec<AppTomlConfigFile>,
    /// Explicit symlink declarations from `[[config_link]]`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub config_links: Vec<AppTomlConfigLink>,

    // Helpful for UI
    pub has_conf_templates: bool,
    pub app_dir: String,
}

/// Runtime-exposed port information for an installed application instance.
#[derive(Debug, Clone, Serialize)]
pub struct ApplicationPort {
    pub ip: String,
    pub private_port: u16,
    pub public_port: Option<u16>,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApplicationInstanceService {
    pub name: String,
    pub container_name: String,
    pub state: String,
    pub health: Option<String>,
    pub ports: Vec<ApplicationPort>,
}

/// Installed application instance model returned to the UI.
/// This reflects persisted metadata plus current compose/container state.
#[derive(Debug, Clone, Serialize)]
pub struct ApplicationInstance {
    pub id: Uuid,
    pub name: String,
    pub template_id: String,
    pub category: String,
    pub state: String,
    pub ports: Vec<ApplicationPort>,
    pub services: Vec<ApplicationInstanceService>,
}

/// Typed install value used by the install request API.
/// Values are accepted in native JSON form and normalized internally.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApplicationInstallValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

/// Normalized resolved value persisted after install.
/// Stores the final string representation and its original input type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationResolvedValue {
    pub key: String,
    pub input: InputType,
    pub value: String,
}

/// Snapshot of the effective install configuration written to disk.
/// Used for later inspection, edits, upgrades, and troubleshooting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationResolvedInstallConfig {
    pub template_id: String,
    pub instance_id: Uuid,
    pub instance_name: String,
    pub values: Vec<ApplicationResolvedValue>,
    pub port_bindings: HashMap<String, u16>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct ApplicationInstallResult {
    pub instance_id: Uuid,
    pub action: DockerActionResponse,
}

/// API payload for installing an application from a template.
#[derive(Debug, Clone, Deserialize)]
pub struct InstallApplicationRequest {
    pub template_id: String,
    pub name: Option<String>,
    #[serde(default)]
    pub typed_values: HashMap<String, ApplicationInstallValue>,
    pub port_bindings: Option<HashMap<String, u16>>,
    pub env: Option<Vec<String>>,
}

/// Persisted metadata for one installed instance.
/// This file is used to locate compose project identity and template linkage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationInstanceMeta {
    pub id: Uuid,
    pub name: String,
    pub template_id: String,
    pub category: String,
}

/// Raw `[metadata]` section from `app.toml`.
/// This is an input-only parsing model before validation/normalization.
#[derive(Debug, Clone, Deserialize)]
struct AppMetadata {
    id: String,
    name: String,
    version: String,
    category: String,
    description: String,
    icon: Option<String>,
    readme: Option<String>,
    compose_file: Option<String>,
}

/// Raw `[[port]]` section entry parsed from `app.toml`.
#[derive(Debug, Clone, Deserialize)]
struct AppTomlPort {
    key: String,
    container_port: Option<u16>,
    protocol: Option<String>,
    default_host_port: Option<u16>,
    required: Option<bool>,
}

/// Raw `[[param]]` section entry parsed from `app.toml`.
#[derive(Debug, Clone, Deserialize)]
struct AppTomlParam {
    key: String,
    label: Option<String>,
    description: Option<String>,
    required: Option<bool>,
    #[serde(rename = "default")]
    default_value: Option<String>,
    input: Option<String>,
    placeholder: Option<String>,
    env: Option<String>,
    env_key: Option<String>,
    rule: Option<String>,
    #[serde(default)]
    options: Vec<String>,
}

/// Raw `[[env]]` section entry parsed from `app.toml`.
#[derive(Debug, Clone, Deserialize)]
struct AppTomlEnv {
    key: String,
    value: Option<String>,
    from: Option<String>,
    required: Option<bool>,
}

/// `[[config_file]]` entry used for parsing and API output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppTomlConfigFile {
    pub path: String,
}

/// `[[config_link]]` entry used for parsing and API output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppTomlConfigLink {
    pub path: String,
    pub target: String,
}

/// Complete `app.toml` parsing model used before conversion into `ApplicationTemplate`.
#[derive(Debug, Clone, Deserialize)]
struct AppTomlDoc {
    metadata: AppMetadata,
    #[serde(default, rename = "param")]
    params: Vec<AppTomlParam>,
    #[serde(default, rename = "port")]
    ports: Vec<AppTomlPort>,
    #[serde(default, rename = "env")]
    env: Vec<AppTomlEnv>,
    #[serde(default, rename = "config_file")]
    config_files: Vec<AppTomlConfigFile>,
    #[serde(default, rename = "config_link")]
    config_links: Vec<AppTomlConfigLink>,
}

/// Application domain service responsible for template loading,
/// instance lifecycle operations, and compose-based orchestration.
#[derive(Clone)]
pub struct ApplicationManager {
    compose: ComposeService,
    app_root_dir: PathBuf,
    compose_projects_dir: PathBuf,
}

impl ApplicationManager {
    pub fn new(app_root_dir: PathBuf) -> Self {
        let compose_projects_dir = Self::resolve_compose_projects_dir(&app_root_dir);

        Self {
            compose: ComposeService::default(),
            app_root_dir,
            compose_projects_dir,
        }
    }

    fn resolve_compose_projects_dir(app_root_dir: &Path) -> PathBuf {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // Keep instance storage stable across different working directories by
        // anchoring to the app root parent (typically backend/.mana-panel/applications).
        let preferred = app_root_dir
            .parent()
            .map(|p| p.join(APP_COMPOSE_BASE_DIR))
            .unwrap_or_else(|| cwd.join(APP_COMPOSE_BASE_DIR));
        if preferred.is_dir() {
            return preferred;
        }

        // Compatibility fallback for older layouts that may already exist.
        let mut legacy_candidates = vec![
            cwd.join(APP_COMPOSE_BASE_DIR),
            cwd.join("backend").join(APP_COMPOSE_BASE_DIR),
        ];

        if let Some(parent) = cwd.parent() {
            legacy_candidates.push(parent.join(APP_COMPOSE_BASE_DIR));
            legacy_candidates.push(parent.join("backend").join(APP_COMPOSE_BASE_DIR));
        }

        if let Some(existing) = legacy_candidates.into_iter().find(|p| p.is_dir()) {
            return existing;
        }

        preferred
    }

    pub fn list_templates(&self) -> Vec<ApplicationTemplate> {
        match self.load_templates_from_app_dirs() {
            Ok(mut templates) => {
                templates.sort_by(|a, b| a.id.cmp(&b.id));
                templates
            }
            Err(err) => {
                tracing::warn!("Failed to load application templates: {}", err);
                Vec::new()
            }
        }
    }

    pub async fn list_instances(&self) -> AppResult<Vec<ApplicationInstance>> {
        self.compose.ensure_available().await?;

        let mut instances = Vec::new();

        for instance_id in self.list_instance_ids()? {
            let meta = match self.load_instance_meta(&instance_id) {
                Ok(meta) => meta,
                Err(err) => {
                    tracing::warn!(
                        "Skipping invalid application instance '{}': {}",
                        instance_id,
                        err
                    );
                    continue;
                }
            };

            let project = self.compose_project_for(&instance_id);
            let statuses = match self.compose.ps(&project).await {
                Ok(items) => items,
                Err(err) => {
                    tracing::warn!(
                        "Failed to inspect compose project '{}' for '{}': {}",
                        project.project_name,
                        instance_id,
                        err
                    );
                    vec![]
                }
            };

            instances.push(self.instance_from_compose_status(&meta, &statuses));
        }

        instances.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(instances)
    }

    pub async fn enqueue_install_application(
        &self,
        req: InstallApplicationRequest,
        docker: Option<DockerService>,
    ) -> ApplicationTask {
        let task_manager = ApplicationTaskManager::global().clone();
        let task = task_manager
            .create_install_task(req.template_id.clone(), req.name.clone())
            .await;
        let task_id = task.id.clone();

        let manager = self.clone();

        tokio::spawn(async move {
            let _ = task_manager
                .mark_running(&task_id, "Starting application installation")
                .await;
            let _ = task_manager
                .append_log(
                    &task_id,
                    ApplicationTaskLogLevel::Info,
                    "Resolving template and rendering compose files",
                )
                .await;

            let template_images = collect_template_images(&manager, &req.template_id);
            if !template_images.is_empty() {
                let _ = task_manager
                    .append_log(
                        &task_id,
                        ApplicationTaskLogLevel::Info,
                        format!("Found {} image(s) to prepare", template_images.len()),
                    )
                    .await;
            }

            if let Some(docker_service) = docker {
                for image in template_images {
                    let _ = task_manager
                        .append_log(
                            &task_id,
                            ApplicationTaskLogLevel::Info,
                            format!("Pulling image {}", image),
                        )
                        .await;

                    if let Err(err) = stream_image_pull_progress(
                        docker_service.clone(),
                        task_manager.clone(),
                        task_id.clone(),
                        image.clone(),
                    )
                    .await
                    {
                        let _ = task_manager
                            .mark_failed(
                                &task_id,
                                format!("Image pull failed for '{}': {}", image, err),
                            )
                            .await;
                        return;
                    }

                    let _ = task_manager
                        .append_log(
                            &task_id,
                            ApplicationTaskLogLevel::Info,
                            format!("Image ready: {}", image),
                        )
                        .await;
                }
            } else if !template_images.is_empty() {
                let _ = task_manager
                    .append_log(
                        &task_id,
                        ApplicationTaskLogLevel::Warn,
                        "Docker API unavailable, skip pre-pull and fallback to compose install",
                    )
                    .await;
            }

            match manager.execute_install_application(req).await {
                Ok(result) => {
                    let _ = task_manager
                        .append_log(
                            &task_id,
                            ApplicationTaskLogLevel::Info,
                            result.action.message.clone(),
                        )
                        .await;
                    let _ = task_manager
                        .mark_succeeded(
                            &task_id,
                            Some(result.instance_id),
                            format!(
                                "Application installed successfully (instance: {})",
                                result.instance_id
                            ),
                        )
                        .await;
                }
                Err(err) => {
                    let _ = task_manager
                        .mark_failed(&task_id, format!("Application install failed: {}", err))
                        .await;
                }
            }

            let _ = task_manager.prune_finished_older_than(180).await;
        });

        task
    }

    async fn execute_install_application(
        &self,
        req: InstallApplicationRequest,
    ) -> AppResult<ApplicationInstallResult> {
        let _guard = app_operation_lock().lock().await;
        self.compose.ensure_available().await?;

        let template = self.find_template(&req.template_id).ok_or_else(|| {
            AppError::Validation(format!("Unknown template_id: {}", req.template_id))
        })?;

        let instance_name = req
            .name
            .clone()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("mana-{}", template.id));

        let instance_id = Uuid::now_v7();
        let resolved_values = self.resolve_install_values(&template, &req)?;
        let resolved_ports = self.resolve_port_bindings(&template, &req)?;

        // Prepare instance directory
        let instance_dir = self.instance_dir(&instance_id);
        fs::create_dir_all(&instance_dir)
            .map_err(|e| AppError::System(format!("Failed to create instance directory: {}", e)))?;

        // Generate .env file
        let resolved_env = self.generate_env_file(
            &template,
            &resolved_values,
            &resolved_ports,
            &instance_dir,
            req.env.as_deref(),
        )?;

        // Build Tera rendering context
        let tera_ctx = self.build_tera_context(
            &resolved_values,
            &instance_name,
            &instance_id,
            req.env.as_deref(),
        );

        let rendered_conf = self.render_conf_templates(&template, &instance_id, &tera_ctx)?;
        let compose_yaml = self.render_compose_template(&template, &tera_ctx, &rendered_conf)?;

        let meta = ApplicationInstanceMeta {
            id: instance_id,
            name: instance_name.clone(),
            template_id: template.id.clone(),
            category: template.category.clone(),
        };

        let resolved_config = self.build_resolved_install_config(
            &template,
            instance_id,
            &instance_name,
            &resolved_values,
            &resolved_ports,
            &resolved_env,
        );

        self.write_instance_files(&meta.id, &compose_yaml, &meta, &resolved_config)?;

        let project = self.compose_project_for(&meta.id);
        let action = self.compose.up(&project).await?;
        Ok(ApplicationInstallResult {
            instance_id,
            action,
        })
    }

    pub async fn start_application(&self, instance_id: &str) -> AppResult<DockerActionResponse> {
        let _guard = app_operation_lock().lock().await;
        self.compose.ensure_available().await?;

        let instance_id = Self::parse_instance_id(instance_id)?;
        self.load_instance_meta(&instance_id)?;
        let project = self.compose_project_for(&instance_id);
        self.compose.start(&project).await
    }

    pub async fn stop_application(&self, instance_id: &str) -> AppResult<DockerActionResponse> {
        self.compose.ensure_available().await?;
        let instance_id = Self::parse_instance_id(instance_id)?;
        self.load_instance_meta(&instance_id)?;
        let project = self.compose_project_for(&instance_id);
        self.compose.stop(&project).await
    }

    pub async fn get_application_logs(&self, instance_id: &str, tail: usize) -> AppResult<DockerActionResponse> {
        self.compose.ensure_available().await?;
        let instance_id = Self::parse_instance_id(instance_id)?;
        self.load_instance_meta(&instance_id)?;
        let project = self.compose_project_for(&instance_id);
        self.compose.logs(&project, tail).await
    }

    pub fn get_application_env(&self, instance_id: &str) -> AppResult<HashMap<String, String>> {
        let instance_id = Self::parse_instance_id(instance_id)?;
        self.load_instance_meta(&instance_id)?;
        let env_path = self.instance_dir(&instance_id).join(".env");
        
        let content = fs::read_to_string(&env_path).unwrap_or_default();
        let mut env_map = HashMap::new();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                env_map.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
        
        Ok(env_map)
    }

    pub async fn update_application_env(
        &self,
        instance_id: &str,
        env_map: HashMap<String, String>,
        docker: Option<DockerService>,
    ) -> AppResult<DockerActionResponse> {
        let parsed_id = Self::parse_instance_id(instance_id)?;
        self.load_instance_meta(&parsed_id)?;
        
        // 1. Write the new .env file
        let env_path = self.instance_dir(&parsed_id).join(".env");
        let mut entries: Vec<_> = env_map.into_iter().collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        let content = entries
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("\n");
            
        fs::write(&env_path, content).map_err(|e| {
            AppError::System(format!("Failed to write updated .env file: {}", e))
        })?;
        
        // 2. Update the application to apply the new environment variables
        self.update_application(instance_id, docker).await
    }

    pub async fn update_application(
        &self,
        instance_id: &str,
        docker: Option<DockerService>,
    ) -> AppResult<DockerActionResponse> {
        let _guard = app_operation_lock().lock().await;
        self.compose.ensure_available().await?;

        let instance_id = Self::parse_instance_id(instance_id)?;
        let meta = self.load_instance_meta(&instance_id)?;

        // Re-render compose file with the latest template so upgrades pick up mount points/env changes
        match self.load_instance_resolved_config(&instance_id) {
            Ok(resolved_config) => {
                if let Some(template) = self.find_template(&meta.template_id) {
                    let mut resolved_values = HashMap::new();
                    for val in &resolved_config.values {
                        resolved_values.insert(val.key.clone(), val.value.clone());
                    }

                    // Build Tera rendering context
                    let tera_ctx =
                        self.build_tera_context(&resolved_values, &meta.name, &instance_id, None);

                    match self.render_conf_templates(&template, &instance_id, &tera_ctx) {
                        Ok(rendered_conf) => {
                            match self.render_compose_template(&template, &tera_ctx, &rendered_conf)
                            {
                                Ok(compose_yaml) => {
                                    if let Err(e) = self.write_instance_files(
                                        &meta.id,
                                        &compose_yaml,
                                        &meta,
                                        &resolved_config,
                                    ) {
                                        tracing::error!(
                                            "Failed to write updated instance files for application update '{}': {}",
                                            instance_id,
                                            e
                                        );
                                    }
                                }
                                Err(e) => tracing::error!(
                                    "Failed to re-render compose template during application update '{}': {}",
                                    instance_id,
                                    e
                                ),
                            }
                        }
                        Err(e) => tracing::error!(
                            "Failed to re-render conf templates during application update '{}': {}",
                            instance_id,
                            e
                        ),
                    }
                } else {
                    tracing::warn!(
                        "Template '{}' not found, skipping compose re-render for application update '{}'",
                        meta.template_id,
                        instance_id
                    );
                }
            }
            Err(e) => {
                // resolved_config.json is required to safely re-render the compose file.
                // If it is missing, we cannot guarantee the container will have the correct
                // volume mounts after compose up – abort to prevent a silent misconfiguration.
                return Err(AppError::System(format!(
                    "Cannot update application '{}': resolved_config.json is missing or corrupt. \
                     Please reinstall the application to restore the configuration snapshot. Error: {}",
                    instance_id, e
                )));
            }
        }
        let project = self.compose_project_for(&instance_id);
        let images = self.extract_images_from_compose_file(&project.compose_file)?;

        if let Some(docker_service) = docker {
            for image in &images {
                docker_service.pull_image(image).await?;
            }
        } else if !images.is_empty() {
            tracing::warn!(
                "Docker API unavailable, skip pre-pull for application update '{}'",
                instance_id
            );
        }

        let mut action = self.compose.up(&project).await?;
        if !images.is_empty() {
            action.message = format!("{} (updated {} image(s))", action.message, images.len());
        }

        Ok(action)
    }

    pub async fn remove_application(
        &self,
        instance_id: &str,
        force: bool,
    ) -> AppResult<DockerActionResponse> {
        self.compose.ensure_available().await?;
        let instance_id = Self::parse_instance_id(instance_id)?;
        self.load_instance_meta(&instance_id)?;
        let project = self.compose_project_for(&instance_id);

        let result = self.compose.down(&project, force).await?;
        let _ = self.delete_instance_dir(&instance_id);

        Ok(result)
    }

    fn find_template(&self, template_id: &str) -> Option<ApplicationTemplate> {
        self.list_templates()
            .into_iter()
            .find(|t| t.id == template_id)
    }

    fn resolve_install_values(
        &self,
        template: &ApplicationTemplate,
        req: &InstallApplicationRequest,
    ) -> AppResult<HashMap<String, String>> {
        let mut out = HashMap::<String, String>::new();

        for (k, v) in &req.typed_values {
            out.insert(k.clone(), Self::typed_value_to_string(v));
        }

        for p in &template.params {
            let existing = out.get(&p.key).map(|s| s.trim().to_string());
            let final_value = existing
                .filter(|s| !s.is_empty())
                .or_else(|| p.default_value.clone().filter(|s| !s.trim().is_empty()));

            match final_value {
                Some(raw) => {
                    let normalized = match p.input {
                        InputType::Boolean => {
                            Self::normalize_boolean_string(&raw).ok_or_else(|| {
                                AppError::Validation(format!(
                                    "Parameter '{}' must be a boolean value",
                                    p.key
                                ))
                            })?
                        }
                        InputType::Number => {
                            let trimmed = raw.trim();
                            if trimmed.parse::<f64>().is_err() {
                                return Err(AppError::Validation(format!(
                                    "Parameter '{}' must be a number",
                                    p.key
                                )));
                            }
                            trimmed.to_string()
                        }
                        InputType::Select => {
                            if !p.options.is_empty() && !p.options.iter().any(|opt| opt == &raw) {
                                return Err(AppError::Validation(format!(
                                    "Parameter '{}' must be one of: {}",
                                    p.key,
                                    p.options.join(", ")
                                )));
                            }
                            raw
                        }
                        _ => raw,
                    };

                    out.insert(p.key.clone(), normalized);
                }
                None if p.required => {
                    return Err(AppError::Validation(format!(
                        "Parameter '{}' is required for template '{}'",
                        p.key, template.id
                    )));
                }
                None => {}
            }
        }

        for e in &template.env {
            if e.required {
                let val = e
                    .value
                    .clone()
                    .or_else(|| e.from.as_ref().and_then(|k| out.get(k).cloned()));
                if val.as_ref().map(|v| v.trim().is_empty()).unwrap_or(true) {
                    return Err(AppError::Validation(format!(
                        "Required env '{}' cannot be resolved for template '{}'",
                        e.key, template.id
                    )));
                }
            }
        }

        Ok(out)
    }

    fn resolve_port_bindings(
        &self,
        template: &ApplicationTemplate,
        req: &InstallApplicationRequest,
    ) -> AppResult<HashMap<String, u16>> {
        let configured = req.port_bindings.clone().unwrap_or_default();
        let mut out = HashMap::new();

        let mut known_template_keys = HashSet::<String>::new();
        let mut known_template_endpoints = HashSet::<String>::new();

        for p in &template.ports {
            let proto = Self::normalize_protocol(&p.protocol);
            let endpoint = format!("{}/{}", p.container_port, proto).to_ascii_lowercase();

            known_template_keys.insert(p.key.to_ascii_lowercase());
            known_template_endpoints.insert(endpoint.clone());

            let by_key = configured.get(&p.key).copied();
            let by_endpoint = configured.get(&endpoint).copied();
            let selected = by_key.or(by_endpoint).or(p.default_host_port);

            match selected {
                Some(port) if port > 0 => {
                    out.insert(endpoint, port);
                }
                None if p.required => {
                    return Err(AppError::Validation(format!(
                        "Port binding for '{}' is required",
                        p.key
                    )));
                }
                _ => {}
            }
        }

        for (binding_key, host_port) in &configured {
            if *host_port == 0 {
                continue;
            }

            let normalized_key = binding_key.trim().to_ascii_lowercase();

            if known_template_keys.contains(&normalized_key)
                || known_template_endpoints.contains(&normalized_key)
            {
                continue;
            }

            let endpoint =
                Self::normalize_endpoint_binding_key(&normalized_key).ok_or_else(|| {
                    AppError::Validation(format!(
                        "Port binding key '{}' must be either a template key or '<container_port>/tcp|udp'",
                        binding_key
                    ))
                })?;

            out.insert(endpoint, *host_port);
        }

        Ok(out)
    }

    fn build_resolved_install_config(
        &self,
        template: &ApplicationTemplate,
        instance_id: Uuid,
        instance_name: &str,
        resolved_values: &HashMap<String, String>,
        resolved_ports: &HashMap<String, u16>,
        resolved_env: &HashMap<String, String>,
    ) -> ApplicationResolvedInstallConfig {
        let mut values = Vec::<ApplicationResolvedValue>::new();
        for p in &template.params {
            if let Some(value) = resolved_values.get(&p.key) {
                values.push(ApplicationResolvedValue {
                    key: p.key.clone(),
                    input: p.input.clone(),
                    value: value.clone(),
                });
            }
        }

        values.sort_by(|a, b| a.key.cmp(&b.key));

        ApplicationResolvedInstallConfig {
            template_id: template.id.clone(),
            instance_id,
            instance_name: instance_name.to_string(),
            values,
            port_bindings: resolved_ports.clone(),
            env: resolved_env.clone(),
        }
    }

    fn generate_env_file(
        &self,
        template: &ApplicationTemplate,
        resolved_values: &HashMap<String, String>,
        resolved_ports: &HashMap<String, u16>,
        instance_dir: &Path,
        env_overrides: Option<&[String]>,
    ) -> AppResult<HashMap<String, String>> {
        let mut env_map = HashMap::new();

        // 1. Standard Magic Variables (1Panel compatible)
        env_map.insert("TZ".to_string(), "Asia/Shanghai".to_string()); // Default, can be overridden

        // HTTP/HTTPS Ports
        if let Some(port) = resolved_ports.get("80/tcp") {
            env_map.insert("PANEL_APP_PORT_HTTP".to_string(), port.to_string());
        }
        if let Some(port) = resolved_ports.get("443/tcp") {
            env_map.insert("PANEL_APP_PORT_HTTPS".to_string(), port.to_string());
        }

        // 2. User Parameters with `env_key`
        for p in &template.params {
            if let Some(env_key) = &p.env_key {
                if let Some(val) = resolved_values.get(&p.key) {
                    let normalized = if env_key.eq_ignore_ascii_case("WEBSITE_DIR") {
                        Self::normalize_compose_host_path(val)
                    } else {
                        val.clone()
                    };
                    env_map.insert(env_key.clone(), normalized);
                }
            }
        }

        // 3. Explicit Environment Variables Overrides (from request)
        if let Some(overrides) = env_overrides {
            for item in overrides {
                if let Some((k, v)) = item.split_once('=') {
                    let key = k.trim().to_string();
                    let value = if key.eq_ignore_ascii_case("WEBSITE_DIR") {
                        Self::normalize_compose_host_path(v)
                    } else {
                        v.trim().to_string()
                    };
                    env_map.insert(key, value);
                }
            }
        }

        // Write to .env file
        let env_path = instance_dir.join(".env");
        let mut content = String::new();
        let mut entries = env_map.iter().collect::<Vec<_>>();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        for (k, v) in entries {
            content.push_str(&format!("{}={}\n", k, v));
        }

        fs::write(&env_path, content)
            .map_err(|e| AppError::System(format!("Failed to write .env file: {}", e)))?;

        Ok(env_map)
    }

    fn build_tera_context(
        &self,
        resolved_values: &HashMap<String, String>,
        instance_name: &str,
        instance_id: &Uuid,
        env_overrides: Option<&[String]>,
    ) -> Context {
        let mut ctx = Context::new();

        // User-supplied and default parameter values
        for (k, v) in resolved_values {
            ctx.insert(k, v);
        }

        // Built-in variables
        ctx.insert("app_name", instance_name);
        ctx.insert("app_id", &instance_id.to_string());

        // Environment variable overrides
        if let Some(envs) = env_overrides {
            for item in envs {
                if let Some((k, v)) = item.split_once('=') {
                    ctx.insert(k.trim(), &v.trim().to_string());
                }
            }
        }

        ctx
    }

    fn render_compose_template(
        &self,
        template: &ApplicationTemplate,
        ctx: &Context,
        rendered_conf_files: &[PathBuf],
    ) -> AppResult<String> {
        let compose_path = PathBuf::from(&template.app_dir).join(&template.compose_file);
        let raw = fs::read_to_string(&compose_path).map_err(|e| {
            AppError::System(format!(
                "Failed to read compose template '{}': {}",
                compose_path.display(),
                e
            ))
        })?;

        let mut full_ctx = ctx.clone();
        if !rendered_conf_files.is_empty() {
            let joined = rendered_conf_files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(",");
            full_ctx.insert("rendered_conf_files", &joined);
        }

        Self::render_tera_template(&raw, &full_ctx)
    }

    fn render_conf_templates(
        &self,
        template: &ApplicationTemplate,
        instance_id: &Uuid,
        ctx: &Context,
    ) -> AppResult<Vec<PathBuf>> {
        let app_dir = PathBuf::from(&template.app_dir);
        let instance_dir = self.instance_dir(instance_id);

        let mut rendered = Vec::new();
        if !template.config_files.is_empty() {
            // ---- Explicit [[config_file]] declarations ----
            for cf in &template.config_files {
                let dst = instance_dir.join(&cf.path);

                let src = app_dir.join(&cf.path);
                let src_metadata = fs::symlink_metadata(&src).map_err(|e| {
                    AppError::Validation(format!(
                        "Config template '{}' does not exist for app '{}': {}",
                        src.display(),
                        template.id,
                        e
                    ))
                })?;

                if let Some(parent) = dst.parent() {
                    fs::create_dir_all(parent).map_err(|e| {
                        AppError::System(format!(
                            "Failed to create directory '{}': {}",
                            parent.display(),
                            e
                        ))
                    })?;
                }

                if src_metadata.file_type().is_symlink() {
                    let link_target = fs::read_link(&src).map_err(|e| {
                        AppError::System(format!(
                            "Failed to read config symlink '{}': {}",
                            src.display(),
                            e
                        ))
                    })?;

                    fs_utils::ensure_symlink(&dst, &link_target)?;
                    rendered.push(dst);
                    continue;
                }

                let raw = fs::read_to_string(&src).map_err(|e| {
                    AppError::System(format!(
                        "Failed to read config template '{}': {}",
                        src.display(),
                        e
                    ))
                })?;

                let content = Self::render_tera_template(&raw, ctx)?;
                fs::write(&dst, &content).map_err(|e| {
                    AppError::System(format!(
                        "Failed to write rendered config '{}': {}",
                        dst.display(),
                        e
                    ))
                })?;

                rendered.push(dst);
            }
        }

        if !template.config_links.is_empty() {
            // ---- Explicit [[config_link]] declarations ----
            for cl in &template.config_links {
                let dst = instance_dir.join(&cl.path);
                let target = PathBuf::from(&cl.target);

                fs_utils::ensure_symlink(&dst, &target)?;
                rendered.push(dst);
            }
        }

        Ok(rendered)
    }

    fn load_templates_from_app_dirs(&self) -> AppResult<Vec<ApplicationTemplate>> {
        if !self.app_root_dir.exists() {
            return Ok(vec![]);
        }

        let entries = fs::read_dir(&self.app_root_dir).map_err(|e| {
            AppError::System(format!(
                "Failed to read app root directory '{}': {}",
                self.app_root_dir.display(),
                e
            ))
        })?;

        let mut templates = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| {
                AppError::System(format!(
                    "Failed to read app root entry '{}': {}",
                    self.app_root_dir.display(),
                    e
                ))
            })?;

            let app_dir = entry.path();
            if !app_dir.is_dir() {
                continue;
            }

            let app_toml = app_dir.join(DEFAULT_APP_TOML);
            if !app_toml.exists() {
                continue;
            }

            let doc = self.parse_app_toml(&app_toml)?;
            let template = self.build_template_from_doc(doc, &app_dir)?;
            templates.push(template);
        }

        self.validate_template_set(&templates)?;
        Ok(templates)
    }

    fn parse_app_toml(&self, path: &Path) -> AppResult<AppTomlDoc> {
        let raw = fs::read_to_string(path).map_err(|e| {
            AppError::System(format!(
                "Failed to read app.toml '{}': {}",
                path.display(),
                e
            ))
        })?;

        let mut doc: AppTomlDoc = toml::from_str(&raw).map_err(|e| {
            AppError::Validation(format!("Invalid app.toml '{}': {}", path.display(), e))
        })?;

        if doc.metadata.version.trim().is_empty() {
            doc.metadata.version = "0.1.0".to_string();
        }

        Ok(doc)
    }

    fn build_template_from_doc(
        &self,
        doc: AppTomlDoc,
        app_dir: &Path,
    ) -> AppResult<ApplicationTemplate> {
        let id = doc.metadata.id.trim().to_string();
        let name = doc.metadata.name.trim().to_string();
        let category = doc.metadata.category.trim().to_string();
        let description = doc.metadata.description.trim().to_string();

        if id.is_empty() {
            return Err(AppError::Validation(format!(
                "metadata.id is required in '{}'",
                app_dir.join(DEFAULT_APP_TOML).display()
            )));
        }
        if !Self::is_valid_slug(&id) {
            return Err(AppError::Validation(format!(
                "metadata.id must be a slug ([a-z0-9-], no leading/trailing '-') in '{}'",
                app_dir.join(DEFAULT_APP_TOML).display()
            )));
        }
        if name.is_empty() {
            return Err(AppError::Validation(format!(
                "metadata.name is required in '{}'",
                app_dir.join(DEFAULT_APP_TOML).display()
            )));
        }
        if category.is_empty() {
            return Err(AppError::Validation(format!(
                "metadata.category is required in '{}'",
                app_dir.join(DEFAULT_APP_TOML).display()
            )));
        }
        if description.is_empty() {
            return Err(AppError::Validation(format!(
                "metadata.description is required in '{}'",
                app_dir.join(DEFAULT_APP_TOML).display()
            )));
        }

        let compose_file = doc
            .metadata
            .compose_file
            .clone()
            .unwrap_or_else(|| DEFAULT_COMPOSE_FILE.to_string());

        let compose_path = app_dir.join(&compose_file);
        if !compose_path.exists() {
            return Err(AppError::Validation(format!(
                "Compose template '{}' does not exist for app '{}'",
                compose_path.display(),
                id
            )));
        }

        let params = doc
            .params
            .into_iter()
            .map(|p| {
                if p.key.trim().is_empty() {
                    return Err(AppError::Validation(format!(
                        "param.key cannot be empty in '{}'",
                        app_dir.join(DEFAULT_APP_TOML).display()
                    )));
                }

                let input_raw = p.input.unwrap_or_else(|| "text".to_string());
                let input_type = match input_raw.trim().to_ascii_lowercase().as_str() {
                    "text" => InputType::Text,
                    "password" => InputType::Password,
                    "textarea" => InputType::Textarea,
                    "number" => InputType::Number,
                    "boolean" | "checkbox" => InputType::Boolean,
                    "select" | "radio" => InputType::Select,
                    _ => InputType::Text,
                };

                Ok(ApplicationTemplateParam {
                    key: p.key.trim().to_string(),
                    label: p.label.unwrap_or_else(|| p.key.clone()),
                    description: p.description.unwrap_or_default(),
                    required: p.required.unwrap_or(false),
                    default_value: p.default_value,
                    input: input_type,
                    placeholder: p.placeholder,
                    env: p.env,
                    env_key: p.env_key,
                    rule: p.rule,
                    options: p.options,
                })
            })
            .collect::<AppResult<Vec<_>>>()?;

        let ports = doc
            .ports
            .into_iter()
            .map(|p| {
                let key = p.key.trim().to_string();
                if key.is_empty() {
                    return Err(AppError::Validation(format!(
                        "port.key cannot be empty in '{}'",
                        app_dir.join(DEFAULT_APP_TOML).display()
                    )));
                }

                let container_port = p.container_port.ok_or_else(|| {
                    AppError::Validation(format!(
                        "port.container_port is required for '{}' in '{}'",
                        key,
                        app_dir.join(DEFAULT_APP_TOML).display()
                    ))
                })?;

                if container_port == 0 {
                    return Err(AppError::Validation(format!(
                        "port.container_port cannot be 0 for '{}' in '{}'",
                        key,
                        app_dir.join(DEFAULT_APP_TOML).display()
                    )));
                }

                let protocol = Self::normalize_protocol(p.protocol.as_deref().unwrap_or("tcp"));

                Ok(ApplicationTemplatePort {
                    key,
                    container_port,
                    protocol,
                    default_host_port: p.default_host_port,
                    required: p.required.unwrap_or(false),
                })
            })
            .collect::<AppResult<Vec<_>>>()?;

        let env = doc
            .env
            .into_iter()
            .map(|e| {
                if e.key.trim().is_empty() {
                    return Err(AppError::Validation(format!(
                        "env.key cannot be empty in '{}'",
                        app_dir.join(DEFAULT_APP_TOML).display()
                    )));
                }

                Ok(ApplicationTemplateEnv {
                    key: e.key.trim().to_string(),
                    value: e.value,
                    from: e.from,
                    required: e.required.unwrap_or(false),
                })
            })
            .collect::<AppResult<Vec<_>>>()?;

        let config_files = doc.config_files;
        let config_links = doc
            .config_links
            .into_iter()
            .map(|c| {
                let path = c.path.trim().to_string();
                if path.is_empty() {
                    return Err(AppError::Validation(format!(
                        "config_link.path cannot be empty in '{}'",
                        app_dir.join(DEFAULT_APP_TOML).display()
                    )));
                }

                let target = c.target.trim().to_string();
                if target.is_empty() {
                    return Err(AppError::Validation(format!(
                        "config_link.target cannot be empty for '{}' in '{}'",
                        path,
                        app_dir.join(DEFAULT_APP_TOML).display()
                    )));
                }

                Ok(AppTomlConfigLink { path, target })
            })
            .collect::<AppResult<Vec<_>>>()?;

        let has_conf_templates = !config_files.is_empty() || !config_links.is_empty();

        let version = if doc.metadata.version.trim().is_empty() {
            "0.1.0".to_string()
        } else {
            doc.metadata.version.clone()
        };

        let template_services = self.extract_template_services(&compose_path)?;

        Ok(ApplicationTemplate {
            id,
            name,
            version,
            category,
            description,
            icon_path: doc
                .metadata
                .icon
                .map(|p| app_dir.join(p).to_string_lossy().to_string()),
            readme_path: doc
                .metadata
                .readme
                .map(|p| app_dir.join(p).to_string_lossy().to_string()),
            compose_file,
            services: template_services,
            params,
            ports,
            env,
            config_files,
            config_links,
            has_conf_templates,
            app_dir: app_dir.to_string_lossy().to_string(),
        })
    }

    fn validate_template_set(&self, templates: &[ApplicationTemplate]) -> AppResult<()> {
        let mut ids = HashSet::<&str>::new();
        for t in templates {
            if !ids.insert(t.id.as_str()) {
                return Err(AppError::Validation(format!(
                    "Duplicate application template id '{}'",
                    t.id
                )));
            }
        }
        Ok(())
    }

    fn instance_dir(&self, instance_id: &Uuid) -> PathBuf {
        self.compose_projects_dir.join(instance_id.to_string())
    }

    fn instance_compose_file(&self, instance_id: &Uuid) -> PathBuf {
        self.instance_dir(instance_id).join(DEFAULT_COMPOSE_FILE)
    }

    fn instance_meta_file(&self, instance_id: &Uuid) -> PathBuf {
        self.instance_dir(instance_id).join(META_FILE_NAME)
    }

    fn instance_resolved_config_file(&self, instance_id: &Uuid) -> PathBuf {
        self.instance_dir(instance_id)
            .join(RESOLVED_CONFIG_FILE_NAME)
    }

    fn write_instance_files(
        &self,
        instance_id: &Uuid,
        compose_yaml: &str,
        meta: &ApplicationInstanceMeta,
        resolved_config: &ApplicationResolvedInstallConfig,
    ) -> AppResult<()> {
        let dir = self.instance_dir(instance_id);
        fs::create_dir_all(&dir).map_err(|e| {
            AppError::System(format!(
                "Failed to create instance directory '{}': {}",
                dir.display(),
                e
            ))
        })?;

        let compose_file = self.instance_compose_file(instance_id);
        fs::write(&compose_file, compose_yaml).map_err(|e| {
            AppError::System(format!(
                "Failed to write compose file '{}': {}",
                compose_file.display(),
                e
            ))
        })?;

        let meta_file = self.instance_meta_file(instance_id);
        let raw = serde_json::to_string_pretty(meta).map_err(|e| {
            AppError::System(format!(
                "Failed to serialize metadata for instance '{}': {}",
                instance_id, e
            ))
        })?;

        fs::write(&meta_file, raw).map_err(|e| {
            AppError::System(format!(
                "Failed to write metadata file '{}': {}",
                meta_file.display(),
                e
            ))
        })?;

        let resolved_file = self.instance_resolved_config_file(instance_id);
        let resolved_raw = serde_json::to_string_pretty(resolved_config).map_err(|e| {
            AppError::System(format!(
                "Failed to serialize resolved config for instance '{}': {}",
                instance_id, e
            ))
        })?;

        fs::write(&resolved_file, resolved_raw).map_err(|e| {
            AppError::System(format!(
                "Failed to write resolved config file '{}': {}",
                resolved_file.display(),
                e
            ))
        })?;

        Ok(())
    }

    /// Find all installed instance IDs that were created from a specific template.
    /// Returns a list of (Uuid, container_name) tuples.
    pub fn list_instance_ids_by_template(
        &self,
        template_id: &str,
    ) -> AppResult<Vec<(uuid::Uuid, String)>> {
        let mut result = Vec::new();
        for id in self.list_instance_ids()? {
            if let Ok(meta) = self.load_instance_meta(&id) {
                if meta.template_id == template_id {
                    // Container name follows the convention set in docker-compose.yml:
                    // container_name: "{{ app_name }}" which resolves to meta.name
                    result.push((id, meta.name.clone()));
                }
            }
        }
        Ok(result)
    }

    /// Returns the filesystem path for a given instance UUID.
    pub fn get_instance_dir(&self, instance_id: &uuid::Uuid) -> PathBuf {
        self.instance_dir(instance_id)
    }

    fn list_instance_ids(&self) -> AppResult<Vec<Uuid>> {
        if !self.compose_projects_dir.exists() {
            return Ok(vec![]);
        }

        let mut ids = Vec::new();
        for entry in fs::read_dir(&self.compose_projects_dir).map_err(|e| {
            AppError::System(format!(
                "Failed to read instance directory '{}': {}",
                self.compose_projects_dir.display(),
                e
            ))
        })? {
            let entry =
                entry.map_err(|e| AppError::System(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(id) = path.file_name().and_then(|n| n.to_str()) {
                    match Uuid::parse_str(id) {
                        Ok(uuid) => ids.push(uuid),
                        Err(_) => tracing::warn!(
                            "Skipping non-UUID application instance directory '{}'",
                            id
                        ),
                    }
                }
            }
        }

        ids.sort();
        Ok(ids)
    }

    pub fn load_instance_meta(&self, instance_id: &Uuid) -> AppResult<ApplicationInstanceMeta> {
        let meta_file = self.instance_meta_file(instance_id);
        if !meta_file.exists() {
            return Err(AppError::NotFound(format!(
                "Application instance '{}' not found",
                instance_id
            )));
        }

        let raw = fs::read_to_string(&meta_file).map_err(|e| {
            AppError::System(format!(
                "Failed to read metadata file '{}': {}",
                meta_file.display(),
                e
            ))
        })?;

        serde_json::from_str(&raw).map_err(|e| {
            AppError::System(format!(
                "Failed to parse metadata file '{}': {}",
                meta_file.display(),
                e
            ))
        })
    }

    pub fn load_instance_resolved_config(
        &self,
        instance_id: &Uuid,
    ) -> AppResult<ApplicationResolvedInstallConfig> {
        let resolved_file = self.instance_resolved_config_file(instance_id);
        if !resolved_file.exists() {
            return Err(AppError::NotFound(format!(
                "Resolved config for instance '{}' not found",
                instance_id
            )));
        }

        let raw = fs::read_to_string(&resolved_file).map_err(|e| {
            AppError::System(format!(
                "Failed to read resolved config file '{}': {}",
                resolved_file.display(),
                e
            ))
        })?;

        serde_json::from_str(&raw).map_err(|e| {
            AppError::System(format!(
                "Failed to parse resolved config file '{}': {}",
                resolved_file.display(),
                e
            ))
        })
    }

    fn delete_instance_dir(&self, instance_id: &Uuid) -> AppResult<()> {
        let dir = self.instance_dir(instance_id);
        if !dir.exists() {
            return Ok(());
        }

        fs::remove_dir_all(&dir).map_err(|e| {
            AppError::System(format!(
                "Failed to delete instance directory '{}': {}",
                dir.display(),
                e
            ))
        })
    }

    fn compose_project_for(&self, instance_id: &Uuid) -> ComposeProject {
        ComposeProject {
            compose_file: self.instance_compose_file(instance_id),
            project_name: format!("mana-{}", instance_id),
        }
    }

    fn instance_from_compose_status(
        &self,
        meta: &ApplicationInstanceMeta,
        statuses: &[ComposeContainerStatus],
    ) -> ApplicationInstance {
        let mut ports = Vec::<ApplicationPort>::new();
        let mut services_by_name = BTreeMap::<String, ApplicationInstanceService>::new();

        for st in statuses {
            let service_name = if st.service.trim().is_empty() {
                st.name.clone()
            } else {
                st.service.clone()
            };

            let service_state = Self::normalize_instance_state(&st.state);
            let service_entry = services_by_name
                .entry(service_name.clone())
                .or_insert_with(|| ApplicationInstanceService {
                    name: service_name.clone(),
                    container_name: st.name.clone(),
                    state: service_state.clone(),
                    health: st.health.clone(),
                    ports: Vec::new(),
                });

            service_entry.container_name = st.name.clone();
            service_entry.state = service_state;
            service_entry.health = st.health.clone();

            for p in &st.publishers {
                let mapped = ApplicationPort {
                    ip: "0.0.0.0".to_string(),
                    private_port: p.target_port.unwrap_or_default(),
                    public_port: p.published_port,
                    protocol: p.protocol.clone().unwrap_or_else(|| "tcp".to_string()),
                };
                ports.push(mapped.clone());
                service_entry.ports.push(mapped);
            }
        }

        ports.sort_by(|a, b| {
            a.public_port
                .unwrap_or_default()
                .cmp(&b.public_port.unwrap_or_default())
        });

        let mut services = services_by_name.into_values().collect::<Vec<_>>();
        services.sort_by(|a, b| a.name.cmp(&b.name));
        for svc in &mut services {
            svc.ports.sort_by(|a, b| {
                a.public_port
                    .unwrap_or_default()
                    .cmp(&b.public_port.unwrap_or_default())
            });
        }

        let aggregated_state = Self::aggregate_instance_state(&services);

        ApplicationInstance {
            id: meta.id.clone(),
            name: meta.name.clone(),
            template_id: meta.template_id.clone(),
            category: meta.category.clone(),
            state: aggregated_state,
            ports,
            services,
        }
    }

    fn normalize_instance_state(state: &str) -> String {
        match state.trim().to_ascii_lowercase().as_str() {
            "running" => "running".to_string(),
            "restarting" => "restarting".to_string(),
            "created" => "created".to_string(),
            "paused" => "paused".to_string(),
            "dead" => "dead".to_string(),
            _ => "exited".to_string(),
        }
    }

    fn aggregate_instance_state(services: &[ApplicationInstanceService]) -> String {
        if services.is_empty() {
            return "exited".to_string();
        }

        let has_running = services.iter().any(|s| s.state == "running");
        let has_restarting = services.iter().any(|s| s.state == "restarting");
        let has_created = services.iter().any(|s| s.state == "created");
        let has_paused = services.iter().any(|s| s.state == "paused");
        let has_dead = services.iter().any(|s| s.state == "dead");

        if has_running {
            return "running".to_string();
        }
        if has_restarting {
            return "restarting".to_string();
        }
        if has_created {
            return "created".to_string();
        }
        if has_paused {
            return "paused".to_string();
        }
        if has_dead {
            return "dead".to_string();
        }

        "exited".to_string()
    }

    fn extract_template_services(
        &self,
        compose_path: &Path,
    ) -> AppResult<Vec<ApplicationTemplateService>> {
        let raw = fs::read_to_string(compose_path).map_err(|e| {
            AppError::System(format!(
                "Failed to read compose template '{}': {}",
                compose_path.display(),
                e
            ))
        })?;

        let value: serde_yaml::Value = serde_yaml::from_str(&raw).map_err(|e| {
            AppError::Validation(format!(
                "Invalid compose template '{}': {}",
                compose_path.display(),
                e
            ))
        })?;

        let services_value = value
            .as_mapping()
            .and_then(|m| m.get(&serde_yaml::Value::String("services".to_string())))
            .and_then(|v| v.as_mapping());

        let mut services = Vec::<ApplicationTemplateService>::new();
        if let Some(mapping) = services_value {
            for (k, v) in mapping {
                let Some(name) = k.as_str() else {
                    continue;
                };

                let image = v
                    .as_mapping()
                    .and_then(|m| m.get(&serde_yaml::Value::String("image".to_string())))
                    .and_then(|img| img.as_str())
                    .map(|s| s.to_string());

                services.push(ApplicationTemplateService {
                    name: name.to_string(),
                    image,
                });
            }
        }

        services.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(services)
    }

    fn extract_images_from_compose_file(&self, compose_path: &Path) -> AppResult<Vec<String>> {
        let raw = fs::read_to_string(compose_path).map_err(|e| {
            AppError::System(format!(
                "Failed to read compose file '{}': {}",
                compose_path.display(),
                e
            ))
        })?;

        let value: serde_yaml::Value = serde_yaml::from_str(&raw).map_err(|e| {
            AppError::Validation(format!(
                "Invalid compose file '{}': {}",
                compose_path.display(),
                e
            ))
        })?;

        let mut images = BTreeSet::<String>::new();
        let services = value
            .as_mapping()
            .and_then(|m| m.get(&serde_yaml::Value::String("services".to_string())))
            .and_then(|v| v.as_mapping());

        if let Some(mapping) = services {
            for (_service_name, service_value) in mapping {
                let image = service_value
                    .as_mapping()
                    .and_then(|m| m.get(&serde_yaml::Value::String("image".to_string())))
                    .and_then(|img| img.as_str())
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());

                if let Some(image) = image {
                    images.insert(image);
                }
            }
        }

        Ok(images.into_iter().collect())
    }

    fn render_tera_template(template_str: &str, ctx: &Context) -> AppResult<String> {
        Tera::one_off(template_str, ctx, false)
            .map_err(|e| AppError::System(format!("Template render failed: {}", e)))
    }

    fn normalize_protocol(input: &str) -> String {
        if input.trim().eq_ignore_ascii_case("udp") {
            "udp".to_string()
        } else {
            "tcp".to_string()
        }
    }

    fn normalize_endpoint_binding_key(input: &str) -> Option<String> {
        let (port_raw, proto_raw) = input.split_once('/')?;
        let port = port_raw.trim().parse::<u16>().ok()?;
        if port == 0 {
            return None;
        }

        let proto = Self::normalize_protocol(proto_raw);
        Some(format!("{}/{}", port, proto))
    }

    fn typed_value_to_string(value: &ApplicationInstallValue) -> String {
        match value {
            ApplicationInstallValue::String(v) => v.clone(),
            ApplicationInstallValue::Integer(v) => v.to_string(),
            ApplicationInstallValue::Float(v) => {
                let mut s = v.to_string();
                if s.contains('.') {
                    while s.ends_with('0') {
                        s.pop();
                    }
                    if s.ends_with('.') {
                        s.push('0');
                    }
                }
                s
            }
            ApplicationInstallValue::Boolean(v) => {
                if *v {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
        }
    }

    fn normalize_boolean_string(raw: &str) -> Option<String> {
        let normalized = raw.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "true" | "1" | "yes" | "on" => Some("true".to_string()),
            "false" | "0" | "no" | "off" => Some("false".to_string()),
            _ => None,
        }
    }

    fn normalize_compose_host_path(input: &str) -> String {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return trimmed.to_string();
        }

        if Path::new(trimmed).is_absolute()
            || trimmed.starts_with("./")
            || trimmed.starts_with("../")
            || trimmed.starts_with("~/")
        {
            return trimmed.to_string();
        }

        format!("./{}", trimmed.trim_start_matches('/'))
    }

    fn is_valid_slug(input: &str) -> bool {
        let s = input.trim();
        !s.is_empty()
            && !s.starts_with('-')
            && !s.ends_with('-')
            && !s.contains("--")
            && s.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    }

    fn parse_instance_id(raw: &str) -> AppResult<Uuid> {
        Uuid::parse_str(raw).map_err(|_| {
            AppError::Validation(format!(
                "Invalid application instance id '{}': expected UUID",
                raw
            ))
        })
    }
}

fn collect_template_images(manager: &ApplicationManager, template_id: &str) -> Vec<String> {
    let mut images = BTreeSet::<String>::new();

    if let Some(template) = manager
        .list_templates()
        .into_iter()
        .find(|tpl| tpl.id == template_id)
    {
        for svc in template.services {
            let Some(raw) = svc.image else {
                continue;
            };
            let normalized = raw.trim();
            if !normalized.is_empty() {
                images.insert(normalized.to_string());
            }
        }
    }

    images.into_iter().collect()
}

fn format_pull_progress_line(image: &str, item: PullProgress) -> String {
    let mut parts = Vec::<String>::new();

    let status = item.status.trim();
    if !status.is_empty() {
        parts.push(status.to_string());
    }

    if let Some(id) = item
        .id
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        parts.push(format!("layer={}", id));
    }

    if let Some(progress) = item
        .progress
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        parts.push(progress);
    }

    if parts.is_empty() {
        format!("[pull:{}] update", image)
    } else {
        format!("[pull:{}] {}", image, parts.join(" | "))
    }
}

async fn stream_image_pull_progress(
    docker: DockerService,
    task_manager: ApplicationTaskManager,
    task_id: String,
    image: String,
) -> AppResult<()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<PullProgress>(128);
    let docker_for_pull = docker.clone();
    let image_for_pull = image.clone();

    let pull_handle =
        tokio::spawn(async move { docker_for_pull.pull_image_stream(&image_for_pull, tx).await });

    let mut last_line = String::new();
    while let Some(item) = rx.recv().await {
        let line = format_pull_progress_line(&image, item);
        if line == last_line {
            continue;
        }
        last_line = line.clone();

        let _ = task_manager
            .append_log(&task_id, ApplicationTaskLogLevel::Info, line)
            .await;
    }

    match pull_handle.await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(err)) => Err(err),
        Err(err) => Err(AppError::System(format!(
            "Image pull worker crashed for '{}': {}",
            image, err
        ))),
    }
}
