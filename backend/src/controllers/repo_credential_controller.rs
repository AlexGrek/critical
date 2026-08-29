use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use crate::db::ArangoDb;
use crate::error::AppError;
use crate::middleware::auth::Auth;
use crate::validation::naming::validate_repo_credential_id;
use crit_shared::data_models::RepoCredential;
use crit_shared::util_models::{Permissions, super_permissions};

use super::gitops_controller::{
    KindController, filter_to_brief, inject_create_defaults, parse_acl, standard_to_external,
    standard_to_internal, strip_unknown_fields,
};

/// Top-level fields accepted by the repo-credential API.
/// `secret`/`passphrase` are accepted here but stripped again in `to_external` —
/// they are write-only, mirroring `password` on `User`.
const REPO_CREDENTIAL_ALLOWED_FIELDS: &[&str] = &[
    "id", "_key",
    "name", "method", "description", "username", "secret", "passphrase",
    "labels", "annotations", "acl", "state", "deletion", "hash_code",
];

pub struct RepoCredentialController {
    pub db: Arc<ArangoDb>,
}

impl RepoCredentialController {
    pub fn new(db: Arc<ArangoDb>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl KindController for RepoCredentialController {
    async fn can_read(&self, user_id: &str, doc: Option<&Value>) -> Result<bool, AppError> {
        let principals = self.db.get_user_principals(user_id).await?;

        if self
            .db
            .has_permission_with_principals(&principals, super_permissions::ADM_CONFIG_EDITOR)
            .await?
        {
            return Ok(true);
        }

        if let Some(doc) = doc {
            if let Ok(acl) = parse_acl(doc) {
                return Ok(acl.check_permission(&principals, Permissions::READ));
            }
        }
        Ok(false)
    }

    async fn can_write(&self, user_id: &str, doc: Option<&Value>) -> Result<bool, AppError> {
        let principals = self.db.get_user_principals(user_id).await?;

        if self
            .db
            .has_permission_with_principals(&principals, super_permissions::ADM_CONFIG_EDITOR)
            .await?
        {
            return Ok(true);
        }

        match doc {
            Some(doc) => {
                if let Ok(acl) = parse_acl(doc) {
                    return Ok(acl.check_permission(&principals, Permissions::MODIFY));
                }
                Ok(false)
            }
            None => {
                let has_perm = self
                    .db
                    .has_permission_with_principals(
                        &principals,
                        super_permissions::USR_CREATE_PROJECTS,
                    )
                    .await?;
                Ok(has_perm)
            }
        }
    }

    fn to_internal(&self, mut body: Value, _auth: &Auth) -> Result<Value, AppError> {
        strip_unknown_fields(&mut body, REPO_CREDENTIAL_ALLOWED_FIELDS);
        if let Some(obj) = body.as_object_mut() {
            if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                let validated_id = validate_repo_credential_id(id).map_err(AppError::Validation)?;
                obj.insert("id".to_string(), Value::String(format!("rc_{validated_id}")));
            }
        }
        Ok(standard_to_internal(body))
    }

    fn to_external(&self, mut doc: Value) -> Value {
        if let Some(obj) = doc.as_object_mut() {
            let has_secret = obj
                .get("secret")
                .and_then(|v| v.as_str())
                .is_some_and(|s| !s.is_empty());
            obj.remove("secret");
            obj.remove("passphrase");
            obj.insert("has_secret".to_string(), json!(has_secret));
        }
        standard_to_external(doc)
    }

    fn to_list_external(&self, doc: Value) -> Value {
        let doc = self.to_external(doc);
        filter_to_brief(doc, RepoCredential::brief_field_names())
    }

    fn list_projection_fields(&self) -> Option<&'static [&'static str]> {
        Some(&["_key", "name", "method", "acl", "labels"])
    }

    fn super_permission(&self) -> Option<&str> {
        Some(super_permissions::ADM_CONFIG_EDITOR)
    }

    fn prepare_create(&self, body: &mut Value, user_id: &str) {
        inject_create_defaults(body, user_id);

        let Some(obj) = body.as_object_mut() else {
            return;
        };

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
        }
    }
}
