use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use crate::db::ArangoDb;
use crate::error::AppError;
use crate::middleware::auth::Auth;
use crit_shared::models::{Permissions, super_permissions};

use super::gitops_controller::{
    KindController, parse_acl, standard_to_external, standard_to_internal,
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
        let is_admin = self
            .db
            .has_permission(user_id, super_permissions::ADM_PROJECT_MANAGER)
            .await?;
        log::debug!(
            "[ACL] ProjectController::can_read: is_admin(ADM_PROJECT_MANAGER)={}",
            is_admin
        );
        if is_admin {
            return Ok(true);
        }

        if let Some(doc) = doc {
            if let Ok(acl) = parse_acl(doc) {
                let principals = self.db.get_user_principals(user_id).await?;
                log::debug!(
                    "[ACL] ProjectController::can_read: principals={:?}",
                    principals
                );
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
        let is_admin = self
            .db
            .has_permission(user_id, super_permissions::ADM_PROJECT_MANAGER)
            .await?;
        log::debug!(
            "[ACL] ProjectController::can_write: is_admin(ADM_PROJECT_MANAGER)={}",
            is_admin
        );
        if is_admin {
            return Ok(true);
        }

        match doc {
            Some(doc) => {
                if let Ok(acl) = parse_acl(doc) {
                    let principals = self.db.get_user_principals(user_id).await?;
                    log::debug!(
                        "[ACL] ProjectController::can_write: principals={:?}",
                        principals
                    );
                    let result = acl.check_permission(&principals, Permissions::WRITE);
                    log::debug!(
                        "[ACL] ProjectController::can_write: check_permission(WRITE)={}",
                        result
                    );
                    return Ok(result);
                }
                Ok(false)
            }
            None => {
                let has_perm = self
                    .db
                    .has_permission(user_id, super_permissions::USR_CREATE_PROJECTS)
                    .await?;
                log::debug!(
                    "[ACL] ProjectController::can_write: new project, has_permission(USR_CREATE_PROJECTS)={}",
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

    fn prepare_create(&self, body: &mut Value, user_id: &str) {
        log::debug!(
            "[ACL] ProjectController::prepare_create: user={}",
            user_id
        );
        let Some(obj) = body.as_object_mut() else {
            return;
        };

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
                "[ACL] ProjectController::prepare_create: added ROOT entry for user",
            );
        }
    }
}
