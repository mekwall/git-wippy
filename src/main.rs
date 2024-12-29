use crate::commands::{
    delete::{delete_wip_branches, DeleteOptions},
    list_wip_branches, restore_wip_changes, save_wip_changes,
};
use crate::utils::GitCommand;
use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod utils;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all WIP branches for the current user
    List,
    /// Save current changes to a WIP branch
    Save {
        /// Save only locally, don't push to remote
        #[arg(short, long)]
        local: bool,
    },
    /// Restore changes from a WIP branch
    Restore,
    /// Delete one or more WIP branches
    Delete {
        /// Branch name to delete
        branch: Option<String>,
        /// Delete all WIP branches
        #[arg(short, long)]
        all: bool,
        /// Skip confirmation prompts
        #[arg(short, long)]
        force: bool,
        /// Delete only local branches
        #[arg(short, long)]
        local_only: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => list_wip_branches().await,
        Commands::Save { local } => save_wip_changes(&GitCommand::new(), local, None, None).await,
        Commands::Restore => restore_wip_changes().await,
        Commands::Delete {
            branch,
            all,
            force,
            local_only,
        } => {
            let options = DeleteOptions {
                branch_name: branch,
                all,
                force,
                local_only,
            };
            delete_wip_branches(&GitCommand::new(), options).await
        }
    }
}
