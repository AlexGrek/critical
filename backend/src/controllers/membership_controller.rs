use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::db::ArangoDb;
use crate::error::AppError;
use crate::middleware::auth::Auth;
use crit_shared::util_models::{Permissions, super_permissions};

use super::gitops_controller::{
    KindController, parse_acl, standard_to_external, standard_to_internal,
};
use super::group_controller::GroupController;

pub struct MembershipController {
    pub db: Arc<ArangoDb>,
}

impl MembershipController {
    pub fn new(db: Arc<ArangoDb>) -> Self {
        Self { db }
    }

    /// Check if a user has MODIFY permission on a group (via admin or group ACL).
    async fn can_modify_group(&self, user_id: &str, group_id: &str) -> Result<bool, AppError> {
        // ADM_USER_MANAGER bypasses group ACL
        let is_admin = self
            .db
            .has_permission(user_id, super_permissions::ADM_USER_MANAGER)
            .await?;
        log::debug!(
            "[ACL] MembershipController::can_modify_group: is_admin(ADM_USER_MANAGER)={}",
            is_admin
        );
        if is_admin {
            return Ok(true);
        }

        // Fetch the group document and check its ACL for MODIFY
        let group_doc = self.db.generic_get("groups", group_id).await?;
        if let Some(doc) = group_doc {
            if let Ok(acl) = parse_acl(&doc) {
                let principals = self.db.get_user_principals(user_id).await?;
                log::debug!(
                    "[ACL] MembershipController::can_modify_group: principals={:?}",
                    principals
                );
                let result = acl.check_permission(&principals, Permissions::MODIFY);
                log::debug!(
                    "[ACL] MembershipController::can_modify_group: group={}, MODIFY={}",
                    group_id, result
                );
                return Ok(result);
            }
        }

        Ok(false)
    }

    /// Extract the group ID from a membership document or request body.
    fn extract_group_id(doc: &Value) -> Option<String> {
        doc.get("group")
            .and_then(|v| v.as_str())
            .map(String::from)
    }
}

#[async_trait]
impl KindController for MembershipController {
    async fn can_read(&self, user_id: &str, doc: Option<&Value>) -> Result<bool, AppError> {
        // ADM_USER_MANAGER can read all memberships
        let is_admin = self
            .db
            .has_permission(user_id, super_permissions::ADM_USER_MANAGER)
            .await?;
        if is_admin {
            return Ok(true);
        }

        // Check if user has READ on the target group
        if let Some(doc) = doc {
            if let Some(group_id) = Self::extract_group_id(doc) {
                let group_doc = self.db.generic_get("groups", &group_id).await?;
                if let Some(gdoc) = group_doc {
                    if let Ok(acl) = parse_acl(&gdoc) {
                        let principals = self.db.get_user_principals(user_id).await?;
                        let result = acl.check_permission(&principals, Permissions::READ);
                        log::debug!(
                            "[ACL] MembershipController::can_read: group={}, READ={}",
                            group_id, result
                        );
                        return Ok(result);
                    }
                }
            }
        }

        Ok(false)
    }

    async fn can_write(&self, user_id: &str, doc: Option<&Value>) -> Result<bool, AppError> {
        // For updates/deletes on existing memberships, check the target group's ACL
        if let Some(doc) = doc {
            if let Some(group_id) = Self::extract_group_id(doc) {
                return self.can_modify_group(user_id, &group_id).await;
            }
        }

        // For creates where doc is None, deny — use can_create which has the body
        Ok(false)
    }

    async fn can_create(&self, user_id: &str, body: &Value) -> Result<bool, AppError> {
        // Extract the target group from the request body and check MODIFY permission
        if let Some(group_id) = Self::extract_group_id(body) {
            return self.can_modify_group(user_id, &group_id).await;
        }

        log::debug!(
            "[ACL] MembershipController::can_create: no group field in body, denying"
        );
        Ok(false)
    }

    fn to_internal(&self, body: Value, _auth: &Auth) -> Result<Value, AppError> {
        Ok(standard_to_internal(body))
    }

    fn to_external(&self, doc: Value) -> Value {
        standard_to_external(doc)
    }

    async fn after_delete(&self, key: &str, db: &ArangoDb) -> Result<(), AppError> {
        // The key format is "{principal}::{group}"
        // After a membership is deleted, check if the group is now empty
        let parts: Vec<&str> = key.splitn(2, "::").collect();
        if parts.len() != 2 {
            return Ok(());
        }
        let group_id = parts[1];

        let count = db.count_group_members(group_id).await?;
        log::debug!(
            "[LIFECYCLE] MembershipController::after_delete: group={}, member_count={}",
            group_id, count
        );

        if count == 0 {
            log::debug!(
                "[LIFECYCLE] MembershipController::after_delete: group {} is empty, deleting",
                group_id
            );
            GroupController::cascade_delete_group(db, group_id).await?;
        }

        Ok(())
    }
}
