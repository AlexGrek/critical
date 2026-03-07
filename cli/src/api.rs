use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Normalize a kind name to its plural API collection name.
/// Already-plural kinds are returned unchanged.
/// e.g. "group" → "groups", "groups" → "groups"
pub fn to_api_kind(kind: &str) -> String {
    if kind.ends_with('s') {
        kind.to_string()
    } else {
        format!("{}s", kind)
    }
}

/// Return the singular form of a kind name.
/// e.g. "groups" → "group", "group" → "group"
pub fn to_singular_kind(kind: &str) -> &str {
    kind.strip_suffix('s').unwrap_or(kind)
}

#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub user: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, Deserialize)]
struct ApiErrorBody {
    error: ApiErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ApiErrorDetail {
    message: String,
    #[allow(dead_code)]
    status: u16,
}

pub async fn login(base_url: &str, user: &str, password: &str) -> Result<LoginResponse> {
    let url = format!("{}/api/v1/login", base_url.trim_end_matches('/'));

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&LoginRequest {
            user: user.to_string(),
            password: password.to_string(),
        })
        .send()
        .await?;

    if resp.status().is_success() {
        Ok(resp.json::<LoginResponse>().await?)
    } else {
        let status = resp.status();
        match resp.json::<ApiErrorBody>().await {
            Ok(body) => bail!("{} ({})", body.error.message, status),
            Err(_) => bail!("login failed with status {}", status),
        }
    }
}

pub async fn list_kind(base_url: &str, token: &str, kind: &str) -> Result<Value> {
    let url = format!("{}/api/v1/global/{}", base_url.trim_end_matches('/'), kind);
    fetch_authenticated(&url, token).await
}

pub async fn get_kind(base_url: &str, token: &str, kind: &str, id: &str) -> Result<Value> {
    let url = format!("{}/api/v1/global/{}/{}", base_url.trim_end_matches('/'), kind, id);
    fetch_authenticated(&url, token).await
}

/// Fetch an existing resource, returning `None` if it does not exist (404).
/// Other HTTP errors are returned as `Err`.
pub async fn try_get_kind(base_url: &str, token: &str, kind: &str, id: &str) -> Result<Option<Value>> {
    let url = format!("{}/api/v1/global/{}/{}", base_url.trim_end_matches('/'), kind, id);
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if resp.status().as_u16() == 404 {
        return Ok(None);
    }
    if resp.status().is_success() {
        return Ok(Some(resp.json::<Value>().await?));
    }
    let status = resp.status();
    match resp.json::<ApiErrorBody>().await {
        Ok(body) => bail!("{} ({})", body.error.message, status),
        Err(_) => bail!("request failed with status {}", status),
    }
}

pub async fn apply_object(base_url: &str, token: &str, kind: &str, id: &str, body: Value) -> Result<Value> {
    let url = format!("{}/api/v1/global/{}/{}", base_url.trim_end_matches('/'), kind, id);
    post_authenticated(&url, token, body).await
}

async fn post_authenticated(url: &str, token: &str, body: Value) -> Result<Value> {
    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&body)
        .send()
        .await?;

    if resp.status().is_success() {
        Ok(resp.json::<Value>().await?)
    } else {
        let status = resp.status();
        match resp.json::<ApiErrorBody>().await {
            Ok(err_body) => bail!("{} ({})", err_body.error.message, status),
            Err(_) => bail!("request failed with status {}", status),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_pluralization() {
        assert_eq!(to_api_kind("group"), "groups");
        assert_eq!(to_api_kind("user"), "users");
        assert_eq!(to_api_kind("project"), "projects");
        assert_eq!(to_api_kind("membership"), "memberships");
        // already plural — unchanged
        assert_eq!(to_api_kind("groups"), "groups");
        assert_eq!(to_api_kind("users"), "users");
    }

    #[test]
    fn kind_singular() {
        assert_eq!(to_singular_kind("groups"), "group");
        assert_eq!(to_singular_kind("users"), "user");
        assert_eq!(to_singular_kind("memberships"), "membership");
        // already singular — unchanged
        assert_eq!(to_singular_kind("group"), "group");
    }
}

async fn fetch_authenticated(url: &str, token: &str) -> Result<Value> {
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if resp.status().is_success() {
        Ok(resp.json::<Value>().await?)
    } else {
        let status = resp.status();
        match resp.json::<ApiErrorBody>().await {
            Ok(body) => bail!("{} ({})", body.error.message, status),
            Err(_) => bail!("request failed with status {}", status),
        }
    }
}
