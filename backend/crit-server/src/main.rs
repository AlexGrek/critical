use axum::{
    Router,
    http::StatusCode,
    middleware::from_fn_with_state,
    response::IntoResponse,
    routing::{get, post},
};
use chrono::Utc;
use crit_shared::entities::User;
use exlogging::{LogLevel, LoggerConfig, configure_log_event, log_event};
use gitops_lib::store::{
    GenericDatabaseProvider, Store, config::StoreConfig, qstorage::KvStorage, qstorage_sled,
};
use log::info;
use tokio::fs;

use crate::{auth::Auth, errors::AppError, middleware::jwt_auth_middleware, state::AppState};
use dotenv::dotenv;
use std::{collections::HashMap, env, path::PathBuf, sync::Arc};
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
mod test;
mod utils;

async fn create_default_user(state: &AppState) -> Result<(), anyhow::Error> {
    let root = state
        .store
        .provider::<User>()
        .try_get_by_key("root")
        .await?;
    match root {
        Some(_) => {
            info!("User root already exists, skipping creation");
            Ok(())
        }
        None => {
            info!("Creating user root with password admin123");
            let hashed_password = state.auth.hash_password("admin123")?;
            let root_user = User {
                uid: "root".to_string(),
                email: "root@cluster.local".to_string(),
                password_hash: Some(hashed_password),
                oauth: None,
                created_at: Utc::now().to_rfc3339(),
                annotations: HashMap::new(),
                has_admin_status: true,
            };
            let insertion_result = state.store.provider::<User>().insert(&root_user).await?;
            Ok(insertion_result)
        }
    }
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "data/sled_db".to_string());
    let database_index_url = env::var("INDEX_DB").unwrap_or_else(|_| "data/index_db".to_string());
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "supersecretjwtkey".to_string());
    let admin_file_path = env::var("ADMIN_FILE_PATH").unwrap_or_else(|_| "admins.txt".to_string());
    let log_file_path = env::var("LOG_FILE_PATH").unwrap_or_else(|_| "application.log".to_string());
    let data_dir_path = env::var("DATA_DIR_PATH").unwrap_or_else(|_| "data".to_string());

    let config_content = fs::read_to_string("config.yaml")
        .await
        .expect("Failed to read config.yaml. Make sure the file exists.");
    let store_config: StoreConfig = serde_yaml::from_str(&config_content).unwrap();

    let store = Store::new(store_config);

    let config = LoggerConfig { log_file_path };
    configure_log_event(config).await.unwrap();

    std::fs::create_dir_all(&data_dir_path)?;

    check_admin_file(&admin_file_path);

    info!("Initializing database at: {}", database_url);

    let auth = Auth::new(jwt_secret.as_bytes());

    let mut index = qstorage_sled::SledKv::new(database_index_url.clone()).unwrap_or_else(|e| {
        log_event(LogLevel::Error, e.to_string(), None::<&str>);
        panic!(
            "Failed to create or open index db: {}, url: {}",
            e.to_string(),
            database_index_url
        )
    });

    db::initialize_index(&mut index);

    let shared_state = Arc::new(AppState {
        // db: app_db,
        auth,
        data_dir_path: PathBuf::from(data_dir_path),
        admin_file_path: PathBuf::from(admin_file_path),
        store: Arc::new(store),
        index: Arc::new(index),
    });

    let failure_in_default_user_creation = create_default_user(&shared_state).await;
    match failure_in_default_user_creation {
        Err(e) => log_event(LogLevel::Error, e.to_string(), None::<&str>),
        _ => (),
    }

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
        .nest(
            "/state",
            Router::new().route(
                "/describe/{kind}",
                get(api::v1::state::fetch::handle_describe),
            ),
        )
        .nest(
            "/personal",
            Router::new().route(
                "/dashboard",
                get(api::v1::dashboard::user_dashboard::handle_user_dashboard),
            ),
        )
        .nest(
            "/ops",
            Router::new()
                .route("/create", post(api::v1::ops::crud::handle_create))
                .route("/upsert", post(api::v1::ops::crud::handle_upsert))
                .route("/list/{kind}", get(api::v1::ops::crud::handle_list)),
        )
        .nest(
            "/adm",
            Router::new().route(
                "/issue_invite",
                post(api::v1::adm::user_managements_endpoints::issue_invite),
            ),
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
