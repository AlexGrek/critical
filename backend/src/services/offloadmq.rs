use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

// ============================================================================
//  Client Error
// ============================================================================

#[derive(Debug)]
pub enum ClientError {
    Reqwest(reqwest::Error),
    Api(String),
    Serialization(serde_json::Error),
    UrlParse(String),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::Reqwest(e) => write!(f, "Network error: {}", e),
            ClientError::Api(e) => write!(f, "API error: {}", e),
            ClientError::Serialization(e) => write!(f, "Serialization error: {}", e),
            ClientError::UrlParse(e) => write!(f, "URL parse error: {}", e),
        }
    }
}

impl Error for ClientError {}

impl From<reqwest::Error> for ClientError {
    fn from(e: reqwest::Error) -> Self {
        ClientError::Reqwest(e)
    }
}

impl From<serde_json::Error> for ClientError {
    fn from(e: serde_json::Error) -> Self {
        ClientError::Serialization(e)
    }
}

// ============================================================================
//  Data Structures (Mirrored from schema.rs)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FileReference {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_clone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub get: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_login: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_auth_header: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_header: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_auth: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskSubmissionRequest {
    pub capability: String,
    #[serde(default)]
    pub urgent: bool,
    #[serde(default)]
    pub restartable: bool,
    pub payload: Value,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fetch_files: Vec<FileReference>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<FileReference>,
    // The client handles injection of this field automatically
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TaskStatus {
    Pending,
    Queued,
    Pinned(String),
    Assigned,
    Starting,
    Running,
    Completed,
    Failed,
    Canceled,
    FailedRetryPending,
    FailedRetryDelayed,
    // Catch-all for forward compatibility
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct TaskId {
    pub cap: String,
    pub id: String,
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}]", self.cap, self.id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatusResponse {
    pub id: TaskId,
    pub status: TaskStatus,
    pub stage: Option<String>,
    pub output: Option<Value>,
    pub log: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitResponse {
    pub id: TaskId,
    pub capability: String,
    pub status: String,
    pub message: String,
}

// ============================================================================
//  API Client
// ============================================================================

pub struct OffloadClient {
    base_url: String,
    api_key: String,
    http: reqwest::Client,
}

impl OffloadClient {
    /// Create a new client instance.
    /// 
    /// # Arguments
    /// * `base_url` - e.g., "http://localhost:3000"
    /// * `api_key` - Your client API key
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key: api_key.into(),
            http: reqwest::Client::new(),
        }
    }

    /// Submit a task to the queue (Non-blocking / Async processing).
    /// This returns immediately with a Task ID. You must poll for results.
    pub async fn submit_task(
        &self,
        capability: &str,
        payload: Value,
        restartable: bool,
    ) -> Result<SubmitResponse, ClientError> {
        let req = TaskSubmissionRequest {
            capability: capability.to_string(),
            urgent: false,
            restartable,
            payload,
            api_key: self.api_key.clone(),
            ..Default::default()
        };

        let url = format!("{}/api/task/submit", self.base_url);
        let resp = self.http.post(&url).json(&req).send().await?;
        
        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(ClientError::Api(format!("Failed to submit task: {}", error_text)));
        }

        let submission: SubmitResponse = resp.json().await?;
        Ok(submission)
    }

    /// Submit an URGENT task (Blocking / Sync processing).
    /// CAUTION: The server waits for an agent to complete this task before returning.
    /// Returns the final result immediately.
    pub async fn submit_urgent_task(
        &self,
        capability: &str,
        payload: Value,
    ) -> Result<Value, ClientError> {
        let req = TaskSubmissionRequest {
            capability: capability.to_string(),
            urgent: true,
            restartable: false,
            payload,
            api_key: self.api_key.clone(),
            ..Default::default()
        };

        let url = format!("{}/api/task/submit_blocking", self.base_url);
        let resp = self.http.post(&url).json(&req).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(ClientError::Api(format!("Failed to submit urgent task: {}", error_text)));
        }

        // Urgent tasks return the raw result payload directly
        let result: Value = resp.json().await?;
        Ok(result)
    }

    /// Submit a task with file dependencies (inputs/artifacts).
    pub async fn submit_task_with_files(
        &self,
        capability: &str,
        payload: Value,
        fetch_files: Vec<FileReference>,
        artifacts: Vec<FileReference>,
    ) -> Result<SubmitResponse, ClientError> {
        let req = TaskSubmissionRequest {
            capability: capability.to_string(),
            urgent: false,
            restartable: true,
            payload,
            fetch_files,
            artifacts,
            api_key: self.api_key.clone(),
        };

        let url = format!("{}/api/task/submit", self.base_url);
        let resp = self.http.post(&url).json(&req).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(ClientError::Api(format!("Failed to submit task: {}", error_text)));
        }

        let submission: SubmitResponse = resp.json().await?;
        Ok(submission)
    }

    /// Poll the status of a specific task.
    pub async fn get_task_status(&self, cap: &str, id: &str) -> Result<TaskStatusResponse, ClientError> {
        // The server expects ApiKeyRequest in the body for polling auth
        let body = serde_json::json!({
            "apiKey": self.api_key
        });

        // The endpoint is /api/task/poll/{cap}/{id}
        // NOTE: TaskId::from_url decodes the cap, so we should be careful about URL encoding if needed.
        // Assuming simple string capabilities here.
        let url = format!("{}/api/task/poll/{}/{}", self.base_url, cap, id);
        
        let resp = self.http.post(&url).json(&body).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(ClientError::Api(format!("Failed to poll status: {}", error_text)));
        }

        let status_resp: TaskStatusResponse = resp.json().await?;
        Ok(status_resp)
    }

    /// Check which capabilities are currently online (served by active agents).
    pub async fn get_online_capabilities(&self) -> Result<Vec<String>, ClientError> {
        let body = serde_json::json!({
            "apiKey": self.api_key
        });

        let url = format!("{}/api/capabilities/online", self.base_url);
        let resp = self.http.post(&url).json(&body).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(ClientError::Api(format!("Failed to fetch capabilities: {}", error_text)));
        }

        let caps: Vec<String> = resp.json().await?;
        Ok(caps)
    }

    /// Helper: Polls a task until it is Completed or Failed, with a delay between checks.
    /// This is a utility function not present in the original API but useful for clients.
    pub async fn wait_for_completion(
        &self, 
        cap: &str, 
        id: &str, 
        poll_interval: std::time::Duration
    ) -> Result<TaskStatusResponse, ClientError> {
        loop {
            let status = self.get_task_status(cap, id).await?;
            match status.status {
                TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Canceled => {
                    return Ok(status);
                }
                _ => {
                    tokio::time::sleep(poll_interval).await;
                }
            }
        }
    }
}
