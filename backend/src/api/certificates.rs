use axum::{
    Json, Router,
    extract::{Multipart, Path, State},
    routing::{delete, get, post},
};
use serde::Deserialize;

use crate::{
    AppState,
    error::{AppError, AppResult},
    services::certificate::CertificateService,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_certificates))
        .route("/issue", post(issue_certificate))
        .route("/upload", post(upload_certificate))
        .route("/{id}", delete(delete_certificate))
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
