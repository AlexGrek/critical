use axum::{
    http::StatusCode,
    middleware::from_fn_with_state,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use exlogging::{configure_log_event, log_event, LogLevel, LoggerConfig};
use log::{error, info};

use crate::{
    auth::Auth, db::issue_tracker::IssueTrackerDb, middleware::jwt_auth_middleware, state::AppState,
};
use dotenv::dotenv;
use std::{env, path::PathBuf, sync::Arc};
use tower_http::{services::ServeDir, trace::TraceLayer};

mod api;
mod auth;
mod cache;
mod db;
mod errors;
mod exlogging;
mod middleware;
mod models;
mod state;
mod utils;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "data/sled_db".to_string());
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "supersecretjwtkey".to_string());
    let admin_file_path = env::var("ADMIN_FILE_PATH").unwrap_or_else(|_| "admins.txt".to_string());
    let log_file_path = env::var("LOG_FILE_PATH").unwrap_or_else(|_| "application.log".to_string());
    let data_dir_path = env::var("DATA_DIR_PATH").unwrap_or_else(|_| "data".to_string());

    let config = LoggerConfig { log_file_path };
    configure_log_event(config).await.unwrap();

    std::fs::create_dir_all(&data_dir_path)?;

    check_admin_file(&admin_file_path);

    info!("Initializing database at: {}", database_url);
    let app_db = match IssueTrackerDb::new(&database_url).await {
        Ok(db) => {
            info!("Database initialized successfully.");
            db
        }
        Err(e) => {
            error!("Failed to initialize database: {:?}", e);
            panic!("Database initialization failed!");
        }
    };
    let auth = Auth::new(jwt_secret.as_bytes());

    let shared_state = Arc::new(AppState {
        db: app_db,
        auth,
        data_dir_path: PathBuf::from(data_dir_path),
        admin_file_path: PathBuf::from(admin_file_path),
    });
    info!("State initialized: {:?}", shared_state);

    // Define a fallback handler for API routes that don't match
    async fn api_fallback() -> impl IntoResponse {
        (StatusCode::NOT_FOUND, "API endpoint not found").into_response()
    }

    // Define the API router with built-in error handling through Result returns
    let api_router = Router::new()
        .route("/register", post(api::v1::auth::register))
        .route("/login", post(api::v1::auth::login))
        .nest(
            "/protected",
            Router::new().route("/check", get(api::v1::auth::get_protected_data)),
        )
        .layer(from_fn_with_state(
            shared_state.clone(),
            jwt_auth_middleware,
        ))
        // Add the API fallback here
        .fallback(api_fallback);

    // We need a special fallback handler for React Router
    // This fallback handler will serve index.html for any non-API, non-file routes
    async fn spa_fallback() -> impl IntoResponse {
        // Serve the index.html content for client-side routing
        let index_content = match tokio::fs::read_to_string("static/index.html").await {
            Ok(content) => content,
            Err(_) => return (StatusCode::NOT_FOUND, "Not Found").into_response(),
        };

        (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "text/html")],
            index_content,
        )
            .into_response()
    }

    // Define our combined app properly for SPA + API:
    let spa_fallback_service = Router::new().fallback(spa_fallback);

    let app = Router::new()
        .nest("/api/v1", api_router) // API routes with proper 404 handling
        // Add the static files service as a fallback before the SPA fallback
        .fallback_service(ServeDir::new("static").fallback(spa_fallback_service))
        .with_state(shared_state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    log::info!("Starting server at http://0.0.0.0:8080");
    log_event(LogLevel::Info, "Application started", None::<&str>);
    axum::serve(listener, app).await?;

    Ok(())
}

// check if admins file is available, if not - print error into the log, if yes - print info with admins file path
fn check_admin_file(path: &str) {
    let admin_file_path = PathBuf::from(path);
    if admin_file_path.exists() {
        log::info!("Admin file found at: {}", admin_file_path.display());
    } else {
        log::error!(
            "Admin file NOT found at: {}. Admin functionality might be limited.",
            admin_file_path.display()
        );
    }
}
