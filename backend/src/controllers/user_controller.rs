use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::db::ArangoDb;
use crate::error::AppError;
use crate::middleware::auth::Auth;
use crit_shared::models::super_permissions;

use super::gitops_controller::{KindController, standard_to_external, rename_id_to_key};

pub struct UserController {
    pub db: Arc<ArangoDb>,
}

impl UserController {
    pub fn new(db: Arc<ArangoDb>) -> Self {
        Self { db }
    }

    pub async fn validate_user(&self, username: &str) -> bool {
        let user_res = self.db.get_user_by_id(username).await;
        user_res.is_ok()
    }
}

#[async_trait]
impl KindController for UserController {
    async fn can_read(&self, _user_id: &str, _doc: Option<&Value>) -> Result<bool, AppError> {
        Ok(true)
    }

    async fn can_write(&self, user_id: &str, _doc: Option<&Value>) -> Result<bool, AppError> {
        let has_perm = self
            .db
            .has_permission(user_id, super_permissions::ADM_USER_MANAGER)
            .await?;
        log::debug!(
            "[ACL] UserController::can_write: has_permission(ADM_USER_MANAGER)={}",
            has_perm
        );
        Ok(has_perm)
    }

    fn to_internal(&self, mut body: Value, auth: &Auth) -> Result<Value, AppError> {
        if let Some(obj) = body.as_object_mut() {
            if let Some(password) = obj.remove("password") {
                if let Some(pw_str) = password.as_str() {
                    let hash = auth.hash_password(pw_str)?;
                    obj.insert("password_hash".to_string(), Value::String(hash));
                }
            }
        }
        rename_id_to_key(&mut body);
        Ok(body)
    }

    fn to_external(&self, mut doc: Value) -> Value {
        if let Some(obj) = doc.as_object_mut() {
            obj.remove("password_hash");
        }
        standard_to_external(doc)
    }
}
