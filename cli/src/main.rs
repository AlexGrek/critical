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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Login { url, user } => commands::login::run(url, user).await,
        Commands::Context { action } => match action {
            None | Some(ContextAction::List) => commands::login::run_context(true),
            Some(ContextAction::Use { name }) => commands::login::use_context(&name),
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
