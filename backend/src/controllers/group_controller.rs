use std::sync::Arc;

use crate::db::DatabaseInterface;

pub struct GroupController {
    pub db: Arc<dyn DatabaseInterface>,
}

impl GroupController {
    pub fn new(db: Arc<dyn DatabaseInterface>) -> Self {
        Self { db }
    }
}
