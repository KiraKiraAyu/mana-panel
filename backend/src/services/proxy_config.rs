use std::path::{Path, PathBuf};

use tera::{Context, Tera};

use crate::db::entities::website::{self, ProxyTargetType, ServerType, SiteType};
use crate::error::{AppError, AppResult};
use crate::services::application::ApplicationManager;
use crate::services::docker::DockerService;
use crate::services::fs_utils;

/// Manages proxy configuration file generation and container reload.
pub struct ProxyConfigService;

impl ProxyConfigService {
    /// Generate and deploy proxy configuration for a website,
    /// then reload the target proxy container.
    pub async fn deploy(
        website: &website::Model,
        app_manager: &ApplicationManager,
        docker: &DockerService,
    ) -> AppResult<()> {
        let config_content = Self::render_config(website, app_manager)?;

        let site_types_vec: Vec<SiteType> =
            serde_json::from_value(website.site_types.clone()).unwrap_or_default();

        // If static site, ensure root_dir exists and is not empty
        if site_types_vec.contains(&SiteType::Static) {
            if let Some(ref root_dir) = website.root_dir {
                let path = Path::new(root_dir);
                if !path.exists() {
                    std::fs::create_dir_all(path).map_err(|e| {
                        AppError::System(format!(
                            "Failed to create static root directory '{}': {}",
                            root_dir, e
                        ))
                    })?;
                    tracing::info!("Created static root directory: {}", root_dir);
                }

                // Check if directory is empty (or only contains hidden files)
                let is_empty = std::fs::read_dir(path)
                    .map(|mut entries| entries.next().is_none())
                    .unwrap_or(true);

                if is_empty {
                    let index_path = path.join("index.html");
                    let welcome_content = Self::render_default_index(
                        &website.name,
                        &index_path.display().to_string(),
                    )?;
                    std::fs::write(&index_path, welcome_content).map_err(|e| {
                        AppError::System(format!("Failed to create default index.html: {}", e))
                    })?;
                    tracing::info!("Created default index.html at {}", index_path.display());
                }
            }
        }

        let (config_path, enabled_path, container_name) =
            Self::resolve_target(website, app_manager).await?;

        // Connect proxy network to target app's network to support internal container name routing
        if site_types_vec.contains(&SiteType::ReverseProxy) {
            if let Some(ProxyTargetType::Application) = website.proxy_target_type {
                if let Some(ref app_id_str) = website.proxy_target_app_id {
                    if let Ok(app_id) = uuid::Uuid::parse_str(app_id_str) {
                        let network_name = format!("mana-{}_default", app_id);
                        if let Err(e) = docker.connect_network(&container_name, &network_name).await
                        {
                            let err_msg = format!(
                                "Failed to connect proxy container '{}' to target network '{}': {}",
                                container_name, network_name, e
                            );
                            tracing::error!("{}", err_msg);
                            return Err(AppError::System(err_msg));
                        }
                    }
                }
            }
        }

        // Backup existing config
        let mut backup_content = None;
        let mut backup_is_legacy_enabled_file = false;
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                backup_content = Some(content);
            }
        } else if let Some(ref enabled_path) = enabled_path {
            if let Ok(metadata) = std::fs::symlink_metadata(enabled_path) {
                if metadata.file_type().is_file() {
                    if let Ok(content) = std::fs::read_to_string(enabled_path) {
                        backup_content = Some(content);
                        backup_is_legacy_enabled_file = true;
                    }
                }
            }
        }

        // Write the new config file
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::System(format!(
                    "Failed to create config directory '{}': {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        std::fs::write(&config_path, &config_content).map_err(|e| {
            AppError::System(format!(
                "Failed to write proxy config '{}': {}",
                config_path.display(),
                e
            ))
        })?;

        tracing::info!(
            "Wrote proxy config for website '{}' to {}",
            website.name,
            config_path.display()
        );

        if let Some(ref enabled_path) = enabled_path {
            fs_utils::ensure_relative_symlink(enabled_path, &config_path)?;
            tracing::info!(
                "Ensured site enable link for website '{}' at {}",
                website.name,
                enabled_path.display()
            );
        }

        // Reload the proxy container, rollback if it fails
        if let Err(e) = Self::reload_container(website, docker, &container_name).await {
            tracing::error!(
                "Failed to reload proxy for '{}', rolling back configuration: {}",
                website.name,
                e
            );
            if let Some(old_content) = backup_content {
                if backup_is_legacy_enabled_file {
                    if let Some(ref enabled_path) = enabled_path {
                        let _ = std::fs::remove_file(enabled_path);
                        let _ = std::fs::write(enabled_path, old_content);
                    }
                    let _ = std::fs::remove_file(&config_path);
                } else {
                    let _ = std::fs::write(&config_path, old_content);
                    if let Some(ref enabled_path) = enabled_path {
                        let _ = fs_utils::ensure_relative_symlink(enabled_path, &config_path);
                    }
                }
                // Attempt to reload the good config back
                let _ = Self::reload_container(website, docker, &container_name).await;
            } else {
                // There was no previous config, delete the bad one
                let _ = std::fs::remove_file(&config_path);
                if let Some(ref enabled_path) = enabled_path {
                    let _ = std::fs::remove_file(enabled_path);
                }
            }
            return Err(e);
        }

        Ok(())
    }

    /// Remove the proxy configuration file for a website and reload the container.
    pub async fn undeploy(
        website: &website::Model,
        app_manager: &ApplicationManager,
        docker: &DockerService,
    ) -> AppResult<()> {
        let (config_path, enabled_path, container_name) =
            Self::resolve_target(website, app_manager).await?;

        let mut removed = false;

        if let Some(enabled_path) = enabled_path {
            if fs_utils::path_exists_or_symlink(&enabled_path) {
                std::fs::remove_file(&enabled_path).map_err(|e| {
                    AppError::System(format!(
                        "Failed to remove proxy config link '{}': {}",
                        enabled_path.display(),
                        e
                    ))
                })?;
                removed = true;
            }
        }

        if config_path.exists() {
            std::fs::remove_file(&config_path).map_err(|e| {
                AppError::System(format!(
                    "Failed to remove proxy config '{}': {}",
                    config_path.display(),
                    e
                ))
            })?;
            removed = true;

            tracing::info!(
                "Removed proxy config for website '{}' at {}",
                website.name,
                config_path.display()
            );
        }

        if removed {
            Self::reload_container(website, docker, &container_name).await?;
        }

        Ok(())
    }

    // ── Internals ──

    /// Returns a lazily initialized Tera engine
    fn get_tera() -> AppResult<&'static Tera> {
        static TERA: std::sync::OnceLock<Tera> = std::sync::OnceLock::new();
        match TERA.get() {
            Some(t) => Ok(t),
            None => {
                let template_dir = Self::template_dir();
                let engine = Tera::new(&format!("{}/*", template_dir.display()))
                    .map_err(|e| AppError::System(format!("Failed to initialize Tera templates: {}", e)))?;
                let _ = TERA.set(engine);
                Ok(TERA.get().unwrap())
            }
        }
    }

    /// Render the default index.html for static sites.
    fn render_default_index(website_name: &str, index_path: &str) -> AppResult<String> {
        let tera = Self::get_tera()?;
        let mut ctx = Context::new();
        ctx.insert("website_name", website_name);
        ctx.insert("index_path", index_path);
        
        tera.render("default_index.tera", &ctx)
            .map_err(|e| AppError::System(format!("Failed to render default index template: {}", e)))
    }

    /// Render the configuration string for this website using the Tera template.
    fn render_config(
        website: &website::Model,
        app_manager: &ApplicationManager,
    ) -> AppResult<String> {
        let template_name = match website.server_type {
            ServerType::Caddy => "caddy_site.tera",
            ServerType::Nginx | ServerType::OpenResty => "nginx_site.tera",
        };

        let tera = Self::get_tera()?;

        let mut ctx = Context::new();
        ctx.insert("website_id", &website.id);
        ctx.insert("primary_domain", &website.primary_domain);

        let aliases: Vec<String> =
            serde_json::from_value(website.aliases.clone()).unwrap_or_default();
        ctx.insert("aliases", &aliases);

        let site_types_vec: Vec<SiteType> =
            serde_json::from_value(website.site_types.clone()).unwrap_or_default();
        
        let has_static = site_types_vec.contains(&SiteType::Static);
        let has_reverse_proxy = site_types_vec.contains(&SiteType::ReverseProxy);

        ctx.insert("has_static", &has_static);
        ctx.insert("has_reverse_proxy", &has_reverse_proxy);

        if has_reverse_proxy {
            let target = Self::build_proxy_target(website, app_manager);
            ctx.insert("proxy_target", &target);
        }
        
        if has_static {
            ctx.insert(
                "root_dir",
                website
                    .root_dir
                    .as_deref()
                    .unwrap_or("/usr/share/nginx/html"),
            );
        }

        tera.render(template_name, &ctx)
            .map_err(|e| AppError::System(format!("Failed to render proxy template: {}", e)))
    }

    /// Build the proxy target URL from the website model.
    fn build_proxy_target(website: &website::Model, app_manager: &ApplicationManager) -> String {
        match &website.proxy_target_type {
            Some(ProxyTargetType::Url) => website.proxy_target_url.clone().unwrap_or_default(),
            Some(ProxyTargetType::Application) => {
                let app_id_str = website
                    .proxy_target_app_id
                    .as_deref()
                    .unwrap_or("localhost");
                let port = website.proxy_target_app_port.unwrap_or(80);

                let target_host = if let Ok(uuid) = uuid::Uuid::parse_str(app_id_str) {
                    if let Ok(meta) = app_manager.load_instance_meta(&uuid) {
                        meta.name // Resolves to container_name in our application compose logic
                    } else {
                        app_id_str.to_string() // Fallback
                    }
                } else {
                    app_id_str.to_string()
                };

                format!("http://{}:{}", target_host, port)
            }
            None => "http://localhost".to_string(),
        }
    }

    /// Resolve the file path where the config should be written and the container name
    /// of the proxy application instance.
    async fn resolve_target(
        website: &website::Model,
        app_manager: &ApplicationManager,
    ) -> AppResult<(PathBuf, Option<PathBuf>, String)> {
        // Derive expected proxy template from the website server type.
        let target_template_id = match website.server_type {
            ServerType::Caddy => "caddy-autossl",
            ServerType::Nginx => "nginx-standard",
            ServerType::OpenResty => "openresty-waf",
        };

        // Preferred path: website is explicitly bound to a server instance.
        let (instance_id, instance_name) = if let Some(instance_id_str) =
            website.server_instance_id.as_deref()
        {
            let trimmed = instance_id_str.trim();
            if trimmed.is_empty() {
                return Err(AppError::Validation(format!(
                    "Website '{}' has an empty server instance binding",
                    website.name
                )));
            }

            let instance_id = uuid::Uuid::parse_str(trimmed).map_err(|_| {
                AppError::Validation(format!(
                    "Website '{}' has invalid server_instance_id '{}'",
                    website.name, trimmed
                ))
            })?;

            let meta = app_manager.load_instance_meta(&instance_id).map_err(|e| {
                AppError::Validation(format!(
                    "Server instance '{}' for website '{}' was not found: {}",
                    trimmed, website.name, e
                ))
            })?;

            if meta.template_id != target_template_id {
                return Err(AppError::Validation(format!(
                    "Website '{}' is bound to instance '{}' (template '{}'), but server type requires template '{}'",
                    website.name, meta.name, meta.template_id, target_template_id
                )));
            }

            (instance_id, meta.name)
        } else {
            // Backward compatibility for legacy rows: pick by server type.
            let instances = app_manager.list_instance_ids_by_template(target_template_id)?;

            if instances.is_empty() {
                return Err(AppError::Validation(format!(
                    "No installed '{}' application instance found. Please install a {} proxy application first.",
                    target_template_id,
                    match website.server_type {
                        ServerType::Caddy => "Caddy",
                        ServerType::Nginx => "Nginx",
                        ServerType::OpenResty => "OpenResty",
                    }
                )));
            }

            if instances.len() > 1 {
                tracing::warn!(
                    "Website '{}' is not bound to a proxy instance. Multiple '{}' instances exist; using first '{}'.",
                    website.name,
                    target_template_id,
                    instances[0].1
                );
            }

            let (instance_id, instance_name) = &instances[0];
            (instance_id.to_owned(), instance_name.clone())
        };

        let instance_dir = app_manager.get_instance_dir(&instance_id);

        // Legacy nginx/openresty instances may miss the static root bind mount.
        // Refresh application files from the latest template so static sites can be served.
        Self::check_and_patch_legacy_instances(
            website,
            app_manager,
            &instance_id,
            &instance_dir
        ).await?;

        let site_filename = format!("{}-{}.conf", website.id, website.name);

        let (config_path, enabled_path) = match website.server_type {
            ServerType::Caddy => {
                (
                    instance_dir.join("conf").join("sites").join(&site_filename),
                    None,
                )
            }
            ServerType::Nginx | ServerType::OpenResty => {
                let available = instance_dir
                    .join("conf")
                    .join("sites-available")
                    .join(&site_filename);
                let enabled = instance_dir
                    .join("conf")
                    .join("sites-enabled")
                    .join(&site_filename);
                (available, Some(enabled))
            }
        };

        Ok((config_path, enabled_path, instance_name))
    }

    /// Check and apply fixes to legacy proxy containers, triggering update_application if necessary.
    async fn check_and_patch_legacy_instances(
        website: &website::Model,
        app_manager: &ApplicationManager,
        instance_id: &uuid::Uuid,
        instance_dir: &PathBuf,
    ) -> AppResult<()> {
        let site_types_vec: Vec<SiteType> =
            serde_json::from_value(website.site_types.clone()).unwrap_or_default();

        match website.server_type {
            ServerType::Nginx | ServerType::OpenResty => {
                if site_types_vec.contains(&SiteType::Static) {
                    let compose_file = instance_dir.join("docker-compose.yml");
                    if compose_file.exists() {
                        if let Ok(content) = std::fs::read_to_string(&compose_file) {
                            let static_mount = "/opt/mana-panel/www:/opt/mana-panel/www:ro";
                            if !content.contains(static_mount) {
                                tracing::info!(
                                    "Detected legacy proxy instance '{}' without static mount, triggering automatic application update",
                                    instance_id
                                );
                                app_manager
                                    .update_application(&instance_id.to_string(), None)
                                    .await
                                    .map_err(|e| {
                                        AppError::System(format!(
                                            "Proxy instance '{}' is missing static mount '{}', and automatic update failed: {}",
                                            instance_id, static_mount, e
                                        ))
                                    })?;
                            }
                        }
                    }
                }
            }
            ServerType::Caddy => {
                let main_caddyfile = instance_dir.join("conf").join("Caddyfile");
                if main_caddyfile.exists() {
                    if let Ok(content) = std::fs::read_to_string(&main_caddyfile) {
                        if !content.contains("import /etc/caddy/sites/*.conf") {
                            let new_content = format!("{}\n\nimport /etc/caddy/sites/*.conf\n", content);
                            if let Err(e) = std::fs::write(&main_caddyfile, new_content) {
                                tracing::error!(
                                    "Failed to append import directive to legacy Caddyfile at {}: {}",
                                    main_caddyfile.display(), e
                                );
                            } else {
                                tracing::info!(
                                    "Triggering automatic application update for patched legacy Caddy instance '{}'",
                                    instance_id
                                );
                                app_manager.update_application(&instance_id.to_string(), None).await
                                    .map_err(|e| {
                                        tracing::error!("Failed to update legacy Caddy instance '{}': {}", instance_id, e);
                                        AppError::System(format!(
                                            "Legacy Caddy instance '{}' was patched but failed to restart with the new volume mount. \
                                             The conf/sites directory may not be available inside the container. Error: {}",
                                            instance_id, e
                                        ))
                                    })?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Send a reload signal to the target proxy container.
    async fn reload_container(
        website: &website::Model,
        docker: &DockerService,
        container_name: &str,
    ) -> AppResult<()> {
        let result = match website.server_type {
            ServerType::Caddy => {
                docker
                    .exec_container(
                        container_name,
                        vec![
                            "caddy",
                            "reload",
                            "--config",
                            "/etc/caddy/Caddyfile",
                            "--adapter",
                            "caddyfile",
                        ],
                    )
                    .await
            }
            ServerType::Nginx | ServerType::OpenResty => {
                docker
                    .exec_container(container_name, vec!["nginx", "-s", "reload"])
                    .await
            }
        };

        match result {
            Ok(output) => {
                tracing::info!(
                    "Reloaded proxy container '{}': {}",
                    container_name,
                    output.trim()
                );
                Ok(())
            }
            Err(e) => {
                let err_msg = format!(
                    "Failed to reload proxy container '{}': {}. Config was written but not applied.",
                    container_name, e
                );
                tracing::warn!("{}", err_msg);
                Err(AppError::System(err_msg))
            }
        }
    }

    /// Build the path to the proxy templates directory.
    fn template_dir() -> PathBuf {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));

        // Try several candidate directories
        let candidates = [
            // CWD / templates/proxy
            Path::new("templates/proxy").to_path_buf(),
            // Backend root / templates/proxy
            Path::new("backend/templates/proxy").to_path_buf(),
        ];

        if let Some(dir) = exe_dir {
            let exe_candidate = dir.join("templates/proxy");
            if exe_candidate.is_dir() {
                return exe_candidate;
            }
        }

        for candidate in &candidates {
            if candidate.is_dir() {
                return candidate.clone();
            }
        }

        // Fallback
        PathBuf::from("templates/proxy")
    }
}
