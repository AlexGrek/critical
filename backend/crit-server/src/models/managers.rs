use std::sync::Arc;

use axum::Json;
use crit_shared::entities::{
    Project, ProjectGitopsSerializable, User, UserGitopsSerializable, UserGitopsUpdate,
};
use gitops_lib::store::{GenericDatabaseProvider, Store};

// use crate::models::entities::{User, UserGitopsUpdate};
use anyhow::Result;

use crate::errors::AppError;

pub struct UserManager {
    store: Arc<Store>,
}

impl UserManager {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    pub async fn list(&self) -> Result<Vec<User>, AppError> {
        self.store
            .provider::<User>()
            .list()
            .await
            .map_err(|e| e.into())
    }

    pub async fn create(&self, item: UserGitopsSerializable) -> Result<(), AppError> {
        self.store
            .provider::<User>()
            .insert(&User::from(item))
            .await
            .map_err(|e| e.into())
    }

    pub async fn list_as_response(&self) -> Result<Json<Vec<UserGitopsSerializable>>, AppError> {
        let users = self.list().await?;
        Ok(Json(users.into_iter().map(|u| u.into()).collect()))
    }

    pub async fn upsert(&self, item: UserGitopsSerializable) -> Result<(), AppError> {
        self.store
            .provider::<User>()
            .upsert(&User::from(item))
            .await
            .map_err(|e| e.into())
    }

    pub async fn delete_by_id(&self, id: &str) -> Result<(), AppError> {
        self.store
            .provider::<Project>()
            .delete(id)
            .await
            .map_err(|e| e.into())
    }
}

pub struct ProjectManager<'a> {
    store: Arc<Store>,
    user: &'a User,
}

impl<'a> ProjectManager<'a> {
    pub fn new(store: Arc<Store>, user: &'a User) -> Self {
        Self { store, user }
    }

    pub async fn create(&self, mut item: ProjectGitopsSerializable) -> Result<(), AppError> {
        item.owner_uid = self.user.uid.clone();
        item.admins_uid = if item.admins_uid.is_empty() {
            vec![item.owner_uid.clone()]
        } else {
            item.admins_uid
        };
        self.store
            .provider::<Project>()
            .insert(&Project::from(item))
            .await
            .map_err(|e| e.into())
    }

    pub async fn upsert(&self, mut item: ProjectGitopsSerializable) -> Result<(), AppError> {
        item.owner_uid = self.user.uid.clone();
        item.admins_uid = if item.admins_uid.is_empty() {
            vec![item.owner_uid.clone()]
        } else {
            item.admins_uid
        };
        self.store
            .provider::<Project>()
            .upsert(&Project::from(item))
            .await
            .map_err(|e| e.into())
    }

    pub async fn delete_by_id(&self, id: &str) -> Result<(), AppError> {
        self.store
            .provider::<Project>()
            .delete(id)
            .await
            .map_err(|e| e.into())
    }

    pub async fn is_project_visible_to_user(&self, proj: &Project) -> Result<bool, AppError> {
        if self.user.has_admin_status {
            return Ok(true);
        }

        if proj.owner_uid == self.user.uid {
            return Ok(true);
        }
        // TODO: handle ownership correctly
        return Ok(false);
    }

    pub async fn list(&self) -> Result<Vec<Project>, AppError> {
        let all = self
            .store
            .provider::<Project>()
            .list()
            .await
            .map_err(|e| AppError::from(e))?;
        let mut visible: Vec<Project> = Vec::with_capacity(all.len());
        for item in all.into_iter() {
            let is_visible = self.is_project_visible_to_user(&item).await?;
            if is_visible {
                visible.push(item);
            }
        }
        Ok(visible)
    }

    pub async fn list_as_response(&self) -> Result<Json<Vec<ProjectGitopsSerializable>>, AppError> {
        let users = self.list().await?;
        Ok(Json(users.into_iter().map(|u| u.into()).collect()))
    }
}
