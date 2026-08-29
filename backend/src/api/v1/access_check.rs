use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use serde_json::{Value, json};

use crit_shared::util_models::{Permissions, super_permissions};

use crate::{
    controllers::gitops_controller::parse_acl,
    error::AppError,
    middleware::auth::AuthenticatedUser,
    state::AppState,
};

/// Check what permissions the authenticated user holds on a global resource.
///
/// `GET /v1/accesscheck/global/{kind}/{id}`
///
/// Returns effective permission bits (as a bitmask and named flags) if the user
/// has at least FETCH access. Returns 404 if the resource doesn't exist **or** if
/// the user has no FETCH permission — deliberately indistinguishable to avoid
/// leaking resource existence to unauthorised callers.
///
/// Godmode users always receive ROOT permissions (all bits set).
pub async fn check_global_access(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((kind, id)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    access_check(&state, &user_id, &kind, &id, None).await
}

/// Check what permissions the authenticated user holds on a project-scoped resource.
///
/// `GET /v1/accesscheck/projects/{project}/{kind}/{id}`
///
/// When the resource's own ACL is empty the project's full ACL is used as a
/// fallback (same rule as the gitops read handlers). Returns 404 when the resource
/// doesn't exist or the user has no FETCH permission.
pub async fn check_scoped_access(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((project, kind, id)): Path<(String, String, String)>,
) -> Result<Json<Value>, AppError> {
    access_check(&state, &user_id, &kind, &id, Some(&project)).await
}

/// Return the calling user's own super-permissions.
///
/// `GET /v1/accesscheck/me/permissions`
///
/// Used by the UI to decide whether to show "create group", "create project",
/// and similar action buttons that are gated on super-permissions rather than
/// per-resource ACLs. Always returns 200 for authenticated users.
pub async fn get_my_permissions(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Json<Value>, AppError> {
    let perms = state
        .db
        .get_user_permissions(&user_id)
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(json!({ "super_permissions": perms })))
}

/// Return a detailed report of the calling user's own access: super-permissions,
/// the full resolved principal chain (self + transitive groups), direct group
/// memberships, and every group/project whose ACL grants them something, along
/// with which principal and permission bits earned that grant.
///
/// `GET /v1/accesscheck/me/acls`
///
/// This reports document-level ACL grants only — it does not fold in super-permission
/// bypasses (e.g. a godmode user may see empty `groups`/`projects` here despite having
/// full access via `is_godmode`/`super_permissions`). Always returns 200 for
/// authenticated users.
pub async fn get_my_acls(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Json<Value>, AppError> {
    let principals = state
        .get_cached_principals(&user_id)
        .await
        .map_err(AppError::Internal)?;

    let super_perms = state
        .db
        .get_user_permissions(&user_id)
        .await
        .map_err(AppError::Internal)?;

    let is_godmode = super_perms
        .iter()
        .any(|p| p == super_permissions::ADM_GODMODE);

    let direct_memberships = state
        .db
        .get_direct_memberships(&user_id)
        .await
        .map_err(AppError::Internal)?;

    let group_grants = state
        .db
        .list_acl_grants("groups", &principals)
        .await
        .map_err(AppError::Internal)?;

    let project_grants = state
        .db
        .list_acl_grants("projects", &principals)
        .await
        .map_err(AppError::Internal)?;

    Ok(Json(json!({
        "user_id": user_id,
        "is_godmode": is_godmode,
        "super_permissions": super_perms,
        "principals": principals,
        "direct_memberships": direct_memberships,
        "groups": shape_grants(group_grants),
        "projects": shape_grants(project_grants),
    })))
}

// ─────────────────────────── internal helper ────────────────────────────────

/// Reshape raw `list_acl_grants` rows into a UI-friendly form: permission bits
/// resolved to named flags, split into the document-wide grant (unscoped entries)
/// and any scope-restricted grants (e.g. project ACL entries scoped to `"tasks"`).
fn shape_grants(raw: Vec<Value>) -> Vec<Value> {
    raw.into_iter()
        .map(|doc| {
            let id = doc.get("id").and_then(|v| v.as_str()).unwrap_or_default();
            let name = doc.get("name").and_then(|v| v.as_str());
            let grants = doc.get("grants").and_then(|v| v.as_array()).cloned().unwrap_or_default();

            let mut unscoped_bits = Permissions::NONE;
            let mut via = std::collections::BTreeSet::new();
            let mut scoped_grants = Vec::new();

            for grant in &grants {
                let bits = grant
                    .get("permissions")
                    .and_then(|v| v.as_u64())
                    .and_then(|b| Permissions::from_bits(b as u8))
                    .unwrap_or(Permissions::NONE);
                let scope = grant.get("scope").and_then(|v| v.as_str());
                let entry_via: Vec<String> = grant
                    .get("via")
                    .and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();

                match scope {
                    None | Some("*") => {
                        unscoped_bits |= bits;
                        via.extend(entry_via);
                    }
                    Some(s) => {
                        scoped_grants.push(json!({
                            "scope": s,
                            "permission": permission_flags(bits),
                            "via_principals": entry_via,
                        }));
                    }
                }
            }

            json!({
                "id": id,
                "name": name,
                "permission": permission_flags(unscoped_bits),
                "via_principals": via.into_iter().collect::<Vec<_>>(),
                "scoped_grants": scoped_grants,
            })
        })
        .collect()
}

fn permission_flags(bits: Permissions) -> Value {
    json!({
        "bits": bits.bits(),
        "can_fetch": bits.contains(Permissions::FETCH),
        "can_list": bits.contains(Permissions::LIST),
        "can_notify": bits.contains(Permissions::NOTIFY),
        "can_create": bits.contains(Permissions::CREATE),
        "can_modify": bits.contains(Permissions::MODIFY),
    })
}

async fn access_check(
    state: &AppState,
    user_id: &str,
    kind: &str,
    id: &str,
    project: Option<&str>,
) -> Result<Json<Value>, AppError> {
    let principals = state
        .get_cached_principals(user_id)
        .await
        .map_err(AppError::Internal)?;

    let super_perms = state
        .db
        .get_user_permissions(user_id)
        .await
        .map_err(AppError::Internal)?;

    let is_godmode = super_perms
        .iter()
        .any(|p| p == super_permissions::ADM_GODMODE);

    let doc = state
        .db
        .generic_get(kind, id)
        .await
        .map_err(AppError::Internal)?;

    // Resource missing — always 404, even for godmode, to keep semantics clean.
    let doc = doc.ok_or_else(|| AppError::not_found("not found"))?;

    if is_godmode {
        log::debug!(
            "[ACCESSCHECK] {} (godmode) → {}/{}: ROOT",
            user_id, kind, id
        );
        return Ok(build_response(kind, id, Permissions::ROOT));
    }

    // Deserialise the resource's ACL using the shared helper which handles both
    // integer-encoded permissions (from `.bits()` calls) and serde string format.
    let resource_acl = parse_acl(&doc).unwrap_or_default();

    // For project-scoped resources with an empty ACL, fall back to the project's ACL.
    let effective_acl = if resource_acl.list.is_empty() {
        if let Some(proj_id) = project {
            state
                .db
                .generic_get("projects", proj_id)
                .await
                .map_err(AppError::Internal)?
                .and_then(|proj| parse_acl(&proj).ok())
                .unwrap_or_default()
        } else {
            resource_acl
        }
    } else {
        resource_acl
    };

    // Accumulate effective bits: OR all ACL entries whose principals overlap ours.
    let effective_bits = effective_acl
        .list
        .iter()
        .filter(|entry| entry.principals.iter().any(|p| principals.contains(p)))
        .fold(Permissions::NONE, |acc, entry| acc | entry.permissions);

    log::debug!(
        "[ACCESSCHECK] {} → {}/{}: bits=0x{:02x}",
        user_id, kind, id, effective_bits.bits()
    );

    // 404 when user lacks FETCH — don't reveal that the resource exists.
    if !effective_bits.contains(Permissions::FETCH) {
        return Err(AppError::not_found("not found"));
    }

    Ok(build_response(kind, id, effective_bits))
}

fn build_response(kind: &str, id: &str, bits: Permissions) -> Json<Value> {
    Json(json!({
        "kind": kind,
        "id": id,
        "permission_bits": bits.bits(),
        "can_fetch":  bits.contains(Permissions::FETCH),
        "can_list":   bits.contains(Permissions::LIST),
        "can_notify": bits.contains(Permissions::NOTIFY),
        "can_create": bits.contains(Permissions::CREATE),
        "can_modify": bits.contains(Permissions::MODIFY),
    }))
}
