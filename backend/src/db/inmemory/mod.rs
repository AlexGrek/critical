// Example implementation structure for in-memory database
use std::collections::HashMap;
use std::sync::RwLock;

use crate::db::{BoxFuture, DatabaseInterface, GroupsRepo, ProjectsRepo, TicketsRepo, UsersRepo};
use crate::error::AppError;
use crate::models::Ticket;

use crate::models::{Group, Project, User};

pub struct InMemoryDatabase {
    users_repo: InMemoryUsersRepo,
    projects_repo: InMemoryProjectsRepo,
    groups_repo: InMemoryGroupsRepo,
    tickets_repo: InMemoryTicketsRepo,
}

impl Default for InMemoryDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryDatabase {
    pub fn new() -> Self {
        Self {
            users_repo: InMemoryUsersRepo::new(),
            projects_repo: InMemoryProjectsRepo::new(),
            groups_repo: InMemoryGroupsRepo::new(),
            tickets_repo: InMemoryTicketsRepo::new(),
        }
    }
}

impl DatabaseInterface for InMemoryDatabase {
    fn users(&self) -> &dyn UsersRepo {
        &self.users_repo
    }

    fn projects(&self) -> &dyn ProjectsRepo {
        &self.projects_repo
    }

    fn groups(&self) -> &dyn GroupsRepo {
        &self.groups_repo
    }

    fn tickets(&self) -> &dyn TicketsRepo {
        &self.tickets_repo
    }

    fn begin_transaction<'a>(&'a self) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            // No-op for in-memory implementation
            Ok(())
        })
    }

    fn commit_transaction<'a>(&'a self) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            // No-op for in-memory implementation
            Ok(())
        })
    }

    fn rollback_transaction<'a>(&'a self) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            // No-op for in-memory implementation
            Ok(())
        })
    }

    fn initialize(&self) -> BoxFuture<'_, Result<(), AppError>> {
        // do nothing, succesfully
        Box::pin(async move { Ok(()) })
    }
}

// In-memory Users Repository
pub struct InMemoryUsersRepo {
    users: RwLock<HashMap<String, User>>,
}

impl Default for InMemoryUsersRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryUsersRepo {
    pub fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
        }
    }
}

impl UsersRepo for InMemoryUsersRepo {
    fn get_user<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<User, AppError>> {
        Box::pin(async move {
            let users = self.users.read().unwrap();
            users
                .get(id)
                .cloned()
                .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))
        })
    }

    fn create_user<'a>(&'a self, user: User) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let mut users = self.users.write().unwrap();
            let id = user.username.clone();
            if users.contains_key(&id.to_string()) {
                return Err(AppError::Conflict(format!("User {} already exists", id)));
            }
            users.insert(id.to_string(), user);
            Ok(())
        })
    }

    fn update_user<'a>(&'a self, id: &'a str, user: User) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let mut users = self.users.write().unwrap();
            if !users.contains_key(id) {
                return Err(AppError::NotFound(format!("User {} not found", id)));
            }
            users.insert(id.to_string(), user);
            Ok(())
        })
    }

    fn delete_user<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let mut users = self.users.write().unwrap();
            users
                .remove(id)
                .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))?;
            Ok(())
        })
    }

    fn list_users<'a>(&'a self) -> BoxFuture<'a, Result<Vec<User>, AppError>> {
        Box::pin(async move {
            let users = self.users.read().unwrap();
            Ok(users.values().cloned().collect())
        })
    }
}

// In-memory Projects Repository
pub struct InMemoryProjectsRepo {
    projects: RwLock<HashMap<String, Project>>,
}

impl Default for InMemoryProjectsRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryProjectsRepo {
    pub fn new() -> Self {
        Self {
            projects: RwLock::new(HashMap::new()),
        }
    }
}

impl ProjectsRepo for InMemoryProjectsRepo {
    fn get_project<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<Project, AppError>> {
        Box::pin(async move {
            let projects = self.projects.read().unwrap();
            projects
                .get(id)
                .cloned()
                .ok_or_else(|| AppError::NotFound(format!("Project {} not found", id)))
        })
    }

