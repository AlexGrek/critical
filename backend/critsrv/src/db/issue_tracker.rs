use chrono::{DateTime, Utc};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use surrealdb::{
    engine::local::{Db, Mem, SurrealKv},
    RecordId,
    Surreal,
};
use uuid::Uuid;

use crate::errors; // Assuming this path is correct for your error definitions

// --- Enums and Structs (updated to use `RecordId`) ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(u8)]
pub enum AdminRole {
    Admin = 0,
    Superadmin = 1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(u8)]
pub enum TicketStatus {
    Reported = 0,
    Confirmed = 1,
    Inprogress = 2,
    Blocked = 3,
    Resolved = 4,
    Rejected = 5,
    Approval = 6,
    Reopened = 7,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(u8)]
pub enum TicketSeverity {
    Unknown = 0,
    Low = 1,
    Mid = 2,
    High = 3,
    Critical = 4,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunnerKind {
    InsecureShell,
    Docker,
    Kubernetes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comment {
    pub user_id: String,
    pub comment_text: String,
    pub datetime: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TicketHistoryEntry {
    pub datetime: DateTime<Utc>,
    pub user_id: String,
    pub changes: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>, // Changed from Thing to RecordId
    pub email: String,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin: Option<AdminRole>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth_id: Option<String>,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn update_from_user_merge(&mut self, other: Self) {
        if other.admin.is_some() {
            self.admin = other.admin;
        }
        self.metadata = other.metadata;
        self.password_hash = other.password_hash;
        if other.oauth_id.is_some() {
            self.oauth_id = other.oauth_id;
        }
    }
}

/// Represents a group of users.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>, // Changed from Thing to RecordId
    pub name: String,
    #[serde(default)]
    pub user_ids: Vec<String>,
}

impl Group {
    pub fn update_from_group_merge(&mut self, other: Self) {
        self.name = other.name;
        self.user_ids = other.user_ids;
    }
}

/// Represents a tenant, acting as a namespace for projects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tenant {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>, // Changed from Thing to RecordId
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creator: Option<String>,
}

impl Tenant {
    pub fn update_from_tenant_merge(&mut self, other: Self) {
        self.name = other.name;
    }
}
/// Represents a project, containing tickets and pipelines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>, // Changed from Thing to RecordId
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub reference: String,
    pub owner: String,
    #[serde(default)]
    pub users_reporters: Vec<String>,
    #[serde(default)]
    pub groups_reporters: Vec<String>,
    #[serde(default)]
    pub users_contributors: Vec<String>,
    #[serde(default)]
    pub groups_contributors: Vec<String>,
    #[serde(default)]
    pub users_admins: Vec<String>,
    #[serde(default)]
    pub groups_admins: Vec<String>,
    #[serde(default)]
    pub is_public: bool,
    #[serde(default)]
    pub pipelines_enabled: bool,
    #[serde(default)]
    pub webhooks: HashMap<String, String>,
    #[serde(default)]
    pub ticket_id_prefix: String,
    pub tenant_id: String,
    #[serde(default)]
    pub next_ticket_sequence: u64,
}

impl Project {
    pub fn update_from_project_merge(&mut self, other: Self) {
        self.description = other.description;
        self.reference = other.reference;
        self.owner = other.owner;
        self.users_reporters = other.users_reporters;
        self.groups_reporters = other.groups_reporters;
        self.users_contributors = other.users_contributors;
        self.groups_contributors = other.groups_contributors;
        self.users_admins = other.users_admins;
        self.groups_admins = other.groups_admins;
        self.is_public = other.is_public;
        self.pipelines_enabled = other.pipelines_enabled;
        self.webhooks = other.webhooks;
        self.ticket_id_prefix = other.ticket_id_prefix;
        self.tenant_id = other.tenant_id;
        self.next_ticket_sequence = other.next_ticket_sequence;
    }
}
/// Represents a ticket within a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ticket {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>, // Changed from Thing to RecordId
    pub project_id: String,
    pub ticket_id: String, // This will be part of the SurrealDB ID
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub status: TicketStatus,
    pub severity: TicketSeverity,
    #[serde(default)]
    pub related: HashMap<String, String>,
    #[serde(default)]
    pub comments: Vec<Comment>,
    #[serde(default)]
    pub history: Vec<TicketHistoryEntry>,
    pub is_closed: bool,
    #[serde(default)]
    pub assigned_to_users: Vec<String>,
    #[serde(default)]
    pub assigned_to_groups: Vec<String>,
    pub mentioned_users: Vec<String>,
    pub mentioned_groups: Vec<String>,
    #[serde(default = "Utc::now")]
    pub last_change_datetime: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub creation_datetime: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creator: Option<String>,
}

