use std::sync::Arc;

use crate::{
    config::{AppConfig, RuntimeConfig},
    controllers::Controller,
    db::DatabaseInterface,
    middleware::auth::Auth,
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub auth: Arc<Auth>,
    pub controller: Arc<Controller>,
    pub db: Arc<dyn DatabaseInterface>,
    pub runtime_config: Arc<RuntimeConfig>,
}

impl AppState {
    pub fn new(config: AppConfig, auth: Auth, database: Arc<dyn DatabaseInterface>) -> Self {
        Self {
            config: Arc::new(config),
            auth: Arc::new(auth),
            db: database.clone(),
            runtime_config: Arc::new(AppConfig::runtime_from_env().unwrap_or_default()),
            controller: Arc::new(Controller::new(database.clone())),
        }
    }
}
