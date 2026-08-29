//! Connectivity probe for project repository links.
//!
//! Fetches exactly one file (`pipelines.js`) from a repository's default
//! branch to verify the URL and credentials are usable. This is a pure
//! connectivity/authorization check — nothing is stored or executed.

use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::Serialize;

use crit_shared::data_models::{RepoAuthMethod, RepoLink, RepoProvider};

use crate::error::AppError;

use crate::services::github::GithubClient;

use super::url::{owner_repo, parse_repo_url};

/// The file every probe looks for. Hardcoded: this check is groundwork for
/// the (not yet implemented) pipelines engine, which will read this file.
const PROBE_PATH: &str = "pipelines.js";

/// Bounds both the git and GitHub-API probe paths so a slow/unreachable host
/// can't hang the request indefinitely.
const PROBE_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeStatus {
    Found,
    Missing,
    Error,
}

#[derive(Debug, Serialize)]
pub struct ProbeOutcome {
    pub status: ProbeStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    pub message: String,
}

impl ProbeOutcome {
    fn error(message: impl Into<String>) -> Self {
        Self {
            status: ProbeStatus::Error,
            branch: None,
            size: None,
            message: message.into(),
        }
    }
}

/// Probe a repository link for `pipelines.js` on its configured (or default)
/// branch. `secret` is the referenced credential's secret (SSH private key or
/// GitHub token) — must be `Some` when `link.auth_method` requires one. Only
/// the secret is needed here; the caller is responsible for the ACL check on
/// the full `RepoCredential` document before extracting it (see
/// `api/v1/repo_check.rs::fetch_readable_credential` — deliberately not
/// deserialized into the typed `RepoCredential` struct here, since raw DB
/// documents store `acl.list[].permissions` as an integer bitmask, not the
/// `|`-separated string form `AccessControlStore`'s derive expects).
///
/// Returns `Err` only for request-shape problems (unparseable URL, missing
/// required credential) — anything that happens while actually talking to
/// the repository (auth failure, timeout, file missing) is reported as an
/// `Ok(ProbeOutcome)` so callers can render it uniformly.
pub async fn probe_repo(link: &RepoLink, secret: Option<&str>) -> Result<ProbeOutcome, AppError> {
    // Validate the URL up front regardless of transport, so malformed input
    // is rejected the same way for every provider.
    parse_repo_url(&link.url)?;

    if link.provider == RepoProvider::Github {
        return probe_github(link, secret).await;
    }
    probe_git(link.clone(), secret.map(str::to_string)).await
}

// ---------------------------------------------------------------------------
// GitHub API path (octocrab)
// ---------------------------------------------------------------------------

async fn probe_github(link: &RepoLink, secret: Option<&str>) -> Result<ProbeOutcome, AppError> {
    let url = parse_repo_url(&link.url)?;
    let (owner, repo) =
        owner_repo(&url).ok_or_else(|| AppError::bad_request("could not determine owner/repo from repository URL"))?;

    let token = if link.auth_method == RepoAuthMethod::GithubToken {
        let token = secret
            .filter(|s| !s.is_empty())
            .ok_or_else(|| AppError::validation("github_token auth requires a credential with a token set"))?;
        Some(token.to_string())
    } else {
        None
    };

    let client = match GithubClient::new(token) {
        Ok(c) => c,
        Err(e) => return Ok(ProbeOutcome::error(format!("failed to build GitHub client: {e}"))),
    };

    let fut = async {
        let default_branch = client.default_branch(&owner, &repo).await;
        let branch = link
            .default_branch
            .clone()
            .or(default_branch.ok().flatten())
            .unwrap_or_else(|| "main".to_string());

        match client.get_file_size(&owner, &repo, &branch, PROBE_PATH).await {
            Ok(Some(size)) => ProbeOutcome {
                status: ProbeStatus::Found,
                branch: Some(branch.clone()),
                size: Some(size),
                message: format!("{PROBE_PATH} found on branch {branch}"),
            },
            Ok(None) => ProbeOutcome {
                status: ProbeStatus::Missing,
                branch: Some(branch.clone()),
                size: None,
                message: format!("{PROBE_PATH} not found on branch {branch}"),
            },
            Err(e) => ProbeOutcome::error(format!("GitHub API error: {e}")),
        }
    };

    match tokio::time::timeout(PROBE_TIMEOUT, fut).await {
        Ok(outcome) => Ok(outcome),
        Err(_) => Ok(ProbeOutcome::error(format!(
            "timed out after {}s talking to the GitHub API",
            PROBE_TIMEOUT.as_secs()
        ))),
    }
}

// ---------------------------------------------------------------------------
// Generic git path (gix) — SSH and anonymous HTTPS
// ---------------------------------------------------------------------------

async fn probe_git(link: RepoLink, secret: Option<String>) -> Result<ProbeOutcome, AppError> {
    if link.auth_method == RepoAuthMethod::Ssh {
        let has_secret = secret.as_deref().is_some_and(|s| !s.is_empty());
        if !has_secret {
            return Err(AppError::validation(
                "ssh auth requires a credential with a private key set",
            ));
        }
    }

    let should_interrupt = Arc::new(AtomicBool::new(false));
    let interrupt_for_task = should_interrupt.clone();
    let handle = tokio::task::spawn_blocking(move || run_git_probe(&link, secret.as_deref(), &interrupt_for_task));

    match tokio::time::timeout(PROBE_TIMEOUT, handle).await {
        Ok(Ok(outcome)) => Ok(outcome),
        Ok(Err(join_err)) => Ok(ProbeOutcome::error(format!("probe task failed: {join_err}"))),
        Err(_elapsed) => {
            should_interrupt.store(true, Ordering::SeqCst);
            Ok(ProbeOutcome::error(format!(
                "timed out after {}s connecting to the repository",
                PROBE_TIMEOUT.as_secs()
            )))
        }
    }
}

