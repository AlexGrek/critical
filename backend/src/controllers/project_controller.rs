use std::sync::Arc;

use crate::db::ArangoDb;

pub struct ProjectController {
    pub db: Arc<ArangoDb>,
}

impl ProjectController {
    pub fn new(db: Arc<ArangoDb>) -> Self {
        Self { db }
    }
}
