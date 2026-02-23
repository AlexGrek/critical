use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use crate::db::ArangoDb;
use crate::error::AppError;
use crate::middleware::auth::Auth;
use crit_shared::data_models::Task;
use crit_shared::util_models::{Permissions, super_permissions};

use super::gitops_controller::{
    KindController, filter_to_brief, parse_acl, standard_to_external, standard_to_internal,
};

pub struct TaskController {
    pub db: Arc<ArangoDb>,
}

impl TaskController {
    pub fn new(db: Arc<ArangoDb>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl KindController for TaskController {
    async fn can_read(&self, user_id: &str, doc: Option<&Value>) -> Result<bool, AppError> {
        // ADM_PROJECT_MANAGER can read any task
        let is_admin = self
            .db
            .has_permission(user_id, super_permissions::ADM_PROJECT_MANAGER)
            .await?;
        log::debug!(
            "[ACL] TaskController::can_read: is_admin(ADM_PROJECT_MANAGER)={}",
            is_admin
        );
        if is_admin {
            return Ok(true);
        }

        if let Some(doc) = doc {
            if let Ok(acl) = parse_acl(doc) {
                let principals = self.db.get_user_principals(user_id).await?;
                let result = acl.check_permission(&principals, Permissions::READ);
                log::debug!(
                    "[ACL] TaskController::can_read: check_permission(READ)={}",
                    result
                );
                return Ok(result);
            }
        }

        Ok(false)
    }

    async fn can_write(&self, user_id: &str, doc: Option<&Value>) -> Result<bool, AppError> {
        let is_admin = self
            .db
            .has_permission(user_id, super_permissions::ADM_PROJECT_MANAGER)
            .await?;
        log::debug!(
            "[ACL] TaskController::can_write: is_admin(ADM_PROJECT_MANAGER)={}",
            is_admin
        );
        if is_admin {
            return Ok(true);
        }

        match doc {
            Some(doc) => {
                if let Ok(acl) = parse_acl(doc) {
                    let principals = self.db.get_user_principals(user_id).await?;
                    let result = acl.check_permission(&principals, Permissions::WRITE);
                    log::debug!(
                        "[ACL] TaskController::can_write: check_permission(WRITE)={}",
                        result
                    );
                    return Ok(result);
                }
                Ok(false)
            }
            None => {
                // Creating a new task requires USR_CREATE_PROJECTS (project membership implied)
                let has_perm = self
                    .db
                    .has_permission(user_id, super_permissions::USR_CREATE_PROJECTS)
                    .await?;
                log::debug!(
                    "[ACL] TaskController::can_write: new task, has_permission(USR_CREATE_PROJECTS)={}",
                    has_perm
                );
                Ok(has_perm)
            }
        }
    }

    fn to_internal(&self, mut body: Value, _auth: &Auth) -> Result<Value, AppError> {
        // Auto-prefix task ID with "t_"
        if let Some(obj) = body.as_object_mut() {
            if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                let prefixed = if id.starts_with("t_") {
                    id.to_string()
                } else {
                    format!("t_{}", id)
                };
                obj.insert("id".to_string(), Value::String(prefixed));
            }
        }
        Ok(standard_to_internal(body))
    }

    fn to_external(&self, doc: Value) -> Value {
        standard_to_external(doc)
    }

    fn to_list_external(&self, doc: Value) -> Value {
        let doc = self.to_external(doc);
        filter_to_brief(doc, Task::brief_field_names())
    }

    fn list_projection_fields(&self) -> Option<&'static [&'static str]> {
        Some(&["_key", "title", "state", "priority", "acl", "meta"])
    }

    fn prepare_create(&self, body: &mut Value, user_id: &str) {
        log::debug!("[ACL] TaskController::prepare_create: user={}", user_id);
        let Some(obj) = body.as_object_mut() else {
            return;
        };

        // Populate meta
        let meta = obj.entry("meta").or_insert_with(|| json!({}));
        if let Some(meta_obj) = meta.as_object_mut() {
            meta_obj
                .entry("created_at")
                .or_insert_with(|| json!(chrono::Utc::now().to_rfc3339()));
            meta_obj
                .entry("created_by")
                .or_insert_with(|| json!(user_id));
            meta_obj
                .entry("updated_at")
                .or_insert_with(|| json!(chrono::Utc::now().to_rfc3339()));
            meta_obj.entry("labels").or_insert_with(|| json!({}));
            meta_obj.entry("annotations").or_insert_with(|| json!({}));
        }

        // Inject creator with ROOT ACL
        let acl = obj
            .entry("acl")
            .or_insert_with(|| json!({"list": [], "last_mod_date": chrono::Utc::now().to_rfc3339()}));
        if let Some(acl_obj) = acl.as_object_mut() {
            let list = acl_obj.entry("list").or_insert_with(|| json!([]));
            if let Some(list_arr) = list.as_array_mut() {
                let already_present = list_arr.iter().any(|entry| {
                    entry
                        .get("principals")
                        .and_then(|p| p.as_array())
                        .is_some_and(|ps| ps.iter().any(|p| p.as_str() == Some(user_id)))
                });
                if !already_present {
                    list_arr.push(json!({
                        "permissions": Permissions::ROOT.bits(),
                        "principals": [user_id],
                    }));
                }
            }
        }
    }
}
