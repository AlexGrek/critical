use std::sync::Arc;

use crate::db::ArangoDb;

pub mod user_controller;
pub mod group_controller;
pub mod gitops_controller;
pub mod membership_controller;
pub mod project_controller;
pub mod repo_credential_controller;
pub mod crd_controller;
pub mod ticket_group_controller;

use crd_controller::CrdController;
use gitops_controller::{DefaultKindController, GitopsController, KindController};
use group_controller::GroupController;
use membership_controller::MembershipController;
use project_controller::ProjectController;
use repo_credential_controller::RepoCredentialController;
use ticket_group_controller::TicketGroupController;
use user_controller::UserController;

pub struct Controller {
    pub user: UserController,
    pub group: GroupController,
    pub gitops: GitopsController,
    pub membership: MembershipController,
    pub project: ProjectController,
    pub repo_credential: RepoCredentialController,
    pub crd: CrdController,
    pub ticket_group: TicketGroupController,
    default: DefaultKindController,
}

impl Controller {
    pub fn new(db: Arc<ArangoDb>) -> Self {
        Self {
            user: UserController::new(db.clone()),
            group: GroupController::new(db.clone()),
            gitops: GitopsController::new(db.clone()),
            membership: MembershipController::new(db.clone()),
            project: ProjectController::new(db.clone()),
            repo_credential: RepoCredentialController::new(db.clone()),
            crd: CrdController::new(db.clone()),
            ticket_group: TicketGroupController::new(db.clone()),
            default: DefaultKindController,
        }
    }

    /// Dispatch to the appropriate kind-specific controller.
    pub fn for_kind(&self, kind: &str) -> &dyn KindController {
        match kind {
            "users" => &self.user,
            "groups" => &self.group,
            "memberships" => &self.membership,
            "projects" => &self.project,
            "repo_credentials" => &self.repo_credential,
            "crds" => &self.crd,
            "ticketgroups" => &self.ticket_group,
            _ => &self.default,
        }
    }
}
