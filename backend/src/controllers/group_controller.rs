use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::db::ArangoDb;
use crate::error::AppError;
use crate::middleware::auth::Auth;
use crit_shared::models::super_permissions;

use super::gitops_controller::{KindController, standard_to_external, standard_to_internal};

pub struct GroupController {
    pub db: Arc<ArangoDb>,
}

impl GroupController {
    pub fn new(db: Arc<ArangoDb>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl KindController for GroupController {
    async fn can_read(&self, _user_id: &str, _doc: Option<&Value>) -> Result<bool, AppError> {
        Ok(true)
    }

    async fn can_write(&self, user_id: &str, _doc: Option<&Value>) -> Result<bool, AppError> {
        let has_perm = self
            .db
            .has_permission(user_id, super_permissions::ADM_USER_MANAGER)
            .await?;
        log::debug!(
            "[ACL] GroupController::can_write: has_permission(ADM_USER_MANAGER)={}",
            has_perm
        );
        Ok(has_perm)
    }

    fn to_internal(&self, body: Value, _auth: &Auth) -> Result<Value, AppError> {
        Ok(standard_to_internal(body))
    }

    fn to_external(&self, doc: Value) -> Value {
        standard_to_external(doc)
    }
}
