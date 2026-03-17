use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Multipart, Path, State},
    routing::{delete, get, post},
};
use serde::Deserialize;

use crate::{
    AppState,
    error::{AppError, AppResult},
    services::certificate::{CertificateService, run_renewal_cycle},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_certificates))
        .route("/issue", post(issue_certificate))
        .route("/upload", post(upload_certificate))
        .route("/renew-check", post(trigger_renewal_check))
        .route("/{id}", delete(delete_certificate))
        .route("/{id}/renew", post(renew_certificate_by_id))
}

#[derive(Debug, Deserialize)]
struct IssueCertificateRequest {
    domain: String,
    #[serde(default)]
    aliases: Vec<String>,
    email: Option<String>,
}

async fn list_certificates(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<crate::db::entities::certificate::Model>>> {
    let certs = CertificateService::list(&state.db).await?;
    Ok(Json(certs))
}

async fn issue_certificate(
    State(state): State<AppState>,
    Json(payload): Json<IssueCertificateRequest>,
) -> AppResult<Json<crate::db::entities::certificate::Model>> {
    let cert = CertificateService::issue_certificate(
        &state.db,
        payload.domain.trim(),
        &payload.aliases,
        payload.email.as_deref(),
    )
    .await?;

    Ok(Json(cert))
}

async fn upload_certificate(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> AppResult<Json<crate::db::entities::certificate::Model>> {
    let mut domain: Option<String> = None;
    let mut cert_pem: Option<String> = None;
    let mut key_pem: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("Failed to read multipart field: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        let data = field
            .text()
            .await
            .map_err(|e| AppError::Validation(format!("Failed to read field '{}': {}", name, e)))?;

        match name.as_str() {
            "domain" => domain = Some(data),
            "cert" | "certificate" => cert_pem = Some(data),
            "key" | "private_key" => key_pem = Some(data),
            _ => {}
        }
    }

    let domain = domain
        .filter(|d| !d.trim().is_empty())
        .ok_or_else(|| AppError::Validation("Missing 'domain' field".to_string()))?;
    let cert_pem = cert_pem.ok_or_else(|| {
        AppError::Validation("Missing 'cert' field (PEM-encoded certificate chain)".to_string())
    })?;
    let key_pem = key_pem.ok_or_else(|| {
        AppError::Validation("Missing 'key' field (PEM-encoded private key)".to_string())
    })?;

    let cert =
        CertificateService::upload_certificate(&state.db, domain.trim(), &cert_pem, &key_pem)
            .await?;

    Ok(Json(cert))
}

async fn delete_certificate(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<serde_json::Value>> {
    CertificateService::delete(&state.db, id).await?;
    Ok(Json(
        serde_json::json!({ "success": true, "message": "Certificate deleted" }),
    ))
}

async fn renew_certificate_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<crate::db::entities::certificate::Model>> {
    use crate::db::entities::certificate::Entity as CertEntity;
    use sea_orm::EntityTrait;

    let cert = CertEntity::find_by_id(id)
        .one(&*state.db)
        .await
        .map_err(|e| AppError::System(format!("Failed to find certificate: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Certificate {} not found", id)))?;

    let renewed = CertificateService::renew_certificate(&state.db, &cert).await?;

    if let Some(docker) = &state.docker {
        use crate::services::application::ApplicationManager;
        use std::path::PathBuf;
        let app_manager = ApplicationManager::new(PathBuf::from(&state.config.app_root_dir));
        let _ = CertificateService::reload_proxies_for_domain(
            &state.db,
            &renewed.domain,
            &app_manager,
            docker,
        )
        .await;
    }

    Ok(Json(renewed))
}

async fn trigger_renewal_check(
    State(state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    let renewal_guard = match Arc::clone(&state.renewal_lock).try_lock_owned() {
        Ok(guard) => guard,
        Err(_) => {
            return Ok(Json(serde_json::json!({
                "success": false,
                "message": "Certificate renewal check is already running"
            })));
        }
    };

    let certs = CertificateService::get_certificates_due_for_renewal(&state.db, 30).await?;
    let count = certs.len();

    let db = Arc::clone(&state.db);
    let config = state.config.clone();
    let docker = state.docker.clone();
    tokio::spawn(async move {
        let _renewal_guard = renewal_guard;
        if let Err(e) = run_renewal_cycle(&db, &config, docker.as_ref()).await {
            tracing::error!("Manual renewal check failed: {}", e);
        }
    });

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("{} certificate(s) due for renewal, processing in background", count)
    })))
}
