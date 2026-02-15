use std::sync::Arc;

use crate::db::ArangoDb;

pub struct TicketController {
    pub db: Arc<ArangoDb>,
}

impl TicketController {
    pub fn new(db: Arc<ArangoDb>) -> Self {
        Self { db }
    }
}
