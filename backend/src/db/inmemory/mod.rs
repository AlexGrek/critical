use crate::db::*;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// In-memory database structure.
#[derive(Clone, Default)]
pub struct InMemoryDb {
    users: Arc<Mutex<HashMap<String, User>>>,
    groups: Arc<Mutex<HashMap<String, Group>>>,
    memberships: Arc<Mutex<HashMap<String, HashSet<String>>>>,
}

impl InMemoryDb {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            groups: Arc::new(Mutex::new(HashMap::new())),
            memberships: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// Dummy transaction object that does nothing.
pub struct DummyTx;

#[async_trait]
impl Transaction for DummyTx {
    async fn commit(&mut self) -> Result<()> {
        Ok(())
    }

    async fn abort(&mut self) -> Result<()> {
        Ok(())
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[async_trait]
impl DatabaseInterface for InMemoryDb {
    async fn begin_transaction(&self) -> Result<Option<BoxTransaction>> {
        // In-memory DB does not support transactions
        Ok(None)
    }

    async fn create_user(&self, user: User, _tx: Option<&mut BoxTransaction>) -> Result<()> {
        let mut map = self.users.lock().unwrap();
        map.insert(user.id.clone(), user);
        Ok(())
    }

    async fn create_group(&self, group: Group, _tx: Option<&mut BoxTransaction>) -> Result<()> {
        let mut map = self.groups.lock().unwrap();
        map.insert(group.id.clone(), group);
        Ok(())
    }

    async fn add_principal_to_group(
        &self,
        principal_id: &str,
        group_id: &str,
        _tx: Option<&mut BoxTransaction>,
    ) -> Result<()> {
        let mut memberships = self.memberships.lock().unwrap();
        let set = memberships
            .entry(group_id.to_string())
            .or_insert_with(HashSet::new);
        set.insert(principal_id.to_string());
        Ok(())
    }

    async fn get_users_list(&self) -> Result<Vec<User>> {
        let map = self.users.lock().unwrap();
        Ok(map.values().cloned().collect())
    }

    async fn get_groups_list(&self) -> Result<Vec<Group>> {
        let map = self.groups.lock().unwrap();
        Ok(map.values().cloned().collect())
    }

    async fn get_users_in_group(&self, group_id: &str) -> Result<Vec<String>> {
        let memberships = self.memberships.lock().unwrap();
        if let Some(set) = memberships.get(group_id) {
            Ok(set
                .iter()
                .filter(|id| id.starts_with("u:"))
                .cloned()
                .collect())
        } else {
            Ok(vec![])
        }
    }

    async fn get_groups_in_group(&self, group_id: &str) -> Result<Vec<String>> {
        let memberships = self.memberships.lock().unwrap();
        if let Some(set) = memberships.get(group_id) {
            Ok(set
                .iter()
                .filter(|id| id.starts_with("g:"))
                .cloned()
                .collect())
        } else {
            Ok(vec![])
        }
    }

        async fn modify_user(&self, user: User, _tx: Option<&mut BoxTransaction>) -> Result<()> {
        let mut map = self.users.lock().unwrap();
        map.insert(user.id.clone(), user);
        Ok(())
    }

    async fn get_user_by_id(&self, user_id: &str) -> Result<Option<User>> {
        let map = self.users.lock().unwrap();
        Ok(map.get(user_id).cloned())
    }

    async fn get_group_by_id(&self, group_id: &str) -> Result<Option<Group>> {
        let map = self.groups.lock().unwrap();
        Ok(map.get(group_id).cloned())
    }
}
