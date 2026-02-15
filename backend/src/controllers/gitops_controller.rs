use std::sync::Arc;

use serde_json::{Value, json};

use crate::db::ArangoDb;
use crate::error::AppError;
use crate::middleware::auth::Auth;
use crit_shared::models::{AccessControlStore, Permissions, super_permissions};

pub struct GitopsController {
    pub db: Arc<ArangoDb>,
}

impl GitopsController {
    pub fn new(db: Arc<ArangoDb>) -> Self {
        Self { db }
    }

    /// Check if a user can read a document of the given kind.
    pub async fn can_read(&self, user_id: &str, kind: &str, doc: Option<&Value>) -> Result<bool, AppError> {
        log::debug!("[ACL] can_read: user={}, kind={}, has_doc={}", user_id, kind, doc.is_some());
        match kind {
            "users" | "groups" => {
                log::debug!("[ACL] can_read: users/groups -> allow all");
                Ok(true)
            }
            "projects" => {
                let is_admin = self.db.has_permission(user_id, super_permissions::ADM_PROJECT_MANAGER).await?;
                log::debug!("[ACL] can_read: projects, is_admin(ADM_PROJECT_MANAGER)={}", is_admin);
                if is_admin {
                    return Ok(true);
                }
                if let Some(doc) = doc {
                    log::debug!("[ACL] can_read: checking doc ACL, doc keys: {:?}", doc.as_object().map(|o| o.keys().collect::<Vec<_>>()));
                    match Self::parse_acl(doc) {
                        Ok(acl) => {
                            log::debug!("[ACL] can_read: parsed ACL, {} entries", acl.list.len());
                            for (i, entry) in acl.list.iter().enumerate() {
                                log::debug!("[ACL] can_read: ACL[{}] permissions={:?}, principals={:?}", i, entry.permissions, entry.principals);
                            }
                            let principals = self.db.get_user_principals(user_id).await?;
                            log::debug!("[ACL] can_read: user principals={:?}", principals);
                            let result = acl.check_permission(&principals, Permissions::READ);
                            log::debug!("[ACL] can_read: check_permission(READ) = {}", result);
                            return Ok(result);
                        }
                        Err(_) => {
                            log::debug!("[ACL] can_read: failed to parse ACL from doc");
                        }
                    }
                }
                log::debug!("[ACL] can_read: projects -> deny");
                Ok(false)
            }
            _ => {
                log::debug!("[ACL] can_read: other kind '{}' -> allow all", kind);
                Ok(true)
            }
        }
    }

    /// Check if a user can write (create/update/delete) a document of the given kind.
    pub async fn can_write(&self, user_id: &str, kind: &str, doc: Option<&Value>) -> Result<bool, AppError> {
        log::debug!("[ACL] can_write: user={}, kind={}, has_doc={}", user_id, kind, doc.is_some());
        match kind {
            "users" | "groups" => {
                let has_perm = self.db.has_permission(user_id, super_permissions::ADM_USER_MANAGER).await?;
                log::debug!("[ACL] can_write: users/groups, has_permission(ADM_USER_MANAGER)={}", has_perm);
                Ok(has_perm)
            }
            "projects" => {
                let is_admin = self.db.has_permission(user_id, super_permissions::ADM_PROJECT_MANAGER).await?;
                log::debug!("[ACL] can_write: projects, is_admin(ADM_PROJECT_MANAGER)={}", is_admin);
                if is_admin {
                    return Ok(true);
                }
                match doc {
                    Some(doc) => {
                        log::debug!("[ACL] can_write: checking existing doc ACL");
                        match Self::parse_acl(doc) {
                            Ok(acl) => {
                                log::debug!("[ACL] can_write: parsed ACL, {} entries", acl.list.len());
                                let principals = self.db.get_user_principals(user_id).await?;
                                log::debug!("[ACL] can_write: user principals={:?}", principals);
                                let result = acl.check_permission(&principals, Permissions::WRITE);
                                log::debug!("[ACL] can_write: check_permission(WRITE) = {}", result);
                                return Ok(result);
                            }
                            Err(_) => {
                                log::debug!("[ACL] can_write: failed to parse ACL from doc");
                            }
                        }
                        Ok(false)
                    }
                    None => {
                        let has_perm = self.db.has_permission(user_id, super_permissions::USR_CREATE_PROJECTS).await?;
                        log::debug!("[ACL] can_write: new project, has_permission(USR_CREATE_PROJECTS)={}", has_perm);
                        Ok(has_perm)
                    }
                }
            }
            _ => {
                log::debug!("[ACL] can_write: other kind '{}' -> allow all", kind);
                Ok(true)
            }
        }
    }

