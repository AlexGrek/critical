use std::sync::Arc;

use crate::{controllers::{group_controller::GroupController, project_controller::ProjectController, ticket_controller::TicketController, user_controller::UserController}, db::DatabaseInterface};
pub mod user_controller;
pub mod project_controller;
pub mod group_controller;
pub mod ticket_controller;

pub struct Controller {
    pub user: UserController,
    pub project: ProjectController,
    pub group: GroupController,
    pub ticket: TicketController,
}


impl Controller {
    pub fn new(db: Arc<dyn DatabaseInterface>) -> Self {
        Self {
            user: UserController::new(db.clone()),
            project: ProjectController::new(db.clone()),
            group: GroupController::new(db.clone()),
            ticket: TicketController::new(db.clone()),
        }
    }
}

