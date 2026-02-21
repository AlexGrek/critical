mod api;
mod commands;
mod context;

use clap::{Parser, Subcommand};

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
    List,
    /// Describe a group (show as YAML)
    Describe {
        /// Group ID
        id: String,
    },
}

#[derive(Subcommand)]
enum UsersAction {
    /// List all users
    List,
    /// Describe a user (show as YAML)
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
            GroupsAction::List => commands::gitops::list_groups().await,
            GroupsAction::Describe { id } => commands::gitops::describe_group(&id).await,
        },
        Commands::Users { action } => match action {
            UsersAction::List => commands::gitops::list_users().await,
            UsersAction::Describe { id } => commands::gitops::describe_user(&id).await,
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
