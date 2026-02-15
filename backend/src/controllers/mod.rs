use std::sync::Arc;

use crate::{controllers::{gitops_controller::GitopsController, group_controller::GroupController, project_controller::ProjectController, ticket_controller::TicketController, user_controller::UserController}, db::ArangoDb};
pub mod user_controller;
pub mod project_controller;
pub mod group_controller;
pub mod ticket_controller;
pub mod gitops_controller;

pub struct Controller {
    pub user: UserController,
    pub project: ProjectController,
    pub group: GroupController,
    pub ticket: TicketController,
    pub gitops: GitopsController,
}

impl Controller {
    pub fn new(db: Arc<ArangoDb>) -> Self {
        Self {
            user: UserController::new(db.clone()),
            project: ProjectController::new(db.clone()),
            group: GroupController::new(db.clone()),
            ticket: TicketController::new(db.clone()),
            gitops: GitopsController::new(db.clone()),
        }
    }
}
