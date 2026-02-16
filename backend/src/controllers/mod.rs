use std::sync::Arc;

use crate::db::ArangoDb;

pub mod user_controller;
pub mod project_controller;
pub mod group_controller;
pub mod ticket_controller;
pub mod gitops_controller;
pub mod membership_controller;

use gitops_controller::{DefaultKindController, GitopsController, KindController};
use group_controller::GroupController;
use membership_controller::MembershipController;
use project_controller::ProjectController;
use ticket_controller::TicketController;
use user_controller::UserController;

pub struct Controller {
    pub user: UserController,
    pub project: ProjectController,
    pub group: GroupController,
    pub ticket: TicketController,
    pub gitops: GitopsController,
    pub membership: MembershipController,
    default: DefaultKindController,
}

impl Controller {
    pub fn new(db: Arc<ArangoDb>) -> Self {
        Self {
            user: UserController::new(db.clone()),
            project: ProjectController::new(db.clone()),
            group: GroupController::new(db.clone()),
            ticket: TicketController::new(db.clone()),
            gitops: GitopsController::new(db.clone()),
            membership: MembershipController::new(db.clone()),
            default: DefaultKindController,
        }
    }

    /// Dispatch to the appropriate kind-specific controller.
    pub fn for_kind(&self, kind: &str) -> &dyn KindController {
        match kind {
            "users" => &self.user,
            "groups" => &self.group,
            "projects" => &self.project,
            "memberships" => &self.membership,
            _ => &self.default,
        }
    }
}
