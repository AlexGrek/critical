use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::db::ArangoDb;
use crate::error::AppError;
use crate::middleware::auth::Auth;
use crit_shared::crd_models::CustomResourceDefinition;
use crit_shared::util_models::super_permissions;

use super::gitops_controller::{
    KindController, inject_create_defaults, standard_to_external, standard_to_internal,
    strip_unknown_fields,
};

/// Top-level fields accepted by the CRD API.
const CRD_ALLOWED_FIELDS: &[&str] = &[
    "id",
    "_key",
    "name",
    "scope",
    "acl_mode",
    "nouns",
    "relations",
    "fields",
    "id_prefix",
    "super_permission",
    "description",
    "labels",
    "annotations",
    "state",
    "deletion",
    "hash_code",
];

pub struct CrdController {
    pub db: Arc<ArangoDb>,
}

impl CrdController {
    pub fn new(db: Arc<ArangoDb>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl KindController for CrdController {
    /// Any authenticated user can read CRDs.
    async fn can_read(&self, _user_id: &str, _doc: Option<&Value>) -> Result<bool, AppError> {
        log::trace!("[ACL] CrdController::can_read: allowed for any authenticated user");
        Ok(true)
    }

    /// Only godmode users can write CRDs.
    async fn can_write(&self, user_id: &str, _doc: Option<&Value>) -> Result<bool, AppError> {
        let principals = self.db.get_user_principals(user_id).await?;
        let has_godmode = self
            .db
            .has_permission_with_principals(&principals, super_permissions::ADM_GODMODE)
            .await?;
        log::debug!(
            "[ACL] CrdController::can_write: user={} godmode={}",
            user_id,
            has_godmode
        );
        Ok(has_godmode)
    }

    fn to_internal(&self, mut body: Value, _auth: &Auth) -> Result<Value, AppError> {
        strip_unknown_fields(&mut body, CRD_ALLOWED_FIELDS);
        // Validate and lowercase the CRD name (used as the collection name and API path segment)
        if let Some(obj) = body.as_object_mut() {
            if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                let validated_id =
                    CustomResourceDefinition::validate_name(id).map_err(AppError::Validation)?;
                obj.insert("id".to_string(), Value::String(validated_id));
            }
        }
        Ok(standard_to_internal(body))
    }

    fn to_external(&self, doc: Value) -> Value {
        standard_to_external(doc)
    }

    fn super_permission(&self) -> Option<&str> {
        Some(super_permissions::ADM_GODMODE)
    }

    fn prepare_create(&self, body: &mut Value, user_id: &str) {
        log::debug!("[ACL] CrdController::prepare_create: user={}", user_id);
        inject_create_defaults(body, user_id);
    }
}