    fn create_project<'a>(&'a self, project: Project) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let mut projects = self.projects.write().unwrap();
            let id = project.id;
            if projects.contains_key(&id.to_string()) {
                return Err(AppError::Conflict(format!("Project {} already exists", id)));
            }
            projects.insert(id.to_string(), project);
            Ok(())
        })
    }

    fn update_project<'a>(
        &'a self,
        id: &'a str,
        project: Project,
    ) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let mut projects = self.projects.write().unwrap();
            if !projects.contains_key(id) {
                return Err(AppError::NotFound(format!("Project {} not found", id)));
            }
            projects.insert(id.to_string(), project);
            Ok(())
        })
    }

    fn delete_project<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let mut projects = self.projects.write().unwrap();
            projects
                .remove(id)
                .ok_or_else(|| AppError::NotFound(format!("Project {} not found", id)))?;
            Ok(())
        })
    }

    fn list_projects<'a>(&'a self) -> BoxFuture<'a, Result<Vec<Project>, AppError>> {
        Box::pin(async move {
            let projects = self.projects.read().unwrap();
            Ok(projects.values().cloned().collect())
        })
    }
}

// In-memory Groups Repository
pub struct InMemoryGroupsRepo {
    groups: RwLock<HashMap<String, Group>>,
}

impl Default for InMemoryGroupsRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryGroupsRepo {
    pub fn new() -> Self {
        Self {
            groups: RwLock::new(HashMap::new()),
        }
    }
}

impl GroupsRepo for InMemoryGroupsRepo {
    fn get_group<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<Group, AppError>> {
        Box::pin(async move {
            let groups = self.groups.read().unwrap();
            groups
                .get(id)
                .cloned()
                .ok_or_else(|| AppError::NotFound(format!("Group {} not found", id)))
        })
    }

    fn create_group<'a>(&'a self, group: Group) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let mut groups = self.groups.write().unwrap();
            let id = group.gid.to_string();
            if groups.contains_key(&id) {
                return Err(AppError::Conflict(format!("Group {} already exists", id)));
            }
            groups.insert(id.to_string(), group);
            Ok(())
        })
    }

    fn update_group<'a>(
        &'a self,
        id: &'a str,
        group: Group,
    ) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let mut groups = self.groups.write().unwrap();
            if !groups.contains_key(id) {
                return Err(AppError::NotFound(format!("Group {} not found", id)));
            }
            groups.insert(id.to_string(), group);
            Ok(())
        })
    }

    fn delete_group<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let mut groups = self.groups.write().unwrap();
            groups
                .remove(id)
                .ok_or_else(|| AppError::NotFound(format!("Group {} not found", id)))?;
            Ok(())
        })
    }

    fn list_groups<'a>(&'a self) -> BoxFuture<'a, Result<Vec<Group>, AppError>> {
        Box::pin(async move {
            let groups = self.groups.read().unwrap();
            Ok(groups.values().cloned().collect())
        })
    }
}

// In-memory Tickets Repository
pub struct InMemoryTicketsRepo {
    tickets: RwLock<HashMap<String, Ticket>>,
}

impl Default for InMemoryTicketsRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryTicketsRepo {
    pub fn new() -> Self {
        Self {
            tickets: RwLock::new(HashMap::new()),
        }
    }
}

impl TicketsRepo for InMemoryTicketsRepo {
    fn get_ticket<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<Ticket, AppError>> {
        Box::pin(async move {
            let tickets = self.tickets.read().unwrap();
            tickets
                .get(id)
                .cloned()
                .ok_or_else(|| AppError::NotFound(format!("Ticket {} not found", id)))
        })
    }

    fn create_ticket<'a>(&'a self, ticket: Ticket) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let mut tickets = self.tickets.write().unwrap();
            let id = ticket.id;
            if tickets.contains_key(&id.to_string()) {
                return Err(AppError::Conflict(format!("Ticket {} already exists", id)));
            }
            tickets.insert(id.to_string(), ticket);
            Ok(())
        })
    }

    fn update_ticket<'a>(
        &'a self,
        id: &'a str,
        ticket: Ticket,
    ) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let mut tickets = self.tickets.write().unwrap();
            if !tickets.contains_key(id) {
                return Err(AppError::NotFound(format!("Ticket {} not found", id)));
            }
            tickets.insert(id.to_string(), ticket);
            Ok(())
        })
    }

    fn delete_ticket<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let mut tickets = self.tickets.write().unwrap();
            tickets
                .remove(id)
                .ok_or_else(|| AppError::NotFound(format!("Ticket {} not found", id)))?;
            Ok(())
        })
    }

    fn list_tickets<'a>(&'a self) -> BoxFuture<'a, Result<Vec<Ticket>, AppError>> {
        Box::pin(async move {
            let tickets = self.tickets.read().unwrap();
            Ok(tickets.values().cloned().collect())
        })
    }
}
