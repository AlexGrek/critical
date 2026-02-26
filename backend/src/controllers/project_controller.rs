use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use crate::db::ArangoDb;
use crate::error::AppError;
use crate::middleware::auth::Auth;
use crit_shared::data_models::Project;
use crit_shared::util_models::{Permissions, super_permissions};

use super::gitops_controller::{
    KindController, filter_to_brief, parse_acl, standard_to_external, standard_to_internal,
};

pub struct ProjectController {
    pub db: Arc<ArangoDb>,
}

impl ProjectController {
    pub fn new(db: Arc<ArangoDb>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl KindController for ProjectController {
    async fn can_read(&self, user_id: &str, doc: Option<&Value>) -> Result<bool, AppError> {
        let principals = self.db.get_user_principals(user_id).await?;

        // ADM_CONFIG_EDITOR can read any project
        if self
            .db
            .has_permission_with_principals(&principals, super_permissions::ADM_CONFIG_EDITOR)
            .await?
        {
            log::debug!(
                "[ACL] ProjectController::can_read: user={} has ADM_CONFIG_EDITOR",
                user_id
            );
            return Ok(true);
        }

        // Document-level ACL
        if let Some(doc) = doc {
            if let Ok(acl) = parse_acl(doc) {
                let result = acl.check_permission(&principals, Permissions::READ);
                log::debug!(
                    "[ACL] ProjectController::can_read: check_permission(READ)={}",
                    result
                );
                return Ok(result);
            }
        }
        Ok(false)
    }

    async fn can_write(&self, user_id: &str, doc: Option<&Value>) -> Result<bool, AppError> {
        let principals = self.db.get_user_principals(user_id).await?;

        // ADM_CONFIG_EDITOR can write any project
        if self
            .db
            .has_permission_with_principals(&principals, super_permissions::ADM_CONFIG_EDITOR)
            .await?
        {
            log::debug!(
                "[ACL] ProjectController::can_write: user={} has ADM_CONFIG_EDITOR",
                user_id
            );
            return Ok(true);
        }

        match doc {
            Some(doc) => {
                // Existing project: check ACL for MODIFY
                if let Ok(acl) = parse_acl(doc) {
                    let result = acl.check_permission(&principals, Permissions::MODIFY);
                    log::debug!(
                        "[ACL] ProjectController::can_write: check_permission(MODIFY)={}",
                        result
                    );
                    return Ok(result);
                }
                Ok(false)
            }
            None => {
                // New project: check usr_create_projects super-permission
                let has_perm = self
                    .db
                    .has_permission_with_principals(
                        &principals,
                        super_permissions::USR_CREATE_PROJECTS,
                    )
                    .await?;
                log::debug!(
                    "[ACL] ProjectController::can_write: new project, USR_CREATE_PROJECTS={}",
                    has_perm
                );
                Ok(has_perm)
            }
        }
    }

    fn to_internal(&self, body: Value, _auth: &Auth) -> Result<Value, AppError> {
        Ok(standard_to_internal(body))
    }

    fn to_external(&self, doc: Value) -> Value {
        standard_to_external(doc)
    }

    fn to_list_external(&self, doc: Value) -> Value {
        let doc = self.to_external(doc);
        filter_to_brief(doc, Project::brief_field_names())
    }

    fn list_projection_fields(&self) -> Option<&'static [&'static str]> {
        Some(&["_key", "name", "acl", "meta"])
    }

    fn super_permission(&self) -> Option<&str> {
        Some(super_permissions::ADM_CONFIG_EDITOR)
    }

    fn prepare_create(&self, body: &mut Value, user_id: &str) {
        log::debug!(
            "[ACL] ProjectController::prepare_create: user={}",
            user_id
        );
        let Some(obj) = body.as_object_mut() else {
            return;
        };

        // Populate meta if not already set
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
            meta_obj
                .entry("labels")
                .or_insert_with(|| json!({}));
            meta_obj
                .entry("annotations")
                .or_insert_with(|| json!({}));
        }

        // Ensure ACL exists with creator having ROOT permissions
        let acl = obj.entry("acl").or_insert_with(|| {
            json!({"list": [], "last_mod_date": chrono::Utc::now().to_rfc3339()})
        });

        let Some(acl_obj) = acl.as_object_mut() else {
            return;
        };
        let list = acl_obj.entry("list").or_insert_with(|| json!([]));

        let Some(list_arr) = list.as_array_mut() else {
            return;
        };

        let already_present = list_arr.iter().any(|entry| {
            entry
                .get("principals")
                .and_then(|p| p.as_array())
                .is_some_and(|principals| {
                    principals.iter().any(|p| p.as_str() == Some(user_id))
                })
        });

        if !already_present {
            list_arr.push(json!({
                "permissions": Permissions::ROOT.bits(),
                "principals": [user_id],
            }));
            log::debug!(
                "[ACL] ProjectController::prepare_create: added ROOT entry for user",
            );
        }
    }
}
