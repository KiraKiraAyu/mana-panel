use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use mana_panel_backend::{
    AppState, api,
    config::Config,
    db,
    services::{docker::DockerService, monitor::SystemMonitor, user::UserService},
};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mana_panel_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();

    tracing::info!("Connecting to database...");
    let db = db::init_database(&config.database_url)
        .await
        .expect("Failed to initialize database");
    let db = Arc::new(db);

    UserService::init_default_admin(&db)
        .await
        .expect("Failed to initialize default admin user");

    // Initialize system monitor
    let monitor = SystemMonitor::new();

    // Initialize Docker service (optional — panel still works without Docker)
    let docker = match DockerService::new() {
        Ok(svc) => {
            tracing::info!("Docker daemon connected successfully");
            Some(svc)
        }
        Err(e) => {
            tracing::warn!(
                "Docker is not available: {}. Docker management features will be disabled.",
                e
            );
            None
        }
    };

    let state = AppState {
        config: config.clone(),
        monitor,
        db,
        docker,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api", api::create_router())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::new(config.host.parse().expect("Invalid host"), config.port);

    tracing::info!("Mana Panel listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
