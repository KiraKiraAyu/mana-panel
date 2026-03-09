use axum::Router;
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(target_family = "unix")]
use std::process::Command;

use mana_panel_backend::{
    AppState, api,
    config::Config,
    db,
    services::{
        docker::DockerService,
        monitor::SystemMonitor,
        root_agent::{RootAgentClient, run_root_agent},
        user::UserService,
    },
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    CreateUser {
        #[arg(short, long)]
        username: String,
    },
    RootAgent,
}

#[tokio::main]
async fn main() {
    // Load config first so runtime setup is driven by centralized config.
    let config = Config::from_env();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            config.log_filter.as_str(),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Check for CLI args first
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::RootAgent) => {
            if let Err(e) = run_root_agent().await {
                tracing::error!("Failed to start root agent: {}", e);
                std::process::exit(1);
            }
            return;
        }
        _ => {}
    }

    let db_conn = db::init_database(&config.database_url)
        .await
        .expect("Failed to initialize database");
    let db = Arc::new(db_conn);

    match &cli.command {
        Some(Commands::CreateUser { username }) => {
            // Generate random password
            use rand::Rng;
            let password: String = rand::thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(16)
                .map(char::from)
                .collect();

            tracing::info!("Creating user: {}", username);

            match UserService::create_user(&db, username, &password).await {
                Ok(_) => {
                    println!("\nSUCCESS: User created successfully!");
                    println!("Username: {}", username);
                    println!("Password: {}", password);
                    println!("\nPlease save these credentials securely.\n");
                }
                Err(e) => {
                    eprintln!("\nERROR: Failed to create user: {}\n", e);
                    std::process::exit(1);
                }
            }
            return;
        }
        None => {
            // Normal Server Startup
        }
        Some(Commands::RootAgent) => {
            // handled above
        }
    }

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

    let root_agent = match RootAgentClient::new() {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to create root agent client: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = root_agent.ping().await {
        tracing::error!("Root agent is required but unavailable: {}", e);
        std::process::exit(1);
    }

    #[cfg(target_family = "unix")]
    {
        let (uid, gid) = match current_uid_gid() {
            Ok(ids) => ids,
            Err(e) => {
                tracing::error!("Failed to resolve backend uid/gid: {}", e);
                std::process::exit(1);
            }
        };

        if let Err(e) = root_agent.ensure_panel_directories(uid, gid).await {
            tracing::error!(
                "Failed to prepare static website root directories via root-agent: {}",
                e
            );
            std::process::exit(1);
        }
    }

    let state = AppState {
        config: config.clone(),
        monitor,
        db,
        docker,
        root_agent,
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

#[cfg(target_family = "unix")]
fn current_uid_gid() -> Result<(u32, u32), String> {
    let uid = read_id_value("-u")?;
    let gid = read_id_value("-g")?;
    Ok((uid, gid))
}

#[cfg(target_family = "unix")]
fn read_id_value(flag: &str) -> Result<u32, String> {
    let output = Command::new("id")
        .arg(flag)
        .output()
        .map_err(|e| format!("failed to execute `id {}`: {}", flag, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("`id {}` failed: {}", flag, stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .trim()
        .parse::<u32>()
        .map_err(|e| format!("invalid `id {}` output '{}': {}", flag, stdout.trim(), e))
}
