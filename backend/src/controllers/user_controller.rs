use std::sync::Arc;

use crate::db::ArangoDb;

pub struct UserController {
    pub db: Arc<ArangoDb>,
}

impl UserController {
    pub fn new(db: Arc<ArangoDb>) -> Self {
        Self { db }
    }

    pub async fn validate_user(&self, username: &str) -> bool {
        let user_res = self.db.get_user_by_id(username).await;
        user_res.is_ok()
    }
}
