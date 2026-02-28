pub mod api;
pub mod cache;
pub mod config;
pub mod controllers;
pub mod db;
pub mod error;
pub mod middleware;
pub use crit_shared::{data_models, util_models};
pub mod schema;
pub mod services;
pub mod state;
pub mod test;
pub mod utils;
pub mod validation;
pub mod godmode;

use std::sync::Arc;

use crate::{
    api::v1::ws::ws_handler,
    db::ArangoDb,
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
        .route("/register", post(api::v1::authentication::login::register))
        .route("/login", post(api::v1::authentication::login::login))
        .route("/logout", post(api::v1::authentication::login::logout))
        .nest(
            "/v1",
            Router::new()
                .route("/ws", get(ws_handler))
                .route(
                    "/global/{kind}",
                    get(api::v1::gitops::list_objects).post(api::v1::gitops::create_object),
                )
                .route(
                    "/global/{kind}/{id}",
                    get(api::v1::gitops::get_object)
                        .post(api::v1::gitops::upsert_object)
                        .put(api::v1::gitops::update_object)
                        .delete(api::v1::gitops::delete_object),
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
                        .route(
                            "/collections",
                            get(api::v1::debug::list_collections),
                        )
                        .route(
                            "/collections/{name}",
                            get(api::v1::debug::get_collection_data),
                        )
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
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api", mainrt.into())
        .route("/health", get(health_check))
        .split_for_parts();
    let router = router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api));

    router.into_make_service()
}

pub async fn create_mock_shared_state() -> Result<AppState, Box<dyn std::error::Error>> {
    let config = config::AppConfig::from_env()?;
    let auth = Auth::new(config.jwt_secret.as_bytes(), config.jwt_expiry_days);
    let db = ArangoDb::connect_basic(&config.database_connection_string, &config.database_user, &config.database_password, &config.database_name).await?;
    let cache = cache::create_default_cache().await;
    Ok(AppState::new(
        config,
        auth,
        Arc::new(db),
        cache,
        None,
        None,
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

    let db = ArangoDb::connect_basic(&config.database_connection_string, &config.database_user, &config.database_password, &config.database_name).await?;

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
    let app_state = AppState::new(
        config.clone(),
        auth,
        db.clone(),
        cache,
        None,
        objectstore,
    );
    let shared_state = Arc::new(app_state);

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
