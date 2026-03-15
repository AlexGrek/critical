mod api;
mod commands;
mod context;

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

/// Output format for get/list commands.
#[derive(Clone, Copy, PartialEq, Default, ValueEnum)]
pub enum OutputFormat {
    /// ASCII table (default, like kubectl)
    #[default]
    Table,
    /// YAML output
    Yaml,
    /// JSON output
    Json,
}

/// cr1t — CLI for Critical project management
#[derive(Parser)]
#[command(name = "cr1t", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authenticate with a Critical server
    Login {
        /// Server URL (e.g. https://critical.example.com)
        #[arg(long)]
        url: Option<String>,

        /// Username
        #[arg(long, short)]
        user: Option<String>,
    },

    /// Show or switch contexts
    Context {
        #[command(subcommand)]
        action: Option<ContextAction>,
    },

    /// Manage groups
    Groups {
        #[command(subcommand)]
        action: GroupsAction,
    },

    /// Manage users
    Users {
        #[command(subcommand)]
        action: UsersAction,
    },

    /// List resources of a kind, or get a single resource by ID
    Get {
        /// Resource kind — singular or plural (e.g. user, users, group, groups)
        kind: String,

        /// Resource ID (omit to list all)
        id: Option<String>,

        /// Output format
        #[arg(short = 'o', long = "output", value_name = "FORMAT", default_value = "table")]
        output: OutputFormat,

        /// Open interactive fuzzy-select (arrow keys + Enter to choose, type to filter)
        #[arg(short = 'i', long = "interactive")]
        interactive: bool,
    },

    /// Describe a single resource in full YAML (with kind field)
    Describe {
        /// Resource kind — singular or plural (e.g. user, users, group, groups)
        kind: String,

        /// Resource ID
        id: String,
    },

    /// Fetch a resource, open in $EDITOR, and save changes back
    Edit {
        /// Resource kind — singular or plural (e.g. group, user)
        kind: String,

        /// Resource ID
        id: String,
    },

    /// Apply a resource from a file or stdin (create or update)
    Apply {
        /// File to apply. Reads from stdin if not specified.
        #[arg(short = 'f', long = "filename", value_name = "FILE")]
        filename: Option<PathBuf>,
    },

    /// Manage ticket groups (project-scoped issue type schemes)
    TicketGroups {
        #[command(subcommand)]
        action: TicketGroupsAction,
    },

    /// Debug and introspection commands (requires godmode)
    Debug {
        #[command(subcommand)]
        action: DebugAction,
    },
}

#[derive(Subcommand)]
enum TicketGroupsAction {
    /// List ticket groups in a project
    List {
        /// Project ID
        #[arg(short = 'p', long = "project")]
        project: String,
        /// Output format
        #[arg(short = 'o', long = "output", value_name = "FORMAT", default_value = "table")]
        output: OutputFormat,
        /// Open interactive fuzzy-select
        #[arg(short = 'i', long = "interactive")]
        interactive: bool,
    },
    /// Get a single ticket group
    Get {
        /// Project ID
        #[arg(short = 'p', long = "project")]
        project: String,
        /// Ticket group ID
        id: String,
        /// Output format
        #[arg(short = 'o', long = "output", value_name = "FORMAT", default_value = "table")]
        output: OutputFormat,
    },
    /// Describe a ticket group in full YAML (with kind field)
    Describe {
        /// Project ID
        #[arg(short = 'p', long = "project")]
        project: String,
        /// Ticket group ID
        id: String,
    },
}

#[derive(Subcommand)]
enum DebugAction {
    /// Show recent events from the event log
    Events {
        /// Output format
        #[arg(short = 'o', long = "output", value_name = "FORMAT", default_value = "table")]
        output: OutputFormat,
    },
}

#[derive(Subcommand)]
enum ContextAction {
    /// List all contexts
    List,
    /// Switch to a named context
    Use {
        /// Context name to switch to
        name: String,
    },
}

#[derive(Subcommand)]
enum GroupsAction {
    /// List all groups
    List {
        /// Output format
        #[arg(short = 'o', long = "output", value_name = "FORMAT", default_value = "table")]
        output: OutputFormat,
    },
    /// Describe a group in full YAML
    Describe {
        /// Group ID
        id: String,
    },
}

#[derive(Subcommand)]
enum UsersAction {
    /// List all users
    List {
        /// Output format
        #[arg(short = 'o', long = "output", value_name = "FORMAT", default_value = "table")]
        output: OutputFormat,
    },
    /// Describe a user in full YAML
    Describe {
        /// User ID
        id: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Login { url, user } => commands::login::run(url, user).await,
        Commands::Context { action } => match action {
            None | Some(ContextAction::List) => commands::login::run_context(true),
            Some(ContextAction::Use { name }) => commands::login::use_context(&name),
        },
        Commands::Groups { action } => match action {
            GroupsAction::List { output } => commands::gitops::list_groups(output).await,
            GroupsAction::Describe { id } => commands::gitops::describe_group(&id).await,
        },
        Commands::Users { action } => match action {
            UsersAction::List { output } => commands::gitops::list_users(output).await,
            UsersAction::Describe { id } => commands::gitops::describe_user(&id).await,
        },
        Commands::Get { kind, id, output, interactive } => match id {
            Some(id) => commands::gitops::get_resource(&kind, &id, output).await,
            None => commands::gitops::list_resources(&kind, output, interactive).await,
        },
        Commands::Describe { kind, id } => commands::gitops::describe_resource(&kind, &id).await,
        Commands::Edit { kind, id } => commands::edit::run(&kind, &id).await,
        Commands::Apply { filename } => commands::apply::run(filename.as_deref()).await,
        Commands::TicketGroups { action } => match action {
            TicketGroupsAction::List { project, output, interactive } => {
                commands::gitops::list_ticket_groups(&project, output, interactive).await
            }
            TicketGroupsAction::Get { project, id, output } => {
                commands::gitops::get_ticket_group(&project, &id, output).await
            }
            TicketGroupsAction::Describe { project, id } => {
                commands::gitops::describe_ticket_group(&project, &id).await
            }
        },
        Commands::Debug { action } => match action {
            DebugAction::Events { output } => commands::debug::events(output).await,
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