impl Ticket {
    pub fn update_from_ticket(&mut self, updated_ticket_data: Self) {
        self.name = updated_ticket_data.name;
        self.description = updated_ticket_data.description;
        self.status = updated_ticket_data.status;
        self.severity = updated_ticket_data.severity;
        self.related = updated_ticket_data.related;
        self.comments = updated_ticket_data.comments;
        self.history = updated_ticket_data.history;
        self.is_closed = updated_ticket_data.is_closed;
        self.assigned_to_users = updated_ticket_data.assigned_to_users;
        self.assigned_to_groups = updated_ticket_data.assigned_to_groups;
        self.mentioned_users = updated_ticket_data.mentioned_users;
        self.mentioned_groups = updated_ticket_data.mentioned_groups;
        self.creator = updated_ticket_data.creator;

        self.last_change_datetime = Utc::now();
    }

    pub fn update_from_ticket_merge(&mut self, updated_ticket_data: Self) {
        self.name = updated_ticket_data.name;
        self.description = updated_ticket_data.description;
        self.status = updated_ticket_data.status;
        self.severity = updated_ticket_data.severity;
        self.related = updated_ticket_data.related;
        self.comments = updated_ticket_data.comments;
        self.history = updated_ticket_data.history;
        self.is_closed = updated_ticket_data.is_closed;
        self.assigned_to_users = updated_ticket_data.assigned_to_users;
        self.assigned_to_groups = updated_ticket_data.assigned_to_groups;
        self.mentioned_users = updated_ticket_data.mentioned_users;
        self.mentioned_groups = updated_ticket_data.mentioned_groups;

        self.last_change_datetime = Utc::now();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pipeline {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>, // Changed from Thing to RecordId
    pub project_id: String,
    pub runner_kind: RunnerKind,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    #[serde(default)]
    pub script: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub name: String, // This will be part of the SurrealDB ID
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub modified_at: DateTime<Utc>,
}

impl Pipeline {
    pub fn update_from_pipeline_merge(&mut self, other: Self) {
        self.project_id = other.project_id;
        self.runner_kind = other.runner_kind;
        self.metadata = other.metadata;
        self.script = other.script;
        self.env = other.env;
        self.name = other.name;
        self.modified_at = Utc::now();
    }
}

pub fn new_random_string() -> String {
    Uuid::new_v4().to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notification {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>, // Changed from Thing to RecordId
    #[serde(default = "new_random_string")]
    pub id_field: String, // Renamed to avoid conflict with `id`
    pub user_id: String,
    pub reason: String,
    pub data: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_link: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticket_link: Option<String>,
    #[serde(default = "Utc::now")]
    pub datetime: DateTime<Utc>,
}

impl Notification {
    pub fn update_from_notification_merge(&mut self, other: Self) {
        self.user_id = other.user_id;
        self.reason = other.reason;
        self.data = other.data;
        self.project_link = other.project_link;
        self.ticket_link = other.ticket_link;
    }
}

// --- IssueTrackerDb with SurrealDB ---

pub struct IssueTrackerDb {
    pub db: Surreal<Db>, // Changed to SurrealKv
}

impl std::fmt::Debug for IssueTrackerDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IssueTrackerDb").finish()
    }
}

impl IssueTrackerDb {
    pub async fn new(path: &str) -> Result<Self, errors::AppError> {
        // Use `Surreal::new::<surrealdb::engine::local::SurrealKv>(path)` for persistent storage.
        // Make sure `path` is a valid file path, e.g., "/tmp/my_database.db"
        let db = Surreal::new::<SurrealKv>(path).await.map_err(|e| {
            errors::AppError::DatabaseError(format!("Failed to connect to SurrealDB: {}", e))
        })?;

        // For embedded databases like SurrealKv, you do not call .use_ns() or .use_db()
        // nor do you call .signin(). These are for client connections to a SurrealDB server.
        // The embedded database handles its internal structure implicitly.

        // Ensure default tenant exists and create if not
        let default_tenant_name = "default".to_string();
        // Create the RecordId from the table name and ID part
        let tenant_record_id: RecordId = ("tenant", default_tenant_name.as_str()).into();

        // Check if tenant exists
        let tenant_exists: Option<Tenant> = db
            .select(tenant_record_id.clone())
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB query error: {}", e)))?;

        if tenant_exists.is_none() {
            info!("Creating default tenant...");
            let mut new_default_tenant = Tenant {
                id: Some(tenant_record_id), // Assign RecordId to the struct
                name: default_tenant_name.clone(),
                creator: None,
            };

            // Use .content() with the struct that now includes the ID
            db.create::<Tenant>("tenant") // Explicit type annotation, table name as string
                .content(new_default_tenant)
                .await
                .map_err(|e| {
                    errors::AppError::DatabaseError(format!(
                        "Failed to create default tenant: {}",
                        e
                    ))
                })?;
            info!("Default tenant created");
        } else {
            info!("Default tenant already exists.");
        }
        Ok(Self { db })
    }