    /// Ensure the creating user has ROOT access in the project's ACL.
    /// If the user is already present in any ACL entry, this is a no-op.
    pub fn ensure_creator_in_acl(body: &mut Value, user_id: &str) {
        log::debug!("[ACL] ensure_creator_in_acl: user={}", user_id);
        let Some(obj) = body.as_object_mut() else {
            log::debug!("[ACL] ensure_creator_in_acl: body is not an object, skipping");
            return;
        };

        let acl = obj
            .entry("acl")
            .or_insert_with(|| json!({"list": [], "last_mod_date": chrono::Utc::now().to_rfc3339()}));

        let Some(acl_obj) = acl.as_object_mut() else {
            log::debug!("[ACL] ensure_creator_in_acl: acl is not an object, skipping");
            return;
        };
        let list = acl_obj
            .entry("list")
            .or_insert_with(|| json!([]));

        let Some(list_arr) = list.as_array_mut() else {
            log::debug!("[ACL] ensure_creator_in_acl: list is not an array, skipping");
            return;
        };

        let already_present = list_arr.iter().any(|entry| {
            entry
                .get("principals")
                .and_then(|p| p.as_array())
                .is_some_and(|principals| principals.iter().any(|p| p.as_str() == Some(user_id)))
        });

        log::debug!("[ACL] ensure_creator_in_acl: already_present={}, list_len_before={}", already_present, list_arr.len());

        if !already_present {
            list_arr.push(json!({
                "permissions": Permissions::ROOT.bits(),
                "principals": [user_id],
            }));
            log::debug!("[ACL] ensure_creator_in_acl: added ROOT entry, list_len_after={}", list_arr.len());
        }
    }

    /// Convert an external gitops request body to an internal ArangoDB document.
    /// Renames `id` → `_key`, hashes password for users.
    pub fn to_internal(kind: &str, mut body: Value, auth: &Auth) -> Result<Value, AppError> {
        if let Some(obj) = body.as_object_mut() {
            if let Some(id) = obj.remove("id") {
                obj.insert("_key".to_string(), id);
            }
            if kind == "users" {
                if let Some(password) = obj.remove("password") {
                    if let Some(pw_str) = password.as_str() {
                        let hash = auth.hash_password(pw_str)?;
                        obj.insert("password_hash".to_string(), Value::String(hash));
                    }
                }
            }
        }
        Ok(body)
    }

    /// Convert an internal ArangoDB document to the external gitops representation.
    /// Renames `_key` → `id`, strips ArangoDB internal fields and sensitive data.
    pub fn to_external(kind: &str, mut doc: Value) -> Value {
        if let Some(obj) = doc.as_object_mut() {
            if let Some(key) = obj.remove("_key") {
                obj.insert("id".to_string(), key);
            }
            obj.remove("_id");
            obj.remove("_rev");
            if kind == "users" {
                obj.remove("password_hash");
            }
        }
        doc
    }

    /// Parse ACL from a raw JSON document.
    /// Handles both numeric permissions (from gitops API) and string flags (from Rust serde).
    fn parse_acl(doc: &Value) -> Result<AccessControlStore, ()> {
        let acl_val = doc.get("acl").ok_or(())?;
        let acl_obj = acl_val.as_object().ok_or(())?;

        let list = acl_obj
            .get("list")
            .and_then(|v| v.as_array())
            .ok_or(())?;

        let mut entries = Vec::new();
        for entry in list {
            let permissions = match entry.get("permissions") {
                Some(Value::Number(n)) => {
                    let bits = n.as_u64().ok_or(())? as u8;
                    Permissions::from_bits(bits).ok_or(())?
                }
                Some(Value::String(_)) => {
                    // Bitflags 2.x serde format: "FETCH | LIST | NOTIFY"
                    serde_json::from_value::<Permissions>(entry.get("permissions").cloned().unwrap()).map_err(|_| ())?
                }
                _ => return Err(()),
            };

            let principals = entry
                .get("principals")
                .and_then(|v| v.as_array())
                .ok_or(())?
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();

            entries.push(crit_shared::models::AccessControlList {
                permissions,
                principals,
            });
        }

        let last_mod_date = acl_obj
            .get("last_mod_date")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_default();

        Ok(AccessControlStore {
            list: entries,
            last_mod_date,
        })
    }
}
