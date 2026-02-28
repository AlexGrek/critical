use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

pub async fn list_groups(base_url: &str, token: &str) -> Result<Value> {
    let url = format!("{}/api/v1/global/groups", base_url.trim_end_matches('/'));
    fetch_authenticated(&url, token).await
}

pub async fn get_group(base_url: &str, token: &str, id: &str) -> Result<Value> {
    let url = format!("{}/api/v1/global/groups/{}", base_url.trim_end_matches('/'), id);
    fetch_authenticated(&url, token).await
}

pub async fn list_users(base_url: &str, token: &str) -> Result<Value> {
    let url = format!("{}/api/v1/global/users", base_url.trim_end_matches('/'));
    fetch_authenticated(&url, token).await
}

pub async fn get_user(base_url: &str, token: &str, id: &str) -> Result<Value> {
    let url = format!("{}/api/v1/global/users/{}", base_url.trim_end_matches('/'), id);
    fetch_authenticated(&url, token).await
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
