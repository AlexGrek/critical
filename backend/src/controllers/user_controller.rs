use std::sync::Arc;

use crate::db::DatabaseInterface;

pub struct UserController {
    pub db: Arc<dyn DatabaseInterface>,
}

impl UserController {
    pub fn new(db: Arc<dyn DatabaseInterface>) -> Self {
        Self { db }
    }

    pub async fn validate_user(&self, username: &str) -> bool {
        let user_res = self.db.users().get_user(username).await;
        user_res.is_ok()
    }   
}
