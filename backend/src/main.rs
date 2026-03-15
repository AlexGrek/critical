pub mod api;
pub mod cache;
pub mod config;
pub mod controllers;
pub mod db;
pub mod error;
pub mod events;
pub mod middleware;
pub use crit_shared::{data_models, util_models};
pub mod godmode;
pub mod schema;
pub mod services;
pub mod state;
pub mod test;
pub mod utils;
pub mod validation;

use std::sync::Arc;

use crate::{api::v1::ws::ws_handler, db::ArangoDb, middleware::auth::Auth, state::AppState};
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use axum::{Json, Router, middleware::from_fn_with_state, routing::*};
use log::info;
use names::{Generator, Name};
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tower_http::{
    cors::CorsLayer,
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
    let mut allowed_origins = vec![
        "http://localhost:5173".parse::<HeaderValue>().unwrap(), // Standard Vite
        "http://127.0.0.1:5173".parse::<HeaderValue>().unwrap(),
    ];
    for origin in &shared_state.config.cors_allowed_origins {
        match origin.parse::<HeaderValue>() {
            Ok(v) => allowed_origins.push(v),
            Err(_) => log::warn!("Invalid CORS origin ignored: {origin}"),
        }
    }

    let mainrt = Router::new()
        // Unauthenticated routes — outside the /v1 auth nest so no JWT is required.
        .route(
            "/v1/register",
            post(api::v1::authentication::login::register),
        )
        .route("/v1/login", post(api::v1::authentication::login::login))
        .route("/v1/logout", post(api::v1::authentication::login::logout))
        // Unauthenticated object-store static files (avatars / wallpapers).
        .route(
            "/v1/static/{*path}",
            get(api::v1::static_files::serve_static),
        )
        .nest(
            "/v1",
            Router::new()
                .route("/ws", get(ws_handler))
                .route(
                    "/global/{kind}",
                    get(api::v1::gitops::list_objects).post(api::v1::gitops::create_object),
                )
                .route(
                    "/global/{kind}/search",
                    get(api::v1::gitops::search_objects),
                )
                .route(
                    "/global/{kind}/{id}",
                    get(api::v1::gitops::get_object)
                        .post(api::v1::gitops::upsert_object)
                        .put(api::v1::gitops::update_object)
                        .delete(api::v1::gitops::delete_object),
                )
                .route(
                    "/global/{kind}/{id}/upload/{upload_type}",
                    post(api::v1::upload::upload_media),
                )
                .route(
                    "/global/permissions/{key}/grant",
                    post(api::v1::permissions::grant_permission),
                )
                .route(
                    "/global/permissions/{key}/revoke",
                    post(api::v1::permissions::revoke_permission),
                )
                // Access-check routes — return the caller's effective permissions
                // on a resource, or 404 if they have no FETCH access.
                .route(
                    "/accesscheck/me/permissions",
                    get(api::v1::access_check::get_my_permissions),
                )
                .route(
                    "/accesscheck/global/{kind}/{id}",
                    get(api::v1::access_check::check_global_access),
                )
                .route(
                    "/accesscheck/projects/{project}/{kind}/{id}",
                    get(api::v1::access_check::check_scoped_access),
                )
                // Project-scoped routes
                .route(
                    "/projects/{project}/{kind}",
                    get(api::v1::scoped_gitops::list_scoped_objects)
                        .post(api::v1::scoped_gitops::create_scoped_object),
                )
                .route(
                    "/projects/{project}/{kind}/{id}",
                    get(api::v1::scoped_gitops::get_scoped_object)
                        .put(api::v1::scoped_gitops::update_scoped_object)
                        .delete(api::v1::scoped_gitops::delete_scoped_object),
                )
                .nest(
                    "/debug",
                    Router::new()
                        .route("/collections", get(api::v1::debug::list_collections))
                        .route(
                            "/collections/{name}",
                            get(api::v1::debug::get_collection_data),
                        )
                        .route("/access", get(api::v1::debug::get_access_debug))
                        .route("/events", get(api::v1::debug::list_events))
                        .layer(from_fn_with_state(
                            shared_state.clone(),
                            middleware::godmode_middleware,
                        )),
                )
                .layer(from_fn_with_state(
                    shared_state.clone(),
                    middleware::jwt_auth_middleware,
                )),
        )
        .with_state(shared_state.clone())
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(allowed_origins)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([CONTENT_TYPE, AUTHORIZATION])
                .allow_credentials(true),
        );
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api", mainrt.into())
        .route("/health", get(health_check))
        .split_for_parts();
    let router = router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api));

    router.into_make_service()
}