    pub async fn update_ticket_optimistic_lock(
        &self,
        updated_ticket_data: Ticket,
    ) -> Result<(), errors::AppError> {
        let ticket_id_str = updated_ticket_data.ticket_id.clone();
        let ticket_record_id: RecordId = ("ticket", ticket_id_str.as_str()).into();

        // Fetch the existing ticket to check the last_change_datetime
        let existing_ticket: Option<Ticket> = self
            .db
            .select(ticket_record_id.clone())
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB query error: {}", e)))?;

        match existing_ticket {
            Some(mut existing_ticket_found) => {
                // Optimistic locking check
                if existing_ticket_found.last_change_datetime
                    != updated_ticket_data.last_change_datetime
                {
                    warn!("Conflict detected: last_change_datetime mismatch. Update rejected.");
                    return Err(errors::AppError::InvalidData(
                        "Ticket updated outside, conflict".to_string(),
                    ));
                }

                // Update the existing ticket's fields
                existing_ticket_found.update_from_ticket_merge(updated_ticket_data);

                // Perform the update in SurrealDB
                let updated: Option<Ticket> = self
                    .db
                    .update::<Ticket>(ticket_record_id) // Explicit type annotation
                    .content(existing_ticket_found) // Using .set() for full content replacement
                    .await
                    .map_err(|e| errors::AppError::DatabaseError(format!("DB update error: {}", e)))?;

                if updated.is_none() {
                    return Err(errors::AppError::DatabaseError(
                        "Failed to update ticket (record not found after check)".to_string(),
                    ));
                }
            }
            None => {
                // If ticket doesn't exist, create it.
                // Ensure the ID is set correctly for creation.
                let mut new_ticket = updated_ticket_data.clone();
                new_ticket.id = Some(ticket_record_id); // Assign RecordId directly
                new_ticket.last_change_datetime = Utc::now();

                // Use .content() with the struct that now includes the ID
                self.db.create::<Ticket>("ticket") // Explicit type annotation, table name as string
                    .content(new_ticket)
                    .await
                    .map_err(|e| errors::AppError::DatabaseError(format!("DB create error: {}", e)))?;
            }
        }
        Ok(())
    }

    pub async fn upsert_user(&self, mut updated_user: User) -> Result<(), errors::AppError> {
        let user_record_id: RecordId = ("user", updated_user.email.as_str()).into();
        updated_user.id = Some(user_record_id); // Assign RecordId directly

        // Use .content() with the struct that now includes the ID
        self.db
            .create::<User>("user") // Explicit type annotation, table name as string
            .content(updated_user)
            .await
            .map_err(|e| {
                errors::AppError::DatabaseError(format!("Failed to upsert user: {}", e))
            })?;
        Ok(())
    }

