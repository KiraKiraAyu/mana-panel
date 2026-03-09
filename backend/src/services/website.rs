use chrono::Utc;
use sea_orm::{ActiveModelTrait, ConnectionTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryOrder, Set, TransactionTrait};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::db::entities::website::{
    self, ActiveModel, Entity as WebsiteEntity, ProxyTargetType, ServerType, SiteType,
    WebsiteStatus,
};
use crate::error::{AppError, AppResult};
use crate::services::application::ApplicationManager;
use crate::services::docker::DockerService;
use crate::services::proxy_config::ProxyConfigService;

// ── API Models ──

#[derive(Debug, Clone, Serialize)]
pub struct WebsiteInfo {
    pub id: i32,
    pub name: String,
    pub primary_domain: String,
    pub aliases: Vec<String>,
    pub server_type: ServerType,
    pub server_instance_id: Option<String>,
    pub site_types: Vec<SiteType>,
    pub proxy_target_type: Option<ProxyTargetType>,
    pub proxy_target_url: Option<String>,
    pub proxy_target_app_id: Option<String>,
    pub proxy_target_app_port: Option<i32>,
    pub root_dir: Option<String>,
    pub status: WebsiteStatus,
    pub error: Option<String>,
    pub has_ssl: bool,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

impl From<website::Model> for WebsiteInfo {
    fn from(m: website::Model) -> Self {
        let aliases: Vec<String> = serde_json::from_value(m.aliases.clone()).unwrap_or_default();
        Self {
            id: m.id,
            name: m.name,
            primary_domain: m.primary_domain,
            aliases,
            server_type: m.server_type,
            server_instance_id: m.server_instance_id,
            site_types: serde_json::from_value(m.site_types).unwrap_or_default(),
            proxy_target_type: m.proxy_target_type,
            proxy_target_url: m.proxy_target_url,
            proxy_target_app_id: m.proxy_target_app_id,
            proxy_target_app_port: m.proxy_target_app_port,
            root_dir: m.root_dir,
            status: m.status,
            error: m.error,
            has_ssl: m.has_ssl,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWebsiteRequest {
    pub name: String,
    pub primary_domain: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub server_instance_id: String,
    pub site_types: Vec<SiteType>,
    pub proxy_target_type: Option<ProxyTargetType>,
    pub proxy_target_url: Option<String>,
    pub proxy_target_app_id: Option<String>,
    pub proxy_target_app_port: Option<i32>,
    pub root_dir: Option<String>,
}

// Use custom double Option deserialization
fn deserialize_some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWebsiteRequest {
    pub name: Option<String>,
    pub primary_domain: Option<String>,
    pub aliases: Option<Vec<String>>,
    pub server_instance_id: Option<String>,
    pub site_types: Option<Vec<SiteType>>,

    #[serde(default, deserialize_with = "deserialize_some")]
    pub proxy_target_type: Option<Option<ProxyTargetType>>,

    #[serde(default, deserialize_with = "deserialize_some")]
    pub proxy_target_url: Option<Option<String>>,

    #[serde(default, deserialize_with = "deserialize_some")]
    pub proxy_target_app_id: Option<Option<String>>,

    #[serde(default, deserialize_with = "deserialize_some")]
    pub proxy_target_app_port: Option<Option<i32>>,

    #[serde(default, deserialize_with = "deserialize_some")]
    pub root_dir: Option<Option<String>>,

    pub has_ssl: Option<bool>,
}

// ── Service ──

pub struct WebsiteService;

impl WebsiteService {
    pub async fn list(db: &DatabaseConnection) -> AppResult<Vec<WebsiteInfo>> {
        let models = WebsiteEntity::find()
            .order_by_asc(website::Column::Id)
            .all(db)
            .await
            .map_err(|e| AppError::System(format!("Failed to list websites: {}", e)))?;

        Ok(models.into_iter().map(WebsiteInfo::from).collect())
    }

    pub async fn get_by_id(db: &DatabaseConnection, id: i32) -> AppResult<WebsiteInfo> {
        let model = WebsiteEntity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| AppError::System(format!("Failed to get website: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Website with id {} not found", id)))?;

        Ok(WebsiteInfo::from(model))
    }

    async fn check_domain_conflict<C: ConnectionTrait>(
        db: &C,
        exclude_id: Option<i32>,
        domains_to_check: &[String],
    ) -> AppResult<()> {
        use sea_orm::{ColumnTrait, Condition, QueryFilter};

        let normalized_domains: Vec<String> = domains_to_check
            .iter()
            .map(|d| d.trim().to_lowercase())
            .collect();

        if normalized_domains.is_empty() {
            return Ok(());
        }

        let mut condition = Condition::any();
        for d in &normalized_domains {
            // Check primary domain
            condition = condition.add(website::Column::PrimaryDomain.eq(d.clone()));
            // Check aliases (basic LIKE match, since SeaORM JSON querying varies by backend)
            condition = condition.add(website::Column::Aliases.like(format!("%\"{}\"%", d)));
        }

        let mut query = WebsiteEntity::find().filter(condition);

        if let Some(id) = exclude_id {
            query = query.filter(website::Column::Id.ne(id));
        }

        let conflicts = query.all(db).await.map_err(|e| {
            AppError::System(format!(
                "Failed to fetch websites for domain validation: {}",
                e
            ))
        })?;

        for site in conflicts {
            let mut site_domains = vec![site.primary_domain.trim().to_lowercase()];
            if let Ok(aliases) = serde_json::from_value::<Vec<String>>(site.aliases.clone()) {
                site_domains.extend(aliases.into_iter().map(|a| a.trim().to_lowercase()));
            }

            for d in &normalized_domains {
                if site_domains.contains(d) {
                    return Err(AppError::Validation(format!(
                        "Domain '{}' is already in use by website '{}'",
                        d, site.name
                    )));
                }
            }
        }

        Ok(())
    }

    pub fn resolve_server_type(
        app_manager: &ApplicationManager,
        instance_id: &str,
    ) -> AppResult<ServerType> {
        let instance_uuid = uuid::Uuid::parse_str(instance_id).map_err(|_| {
            AppError::Validation(format!(
                "'{}' is not a valid server instance id",
                instance_id
            ))
        })?;

        let meta = app_manager
            .load_instance_meta(&instance_uuid)
            .map_err(|e| {
                AppError::Validation(format!(
                    "Server instance '{}' was not found or is invalid: {}",
                    instance_id, e
                ))
            })?;

        match meta.template_id.as_str() {
            "caddy-autossl" => Ok(ServerType::Caddy),
            "nginx-standard" => Ok(ServerType::Nginx),
            "openresty-waf" => Ok(ServerType::OpenResty),
            _ => Err(AppError::Validation(format!(
                "Selected instance '{}' (template '{}') is not a supported proxy server",
                meta.name, meta.template_id
            ))),
        }
    }

    /// Returns the raw Sea-ORM model for use by internal services (e.g. ProxyConfigService).
    pub async fn get_model_by_id<C: ConnectionTrait>(db: &C, id: i32) -> AppResult<website::Model> {
        WebsiteEntity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| AppError::System(format!("Failed to get website: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Website with id {} not found", id)))
    }

    pub async fn create(
        db: &DatabaseConnection,
        app_manager: &ApplicationManager,
        docker: &DockerService,
        req: CreateWebsiteRequest,
    ) -> AppResult<WebsiteInfo> {
        let resolved_server_type = Self::resolve_server_type(app_manager, &req.server_instance_id)?;
        Self::validate_create(&req)?;

        let mut domains_to_check = vec![req.primary_domain.clone()];
        domains_to_check.extend(req.aliases.clone());
        Self::check_domain_conflict(db, None, &domains_to_check).await?;

        let now = Utc::now();

        let model = ActiveModel {
            name: Set(req.name),
            primary_domain: Set(req.primary_domain),
            aliases: Set(serde_json::to_value(&req.aliases).unwrap_or_default()),
            server_type: Set(resolved_server_type),
            server_instance_id: Set(Some(req.server_instance_id.clone())),
            site_types: Set(serde_json::to_value(&req.site_types).unwrap_or_default()),
            proxy_target_type: Set(if req.site_types.contains(&SiteType::ReverseProxy) {
                req.proxy_target_type.clone()
            } else {
                None
            }),
            proxy_target_url: Set(if req.site_types.contains(&SiteType::ReverseProxy) && matches!(&req.proxy_target_type, Some(ProxyTargetType::Url)) {
                req.proxy_target_url.clone()
            } else {
                None
            }),
            proxy_target_app_id: Set(if req.site_types.contains(&SiteType::ReverseProxy) && matches!(&req.proxy_target_type, Some(ProxyTargetType::Application)) {
                req.proxy_target_app_id.clone()
            } else {
                None
            }),
            proxy_target_app_port: Set(if req.site_types.contains(&SiteType::ReverseProxy) && matches!(&req.proxy_target_type, Some(ProxyTargetType::Application)) {
                req.proxy_target_app_port
            } else {
                None
            }),
            root_dir: Set(if req.site_types.contains(&SiteType::Static) {
                req.root_dir.clone()
            } else {
                None
            }),
            status: Set(WebsiteStatus::Running),
            has_ssl: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let txn = db.begin().await.map_err(|e| AppError::System(format!("Failed to begin transaction: {}", e)))?;

        let result = model
            .insert(&txn)
            .await
            .map_err(|e| AppError::System(format!("Failed to create website: {}", e)))?;

        if let Err(e) = ProxyConfigService::deploy(&result, app_manager, docker).await {
            txn.rollback().await.ok();
            return Err(AppError::System(format!("Failed to deploy proxy configuration: {}", e)));
        }

        txn.commit().await.map_err(|e| AppError::System(format!("Failed to commit transaction: {}", e)))?;

        Ok(WebsiteInfo::from(result))
    }

    pub async fn update(
        db: &DatabaseConnection,
        app_manager: &ApplicationManager,
        docker: &DockerService,
        id: i32,
        req: UpdateWebsiteRequest,
    ) -> AppResult<WebsiteInfo> {
        let txn = db.begin().await.map_err(|e| AppError::System(format!("Failed to begin transaction: {}", e)))?;

        let existing = WebsiteEntity::find_by_id(id)
            .one(&txn)
            .await
            .map_err(|e| AppError::System(format!("Failed to get website: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Website with id {} not found", id)))?;

        let mut am: ActiveModel = existing.clone().into();

        if let Some(name) = req.name {
            if name.trim().is_empty() {
                return Err(AppError::Validation(
                    "Website name cannot be empty".to_string(),
                ));
            }
            am.name = Set(name);
        }

        let mut domains_to_check = vec![existing.primary_domain.clone()];
        if let Ok(existing_aliases) =
            serde_json::from_value::<Vec<String>>(existing.aliases.clone())
        {
            domains_to_check.extend(existing_aliases);
        }
        let mut check_needed = false;

        if let Some(ref primary_domain) = req.primary_domain {
            if primary_domain.trim().is_empty() {
                return Err(AppError::Validation(
                    "Primary domain cannot be empty".to_string(),
                ));
            }
            if !is_valid_hostname(primary_domain.trim()) {
                return Err(AppError::Validation(format!(
                    "'{}' is not a valid domain name. Domains may only contain letters, digits, hyphens and dots.",
                    primary_domain
                )));
            }
            if primary_domain != &existing.primary_domain {
                check_needed = true;
                domains_to_check[0] = primary_domain.clone();
            }
            am.primary_domain = Set(primary_domain.clone());
        }

        if let Some(ref aliases) = req.aliases {
            // Validate each alias hostname format
            for alias in aliases {
                if !is_valid_hostname(alias.trim()) {
                    return Err(AppError::Validation(format!(
                        "'{}' is not a valid alias domain. Domains may only contain letters, digits, hyphens and dots.",
                        alias
                    )));
                }
            }
            check_needed = true;
            domains_to_check.truncate(1);
            domains_to_check.extend(aliases.clone());
            am.aliases = Set(serde_json::to_value(aliases).unwrap_or_default());
        }

        if check_needed {
            Self::check_domain_conflict(&txn, Some(id), &domains_to_check).await?;
        }

        if let Some(server_instance_id) = req.server_instance_id {
            if server_instance_id.trim().is_empty() {
                return Err(AppError::Validation(
                    "Server instance cannot be empty".to_string(),
                ));
            }
            am.server_instance_id = Set(Some(server_instance_id.clone()));
            let resolved_server_type = Self::resolve_server_type(app_manager, &server_instance_id)?;
            am.server_type = Set(resolved_server_type);
        }

        if let Some(proxy_target_type) = req.proxy_target_type {
            am.proxy_target_type = Set(proxy_target_type);
        }

        let mut final_proxy_url = existing.proxy_target_url.clone();
        if let Some(proxy_target_url) = req.proxy_target_url {
            // proxy_target_url is Option<Option<String>>; the outer Some means "field is being updated"
            // the inner value is the new URL (None = clear it)
            if let Some(ref url_str) = proxy_target_url {
                if !url_str.trim().is_empty() && !is_valid_http_url(url_str) {
                    return Err(AppError::Validation(
                        "Proxy target URL must start with http:// or https://".to_string(),
                    ));
                }
                final_proxy_url = Some(url_str.clone());
            } else {
                final_proxy_url = None;
            }
            am.proxy_target_url = Set(proxy_target_url);
        }

        let mut final_proxy_app = existing.proxy_target_app_id.clone();
        if let Some(proxy_target_app_id) = req.proxy_target_app_id {
            final_proxy_app = proxy_target_app_id.clone();
            am.proxy_target_app_id = Set(proxy_target_app_id);
        }

        if let Some(proxy_target_app_port) = req.proxy_target_app_port {
            am.proxy_target_app_port = Set(proxy_target_app_port);
        }

        let mut final_root_dir = existing.root_dir.clone();
        if let Some(root_dir) = req.root_dir {
            final_root_dir = root_dir.clone();
            am.root_dir = Set(root_dir);
        }

        let existing_site_types: Vec<SiteType> = serde_json::from_value(existing.site_types.clone()).unwrap_or_default();
        let mut final_site_types = existing_site_types.clone();

        if let Some(site_types) = req.site_types {
            final_site_types = site_types.clone();
            am.site_types = Set(serde_json::to_value(site_types).unwrap_or_default());
        }

        let mut final_proxy_target_type = match &am.proxy_target_type {
            sea_orm::ActiveValue::Set(v) => v.clone(),
            _ => existing.proxy_target_type.clone(),
        };
        let mut final_proxy_app_port = match &am.proxy_target_app_port {
            sea_orm::ActiveValue::Set(v) => *v,
            _ => existing.proxy_target_app_port,
        };

        // Priority: Explicitly clear opposing proxy target type fields before validation
        if !final_site_types.contains(&SiteType::ReverseProxy) {
            am.proxy_target_type = Set(None);
            final_proxy_target_type = None;
            am.proxy_target_url = Set(None);
            final_proxy_url = None;
            am.proxy_target_app_id = Set(None);
            final_proxy_app = None;
            am.proxy_target_app_port = Set(None);
            final_proxy_app_port = None;
        } else {
            if final_proxy_target_type == Some(ProxyTargetType::Url) {
                am.proxy_target_app_id = Set(None);
                final_proxy_app = None;
                am.proxy_target_app_port = Set(None);
                final_proxy_app_port = None;
            } else if final_proxy_target_type == Some(ProxyTargetType::Application) {
                am.proxy_target_url = Set(None);
                final_proxy_url = None;
            }
        }

        if !final_site_types.contains(&SiteType::Static) {
            am.root_dir = Set(None);
            final_root_dir = None;
        }

        Self::validate_site_config(
            &final_site_types,
            final_proxy_target_type.as_ref(),
            final_proxy_url.as_deref(),
            final_proxy_app.as_deref(),
            final_proxy_app_port,
            final_root_dir.as_deref(),
        )?;

        if let Some(has_ssl) = req.has_ssl {
            am.has_ssl = Set(has_ssl);
        }

        am.status = Set(WebsiteStatus::Running);
        am.error = Set(None);
        am.updated_at = Set(Utc::now());

        let result = am
            .update(&txn)
            .await
            .map_err(|e| AppError::System(format!("Failed to update website: {}", e)))?;

        if let Err(e) = ProxyConfigService::deploy(&result, app_manager, docker).await {
            txn.rollback().await.ok();
            return Err(AppError::System(format!("Failed to re-deploy proxy configuration: {}", e)));
        }

        txn.commit().await.map_err(|e| AppError::System(format!("Failed to commit transaction: {}", e)))?;

        Ok(WebsiteInfo::from(result))
    }

    pub async fn delete(
        db: &DatabaseConnection,
        app_manager: &ApplicationManager,
        docker: &DockerService,
        id: i32,
    ) -> AppResult<()> {
        let txn = db.begin().await.map_err(|e| AppError::System(format!("Failed to begin transaction: {}", e)))?;

        let existing = WebsiteEntity::find_by_id(id)
            .one(&txn)
            .await
            .map_err(|e| AppError::System(format!("Failed to get website: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Website with id {} not found", id)))?;

        if let Err(e) = ProxyConfigService::undeploy(&existing, app_manager, docker).await {
            txn.rollback().await.ok();
            return Err(AppError::System(format!("Failed to undeploy proxy configuration: {}", e)));
        }

        existing.delete(&txn).await.map_err(|e| AppError::System(format!("Failed to delete website: {}", e)))?;

        txn.commit().await.map_err(|e| AppError::System(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }

    // ── Validation ──

    fn validate_create(req: &CreateWebsiteRequest) -> AppResult<()> {
        if req.name.trim().is_empty() {
            return Err(AppError::Validation(
                "Website name cannot be empty".to_string(),
            ));
        }

        if req.primary_domain.trim().is_empty() {
            return Err(AppError::Validation(
                "Primary domain cannot be empty".to_string(),
            ));
        }

        if req.server_instance_id.trim().is_empty() {
            return Err(AppError::Validation(
                "Server instance is required".to_string(),
            ));
        }

        // Validate domain format: must look like a valid hostname
        let all_domains = std::iter::once(req.primary_domain.as_str())
            .chain(req.aliases.iter().map(|s| s.as_str()));
        for domain in all_domains {
            if !is_valid_hostname(domain.trim()) {
                return Err(AppError::Validation(format!(
                    "'{}' is not a valid domain name. Domains may only contain letters, digits, hyphens and dots.",
                    domain
                )));
            }
        }

        Self::validate_site_config(
            &req.site_types,
            req.proxy_target_type.as_ref(),
            req.proxy_target_url.as_deref(),
            req.proxy_target_app_id.as_deref(),
            req.proxy_target_app_port,
            req.root_dir.as_deref(),
        )
    }

    fn validate_site_config(
        site_types: &[SiteType],
        proxy_target_type: Option<&ProxyTargetType>,
        proxy_target_url: Option<&str>,
        proxy_target_app_id: Option<&str>,
        proxy_target_app_port: Option<i32>,
        root_dir: Option<&str>,
    ) -> AppResult<()> {
        if site_types.is_empty() {
            return Err(AppError::Validation(
                "At least one site type must be specified".to_string(),
            ));
        }

        if site_types.contains(&SiteType::ReverseProxy) {
            match proxy_target_type {
                Some(ProxyTargetType::Url) => {
                    let has_url = proxy_target_url
                        .map(|u| !u.trim().is_empty())
                        .unwrap_or(false);
                    if !has_url {
                        return Err(AppError::Validation(
                            "Reverse proxy with 'url' target type must specify a target URL"
                                .to_string(),
                        ));
                    }
                    if let Some(url) = proxy_target_url {
                        if !is_valid_http_url(url) {
                            return Err(AppError::Validation(
                                "Proxy target URL must start with http:// or https://".to_string(),
                            ));
                        }
                    }
                }
                Some(ProxyTargetType::Application) => {
                    let has_app = proxy_target_app_id
                        .map(|a| !a.trim().is_empty())
                        .unwrap_or(false);
                    if !has_app {
                        return Err(AppError::Validation(
                            "Reverse proxy with 'application' target type must specify an application instance"
                                .to_string(),
                        ));
                    }

                    let valid_port = proxy_target_app_port
                        .map(|p| p > 0 && p <= 65535)
                        .unwrap_or(false);
                    if !valid_port {
                        return Err(AppError::Validation(
                            "Application proxy target requires a valid port number".to_string(),
                        ));
                    }
                }
                None => {
                    return Err(AppError::Validation(
                        "Reverse proxy site must specify a proxy target type".to_string(),
                    ));
                }
            }
        }

        if site_types.contains(&SiteType::Static) {
            let has_root = root_dir
                .map(|r| !r.trim().is_empty())
                .unwrap_or(false);
            if !has_root {
                return Err(AppError::Validation(
                    "Static site must specify a root directory".to_string(),
                ));
            }

            if let Some(r) = root_dir {
                let path = PathBuf::from(r.trim());
                if !path.starts_with("/opt/mana-panel/www") {
                    return Err(AppError::Validation(
                        "Static site root directory must be located inside /opt/mana-panel/www to ensure proper volume mounting".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }
}

/// Returns true if the string looks like a valid hostname or IP address.
/// Allows letters, digits, hyphens and dots; each label must not start/end with a hyphen.
fn is_valid_hostname(s: &str) -> bool {
    if s.is_empty() || s.len() > 253 {
        return false;
    }
    // Strip optional port suffix for validation purposes
    let host = s.split(':').next().unwrap_or(s);
    for label in host.split('.') {
        if label.is_empty() || label.len() > 63 {
            return false;
        }
        if label.starts_with('-') || label.ends_with('-') {
            return false;
        }
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return false;
        }
    }
    true
}

/// Returns true if the string starts with http:// or https://
fn is_valid_http_url(s: &str) -> bool {
    let lower = s.trim().to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}
