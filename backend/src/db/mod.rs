use std::any::Any;

use crate::models::*;
use anyhow::Result;
use async_trait::async_trait;

pub mod arangodb;
pub mod inmemory;

/// Transaction trait object: async commit/abort plus downcast helper.
/// Implementors MUST implement `as_any` to allow downcasting.
#[async_trait]
pub trait Transaction: Send + Sync {
    async fn commit(&mut self) -> Result<()>;
    async fn abort(&mut self) -> Result<()>;
    fn as_any(&mut self) -> &mut dyn Any;
}

pub type BoxTransaction = Box<dyn Transaction>;

// ------------ DATABASE INTERFACE ------------

#[async_trait]
pub trait DatabaseInterface: Send + Sync {
    /// Begin a server-side transaction. Return `None` if the backend doesn't support transactions.
    async fn begin_transaction(&self) -> Result<Option<BoxTransaction>>;

    /// Create a user (optionally inside tx)
    async fn create_user(&self, user: User, tx: Option<&mut BoxTransaction>) -> Result<()>;

    /// Create a group (optionally inside tx)
    async fn create_group(&self, group: Group, tx: Option<&mut BoxTransaction>) -> Result<()>;

    /// Add principal (user or group) to group (optionally inside tx)
    async fn add_principal_to_group(
        &self,
        principal_id: &str,
        group_id: &str,
        tx: Option<&mut BoxTransaction>,
    ) -> Result<()>;

    /// List users
    async fn get_users_list(&self) -> Result<Vec<User>>;

    /// List groups
    async fn get_groups_list(&self) -> Result<Vec<Group>>;

    /// Get direct user principals in group (returns principal ids like "u:alice")
    async fn get_users_in_group(&self, group_id: &str) -> Result<Vec<String>>;

    /// Get direct group principals in group (returns principal ids like "g:admins")
    async fn get_groups_in_group(&self, group_id: &str) -> Result<Vec<String>>;

    /// Modify user by ID (replace the full User struct)
    async fn modify_user(&self, user: User, tx: Option<&mut BoxTransaction>) -> Result<()>;

    /// Get user by ID
    async fn get_user_by_id(&self, user_id: &str) -> Result<Option<User>>;

    /// Get group by ID
    async fn get_group_by_id(&self, group_id: &str) -> Result<Option<Group>>;
}
