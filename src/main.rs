use crate::utils::GitCommand;
use anyhow::Result;
use clap::{Arg, Command};

mod commands;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = build_cli().get_matches();

    match matches.subcommand() {
        Some(("save", sub_m)) => {
            let local = sub_m.get_flag("local");
            let git = GitCommand::new();
            commands::save_wip_changes(&git, local, None, None).await?;
        }
        Some(("restore", _sub_m)) => {
            commands::restore_wip_changes().await?;
        }
        Some(("list", _sub_m)) => {
            commands::list_wip_branches().await?;
        }
        _ => {
            // Show help text when no subcommand is provided
            build_cli().print_help()?;
            std::process::exit(1);
        }
    }

    Ok(())
}

fn build_cli() -> Command {
    Command::new("git-wippy")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("A Git utility for managing work-in-progress changes across branches")
        .subcommand(
            Command::new("save")
                .about("Saves current WIP changes to a temporary branch")
                .long_about("Creates a new branch with your work-in-progress changes, allowing you to switch contexts safely")
                .arg(
                    Arg::new("local")
                        .long("local")
                        .action(clap::ArgAction::SetTrue)
                        .help("Saves the WIP branch locally without pushing to remote origin"),
                ),
        )
        .subcommand(
            Command::new("restore")
                .about("Restores WIP changes from a temporary branch")
                .long_about("Retrieves your saved work-in-progress changes from a temporary branch and applies them to your current branch"),
        )
        .subcommand(
            Command::new("list")
                .about("List all WIP branches for the current user")
                .long_about("Displays all WIP branches created by the current user, including creation date and source branch"),
        )
}
