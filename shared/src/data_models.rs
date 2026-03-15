use serde::{Deserialize, Serialize};

use crate::util_models::PrincipalId;

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
// Users
// ---------------------------------------------------------------------------

/// Users don't have per-resource ACL — access controlled by super-permissions.
#[crit_derive::crit_resource(collection = "users", prefix = "u_", no_acl)]
pub struct User {
    pub password_hash: String,
    #[brief]
    pub personal: PersonalInfo,
    /// ULID of the user's current avatar (no extension). Set when an avatar upload is accepted;
    /// the processed WebP files live in `user_avatars/{ulid}_hd.webp` and `_thumb.webp`.
    #[serde(default)]
    #[brief]
    pub avatar_ulid: Option<String>,
    /// ULID of the user's current profile wallpaper. Files in `user_wallpapers/`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallpaper_ulid: Option<String>,
}

// ---------------------------------------------------------------------------
// Groups
// ---------------------------------------------------------------------------

#[crit_derive::crit_resource(collection = "groups", prefix = "g_")]
pub struct Group {
    #[brief]
    pub name: String,
    pub description: Option<String>,
    /// ULID of the group's avatar. Files in `group_avatars/`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[brief]
    pub avatar_ulid: Option<String>,
    /// ULID of the group's wallpaper. Files in `group_wallpapers/`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallpaper_ulid: Option<String>,
}

// ---------------------------------------------------------------------------
// Service Accounts (non-human principals for integrations)
// ---------------------------------------------------------------------------

#[crit_derive::crit_resource(collection = "service_accounts", prefix = "sa_")]
pub struct ServiceAccount {
    #[brief]
    pub name: String,
    pub description: Option<String>,
    /// Hashed API token for authentication.
    pub token_hash: String,
    /// ULID of the service account's avatar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[brief]
    pub avatar_ulid: Option<String>,
    /// ULID of the service account's wallpaper.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallpaper_ulid: Option<String>,
}

// ---------------------------------------------------------------------------
// Pipeline Accounts (non-human principals scoped to CI/CD)
// ---------------------------------------------------------------------------

#[crit_derive::crit_resource(collection = "pipeline_accounts", prefix = "pa_")]
pub struct PipelineAccount {
    #[brief]
    pub name: String,
    pub description: Option<String>,
    /// Scoped to a specific pipeline or project.
    pub scope: Option<String>,
    /// Hashed API token for authentication.
    pub token_hash: String,
    /// ULID of the pipeline account's avatar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[brief]
    pub avatar_ulid: Option<String>,
    /// ULID of the pipeline account's wallpaper.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallpaper_ulid: Option<String>,
}

// ---------------------------------------------------------------------------
// Principal info (unified cross-type identity card)
// ---------------------------------------------------------------------------

/// Lightweight identity card for any principal, used by the resolve endpoint.
/// `name` is the display name: `personal.name` for users, `name` for groups/sa/pa.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrincipalInfo {
    /// Principal type: `"user"`, `"group"`, `"service_account"`, `"pipeline_account"`.
    #[serde(rename = "type")]
    pub principal_type: String,
    /// Display name (`personal.name` for users, `name` for groups/sa/pa).
    pub name: String,
    /// ULID of the principal's avatar, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_ulid: Option<String>,
}

// ---------------------------------------------------------------------------
// Membership (edge collection — manual definition, no macro)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Global permissions (simple key-value, no lifecycle — manual definition)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPermission {
    #[serde(rename = "_key")]
    pub id: String,
    pub principals: Vec<PrincipalId>,
}

// ---------------------------------------------------------------------------
// Project sub-types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RepoProvider {
    #[default]
    Git,
    Github,
    Gitlab,
    Bitbucket,
    Svn,
    Mercurial,
    Custom,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepoLink {
    pub url: String,
    #[serde(default)]
    pub provider: RepoProvider,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Primary branch (git-based providers only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<String>,
}

// ---------------------------------------------------------------------------
// Project services
// ---------------------------------------------------------------------------

/// Toggleable feature modules available for a project.
/// Controls which tabs are visible in the UI.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ProjectService {
    /// Webhooks, GitHub Apps, and third-party integrations.
    Integrations,
    /// Built-in CI/CD pipeline engine.
    Pipelines,
    /// State-controlled deployment management.
    Deployments,
    /// Secret management (HashiCorp Vault alternative).
    Secrets,
    /// Git-backed documentation wiki (Confluence alternative).
    Wikis,
    /// Custom internal tools and micro-apps.
    Apps,
    /// Issue tracking with kanban boards (JIRA alternative).
    Tasks,
    /// Team discussion boards and threads.
    Talks,
    /// Version tagging, changelogs, and release artifacts.
    Releases,
    /// Dev/staging/prod environment configuration management.
    Environments,
    /// Project analytics, burndown charts, and metrics.
    Insights,
}

// ---------------------------------------------------------------------------
// Projects
// ---------------------------------------------------------------------------

/// Projects are namespaces for all work items (issues, sprints, pipelines, wiki).
/// Plain IDs with no prefix -- the project ID doubles as the namespace key.
#[crit_derive::crit_resource(collection = "projects", prefix = "")]
pub struct Project {
    #[brief]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Source code repositories linked to this project.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repositories: Vec<RepoLink>,
    /// Feature modules enabled for this project (controls visible UI tabs).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enabled_services: Vec<ProjectService>,
}

// ---------------------------------------------------------------------------
// TicketGroup sub-types
// ---------------------------------------------------------------------------

/// Supported field types for custom ticket fields.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TicketFieldType {
    Text,
    Number,
    Boolean,
    Date,
    /// Reference to a principal (user, group, etc.)
    User,
    /// One value chosen from `options`
    SingleSelect,
    /// Multiple values chosen from `options`
    MultiSelect,
}

/// A custom field definition within a ticket type.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TicketFieldDef {
    pub name: String,
    pub field_type: TicketFieldType,
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Allowed values for SingleSelect / MultiSelect fields.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<String>,
}

/// Workflow category that classifies a status for burndown / progress tracking.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StatusCategory {
    Todo,
    InProgress,
    Done,
}

/// A named workflow status belonging to a ticket type.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TicketStatus {
    pub name: String,
    pub category: StatusCategory,
}

/// A ticket type definition within a group (e.g. "Bug", "Story", "Epic").
/// Each type has its own workflow statuses and optional custom fields.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TicketTypeDef {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Ordered workflow statuses for this ticket type.
    pub statuses: Vec<TicketStatus>,
    /// Custom fields beyond the standard set (title, assignee, priority, etc.).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<TicketFieldDef>,
}

// ---------------------------------------------------------------------------
// TicketGroup (project-scoped)
// ---------------------------------------------------------------------------

/// A JIRA-style issue type scheme scoped to a project.
///
/// The resource `id` must be 2-6 capital letters A-Z (e.g. `BUG`, `TASK`, `FEAT`).
/// It is stored in ArangoDB as `_key = "tg_{id}"` (prefix added by the controller).
/// Each group defines one or more ticket types, each with their own workflow
/// statuses and optional custom fields.
#[crit_derive::crit_resource(collection = "ticketgroups", prefix = "tg_")]
pub struct TicketGroup {
    /// Display name of the ticket group (e.g. "Engineering Bugs").
    #[brief]
    pub name: String,
    /// Human-readable description of what ticket types this group covers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Ticket type definitions — each type has its own statuses and fields.
    #[serde(default)]
    pub ticket_types: Vec<TicketTypeDef>,
    /// Parent project `_key` — injected by the scoped gitops handler on create.
    pub project: String,
}
