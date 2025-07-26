use std::sync::Arc;

use axum::Json;
use crit_shared::state_entities::{ProjectStateResponse, UserDashboard};
use crit_shared::{
    entities::{Project, ProjectGitopsSerializable, User, UserGitopsSerializable},
    state_entities::ProjectState,
};
use gitops_lib::store::qstorage::KvStorage;
use gitops_lib::store::{GenericDatabaseProvider, StorageError, Store};

// use crate::models::entities::{User, UserGitopsUpdate};
use anyhow::Result;
use gitops_lib::GitopsResourceRoot;

use crate::db::index_view::IndexView;
use crate::db::indexable_consts::USER_TO_PROJECTS;
use crate::errors::AppError;
use crate::state::AppState;

pub trait DataManager<T: GitopsResourceRoot> {
    async fn fetch(&self, key: &str) -> Result<T, StorageError>;

    async fn try_fetch(&self, key: &str) -> Result<Option<T>, StorageError> {
        let result = self.fetch(key).await;
        match result {
            Err(StorageError::ItemNotFound { .. }) => Ok(None),
            Ok(val) => Ok(Some(val)),
            Err(e) => Err(e),
        }
    }

    async fn try_fetch_all<'a>(&self, keys: Vec<&'a str>) -> Result<Vec<T>, StorageError> {
        let mut items = Vec::with_capacity(keys.len());
        for key in keys {
            let result = self.try_fetch(key).await?;
            if let Some(item) = result {
                items.push(item);
            }
        }
        Ok(items)
    }

    async fn try_fetch_all_owned<'a>(&self, keys: Vec<String>) -> Result<Vec<T>, StorageError> {
        let mut items = Vec::with_capacity(keys.len());
        for key in keys {
            let result = self.try_fetch(&key).await?;
            if let Some(item) = result {
                items.push(item);
            }
        }
        Ok(items)
    }
}

pub struct SpecificUserManager<'a> {
    store: Arc<Store>,
    index: Arc<dyn KvStorage>,
    user: &'a User,
}

impl<'a> SpecificUserManager<'a> {
    pub fn new(store: Arc<Store>, index: Arc<dyn KvStorage>, user: &'a User) -> Self {
        Self { store, user, index }
    }

    pub async fn get_referenced_projects(&self) -> Result<Vec<Project>, StorageError> {
        let projects =
            IndexView::new(self.index.clone(), USER_TO_PROJECTS).get_all(&self.user.uid)?;
        let pm = ProjectManager::new(self.store.clone(), self.index.clone(), self.user);
        return pm.try_fetch_all_owned(projects).await;
    }

    pub async fn gen_dashboard(&self) -> Result<UserDashboard, AppError> {
        let mut dashboard = UserDashboard::default();
        dashboard.recent_and_owned_projects = self
            .get_referenced_projects()
            .await?
            .into_iter()
            .map(|p| p.into_serializable())
            .collect();

        Ok(dashboard)
    }
}

pub struct UserManager {
    store: Arc<Store>,
    index: Arc<dyn KvStorage>,
}

impl UserManager {
    pub fn from_app_state(state: &AppState) -> Self {
        Self {
            index: state.index.clone(),
            store: state.store.clone(),
        }
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
    index: Arc<dyn KvStorage>,
}

impl<'a> ProjectManager<'a> {
    pub fn from_app_state(state: &AppState, user: &'a User) -> Self {
        Self {
            index: state.index.clone(),
            store: state.store.clone(),
            user,
        }
    }

    pub fn new(store: Arc<Store>, index: Arc<dyn KvStorage>, user: &'a User) -> Self {
        Self { store, user, index }
    }

    pub async fn create(&self, mut item: ProjectGitopsSerializable) -> Result<(), AppError> {
        item.owner_uid = self.user.uid.clone();
        item.admins_uid = if item.admins_uid.is_empty() {
            vec![item.owner_uid.clone()]
        } else {
            item.admins_uid
        };
        let user_to_projects_mapping = IndexView::new(self.index.clone(), USER_TO_PROJECTS);
        for user in item.admins_uid.iter() {
            user_to_projects_mapping.append_unique(user, &item.name_id)?;
        }
        user_to_projects_mapping.append_unique(&item.owner_uid, &item.name_id);
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

    pub async fn describe(&self, id: &str) -> Result<ProjectStateResponse, AppError> {
        self.store
            .provider::<Project>()
            .get_by_key(id)
            .await
            .map_err(|e| e.into())
            .map(|item| ProjectStateResponse {
                meta: item.into(),
                state: ProjectState { total_tickets: 0 },
            })
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

impl<'a> DataManager<Project> for ProjectManager<'a> {
    async fn fetch(&self, key: &str) -> Result<Project, StorageError> {
        self.store.provider::<Project>().get_by_key(key).await
    }
}

impl DataManager<User> for UserManager {
    async fn fetch(&self, key: &str) -> Result<User, StorageError> {
        self.store.provider::<User>().get_by_key(key).await
    }
}
