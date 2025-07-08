use std::sync::Arc;

use axum::Json;
use crit_shared::entities::{User, UserGitopsSerializable};
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

    pub async fn list_as_response(&self) -> Result<Json<Vec<UserGitopsSerializable>>, AppError> {
        let users = self.list().await?;
        Ok(Json(users.into_iter().map(|u| u.into()).collect()))
    }
}
