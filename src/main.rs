use anyhow::Result;
use clap::{Arg, Command};
use tokio::main;

mod commands;
mod utils;

#[main]
async fn main() -> Result<()> {
    let matches = Command::new("git-wippy")
        .version("1.0")
        .author("Your Name <your_email@example.com>")
        .about("Manages work-in-progress (WIP) changes in Git repositories")
        .subcommand(
            Command::new("save")
                .about("Saves current WIP changes to a temporary branch")
                .arg(
                    Arg::new("local")
                        .long("local")
                        .action(clap::ArgAction::SetTrue)
                        .help("Saves the WIP branch locally without pushing to remote origin"),
                ),
        )
        .subcommand(
            Command::new("restore")
                .about("Restores WIP changes from a temporary branch to the source branch"),
        )
        .subcommand(Command::new("list").about("List all WIP branches for the current user"))
        .get_matches();

    match matches.subcommand() {
        Some(("save", sub_m)) => {
            let local = sub_m.get_flag("local");
            commands::save_wip_changes(local).await?;
        }
        Some(("restore", _sub_m)) => {
            // For simplicity, let's assume the restore command internally handles branch selection
            commands::restore_wip_changes().await?;
        }
        Some(("list", _sub_m)) => {
            // For simplicity, let's assume the restore command internally handles branch selection
            commands::list_wip_branches().await?;
        }
        _ => {}
    }

    Ok(())
}
