use chrono::{DateTime, Utc};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

pub type PrincipalId = String;

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
    pub principals: Vec<String>,
}

impl AccessControlStore {
    /// Check if any of the given principals has the required permission in this ACL.
    pub fn check_permission(&self, principals: &[String], required: Permissions) -> bool {
        self.list.iter().any(|acl| {
            acl.permissions.contains(required)
                && acl.principals.iter().any(|p| principals.contains(p))
        })
    }
}

pub mod super_permissions {
    pub const ADM_USER_MANAGER: &str = "adm_user_manager";
    pub const ADM_PROJECT_MANAGER: &str = "adm_project_manager";
    pub const USR_CREATE_PROJECTS: &str = "usr_create_projects";
    pub const ADM_CONFIG_EDITOR: &str = "adm_config_editor";
    pub const USR_CREATE_GROUPS: &str = "usr_create_groups";
}
