use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use crate::db::ArangoDb;
use crate::error::AppError;
use crate::middleware::auth::Auth;
use crate::validation::naming::validate_group_id;
use crit_shared::data_models::Group;
use crit_shared::util_models::{Permissions, super_permissions};

use super::gitops_controller::{
    KindController, filter_to_brief, parse_acl, standard_to_external, standard_to_internal,
};

pub struct GroupController {
    pub db: Arc<ArangoDb>,
}

impl GroupController {
    pub fn new(db: Arc<ArangoDb>) -> Self {
        Self { db }
    }

    /// Remove all membership references for a group and return parent groups
    /// that became empty as a result.
    async fn cleanup_group_references(db: &ArangoDb, group_id: &str) -> Result<Vec<String>, AppError> {
        // Remove all membership edges where this group is the target (members OF this group)
        db.remove_all_members_of_group(group_id).await?;

        // Remove this group as a member of all parent groups, get list of now-empty parents
        let empty_parents = db.remove_principal_from_all_groups(group_id).await?;

        Ok(empty_parents)
    }

    /// Recursively delete a group and cascade: remove it from parent groups,
    /// delete any parent groups that become empty.
    pub async fn cascade_delete_group(db: &ArangoDb, group_id: &str) -> Result<(), AppError> {
        log::debug!(
            "[CASCADE] GroupController::cascade_delete_group: group={}",
            group_id
        );

        let empty_parents = Self::cleanup_group_references(db, group_id).await?;

        // Delete the group document itself
        // Ignore errors if already deleted (e.g. during recursive cascade)
        let _ = db.generic_delete("groups", group_id).await;

        // Recursively cascade for any parent groups that became empty
        for parent_id in empty_parents {
            log::debug!(
                "[CASCADE] GroupController: parent group {} is now empty, deleting",
                parent_id
            );
            Box::pin(Self::cascade_delete_group(db, &parent_id)).await?;
        }

        Ok(())
    }
}

#[async_trait]
impl KindController for GroupController {
    async fn can_read(&self, user_id: &str, doc: Option<&Value>) -> Result<bool, AppError> {
        // ADM_USER_MANAGER can read any group
        let is_admin = self
            .db
            .has_permission(user_id, super_permissions::ADM_USER_MANAGER)
            .await?;
        log::debug!(
            "[ACL] GroupController::can_read: is_admin(ADM_USER_MANAGER)={}",
            is_admin
        );
        if is_admin {
            return Ok(true);
        }

        // Check document-level ACL for READ
        if let Some(doc) = doc {
            if let Ok(acl) = parse_acl(doc) {
                let principals = self.db.get_user_principals(user_id).await?;
                log::debug!(
                    "[ACL] GroupController::can_read: principals={:?}",
                    principals
                );
                let result = acl.check_permission(&principals, Permissions::READ);
                log::debug!(
                    "[ACL] GroupController::can_read: check_permission(READ)={}",
                    result
                );
                return Ok(result);
            }
        }

        Ok(false)
    }

    async fn can_write(&self, user_id: &str, doc: Option<&Value>) -> Result<bool, AppError> {
        // ADM_USER_MANAGER can write any group
        let is_admin = self
            .db
            .has_permission(user_id, super_permissions::ADM_USER_MANAGER)
            .await?;
        log::debug!(
            "[ACL] GroupController::can_write: is_admin(ADM_USER_MANAGER)={}",
            is_admin
        );
        if is_admin {
            return Ok(true);
        }

        match doc {
            Some(doc) => {
                // Existing group: check ACL for MODIFY permission
                if let Ok(acl) = parse_acl(doc) {
                    let principals = self.db.get_user_principals(user_id).await?;
                    log::debug!(
                        "[ACL] GroupController::can_write: principals={:?}",
                        principals
                    );
                    let result = acl.check_permission(&principals, Permissions::MODIFY);
                    log::debug!(
                        "[ACL] GroupController::can_write: check_permission(MODIFY)={}",
                        result
                    );
                    return Ok(result);
                }
                Ok(false)
            }
            None => {
                // New group: check usr_create_groups super-permission
                let has_perm = self
                    .db
                    .has_permission(user_id, super_permissions::USR_CREATE_GROUPS)
                    .await?;
                log::debug!(
                    "[ACL] GroupController::can_write: new group, has_permission(USR_CREATE_GROUPS)={}",
                    has_perm
                );
                Ok(has_perm)
            }
        }
    }

    fn to_internal(&self, mut body: Value, _auth: &Auth) -> Result<Value, AppError> {
        // Validate and auto-prefix group ID
        if let Some(obj) = body.as_object_mut() {
            if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                // Validate (strips g_ prefix if present, validates, returns without prefix)
                let validated_id = validate_group_id(id)
                    .map_err(AppError::Validation)?;

                // Add g_ prefix
                let prefixed_id = format!("g_{}", validated_id);
                obj.insert("id".to_string(), Value::String(prefixed_id));
            }
        }

        Ok(standard_to_internal(body))
    }

    fn to_external(&self, doc: Value) -> Value {
        standard_to_external(doc)
    }

    fn to_list_external(&self, doc: Value) -> Value {
        let doc = self.to_external(doc);
        filter_to_brief(doc, Group::brief_field_names())
    }

    fn list_projection_fields(&self) -> Option<&'static [&'static str]> {
        // _key → "id" after to_external; "acl" needed by can_read for ACL checks
        Some(&["_key", "name", "acl", "meta"])
    }

    fn prepare_create(&self, body: &mut Value, user_id: &str) {
        log::debug!(
            "[ACL] GroupController::prepare_create: user={}",
            user_id
        );
        let Some(obj) = body.as_object_mut() else {
            return;
        };

        // Populate meta (created_at, created_by) if not already set
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
        let acl = obj
            .entry("acl")
            .or_insert_with(|| {
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
                "[ACL] GroupController::prepare_create: added ROOT entry for user",
            );
        }
    }

    async fn after_create(&self, key: &str, user_id: &str, db: &ArangoDb) -> Result<(), AppError> {
        log::debug!(
            "[LIFECYCLE] GroupController::after_create: group={}, creator={}",
            key, user_id
        );

        // Insert creator as a member of the new group
        db.add_principal_to_group(user_id, key, None).await?;
        log::debug!(
            "[LIFECYCLE] GroupController::after_create: added creator {} as member of group {}",
            user_id, key
        );

        Ok(())
    }

    async fn after_delete(&self, key: &str, db: &ArangoDb) -> Result<(), AppError> {
        log::debug!(
            "[LIFECYCLE] GroupController::after_delete: group={}",
            key
        );

        let empty_parents = Self::cleanup_group_references(db, key).await?;

        // Recursively cascade for any parent groups that became empty
        for parent_id in empty_parents {
            log::debug!(
                "[LIFECYCLE] GroupController::after_delete: parent group {} is now empty, cascading",
                parent_id
            );
            Self::cascade_delete_group(db, &parent_id).await?;
        }

        Ok(())
    }

    async fn after_update(&self, key: &str, db: &ArangoDb) -> Result<(), AppError> {
        // Check if the group is now empty (zero members) and delete if so
        let count = db.count_group_members(key).await?;
        log::debug!(
            "[LIFECYCLE] GroupController::after_update: group={}, member_count={}",
            key, count
        );
        if count == 0 {
            log::debug!(
                "[LIFECYCLE] GroupController::after_update: group {} is empty, deleting",
                key
            );
            Self::cascade_delete_group(db, key).await?;
        }
        Ok(())
    }
}
