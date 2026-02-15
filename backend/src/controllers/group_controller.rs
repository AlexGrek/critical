use std::sync::Arc;

use crate::db::ArangoDb;

pub struct GroupController {
    pub db: Arc<ArangoDb>,
}

impl GroupController {
    pub fn new(db: Arc<ArangoDb>) -> Self {
        Self { db }
    }
}
