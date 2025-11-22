pub mod inmemory;
pub mod arangodb;

use crate::{error::AppError, models::{Group, Project, Ticket, User}, utils::BoxFuture};

// Individual repository traits
pub trait UsersRepo: Send + Sync {
    fn get_user<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<User, AppError>>;
    fn create_user<'a>(&'a self, user: User) -> BoxFuture<'a, Result<(), AppError>>;
    fn update_user<'a>(&'a self, id: &'a str, user: User) -> BoxFuture<'a, Result<(), AppError>>;
    fn delete_user<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<(), AppError>>;
    fn list_users<'a>(&'a self) -> BoxFuture<'a, Result<Vec<User>, AppError>>;
}

pub trait ProjectsRepo: Send + Sync {
    fn get_project<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<Project, AppError>>;
    fn create_project<'a>(&'a self, project: Project) -> BoxFuture<'a, Result<(), AppError>>;
    fn update_project<'a>(&'a self, id: &'a str, project: Project) -> BoxFuture<'a, Result<(), AppError>>;
    fn delete_project<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<(), AppError>>;
    fn list_projects<'a>(&'a self) -> BoxFuture<'a, Result<Vec<Project>, AppError>>;
}

pub trait GroupsRepo: Send + Sync {
    fn get_group<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<Group, AppError>>;
    fn create_group<'a>(&'a self, group: Group) -> BoxFuture<'a, Result<(), AppError>>;
    fn update_group<'a>(&'a self, id: &'a str, group: Group) -> BoxFuture<'a, Result<(), AppError>>;
    fn delete_group<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<(), AppError>>;
    fn list_groups<'a>(&'a self) -> BoxFuture<'a, Result<Vec<Group>, AppError>>;
}

pub trait TicketsRepo: Send + Sync {
    fn get_ticket<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<Ticket, AppError>>;
    fn create_ticket<'a>(&'a self, ticket: Ticket) -> BoxFuture<'a, Result<(), AppError>>;
    fn update_ticket<'a>(&'a self, id: &'a str, ticket: Ticket) -> BoxFuture<'a, Result<(), AppError>>;
    fn delete_ticket<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<(), AppError>>;
    fn list_tickets<'a>(&'a self) -> BoxFuture<'a, Result<Vec<Ticket>, AppError>>;
}

// Main database interface that provides access to all repositories
pub trait DatabaseInterface: Send + Sync {
    // Access to individual repositories
    fn users(&self) -> &dyn UsersRepo;
    fn projects(&self) -> &dyn ProjectsRepo;
    fn groups(&self) -> &dyn GroupsRepo;
    fn tickets(&self) -> &dyn TicketsRepo;
    
    // Transaction support (optional but recommended)
    fn begin_transaction<'a>(&'a self) -> BoxFuture<'a, Result<(), AppError>>;
    fn commit_transaction<'a>(&'a self) -> BoxFuture<'a, Result<(), AppError>>;
    fn rollback_transaction<'a>(&'a self) -> BoxFuture<'a, Result<(), AppError>>;

    // Initialization (called on app start, can do migrations, db creation)
    fn initialize(&self) -> BoxFuture<'_, Result<(), AppError>>;
}
