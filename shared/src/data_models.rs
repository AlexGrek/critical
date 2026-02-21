use std::collections::HashMap;

use chrono::{DateTime, Utc};
use crit_derive::Brief;
use serde::{Deserialize, Serialize};

use crate::util_models::{AccessControlStore, PrincipalId};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PersonalInfo {
    pub name: String,
    pub gender: String,
    pub job_title: String,
    pub manager: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, Brief)]
pub struct User {
    #[brief]
    #[serde(rename = "_key")]
    pub id: PrincipalId,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>, // user ID who created this user, if not self-registered
    #[brief]
    pub deactivated: bool,
    #[brief]
    pub personal: PersonalInfo,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: uuid::Uuid,
    pub acl: AccessControlStore,
    pub tickets: Vec<TicketGroup>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TicketGroup {
    pub prefix: String,
    pub acl: AccessControlStore,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ticket {
    pub id: i64,
    pub title: String,
    pub severity: (u8, String),
    pub description: String,
    pub created_by: String,     // only user
    pub assigned_to: String,    // can be group
    pub mentioned: Vec<String>, // principals
    pub last_modification: DateTime<Utc>,
    pub creation_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Brief)]
pub struct Group {
    #[brief]
    #[serde(rename = "_key")]
    pub id: PrincipalId,
    #[brief]
    pub name: String,
    #[serde(default)]
    pub acl: AccessControlStore,
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