fn generate_node_name() -> String {
    std::env::var("HOSTNAME").unwrap_or_else(|_| {
        let mut name_gen = Generator::with_naming(Name::Numbered);
        name_gen.next().unwrap()
    })
}

pub async fn create_mock_shared_state() -> Result<AppState, Box<dyn std::error::Error>> {
    let config = config::AppConfig::from_env()?;
    let auth = Auth::new(config.jwt_secret.as_bytes(), config.jwt_expiry_days);
    let db = ArangoDb::connect_basic(
        &config.database_connection_string,
        &config.database_user,
        &config.database_password,
        &config.database_name,
    )
    .await?;
    let db = Arc::new(db);
    let cache = cache::create_default_cache().await;
    let node_id = generate_node_name();
    let event_logger = Arc::new(events::EventLogger::new(db.clone(), node_id));
    Ok(AppState::new(config, auth, db, cache, None, None, event_logger))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    // tracing_subscriber::init();

    let config = config::AppConfig::from_env()?;
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    info!(
        "Log level: {}",
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info (default)".to_string())
    );

    let node_id = generate_node_name();

    info!("Starting application with config:");
    info!("  Node ID: {}", node_id);
    info!("  Host: {}", config.host);
    info!("  Port: {}", config.port);
    info!(
        "  Database connection: {}",
        config.database_connection_string
    );
    info!("  Database name: {}", config.database_name);
    info!("  Client API keys: {:?}", config.client_api_keys);
    info!("  CORS extra origins: {:?}", config.cors_allowed_origins);

    let db = ArangoDb::connect_basic(
        &config.database_connection_string,
        &config.database_user,
        &config.database_password,
        &config.database_name,
    )
    .await?;

    // Seed root account if it doesn't exist
    let auth = Auth::new(config.jwt_secret.as_bytes(), config.jwt_expiry_days);
    let db = Arc::new(db);
    if db.get_user_by_id("u_root").await?.is_none() {
        use crate::controllers::gitops_controller::inject_create_defaults;
        let mut body = serde_json::json!({
            "id": "u_root",
            "password": &config.root_password,
        });
        inject_create_defaults(&mut body, "u_root");
        let ctrl = controllers::Controller::new(db.clone());
        let doc = ctrl.for_kind("users").to_internal(body, &auth)?;
        db.generic_create("users", doc).await?;
        info!("Root account created (username: root)");
    }

    // Ensure root has ADM_GODMODE (idempotent — safe to call on every startup)
    db.grant_permission(
        crit_shared::util_models::super_permissions::ADM_GODMODE,
        "u_root",
    )
    .await?;

    // Create app state
    let cache = cache::create_default_cache().await;
    let objectstore = services::objectstore::ObjectStoreService::try_from_config(&config);
    let event_logger = Arc::new(events::EventLogger::new(db.clone(), node_id.clone()));
    let app_state = AppState::new(config.clone(), auth, db.clone(), cache, None, objectstore, event_logger.clone());
    let shared_state = Arc::new(app_state);

    // Log server startup event
    event_logger.server_event(
        crit_shared::event_models::EventPriority::Lifecycle,
        Some(serde_json::json!({ "host": config.host, "port": config.port })),
    ).await;

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
