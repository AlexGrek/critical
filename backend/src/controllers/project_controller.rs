use std::sync::Arc;

use crate::db::DatabaseInterface;

pub struct ProjectController {
    pub db: Arc<dyn DatabaseInterface>,
}

impl ProjectController {
    pub fn new(db: Arc<dyn DatabaseInterface>) -> Self {
        Self { db }
    }
}