    pub async fn upsert_group(&self, mut updated_group: Group) -> Result<(), errors::AppError> {
        let group_record_id: RecordId = ("group", updated_group.name.as_str()).into();
        updated_group.id = Some(group_record_id); // Assign RecordId directly

        // Use .content() with the struct that now includes the ID
        self.db
            .create::<Group>("group") // Explicit type annotation, table name as string
            .content(updated_group)
            .await
            .map_err(|e| {
                errors::AppError::DatabaseError(format!("Failed to upsert group: {}", e))
            })?;
        Ok(())
    }

    pub async fn upsert_tenant(&self, mut updated_tenant: Tenant) -> Result<(), errors::AppError> {
        let tenant_record_id: RecordId = ("tenant", updated_tenant.name.as_str()).into();
        updated_tenant.id = Some(tenant_record_id); // Assign RecordId directly

        // Use .content() with the struct that now includes the ID
        self.db
            .create::<Tenant>("tenant") // Explicit type annotation, table name as string
            .content(updated_tenant)
            .await
            .map_err(|e| {
                errors::AppError::DatabaseError(format!("Failed to upsert tenant: {}", e))
            })?;
        Ok(())
    }

    pub async fn upsert_project(
        &self,
        mut updated_project: Project,
    ) -> Result<(), errors::AppError> {
        let project_record_id: RecordId = ("project", updated_project.name.as_str()).into();
        updated_project.id = Some(project_record_id); // Assign RecordId directly

        // Use .content() with the struct that now includes the ID
        self.db
            .create::<Project>("project") // Explicit type annotation, table name as string
            .content(updated_project)
            .await
            .map_err(|e| {
                errors::AppError::DatabaseError(format!("Failed to upsert project: {}", e))
            })?;
        Ok(())
    }

    pub async fn upsert_pipeline(
        &self,
        mut updated_pipeline: Pipeline,
    ) -> Result<(), errors::AppError> {
        let pipeline_record_id: RecordId = ("pipeline", updated_pipeline.name.as_str()).into();
        updated_pipeline.id = Some(pipeline_record_id); // Assign RecordId directly

        // Use .content() with the struct that now includes the ID
        self.db
            .create::<Pipeline>("pipeline") // Explicit type annotation, table name as string
            .content(updated_pipeline)
            .await
            .map_err(|e| {
                errors::AppError::DatabaseError(format!("Failed to upsert pipeline: {}", e))
            })?;
        Ok(())
    }

    pub async fn upsert_notification(
        &self,
        mut updated_notification: Notification,
    ) -> Result<(), errors::AppError> {
        let notification_record_id: RecordId = ("notification", updated_notification.id_field.as_str()).into();
        updated_notification.id = Some(notification_record_id); // Assign RecordId directly

        // Use .content() with the struct that now includes the ID
        self.db
            .create::<Notification>("notification") // Explicit type annotation, table name as string
            .content(updated_notification)
            .await
            .map_err(|e| {
                errors::AppError::DatabaseError(format!("Failed to upsert notification: {}", e))
            })?;
        Ok(())
    }

    pub async fn get_user(&self, email: &str) -> Result<Option<User>, errors::AppError> {
        let user_record_id: RecordId = ("user", email.as_str()).into();
        let user: Option<User> = self
            .db
            .select(user_record_id)
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB query error: {}", e)))?;
        Ok(user)
    }

    pub async fn list_users(&self) -> Result<Vec<User>, errors::AppError> {
        let users: Vec<User> = self
            .db
            .query("SELECT * FROM user")
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB query error: {}", e)))?
            .take::<Vec<User>>(0) // Explicit type annotation
            .map_err(|e| errors::AppError::DatabaseError(format!("Failed to get results: {}", e)))?;
        Ok(users)
    }

