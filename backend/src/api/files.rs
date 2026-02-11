use axum::{
    extract::{Multipart, Query},
    body::Body,
    http::header,
    response::Response,
    routing::{get, patch, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncReadExt;

use crate::{error::{AppError, AppResult}, AppState};

#[derive(Debug, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: i64,
    pub permissions: String,
    pub owner: String,
    pub group: String,
}

#[derive(Debug, Deserialize)]
pub struct PathQuery {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct FileContentRequest {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct PermissionRequest {
    pub path: String,
    pub mode: Option<String>,
    pub owner: Option<String>,
    pub group: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MkdirRequest {
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct FileContentResponse {
    pub path: String,
    pub content: String,
    pub encoding: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_files).delete(delete_path))
        .route("/download", get(download_file))
        .route("/upload", post(upload_file))
        .route("/content", get(read_file).put(write_file))
        .route("/permissions", patch(set_permissions))
        .route("/mkdir", post(create_directory))
}

/// Validate and canonicalize path to prevent directory traversal
fn validate_path(path: &str) -> AppResult<PathBuf> {
    let path = PathBuf::from(path);
    
    // Basic path traversal check
    let path_str = path.to_string_lossy();
    if path_str.contains("..") {
        return Err(AppError::Validation("Path traversal not allowed".to_string()));
    }
    
    // Ensure absolute path
    if !path.is_absolute() {
        return Err(AppError::Validation("Absolute path required".to_string()));
    }
    
    Ok(path)
}

async fn list_files(Query(query): Query<PathQuery>) -> AppResult<Json<Vec<FileEntry>>> {
    let path = validate_path(&query.path)?;
    
    if !path.exists() {
        return Err(AppError::NotFound(format!("Path not found: {}", query.path)));
    }
    
    if !path.is_dir() {
        return Err(AppError::Validation("Path is not a directory".to_string()));
    }
    
    let mut entries = Vec::new();
    let mut dir = fs::read_dir(&path).await?;
    
    while let Some(entry) = dir.next_entry().await? {
        let metadata = entry.metadata().await?;
        let file_type = metadata.file_type();
        
        entries.push(FileEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path().to_string_lossy().to_string(),
            is_dir: file_type.is_dir(),
            size: metadata.len(),
            modified: metadata
                .modified()
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64)
                .unwrap_or(0),
            permissions: format!("{:o}", metadata.permissions().mode() & 0o777),
            owner: String::new(),
            group: String::new(),
        });
    }
    
    // Sort: directories first, then alphabetically
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });
    
    Ok(Json(entries))
}

async fn download_file(Query(query): Query<PathQuery>) -> AppResult<Response<Body>> {
    let path = validate_path(&query.path)?;
    
    if !path.exists() {
        return Err(AppError::NotFound(format!("File not found: {}", query.path)));
    }
    
    if !path.is_file() {
        return Err(AppError::Validation("Path is not a file".to_string()));
    }
    
    let file_name = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "download".to_string());
    
    let content = fs::read(&path).await?;
    
    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file_name),
        )
        .body(Body::from(content))
        .unwrap())
}

async fn upload_file(mut multipart: Multipart) -> AppResult<Json<serde_json::Value>> {
    let mut target_path: Option<String> = None;
    
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::Validation(e.to_string()))? {
        let name = field.name().unwrap_or_default().to_string();
        
        if name == "path" {
            target_path = Some(field.text().await.map_err(|e| AppError::Validation(e.to_string()))?);
        } else if name == "file" {
            let path = target_path.as_ref()
                .ok_or_else(|| AppError::Validation("Path must be provided before file".to_string()))?;
            
            let file_path = validate_path(path)?;
            let data = field.bytes().await.map_err(|e| AppError::Validation(e.to_string()))?;
            
            fs::write(&file_path, &data).await?;
            
            return Ok(Json(serde_json::json!({
                "success": true,
                "path": file_path.to_string_lossy()
            })));
        }
    }
    
    Err(AppError::Validation("No file provided".to_string()))
}

async fn read_file(Query(query): Query<PathQuery>) -> AppResult<Json<FileContentResponse>> {
    let path = validate_path(&query.path)?;
    
    if !path.exists() {
        return Err(AppError::NotFound(format!("File not found: {}", query.path)));
    }
    
    if !path.is_file() {
        return Err(AppError::Validation("Path is not a file".to_string()));
    }
    
    let mut file = fs::File::open(&path).await?;
    let mut content = Vec::new();
    file.read_to_end(&mut content).await?;
    
    // Try to decode as UTF-8, otherwise return base64
    match String::from_utf8(content.clone()) {
        Ok(text) => Ok(Json(FileContentResponse {
            path: query.path,
            content: text,
            encoding: "utf-8".to_string(),
        })),
        Err(_) => {
            use base64::Engine;
            Ok(Json(FileContentResponse {
                path: query.path,
                content: base64::engine::general_purpose::STANDARD.encode(&content),
                encoding: "base64".to_string(),
            }))
        }
    }
}

async fn write_file(Json(payload): Json<FileContentRequest>) -> AppResult<Json<serde_json::Value>> {
    let path = validate_path(&payload.path)?;
    
    fs::write(&path, &payload.content).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "path": payload.path
    })))
}

async fn set_permissions(Json(payload): Json<PermissionRequest>) -> AppResult<Json<serde_json::Value>> {
    let path = validate_path(&payload.path)?;
    
    if !path.exists() {
        return Err(AppError::NotFound(format!("Path not found: {}", payload.path)));
    }
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        
        if let Some(ref mode) = payload.mode {
            let mode = u32::from_str_radix(mode, 8)
                .map_err(|_| AppError::Validation("Invalid permission mode".to_string()))?;
            let permissions = std::fs::Permissions::from_mode(mode);
            fs::set_permissions(&path, permissions).await?;
        }
    }
    
    Ok(Json(serde_json::json!({
        "success": true,
        "path": payload.path
    })))
}

async fn create_directory(Json(payload): Json<MkdirRequest>) -> AppResult<Json<serde_json::Value>> {
    let path = validate_path(&payload.path)?;
    
    fs::create_dir_all(&path).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "path": payload.path
    })))
}

async fn delete_path(Query(query): Query<PathQuery>) -> AppResult<Json<serde_json::Value>> {
    let path = validate_path(&query.path)?;
    
    if !path.exists() {
        return Err(AppError::NotFound(format!("Path not found: {}", query.path)));
    }
    
    if path.is_dir() {
        fs::remove_dir_all(&path).await?;
    } else {
        fs::remove_file(&path).await?;
    }
    
    Ok(Json(serde_json::json!({
        "success": true,
        "path": query.path
    })))
}

// Trait extension for file permissions (Unix-only)
#[cfg(unix)]
trait PermissionsExt {
    fn mode(&self) -> u32;
}

#[cfg(unix)]
impl PermissionsExt for std::fs::Permissions {
    fn mode(&self) -> u32 {
        std::os::unix::fs::PermissionsExt::mode(self)
    }
}

#[cfg(not(unix))]
trait PermissionsExt {
    fn mode(&self) -> u32;
}

#[cfg(not(unix))]
impl PermissionsExt for std::fs::Permissions {
    fn mode(&self) -> u32 {
        0o755 // Default for non-Unix
    }
}
