use chrono::{DateTime, Utc};
use crit_derive::Brief;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::util_models::{AccessControlStore, Labels, LifecycleState, PrincipalId, ResourceMeta};

// ---------------------------------------------------------------------------
// Shared sub-types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PersonalInfo {
    pub name: String,
    pub gender: String,
    pub job_title: String,
    pub manager: Option<String>,
}

// ---------------------------------------------------------------------------
// Users & Groups
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, Default, Brief)]
pub struct User {
    #[brief]
    #[serde(rename = "_key")]
    pub id: PrincipalId,
    pub password_hash: String,
    /// Replaces the previous `deactivated: bool`.
    #[brief]
    pub state: LifecycleState,
    #[brief]
    pub personal: PersonalInfo,
    /// Replaces `metadata: HashMap<String, String>` and top-level
    /// `created_at` / `created_by` fields.
    #[brief]
    #[serde(default)]
    pub meta: ResourceMeta,
}

#[derive(Debug, Serialize, Deserialize, Clone, Brief)]
pub struct Group {
    #[brief]
    #[serde(rename = "_key")]
    pub id: PrincipalId,
    #[brief]
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub acl: AccessControlStore,
    #[serde(default)]
    pub meta: ResourceMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMembership {
    #[serde(rename = "_key")]
    pub id: String,
    #[serde(rename = "_from")]
    pub from: String,
    #[serde(rename = "_to")]
    pub to: String,
    pub principal: PrincipalId,
    pub group: PrincipalId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPermission {
    #[serde(rename = "_key")]
    pub id: String,
    pub principals: Vec<PrincipalId>,
}

// ---------------------------------------------------------------------------
// Projects
// ---------------------------------------------------------------------------

/// A project acts as a namespace for tasks, sprints, features, pages, etc.
#[derive(Debug, Serialize, Deserialize, Clone, Brief)]
pub struct Project {
    #[brief]
    #[serde(rename = "_key")]
    pub id: String, // p_ prefix
    #[brief]
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub acl: AccessControlStore,
    #[brief]
    #[serde(default)]
    pub state: LifecycleState,
    #[serde(default)]
    pub meta: ResourceMeta,
}

// ---------------------------------------------------------------------------
// Tasks (replaces Ticket / TicketGroup)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    #[default]
    Backlog,
    Open,
    InProgress,
    InReview,
    Done,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize, Clone, Brief)]
pub struct Task {
    #[brief]
    #[serde(rename = "_key")]
    pub id: String, // t_ prefix
    #[brief]
    pub title: String,
    pub description: String,
    #[brief]
    pub state: TaskState,
    #[brief]
    pub priority: Priority,
    pub severity: Option<(u8, String)>,
    pub assigned_to: Option<PrincipalId>,
    pub mentioned: Vec<PrincipalId>,
    #[serde(default)]
    pub acl: AccessControlStore,
    #[serde(default)]
    pub meta: ResourceMeta,
}

// ---------------------------------------------------------------------------
// Sprints
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SprintState {
    #[default]
    Planning,
    Active,
    Completed,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Clone, Brief)]
pub struct Sprint {
    #[brief]
    #[serde(rename = "_key")]
    pub id: String, // sp_ prefix
    #[brief]
    pub name: String,
    pub goal: Option<String>,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    #[brief]
    pub state: SprintState,
    #[serde(default)]
    pub acl: AccessControlStore,
    #[serde(default)]
    pub meta: ResourceMeta,
}

// ---------------------------------------------------------------------------
// Features (epics / requirements)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, Brief)]
pub struct Feature {
    #[brief]
    #[serde(rename = "_key")]
    pub id: String, // f_ prefix
    #[brief]
    pub name: String,
    pub description: Option<String>,
    /// Reuses TaskState: Backlog → Open → InProgress → Done.
    #[brief]
    pub state: TaskState,
    #[serde(default)]
    pub acl: AccessControlStore,
    #[serde(default)]
    pub meta: ResourceMeta,
}

