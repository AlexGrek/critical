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
}

// ---------------------------------------------------------------------------
// Groups
// ---------------------------------------------------------------------------

#[crit_derive::crit_resource(collection = "groups", prefix = "g_")]
pub struct Group {
    #[brief]
    pub name: String,
    pub description: Option<String>,
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
}
