use anyhow::Result;
use octocrab::Octocrab;
use octocrab::models::IssueState;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Branch {
    pub name: String,
    pub commit: CommitRef,
}

#[derive(Debug, Deserialize)]
pub struct CommitRef {
    pub sha: String,
}

#[derive(Debug)]
pub struct Issue {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
}

impl From<octocrab::models::issues::Issue> for Issue {
    fn from(issue: octocrab::models::issues::Issue) -> Self {
        Self {
            id: issue.id.0,
            number: issue.number,
            title: issue.title,
            body: issue.body,
            state: match issue.state {
                IssueState::Open => "open".to_string(),
                IssueState::Closed => "closed".to_string(),
                _ => "unknown".to_string(),
            },
        }
    }
}

/// Thin wrapper around `octocrab` for the GitHub-provider repository probe
/// and (currently unused elsewhere) issue helpers.
#[derive(Clone)]
pub struct GithubClient {
    octo: Octocrab,
}

impl GithubClient {
    /// `token` is a personal access token (or installation token). `None`
    /// means anonymous access, which only works for public repos.
    pub fn new(token: Option<String>) -> Result<Self> {
        let mut builder = Octocrab::builder();
        if let Some(token) = token {
            builder = builder.personal_token(token);
        }
        Ok(Self { octo: builder.build()? })
    }

    /// The repository's default branch. `Ok(None)` if the repo doesn't exist
    /// or isn't accessible with the current credentials.
    pub async fn default_branch(&self, owner: &str, repo: &str) -> Result<Option<String>> {
        match self.octo.repos(owner, repo).get().await {
            Ok(info) => Ok(info.default_branch),
            Err(e) if is_not_found(&e) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Size in bytes of the file at `path` on `branch`. `Ok(None)` if the
    /// path doesn't exist there.
    pub async fn get_file_size(&self, owner: &str, repo: &str, branch: &str, path: &str) -> Result<Option<u64>> {
        match self
            .octo
            .repos(owner, repo)
            .get_content()
            .path(path)
            .r#ref(branch)
            .send()
            .await
        {
            Ok(items) => Ok(items.items.first().map(|c| c.size.max(0) as u64)),
            Err(e) if is_not_found(&e) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn list_branches(&self, owner: &str, repo: &str) -> Result<Vec<Branch>> {
        let page = self.octo.repos(owner, repo).list_branches().send().await?;
        Ok(page
            .items
            .into_iter()
            .map(|b| Branch {
                name: b.name,
                commit: CommitRef { sha: b.commit.sha },
            })
            .collect())
    }

    pub async fn list_issues(&self, owner: &str, repo: &str) -> Result<Vec<Issue>> {
        let page = self.octo.issues(owner, repo).list().send().await?;
        Ok(page.items.into_iter().map(Issue::from).collect())
    }

    pub async fn create_issue(&self, owner: &str, repo: &str, title: &str, body: &str) -> Result<Issue> {
        let issue = self.octo.issues(owner, repo).create(title).body(body).send().await?;
        Ok(issue.into())
    }
}

/// `true` if the GitHub API responded 404 (repo/path doesn't exist or isn't
/// visible with the current credentials).
fn is_not_found(e: &octocrab::Error) -> bool {
    matches!(e, octocrab::Error::GitHub { source, .. } if source.status_code.as_u16() == 404)
}
