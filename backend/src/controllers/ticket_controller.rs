use std::sync::Arc;

use crate::db::DatabaseInterface;

pub struct TicketController {
    pub db: Arc<dyn DatabaseInterface>,
}

impl TicketController {
    pub fn new(db: Arc<dyn DatabaseInterface>) -> Self {
        Self { db }
    }
}
