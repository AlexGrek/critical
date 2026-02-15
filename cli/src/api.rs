use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

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
    let url = format!("{}/login", base_url.trim_end_matches('/'));

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
