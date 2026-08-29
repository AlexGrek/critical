//! URL parsing and validation for repository links.

use crate::error::AppError;

/// Parse and validate a repository URL. Rejects schemes we cannot use for a
/// network fetch (`file://`, remote-helper forms, and other non-standard
/// schemes) — the only accepted schemes are the ones a real git remote can use.
pub fn parse_repo_url(raw: &str) -> Result<gix::Url, AppError> {
    let url = gix::url::parse(raw)
        .map_err(|e| AppError::bad_request(format!("invalid repository URL: {e}")))?;

    match url.scheme {
        gix::url::Scheme::Http | gix::url::Scheme::Https | gix::url::Scheme::Ssh | gix::url::Scheme::Git => Ok(url),
        other => Err(AppError::bad_request(format!(
            "unsupported repository URL scheme: {other:?}"
        ))),
    }
}

/// Best-effort extraction of `(owner, repo)` from a URL path, e.g.
/// `/org/repo.git` (https) or `org/repo.git` (scp-style ssh) -> `("org", "repo")`.
pub fn owner_repo(url: &gix::Url) -> Option<(String, String)> {
    let path = url.path.to_string();
    let trimmed = path
        .trim_start_matches('/')
        .trim_end_matches('/')
        .trim_end_matches(".git");
    let mut parts = trimmed.rsplitn(2, '/');
    let repo = parts.next()?.to_string();
    let owner = parts.next()?.to_string();
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((owner, repo))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_https_url() {
        let url = parse_repo_url("https://github.com/org/repo.git").unwrap();
        assert_eq!(url.scheme, gix::url::Scheme::Https);
    }

    #[test]
    fn accepts_scp_style_ssh_url() {
        let url = parse_repo_url("git@github.com:org/repo.git").unwrap();
        assert_eq!(url.scheme, gix::url::Scheme::Ssh);
    }

    #[test]
    fn accepts_explicit_ssh_url() {
        let url = parse_repo_url("ssh://git@github.com/org/repo.git").unwrap();
        assert_eq!(url.scheme, gix::url::Scheme::Ssh);
    }

    #[test]
    fn rejects_file_scheme() {
        assert!(parse_repo_url("file:///etc/passwd").is_err());
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_repo_url("not a url at all \0").is_err());
    }

    #[test]
    fn owner_repo_from_https_url() {
        let url = parse_repo_url("https://github.com/org/repo.git").unwrap();
        assert_eq!(owner_repo(&url), Some(("org".to_string(), "repo".to_string())));
    }

    #[test]
    fn owner_repo_from_scp_style_url() {
        let url = parse_repo_url("git@github.com:org/repo.git").unwrap();
        assert_eq!(owner_repo(&url), Some(("org".to_string(), "repo".to_string())));
    }

    #[test]
    fn owner_repo_from_url_without_git_suffix() {
        let url = parse_repo_url("https://github.com/org/repo").unwrap();
        assert_eq!(owner_repo(&url), Some(("org".to_string(), "repo".to_string())));
    }

    #[test]
    fn owner_repo_missing_when_path_too_short() {
        let url = parse_repo_url("https://github.com/repo").unwrap();
        assert_eq!(owner_repo(&url), None);
    }
}
