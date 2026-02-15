use std::sync::Arc;

use crate::{
    config::{AppConfig, RuntimeConfig},
    controllers::Controller,
    db::ArangoDb,
    middleware::auth::Auth, services::offloadmq::OffloadClient,
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub auth: Arc<Auth>,
    pub controller: Arc<Controller>,
    pub db: Arc<ArangoDb>,
    pub runtime_config: Arc<RuntimeConfig>,
    pub offloadmq: Arc<Option<OffloadClient>>,
}

impl AppState {
    pub fn new(config: AppConfig, auth: Auth, database: Arc<ArangoDb>, offloadmq: Option<OffloadClient>) -> Self {
        Self {
            config: Arc::new(config),
            auth: Arc::new(auth),
            db: database.clone(),
            runtime_config: Arc::new(AppConfig::runtime_from_env().unwrap_or_default()),
            controller: Arc::new(Controller::new(database.clone())),
            offloadmq: Arc::new(offloadmq)
        }
    }
}