    pub async fn delete_user(&self, email: &str) -> Result<(), errors::AppError> {
        let user_record_id: RecordId = ("user", email.as_str()).into();
        let deleted: Option<User> = self
            .db
            .delete::<User>(user_record_id) // Explicit type annotation
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB delete error: {}", e)))?;

        if deleted.is_none() {
            return Err(errors::AppError::InvalidData(format!(
                "Cannot remove entity {}: not found",
                email
            )));
        }
        Ok(())
    }

    pub async fn get_group(&self, name: &str) -> Result<Option<Group>, errors::AppError> {
        let group_record_id: RecordId = ("group", name.as_str()).into();
        let group: Option<Group> = self
            .db
            .select(group_record_id)
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB query error: {}", e)))?;
        Ok(group)
    }

    pub async fn list_groups(&self) -> Result<Vec<Group>, errors::AppError> {
        let groups: Vec<Group> = self
            .db
            .query("SELECT * FROM group")
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB query error: {}", e)))?
            .take::<Vec<Group>>(0) // Explicit type annotation
            .map_err(|e| errors::AppError::DatabaseError(format!("Failed to get results: {}", e)))?;
        Ok(groups)
    }

    pub async fn delete_group(&self, name: &str) -> Result<(), errors::AppError> {
        let group_record_id: RecordId = ("group", name.as_str()).into();
        let deleted: Option<Group> = self
            .db
            .delete::<Group>(group_record_id) // Explicit type annotation
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB delete error: {}", e)))?;

        if deleted.is_none() {
            return Err(errors::AppError::InvalidData(format!(
                "Cannot remove group {}: not found",
                name
            )));
        }
        Ok(())
    }

    pub async fn get_tenant(&self, name: &str) -> Result<Option<Tenant>, errors::AppError> {
        let tenant_record_id: RecordId = ("tenant", name.as_str()).into();
        let tenant: Option<Tenant> = self
            .db
            .select(tenant_record_id)
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB query error: {}", e)))?;
        Ok(tenant)
    }

    pub async fn list_tenants(&self) -> Result<Vec<Tenant>, errors::AppError> {
        let tenants: Vec<Tenant> = self
            .db
            .query("SELECT * FROM tenant")
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB query error: {}", e)))?
            .take::<Vec<Tenant>>(0) // Explicit type annotation
            .map_err(|e| errors::AppError::DatabaseError(format!("Failed to get results: {}", e)))?;
        Ok(tenants)
    }

    pub async fn delete_tenant(&self, name: &str) -> Result<(), errors::AppError> {
        let tenant_record_id: RecordId = ("tenant", name.as_str()).into();
        let deleted: Option<Tenant> = self
            .db
            .delete::<Tenant>(tenant_record_id) // Explicit type annotation
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB delete error: {}", e)))?;

        if deleted.is_none() {
            return Err(errors::AppError::InvalidData(format!(
                "Cannot remove tenant {}: not found",
                name
            )));
        }
        Ok(())
    }

