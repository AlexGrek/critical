use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};

use crit_shared::data_models::RepoCredential;
use crit_shared::event_models::{EventKind, EventPriority};
use serde_json::json;

use crate::{
    controllers::gitops_controller::KindController,
    error::AppError,
    middleware::auth::AuthenticatedUser,
    services::git::{ProbeOutcome, probe_repo},
    state::AppState,
};

/// Probe a repository link for `pipelines.js` on its configured (or default)
/// branch, without saving anything. Used by the frontend both to validate an
/// unsaved repo entry in the add/edit form and to re-check an already-saved one.
///
/// `POST /v1/global/{kind}/{id}/repocheck` — only `kind == "projects"` is
/// supported today; other kinds 404. Mirrors the `/upload/{upload_type}`
/// route shape since axum can't mix a literal `projects` segment with the
/// `{kind}` param already used at that position by the generic gitops routes.
///
/// The body is a full `RepoLink` (not an index into the project's saved
/// list) so the same endpoint serves both cases. Requires MODIFY on the
/// project; if a credential is referenced, also requires READ on it. Both
/// checks return 404 on denial, matching the rest of the ACL model.
///
/// Always `200` once the probe actually runs, even if the file is missing or
/// the connection fails — those are reported as `status: "missing"` /
/// `"error"` in the body. `4xx` is reserved for request-shape problems:
/// unknown project, denied access, or an unparseable URL.
pub async fn check_project_repository(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((kind, project_id)): Path<(String, String)>,
    Json(link): Json<crit_shared::data_models::RepoLink>,
) -> Result<Json<ProbeOutcome>, AppError> {
    if kind != "projects" {
        return Err(AppError::not_found("repository check not supported for this resource kind"));
    }

    check_project_write_access(&state, &user_id, &project_id).await?;

    let credential = match &link.credential {
        Some(cred_id) => Some(fetch_readable_credential(&state, &user_id, cred_id).await?),
        None => None,
    };

    let outcome = probe_repo(&link, credential.as_ref()).await?;

    state
        .events
        .log(
            EventPriority::Note,
            EventKind::EntityManagement,
            Some(&user_id),
            vec![format!("projects/{project_id}")],
            Some(json!({ "action": "repo_check", "url": link.url, "status": outcome.status })),
        )
        .await;

    Ok(Json(outcome))
}

/// Same access rule as a project PUT: godmode, or MODIFY on the project's ACL.
/// 404 on both "project doesn't exist" and "caller can't write it".
async fn check_project_write_access(state: &AppState, user_id: &str, project_id: &str) -> Result<(), AppError> {
    if state.has_godmode(user_id).await.unwrap_or(false) {
        return Ok(());
    }

    let doc = state
        .db
        .generic_get("projects", project_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("projects/{project_id}")))?;

    let allowed = state.controller.project.can_write(user_id, Some(&doc)).await?;
    if !allowed {
        return Err(AppError::not_found(format!("projects/{project_id}")));
    }
    Ok(())
}

/// Fetch a `repo_credentials` document the caller may read. 404 if it
/// doesn't exist or the caller lacks READ — this is load-bearing: without it,
/// anyone who can edit a project could attach a credential ID they don't own
/// and have the server use its secret against a URL of their choosing.
async fn fetch_readable_credential(
    state: &AppState,
    user_id: &str,
    credential_id: &str,
) -> Result<RepoCredential, AppError> {
    let doc = state
        .db
        .generic_get("repo_credentials", credential_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("repo_credentials/{credential_id}")))?;

    let godmode = state.has_godmode(user_id).await.unwrap_or(false);
    if !godmode && !state.controller.repo_credential.can_read(user_id, Some(&doc)).await? {
        return Err(AppError::not_found(format!("repo_credentials/{credential_id}")));
    }

    serde_json::from_value(doc)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt repo_credentials document: {e}")))
}