/// Blocking: shallow bare-clones the repo into a temp dir, then reads
/// `pipelines.js` out of the resulting tree. Runs on a `spawn_blocking`
/// thread — gix's network client is synchronous.
fn run_git_probe(link: &RepoLink, secret: Option<&str>, should_interrupt: &AtomicBool) -> ProbeOutcome {
    let tmp = match tempfile::tempdir() {
        Ok(t) => t,
        Err(e) => return ProbeOutcome::error(format!("failed to create temp dir: {e}")),
    };

    let mut prepare = match gix::prepare_clone_bare(link.url.as_str(), tmp.path()) {
        Ok(p) => p,
        Err(e) => return ProbeOutcome::error(format!("invalid repository URL: {e}")),
    };

    prepare = prepare.with_shallow(gix::remote::fetch::Shallow::DepthAtRemote(
        1.try_into().expect("1 is non-zero"),
    ));

    // Keep the key file alive for the duration of the fetch; it is deleted
    // (via TempDir/NamedTempFile drop) as soon as we leave this function.
    let mut _key_file_guard = None;
    if link.auth_method == RepoAuthMethod::Ssh {
        let key_file = match write_ssh_key(secret.unwrap_or_default()) {
            Ok(f) => f,
            Err(e) => return ProbeOutcome::error(format!("failed to stage SSH key: {e}")),
        };
        // Passed as an in-memory config override (not GIT_SSH_COMMAND) so it
        // can't leak into or race with other concurrent probes on this process.
        let ssh_cmd = format!(
            "ssh -i {} -o IdentitiesOnly=yes -o StrictHostKeyChecking=accept-new -o UserKnownHostsFile=/dev/null",
            key_file.path().display()
        );
        prepare = prepare.with_in_memory_config_overrides([format!("core.sshCommand={ssh_cmd}")]);
        _key_file_guard = Some(key_file);
    }

    if let Some(branch) = &link.default_branch {
        prepare = match prepare.with_ref_name(Some(branch.as_str())) {
            Ok(p) => p,
            Err(e) => return ProbeOutcome::error(format!("invalid branch name {branch:?}: {e}")),
        };
    }

    let mut progress = gix::progress::Discard;
    let repo = match prepare.fetch_only(&mut progress, should_interrupt) {
        Ok((repo, _outcome)) => repo,
        Err(e) => return ProbeOutcome::error(format!("failed to connect to repository: {e}")),
    };
    drop(_key_file_guard);

    let branch_label = link.default_branch.clone().unwrap_or_else(|| "HEAD".to_string());

    let commit = match resolve_commit(&repo, link.default_branch.as_deref()) {
        Ok(c) => c,
        Err(e) => return ProbeOutcome::error(format!("failed to resolve branch {branch_label:?}: {e}")),
    };

    let tree = match commit.tree() {
        Ok(t) => t,
        Err(e) => return ProbeOutcome::error(format!("failed to read commit tree: {e}")),
    };

    match tree.lookup_entry_by_path(PROBE_PATH) {
        Ok(Some(entry)) => {
            let size = entry.object().ok().map(|o| o.data.len() as u64);
            ProbeOutcome {
                status: ProbeStatus::Found,
                branch: Some(branch_label.clone()),
                size,
                message: format!("{PROBE_PATH} found on branch {branch_label}"),
            }
        }
        Ok(None) => ProbeOutcome {
            status: ProbeStatus::Missing,
            branch: Some(branch_label.clone()),
            size: None,
            message: format!("{PROBE_PATH} not found on branch {branch_label}"),
        },
        Err(e) => ProbeOutcome::error(format!("failed to read repository tree: {e}")),
    }
}

/// Resolve the commit for `branch` (checking the remote-tracking ref first,
/// then a couple of fallback ref shapes), or the fetched HEAD if no branch
/// was requested.
fn resolve_commit<'repo>(
    repo: &'repo gix::Repository,
    branch: Option<&str>,
) -> Result<gix::Commit<'repo>, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(branch) = branch {
        for candidate in [
            format!("refs/remotes/origin/{branch}"),
            format!("refs/heads/{branch}"),
            branch.to_string(),
        ] {
            let Ok(mut reference) = repo.find_reference(candidate.as_str()) else {
                continue;
            };
            let Ok(id) = reference.peel_to_id() else {
                continue;
            };
            let Ok(object) = id.object() else {
                continue;
            };
            if let Ok(commit) = object.try_into_commit() {
                return Ok(commit);
            }
        }
    }
    Ok(repo.head_commit()?)
}

/// Write an SSH private key to a temp file with `0600` permissions so
/// `ssh -i` will accept it without complaining about loose permissions.
fn write_ssh_key(secret: &str) -> std::io::Result<tempfile::NamedTempFile> {
    let mut file = tempfile::NamedTempFile::new()?;
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = file.as_file().metadata()?.permissions();
        perms.set_mode(0o600);
        file.as_file().set_permissions(perms)?;
    }
    let mut contents = secret.to_string();
    if !contents.ends_with('\n') {
        contents.push('\n');
    }
    file.write_all(contents.as_bytes())?;
    file.flush()?;
    Ok(file)
}