    pub async fn get_project(&self, name: &str) -> Result<Option<Project>, errors::AppError> {
        let project_record_id: RecordId = ("project", name.as_str()).into();
        let project: Option<Project> = self
            .db
            .select(project_record_id)
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB query error: {}", e)))?;
        Ok(project)
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>, errors::AppError> {
        let projects: Vec<Project> = self
            .db
            .query("SELECT * FROM project")
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB query error: {}", e)))?
            .take::<Vec<Project>>(0) // Explicit type annotation
            .map_err(|e| errors::AppError::DatabaseError(format!("Failed to get results: {}", e)))?;
        Ok(projects)
    }

    pub async fn delete_project(&self, name: &str) -> Result<(), errors::AppError> {
        let project_record_id: RecordId = ("project", name.as_str()).into();
        let deleted: Option<Project> = self
            .db
            .delete::<Project>(project_record_id) // Explicit type annotation
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB delete error: {}", e)))?;

        if deleted.is_none() {
            return Err(errors::AppError::InvalidData(format!(
                "Cannot remove project {}: not found",
                name
            )));
        }
        Ok(())
    }

    pub async fn get_ticket(
        &self,
        ticket_id_str: &str,
    ) -> Result<Option<Ticket>, errors::AppError> {
        let ticket_record_id: RecordId = ("ticket", ticket_id_str.as_str()).into();
        let ticket: Option<Ticket> = self
            .db
            .select(ticket_record_id)
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB query error: {}", e)))?;
        Ok(ticket)
    }

    pub async fn list_tickets(&self) -> Result<Vec<Ticket>, errors::AppError> {
        let tickets: Vec<Ticket> = self
            .db
            .query("SELECT * FROM ticket")
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB query error: {}", e)))?
            .take::<Vec<Ticket>>(0) // Explicit type annotation
            .map_err(|e| errors::AppError::DatabaseError(format!("Failed to get results: {}", e)))?;
        Ok(tickets)
    }

    pub async fn delete_ticket(&self, ticket_id_str: &str) -> Result<(), errors::AppError> {
        let ticket_record_id: RecordId = ("ticket", ticket_id_str.as_str()).into();
        let deleted: Option<Ticket> = self
            .db
            .delete::<Ticket>(ticket_record_id) // Explicit type annotation
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB delete error: {}", e)))?;

        if deleted.is_none() {
            return Err(errors::AppError::InvalidData(format!(
                "Cannot remove ticket {}: not found",
                ticket_id_str
            )));
        }
        Ok(())
    }

    pub async fn get_pipeline(&self, name: &str) -> Result<Option<Pipeline>, errors::AppError> {
        let pipeline_record_id: RecordId = ("pipeline", name.as_str()).into();
        let pipeline: Option<Pipeline> = self
            .db
            .select(pipeline_record_id)
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB query error: {}", e)))?;
        Ok(pipeline)
    }

    pub async fn list_pipelines(&self) -> Result<Vec<Pipeline>, errors::AppError> {
        let pipelines: Vec<Pipeline> = self
            .db
            .query("SELECT * FROM pipeline")
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB query error: {}", e)))?
            .take::<Vec<Pipeline>>(0) // Explicit type annotation
            .map_err(|e| errors::AppError::DatabaseError(format!("Failed to get results: {}", e)))?;
        Ok(pipelines)
    }

    pub async fn delete_pipeline(&self, name: &str) -> Result<(), errors::AppError> {
        let pipeline_record_id: RecordId = ("pipeline", name.as_str()).into();
        let deleted: Option<Pipeline> = self
            .db
            .delete::<Pipeline>(pipeline_record_id) // Explicit type annotation
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB delete error: {}", e)))?;

        if deleted.is_none() {
            return Err(errors::AppError::InvalidData(format!(
                "Cannot remove pipeline {}: not found",
                name
            )));
        }
        Ok(())
    }

    pub async fn get_notification(
        &self,
        notification_id_str: &str,
    ) -> Result<Option<Notification>, errors::AppError> {
        let notification_record_id: RecordId = ("notification", notification_id_str.as_str()).into();
        let notification: Option<Notification> = self
            .db
            .select(notification_record_id)
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB query error: {}", e)))?;
        Ok(notification)
    }

    pub async fn list_notifications(&self) -> Result<Vec<Notification>, errors::AppError> {
        let notifications: Vec<Notification> = self
            .db
            .query("SELECT * FROM notification")
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB query error: {}", e)))?
            .take::<Vec<Notification>>(0) // Explicit type annotation
            .map_err(|e| errors::AppError::DatabaseError(format!("Failed to get results: {}", e)))?;
        Ok(notifications)
    }

    pub async fn delete_notification(
        &self,
        notification_id_str: &str,
    ) -> Result<(), errors::AppError> {
        let notification_record_id: RecordId = ("notification", notification_id_str.as_str()).into();
        let deleted: Option<Notification> = self
            .db
            .delete::<Notification>(notification_record_id) // Explicit type annotation
            .await
            .map_err(|e| errors::AppError::DatabaseError(format!("DB delete error: {}", e)))?;

        if deleted.is_none() {
            return Err(errors::AppError::InvalidData(format!(
                "Cannot remove notification {}: not found",
                notification_id_str
            )));
        }
        Ok(())
    }
}
