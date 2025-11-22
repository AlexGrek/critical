use anyhow::Result;
use reqwest::{Client, Method};
use serde::de::DeserializeOwned;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Branch {
    pub name: String,
    pub commit: CommitRef,
}

#[derive(Debug, Deserialize)]
pub struct CommitRef {
    pub sha: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Issue {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
}

#[derive(Clone)]
pub struct GithubClient {
    http: Client,
    token: String, // Personal access token OR installation token
}

impl GithubClient {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            token: token.into(),
        }
    }

    async fn request<T: DeserializeOwned>(&self, method: Method, url: &str) -> Result<T> {
        let res = self
            .http
            .request(method, url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "critical")
            .send()
            .await?
            .error_for_status()?
            .json::<T>()
            .await?;

        Ok(res)
    }

    pub async fn list_branches(&self, owner: &str, repo: &str) -> Result<Vec<Branch>> {
        let url = format!("https://api.github.com/repos/{}/{}/branches", owner, repo);
        self.request(Method::GET, &url).await
    }

    pub async fn list_issues(&self, owner: &str, repo: &str) -> Result<Vec<Issue>> {
        let url = format!("https://api.github.com/repos/{}/{}/issues", owner, repo);
        self.request(Method::GET, &url).await
    }

    pub async fn create_issue(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        body: &str,
    ) -> Result<Issue> {
        let url = format!("https://api.github.com/repos/{}/{}/issues", owner, repo);
        let payload = serde_json::json!({
            "title": title,
            "body": body,
        });

        let issue = self
            .http
            .post(url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "critical")
            .json(&payload)
            .send()
            .await?
            .error_for_status()?
            .json::<Issue>()
            .await?;

        Ok(issue)
    }
}
