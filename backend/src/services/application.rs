use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tera::{Context, Tera};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::services::compose::{ComposeContainerStatus, ComposeProject, ComposeService};
use crate::services::docker::{DockerActionResponse, DockerService};

const APP_COMPOSE_BASE_DIR: &str = ".mana-panel/applications";

const META_FILE_NAME: &str = "meta.json";
const DEFAULT_APP_TOML: &str = "app.toml";
const DEFAULT_COMPOSE_FILE: &str = "docker-compose.yml";

static APP_OPERATION_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn app_operation_lock() -> &'static Mutex<()> {
    APP_OPERATION_LOCK.get_or_init(|| Mutex::new(()))
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationTemplatePort {
    pub key: String,
    pub container_port: u16,
    pub protocol: String, // tcp | udp
    pub default_host_port: Option<u16>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationTemplateEnv {
    pub key: String,
    pub value: Option<String>,
    pub from: Option<String>,
    pub required: bool,
}

/// Declares a configuration file that must be rendered at install time and
/// mounted into the container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationTemplateConfigFile {
    /// Relative path to the Tera template inside the app package.
    pub template: String,
    /// Destination inside the instance directory (rendered output).
    pub target: String,
}

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
    #[serde(skip_serializing)]
    pub image: Option<String>,

    // Dynamic form + runtime mappings
    pub params: Vec<ApplicationTemplateParam>,
    pub ports: Vec<ApplicationTemplatePort>,
    pub env: Vec<ApplicationTemplateEnv>,

    /// Explicit configuration file declarations from `[[config_file]]`.
    pub config_files: Vec<ApplicationTemplateConfigFile>,

    // Helpful for UI
    pub has_conf_templates: bool,
    pub app_dir: String,
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
    pub id: Uuid,
    pub name: String,
    pub template_id: String,
    pub category: String,
    pub state: String,
    pub ports: Vec<ApplicationPort>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InstallApplicationRequest {
    pub template_id: String,
    pub name: Option<String>,
    pub values: Option<HashMap<String, String>>,
    pub port_bindings: Option<HashMap<String, u16>>,
    pub env: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApplicationInstanceMeta {
    pub id: Uuid,
    pub name: String,
    pub template_id: String,
    pub category: String,
    pub project_name: String,
}

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
    image: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct AppTomlPort {
    key: String,
    container_port: Option<u16>,
    protocol: Option<String>,
    default_host_port: Option<u16>,
    required: Option<bool>,
}

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

#[derive(Debug, Clone, Deserialize)]
struct AppTomlEnv {
    key: String,
    value: Option<String>,
    from: Option<String>,
    required: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
struct AppTomlConfigFile {
    template: String,
    target: String,
}

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
}

#[derive(Clone)]
pub struct ApplicationManager {
    docker: DockerService,
    compose: ComposeService,
    app_root_dir: PathBuf,
    compose_projects_dir: PathBuf,
}

impl ApplicationManager {
    pub fn new(docker: DockerService, app_root_dir: PathBuf) -> Self {
        let compose_projects_dir = Self::resolve_compose_projects_dir(&app_root_dir);

        Self {
            docker,
            compose: ComposeService::default(),
            app_root_dir,
            compose_projects_dir,
        }
    }

    fn resolve_compose_projects_dir(app_root_dir: &Path) -> PathBuf {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut candidates = Vec::<PathBuf>::new();

        // Typical layouts:
        // - repo root run:      ./.mana-panel/applications
        // - repo root + backend: ./backend/.mana-panel/applications
        // - backend run:        ./.mana-panel/applications
        candidates.push(cwd.join(APP_COMPOSE_BASE_DIR));
        candidates.push(cwd.join("backend").join(APP_COMPOSE_BASE_DIR));

        if let Some(parent) = cwd.parent() {
            candidates.push(parent.join(APP_COMPOSE_BASE_DIR));
            candidates.push(parent.join("backend").join(APP_COMPOSE_BASE_DIR));
        }

        // Derive from app_root_dir (usually backend/apps) for stable resolution.
        if let Some(app_parent) = app_root_dir.parent() {
            candidates.push(app_parent.join(APP_COMPOSE_BASE_DIR));
            if let Some(project_root) = app_parent.parent() {
                candidates.push(project_root.join(APP_COMPOSE_BASE_DIR));
            }
        }

        if let Some(existing) = candidates.iter().find(|p| p.is_dir()) {
            return existing.clone();
        }

        app_root_dir
            .parent()
            .map(|p| p.join(APP_COMPOSE_BASE_DIR))
            .unwrap_or_else(|| cwd.join(APP_COMPOSE_BASE_DIR))
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

            let project = self.compose_project_for(&instance_id, &meta.project_name);
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

        Ok(images.into_iter().any(|img| {
            img.repo_tags
                .iter()
                .any(|tag| tag == target || tag == &normalized)
        }))
    }

    pub async fn install_application(
        &self,
        req: InstallApplicationRequest,
    ) -> AppResult<DockerActionResponse> {
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
        let image = template
            .image
            .clone()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_default();

        if image.trim().is_empty() {
            return Err(AppError::Validation(format!(
                "Template '{}' does not define an image",
                template.id
            )));
        }

        let resolved_values = self.resolve_install_values(&template, &req)?;
        let resolved_ports = self.resolve_port_bindings(&template, &req)?;

        // Prepare instance directory
        let instance_dir = self.instance_dir(&instance_id);
        fs::create_dir_all(&instance_dir)
            .map_err(|e| AppError::System(format!("Failed to create instance directory: {}", e)))?;

        // Generate .env file
        self.generate_env_file(
            &template,
            &resolved_values,
            &resolved_ports,
            &instance_dir,
            req.env.as_deref(),
        )?;

        // Build Tera rendering context
        let tera_ctx = self.build_tera_context(
            &template,
            &resolved_values,
            &resolved_ports,
            &instance_name,
            &instance_id,
            &image,
            req.env.as_deref(),
        );

        let rendered_conf = self.render_conf_templates(&template, &instance_id, &tera_ctx)?;
        let compose_yaml = self.render_compose_template(&template, &tera_ctx, &rendered_conf)?;

        let project_name = format!("mana-{}", instance_id);

        let meta = ApplicationInstanceMeta {
            id: instance_id,
            name: instance_name,
            template_id: template.id.clone(),
            category: template.category.clone(),
            project_name: project_name.clone(),
        };

        self.write_instance_files(&meta.id, &compose_yaml, &meta)?;

        let project = self.compose_project_for(&meta.id, &project_name);
        self.compose.up(&project).await
    }

    pub async fn start_application(&self, instance_id: &str) -> AppResult<DockerActionResponse> {
        let _guard = app_operation_lock().lock().await;
        self.compose.ensure_available().await?;

        let instance_id = Self::parse_instance_id(instance_id)?;
        let meta = self.load_instance_meta(&instance_id)?;
        let project = self.compose_project_for(&instance_id, &meta.project_name);
        self.compose.start(&project).await
    }

    pub async fn stop_application(&self, instance_id: &str) -> AppResult<DockerActionResponse> {
        self.compose.ensure_available().await?;
        let instance_id = Self::parse_instance_id(instance_id)?;
        let meta = self.load_instance_meta(&instance_id)?;
        let project = self.compose_project_for(&instance_id, &meta.project_name);
        self.compose.stop(&project).await
    }

    pub async fn remove_application(
        &self,
        instance_id: &str,
        force: bool,
    ) -> AppResult<DockerActionResponse> {
        self.compose.ensure_available().await?;
        let instance_id = Self::parse_instance_id(instance_id)?;
        let meta = self.load_instance_meta(&instance_id)?;
        let project = self.compose_project_for(&instance_id, &meta.project_name);

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

        if let Some(vals) = &req.values {
            for (k, v) in vals {
                out.insert(k.clone(), v.clone());
            }
        }

        for p in &template.params {
            let existing = out.get(&p.key).map(|s| s.trim().to_string());
            let final_value = existing
                .filter(|s| !s.is_empty())
                .or_else(|| p.default_value.clone().filter(|s| !s.trim().is_empty()));

            match final_value {
                Some(v) => {
                    out.insert(p.key.clone(), v);
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

        for p in &template.ports {
            let proto = Self::normalize_protocol(&p.protocol);
            let endpoint = format!("{}/{}", p.container_port, proto);

            let by_key = configured.get(&p.key).copied();
            let selected = by_key.or(p.default_host_port);

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

        Ok(out)
    }

    fn generate_env_file(
        &self,
        template: &ApplicationTemplate,
        resolved_values: &HashMap<String, String>,
        resolved_ports: &HashMap<String, u16>,
        instance_dir: &Path,
        env_overrides: Option<&[String]>,
    ) -> AppResult<()> {
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
        for (k, v) in env_map {
            content.push_str(&format!("{}={}\n", k, v));
        }

        fs::write(&env_path, content)
            .map_err(|e| AppError::System(format!("Failed to write .env file: {}", e)))?;

        Ok(())
    }

    fn build_tera_context(
        &self,
        template: &ApplicationTemplate,
        resolved_values: &HashMap<String, String>,
        resolved_ports: &HashMap<String, u16>,
        instance_name: &str,
        instance_id: &Uuid,
        image: &str,
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
        ctx.insert("app_image", image);
        ctx.insert("image", image);

        // Port context variables: port_80_tcp = 8080, port_http = 8080
        for (endpoint, host_port) in resolved_ports {
            let key = format!("port_{}", Self::sanitize_for_fs(endpoint)).to_lowercase();
            ctx.insert(&key, host_port);
        }
        for p in &template.ports {
            let proto = Self::normalize_protocol(&p.protocol);
            let endpoint = format!("{}/{}", p.container_port, proto);
            if let Some(host) = resolved_ports.get(&endpoint) {
                let key = format!("port_{}", Self::sanitize_for_fs(&p.key)).to_lowercase();
                ctx.insert(&key, host);
            }
        }

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

        if !template.config_files.is_empty() {
            // ---- Explicit [[config_file]] declarations ----
            let mut rendered = Vec::new();
            for cf in &template.config_files {
                let src = app_dir.join(&cf.template);
                if !src.exists() {
                    return Err(AppError::Validation(format!(
                        "Config template '{}' does not exist for app '{}'",
                        src.display(),
                        template.id
                    )));
                }

                let dst = instance_dir.join(&cf.target);
                if let Some(parent) = dst.parent() {
                    fs::create_dir_all(parent).map_err(|e| {
                        AppError::System(format!(
                            "Failed to create directory '{}': {}",
                            parent.display(),
                            e
                        ))
                    })?;
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
            Ok(rendered)
        } else {
            Ok(vec![])
        }
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

        let config_files: Vec<ApplicationTemplateConfigFile> = doc
            .config_files
            .into_iter()
            .map(|cf| ApplicationTemplateConfigFile {
                template: cf.template,
                target: cf.target,
            })
            .collect();

        let has_conf_templates = !config_files.is_empty();

        let version = if doc.metadata.version.trim().is_empty() {
            "0.1.0".to_string()
        } else {
            doc.metadata.version.clone()
        };

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
            image: doc.metadata.image,
            params,
            ports,
            env,
            config_files,
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

    fn write_instance_files(
        &self,
        instance_id: &Uuid,
        compose_yaml: &str,
        meta: &ApplicationInstanceMeta,
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

        Ok(())
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

    fn load_instance_meta(&self, instance_id: &Uuid) -> AppResult<ApplicationInstanceMeta> {
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

    fn compose_project_for(&self, instance_id: &Uuid, project_name: &str) -> ComposeProject {
        ComposeProject {
            compose_file: self.instance_compose_file(instance_id),
            project_name: project_name.to_string(),
        }
    }

    fn instance_from_compose_status(
        &self,
        meta: &ApplicationInstanceMeta,
        statuses: &[ComposeContainerStatus],
    ) -> ApplicationInstance {
        let mut ports = Vec::<ApplicationPort>::new();
        for st in statuses {
            for p in &st.publishers {
                ports.push(ApplicationPort {
                    ip: "0.0.0.0".to_string(),
                    private_port: p.target_port.unwrap_or_default(),
                    public_port: p.published_port,
                    protocol: p.protocol.clone().unwrap_or_else(|| "tcp".to_string()),
                });
            }
        }

        ports.sort_by(|a, b| {
            a.public_port
                .unwrap_or_default()
                .cmp(&b.public_port.unwrap_or_default())
        });

        let primary_state = statuses
            .first()
            .map(|s| s.state.as_str())
            .unwrap_or("exited");

        ApplicationInstance {
            id: meta.id.clone(),
            name: meta.name.clone(),
            template_id: meta.template_id.clone(),
            category: meta.category.clone(),
            state: Self::normalize_instance_state(primary_state),
            ports,
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

    fn render_tera_template(template_str: &str, ctx: &Context) -> AppResult<String> {
        Tera::one_off(template_str, ctx, false)
            .map_err(|e| AppError::System(format!("Template render failed: {}", e)))
    }

    fn sanitize_for_fs(raw: &str) -> String {
        let mut out = String::with_capacity(raw.len());
        for ch in raw.chars() {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                out.push(ch);
            } else {
                out.push('_');
            }
        }
        let trimmed = out.trim_matches('_');
        if trimmed.is_empty() {
            "app".to_string()
        } else {
            trimmed.to_string()
        }
    }

    fn normalize_protocol(input: &str) -> String {
        if input.trim().eq_ignore_ascii_case("udp") {
            "udp".to_string()
        } else {
            "tcp".to_string()
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
