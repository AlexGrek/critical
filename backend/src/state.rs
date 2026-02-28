use std::sync::Arc;

use serde_json::json;
use tokio::sync::Semaphore;

use crate::{
    cache::CacheStore,
    config::{AppConfig, RuntimeConfig},
    controllers::Controller,
    db::ArangoDb,
    godmode,
    middleware::auth::Auth,
    services::objectstore::ObjectStoreService,
    services::offloadmq::OffloadClient,
};
use crit_shared::util_models::super_permissions;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub auth: Arc<Auth>,
    pub controller: Arc<Controller>,
    pub db: Arc<ArangoDb>,
    pub cache: Arc<CacheStore>,
    pub runtime_config: Arc<RuntimeConfig>,
    pub offloadmq: Arc<Option<OffloadClient>>,
    pub objectstore: Arc<Option<ObjectStoreService>>,
    /// Limits background image conversion to one task at a time.
    /// All other upload tasks queue up and wait their turn.
    pub image_processing_semaphore: Arc<Semaphore>,
}

impl AppState {
    pub fn new(config: AppConfig, auth: Auth, database: Arc<ArangoDb>, cache: Arc<CacheStore>, offloadmq: Option<OffloadClient>, objectstore: Option<ObjectStoreService>) -> Self {
        Self {
            config: Arc::new(config),
            auth: Arc::new(auth),
            db: database.clone(),
            cache,
            runtime_config: Arc::new(AppConfig::runtime_from_env().unwrap_or_default()),
            controller: Arc::new(Controller::new(database.clone())),
            offloadmq: Arc::new(offloadmq),
            objectstore: Arc::new(objectstore),
            image_processing_semaphore: Arc::new(Semaphore::new(1)),
        }
    }

    /// Check if a user has ADM_GODMODE, using the special_access_cache with
    /// 5-minute TTL. Falls back to a DB query on cache miss.
    pub async fn has_godmode(&self, user_id: &str) -> Result<bool, anyhow::Error> {
        let cache_key = godmode::godmode_cache_key(user_id);

        // Check cache first
        if let Some(val) = self.cache.get(godmode::SPECIAL_ACCESS_CACHE, &cache_key).await {
            if let Some(b) = val.as_bool() {
                log::debug!("[GODMODE] cache hit for {}: {}", user_id, b);
                return Ok(b);
            }
        }

        // Cache miss — query DB
        let has = self
            .db
            .has_permission(user_id, super_permissions::ADM_GODMODE)
            .await?;
        log::debug!("[GODMODE] cache miss for {}, queried DB: {}", user_id, has);

        // Store in cache
        self.cache
            .set(
                godmode::SPECIAL_ACCESS_CACHE,
                cache_key,
                json!(has),
            )
            .await;

        Ok(has)
    }
}
