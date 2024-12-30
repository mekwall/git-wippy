mod cli;
mod commands;
mod i18n;
mod output;
mod utils;

use crate::cli::{Cli, Commands};
use crate::commands::{
    delete::delete_wip_branches, delete::DeleteOptions, list::list_wip_branches,
    restore::restore_wip_changes, restore::RestoreOptions, save::save_wip_changes,
};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::new();

    match cli.command {
        Commands::Save(options) => {
            save_wip_changes(options.local, options.username, options.datetime).await?;
        }
        Commands::List => {
            list_wip_branches().await?;
        }
        Commands::Delete(options) => {
            delete_wip_branches(DeleteOptions {
                branch_name: options.branch,
                all: options.all,
                force: options.force,
                local_only: options.local,
            })
            .await?;
        }
        Commands::Restore(options) => {
            restore_wip_changes(RestoreOptions {
                branch_name: options.branch,
                force: options.force,
                autostash: options.autostash,
            })
            .await?;
        }
    }

    Ok(())
}