// ---------------------------------------------------------------------------
// CI/CD — Pipelines, PipelineRuns, Artifacts, Deployments
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
    #[default]
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Clone, Brief)]
pub struct Pipeline {
    #[brief]
    #[serde(rename = "_key")]
    pub id: String, // pl_ prefix
    #[brief]
    pub name: String,
    pub repo_url: Option<String>,
    /// Branch patterns or tag globs that trigger this pipeline.
    pub triggers: Vec<String>,
    #[serde(default)]
    pub acl: AccessControlStore,
    #[serde(default)]
    pub meta: ResourceMeta,
}

#[derive(Debug, Serialize, Deserialize, Clone, Brief)]
pub struct PipelineRun {
    #[brief]
    #[serde(rename = "_key")]
    pub id: String, // plr_ prefix
    #[brief]
    pub pipeline_id: String,
    #[brief]
    pub state: RunState,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub triggered_by: PrincipalId,
    pub log_url: Option<String>,
    #[serde(default)]
    pub meta: ResourceMeta,
}

/// A build output: Docker image, binary, npm package, etc.
#[derive(Debug, Serialize, Deserialize, Clone, Brief)]
pub struct Artifact {
    #[brief]
    #[serde(rename = "_key")]
    pub id: String, // art_ prefix
    #[brief]
    pub name: String,
    /// e.g. "docker-image", "binary", "npm-package"
    pub artifact_type: String,
    pub uri: String,
    /// Content digest (e.g. sha256:…) for integrity verification.
    pub digest: Option<String>,
    #[serde(default)]
    pub meta: ResourceMeta,
}

#[derive(Debug, Serialize, Deserialize, Clone, Brief)]
pub struct Deployment {
    #[brief]
    #[serde(rename = "_key")]
    pub id: String, // dep_ prefix
    /// Target environment: "prod", "staging", "dev", etc.
    #[brief]
    pub env: String,
    #[brief]
    pub state: RunState,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub deployed_by: PrincipalId,
    #[serde(default)]
    pub acl: AccessControlStore,
    #[serde(default)]
    pub meta: ResourceMeta,
}

// ---------------------------------------------------------------------------
// Releases
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseState {
    #[default]
    Draft,
    Candidate,
    Released,
    Yanked,
}

#[derive(Debug, Serialize, Deserialize, Clone, Brief)]
pub struct Release {
    #[brief]
    #[serde(rename = "_key")]
    pub id: String, // rel_ prefix
    #[brief]
    pub version: String,
    pub changelog: Option<String>,
    #[brief]
    pub state: ReleaseState,
    pub released_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub acl: AccessControlStore,
    #[serde(default)]
    pub meta: ResourceMeta,
}

// ---------------------------------------------------------------------------
// Pages (wiki / Confluence replacement)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, Brief)]
pub struct Page {
    #[brief]
    #[serde(rename = "_key")]
    pub id: String, // pg_ prefix
    #[brief]
    pub title: String,
    /// Markdown content.
    pub content: String,
    #[serde(default)]
    pub acl: AccessControlStore,
    #[serde(default)]
    pub meta: ResourceMeta,
}

// ---------------------------------------------------------------------------
// Policies (approval gates)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Policy {
    #[serde(rename = "_key")]
    pub id: String, // pol_ prefix
    pub name: String,
    /// Resource kind this policy applies to, e.g. "deployment", "release".
    pub match_kind: String,
    /// Label conditions that must all match for the policy to apply.
    pub match_labels: Labels,
    /// Principals (users or groups) whose approval is required.
    pub require_approvers: Vec<PrincipalId>,
    #[serde(default)]
    pub meta: ResourceMeta,
}

// ---------------------------------------------------------------------------
// Audit history
// ---------------------------------------------------------------------------

/// Immutable audit record: a point-in-time snapshot of any resource.
/// Written by controller hooks; never updated or deleted via the public API.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourceRevision {
    /// Composite key: `{kind}_{resource_id}_{revision}`.
    #[serde(rename = "_key")]
    pub id: String,
    pub resource_kind: String,
    pub resource_id: String,
    pub revision: u64,
    /// Full document at this point in time.
    pub snapshot: Value,
    pub changed_by: PrincipalId,
    pub changed_at: DateTime<Utc>,
}
