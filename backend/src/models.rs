use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::schema;
use bitflags::bitflags;

bitflags! {
    // derive common traits for easier usage
    #[derive(Default, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Permissions: u8 {
        const NONE = 0;
        const FETCH    = 1 << 0; // 0000 0001
        const LIST   = 1 << 1; // 0000 0010
        const NOTIFY = 1 << 2; // 0000 0100
        const CREATE = 1 << 3;
        const MODIFY = 1 << 4; // it also auto allows deletion
        const CUSTOM1 = 1 << 5;
        const CUSTOM2 = 1 << 6;
        const READ = Self::FETCH.bits() | Self::LIST.bits() | Self::NOTIFY.bits();
        const WRITE = Self::CREATE.bits() | Self::MODIFY.bits() | Self::READ.bits();

        // You can define composite flags (shortcuts)
        const ROOT     = Self::READ.bits() | Self::WRITE.bits() | Self::CUSTOM1.bits() | Self::CUSTOM2.bits();
        const DEFAULT = Self::NONE.bits();
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AccessControlStore {
    pub list: Vec<AccessControlList>,
    pub last_mod_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AccessControlList {
    pub permissions: Permissions,
    pub principals: Vec<String>
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PersonalInfo {
    pub name: String,
    pub gender: String,
    pub job_title: String,
    pub manager: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct User {
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>, // user ID who created this user, if not self-registered
    pub deactivated: bool,
    pub personal: PersonalInfo,
    pub metadata: HashMap<String, String>,
}

impl From<crate::schema::User> for User {
    fn from(src: schema::User) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("registered_at".to_string(), Utc::now().to_rfc3339());

        Self {
            username: src.username,
            password_hash: src.password_hash,
            metadata,
            ..Self::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: uuid::Uuid,
    pub acl: AccessControlStore,
    pub tickets: Vec<TicketGroup>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TicketGroup {
    pub prefix: String,
    pub acl: AccessControlStore
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ticket {
    pub id: i64,
    pub title: String,
    pub severity: (u8, String),
    pub description: String,
    pub created_by: String, // only user
    pub assigned_to: String, // can be group
    pub mentioned: Vec<String>, // principals
    pub last_modification: DateTime<Utc>,
    pub creation_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    pub gid: String,
    pub name: String,
    pub principals: Vec<String>
}
