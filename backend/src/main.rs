pub mod api;
pub mod config;
pub mod controllers;
pub mod db;
pub mod error;
pub mod middleware;
pub mod models;
pub mod schema;
pub mod state;
pub mod test;
pub mod utils;
pub mod validation;

use std::sync::Arc;

use crate::{
    api::v1::ws::ws_handler,
    db::{
        DatabaseInterface,
        arangodb::{ArangoDatabase, connect_or_create_db_no_auth},
        inmemory::InMemoryDatabase,
    },
    middleware::auth::Auth,
    state::AppState,
};
use axum::{Json, Router, middleware::from_fn_with_state, routing::*};
use log::info;
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;



// Uncomment on build if you want swagger UI, currently enabling this makes IDE fail.
// use utoipauto::utoipauto;
// #[utoipauto]
#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

pub fn create_app(shared_state: Arc<AppState>) -> IntoMakeService<Router> {
    let mainrt = Router::new()
        // Health check and stats
        .route(
            "/register",
            post(api::v1::authentication::login::register),
        )
        .route("/login", post(api::v1::authentication::login::login))
        .nest(
            "/v1",
            Router::new()
                .route("/ws", get(ws_handler))
                .layer(from_fn_with_state(
                    shared_state.clone(),
                    middleware::jwt_auth_middleware,
                )),
        )
        .with_state(shared_state.clone())
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api", mainrt.into())
        .route("/health", get(health_check))
        .split_for_parts();
    let router = router.merge(
        SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", api),
    );

    router.into_make_service()
}

pub fn create_mock_shared_state() -> Result<AppState, Box<dyn std::error::Error>> {
    let config = config::AppConfig::from_env()?;
    let auth = Auth::new(config.jwt_secret.as_bytes());
    Ok(AppState::new(
        config,
        auth,
        Arc::new(InMemoryDatabase::new()),
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    // tracing_subscriber::init();

    let config = config::AppConfig::from_env()?;
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    info!("Starting application with config:");
    info!("  Host: {}", config.host);
    info!("  Port: {}", config.port);
    info!(
        "  Database connection: {}",
        config.database_connection_string
    );
    info!("  Database name: {}", config.database_name);
    info!("  Client API keys: {:?}", config.client_api_keys);
    info!("  Management token: {}", config.management_token);

    let mut database: Option<Arc<dyn DatabaseInterface>> = None;

    if config.database_connection_string.starts_with("http") {
        info!("Using ArangoDB as database backend");
        let conn =
            arangors::Connection::establish_without_auth(config.database_connection_string.clone())
                .await?;
        let db = connect_or_create_db_no_auth(&conn, &config.database_name).await?;
        let wrapper = ArangoDatabase::new(db);
        database = Some(Arc::new(wrapper));
    }

    // Create app state
    let auth = Auth::new(config.jwt_secret.as_bytes());
    let app_state = AppState::new(
        config.clone(),
        auth,
        database.unwrap_or(Arc::new(InMemoryDatabase::new())),
    );
    let shared_state = Arc::new(app_state);

    // Init the database
    info!("  Database initialization...");
    shared_state.db.initialize().await?;
    info!("  Database initialization complete");

    // Build the application router
    let app = create_app(shared_state);

    // Start the server
    let bind_address = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&bind_address).await?;
    info!("Server starting on http://{}", bind_address);
    axum::serve(listener, app).await?;

    Ok(())
}

// Utility handlers
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    }))
}
