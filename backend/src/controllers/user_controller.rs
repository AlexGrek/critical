use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::db::ArangoDb;
use crate::error::AppError;
use crate::middleware::auth::Auth;
use crate::validation::naming::validate_username;
use crit_shared::data_models::User;
use crit_shared::util_models::super_permissions;

use super::gitops_controller::{KindController, filter_to_brief, standard_to_external, rename_id_to_key};

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
            // Validate and auto-prefix user ID
            if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                // Strip u_ prefix if present (for backward compatibility)
                let id_without_prefix = id.strip_prefix("u_").unwrap_or(id);

                // Validate username
                let validated_username = validate_username(id_without_prefix)
                    .map_err(AppError::Validation)?;

                // Add u_ prefix
                let prefixed_id = format!("u_{}", validated_username);
                obj.insert("id".to_string(), Value::String(prefixed_id));
            }

            // Hash password if provided
            if let Some(password) = obj.remove("password") {
                if let Some(pw_str) = password.as_str() {
                    let hash = auth.hash_password(pw_str)?;
                    obj.insert("password_hash".to_string(), Value::String(hash));
                }
            }

            // Ensure personal field has a default value if missing
            if !obj.contains_key("personal") {
                obj.insert("personal".to_string(), serde_json::json!({
                    "name": "",
                    "gender": "",
                    "job_title": "",
                    "manager": null
                }));
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

    fn to_list_external(&self, doc: Value) -> Value {
        let doc = self.to_external(doc);
        filter_to_brief(doc, User::brief_field_names())
    }

    fn list_projection_fields(&self) -> Option<&'static [&'static str]> {
        // _key maps to "id" after to_external
        Some(&["_key", "personal", "labels", "annotations"])
    }

    async fn after_delete(&self, key: &str, db: &ArangoDb) -> Result<(), AppError> {
        log::debug!(
            "[LIFECYCLE] UserController::after_delete: user={}",
            key
        );

        // Remove user from all groups, get list of now-empty groups
        let empty_groups = db.remove_principal_from_all_groups(key).await?;

        // Cascade: delete any groups that became empty
        for group_id in empty_groups {
            log::debug!(
                "[LIFECYCLE] UserController::after_delete: group {} is now empty, cascading",
                group_id
            );
            super::group_controller::GroupController::cascade_delete_group(db, &group_id).await?;
        }

        Ok(())
    }
}
