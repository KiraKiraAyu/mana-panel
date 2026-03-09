pub mod api;
pub mod config;
pub mod db;
pub mod error;
pub mod middleware;
pub mod services;

use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub use config::Config;
pub use services::docker::DockerService;
pub use services::monitor::SystemMonitor;
pub use services::root_agent::RootAgentClient;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub monitor: SystemMonitor,
    pub db: Arc<DatabaseConnection>,
    pub docker: Option<DockerService>,
    pub root_agent: RootAgentClient,
}
