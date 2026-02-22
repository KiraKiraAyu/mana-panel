use serde::Deserialize;
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    pub app_root_dir: String,
    pub log_filter: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("PORT must be a number"),
            database_url: Self::resolve_database_url(),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-super-secret-key-change-in-production".to_string()),
            jwt_expiry_hours: env::var("JWT_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .expect("JWT_EXPIRY_HOURS must be a number"),
            app_root_dir: Self::resolve_app_root_dir(),
            log_filter: env::var("RUST_LOG")
                .unwrap_or_else(|_| "mana_panel_backend=debug,tower_http=debug".to_string()),
        }
    }

    fn resolve_database_url() -> String {
        if let Ok(raw) = env::var("DATABASE_URL") {
            let trimmed = raw.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }

        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // Prefer project root DB path so running from either repo root or backend/
        // uses the same sqlite file.
        let project_root = if cwd.join("backend/Cargo.toml").is_file() {
            cwd.clone()
        } else if cwd.join("Cargo.toml").is_file()
            && cwd.file_name().and_then(|s| s.to_str()) == Some("backend")
        {
            cwd.parent()
                .map(PathBuf::from)
                .unwrap_or_else(|| cwd.clone())
        } else if cwd.join("../backend/Cargo.toml").is_file() {
            cwd.join("..")
        } else {
            cwd.clone()
        };

        let db_path = project_root.join("mana-panel.db");
        format!("sqlite:{}?mode=rwc", db_path.to_string_lossy())
    }

    fn resolve_app_root_dir() -> String {
        if let Ok(raw) = env::var("MANA_PANEL_APP_ROOT_DIR") {
            let trimmed = raw.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }

        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let candidates = vec![
            cwd.join("backend/apps"),
            cwd.join("apps"),
            cwd.join("../backend/apps"),
        ];

        for candidate in candidates {
            if candidate.is_dir() {
                return candidate.to_string_lossy().to_string();
            }
        }

        cwd.join("backend/apps").to_string_lossy().to_string()
    }
}
