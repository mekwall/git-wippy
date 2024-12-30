#[allow(unused_imports)]
use crate::i18n;
use crate::i18n::t;
use clap::{Args, CommandFactory, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "git-wippy")]
#[command(version)]
#[command(
    help_template = "{before-help}{name} {version}\n\n{usage-heading}\n  {usage}\n\n{all-args}{after-help}"
)]
#[command(about = t("app-about"))]
#[command(subcommand_required = true)]
#[command(arg_required_else_help = true)]
#[command(next_help_heading = "Help:")]
#[command(next_display_order = 0)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Args)]
pub struct SaveArgs {
    /// Don't push changes to remote repository
    #[arg(short, long, action = clap::ArgAction::SetTrue, help = t("save-local-help"))]
    pub local: bool,

    /// Specify a custom username
    #[arg(short, long, value_name = "USERNAME", help = t("save-username-help"))]
    pub username: Option<String>,

    /// Specify a custom date and time
    #[arg(short, long, value_name = "DATETIME", help = t("save-datetime-help"))]
    pub datetime: Option<String>,
}

#[derive(Args)]
pub struct DeleteArgs {
    /// Name of the branch to delete
    #[arg(value_name = "BRANCH", help = t("delete-branch-help"))]
    pub branch: Option<String>,

    /// Delete all WIP branches
    #[arg(short, long, action = clap::ArgAction::SetTrue, help = t("delete-all-help"))]
    pub all: bool,

    /// Skip confirmation prompt
    #[arg(short, long, action = clap::ArgAction::SetTrue, help = t("delete-force-help"))]
    pub force: bool,

    /// Only delete local branches
    #[arg(short, long, action = clap::ArgAction::SetTrue, help = t("delete-local-help"))]
    pub local: bool,
}

#[derive(Args)]
pub struct RestoreArgs {
    /// Name of the branch to restore
    #[arg(value_name = "BRANCH", help = t("restore-branch-help"))]
    pub branch: Option<String>,

    /// Skip confirmation prompt
    #[arg(short = 'y', action = clap::ArgAction::SetTrue, help = t("restore-force-help"))]
    pub force: bool,

    /// Automatically stash and reapply local changes
    #[arg(long = "autostash", action = clap::ArgAction::SetTrue, help = t("restore-autostash-help"))]
    pub autostash: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(alias = "s")]
    #[command(about = t("save-command-about"))]
    #[command(long_about = t("save-command-long-about"))]
    Save(SaveArgs),

    #[command(alias = "l")]
    #[command(about = t("list-command-about"))]
    #[command(long_about = t("list-command-long-about"))]
    List,

    #[command(alias = "d")]
    #[command(about = t("delete-command-about"))]
    #[command(long_about = t("delete-command-long-about"))]
    Delete(DeleteArgs),

    #[command(alias = "r")]
    #[command(about = t("restore-command-about"))]
    #[command(long_about = t("restore-command-long-about"))]
    Restore(RestoreArgs),
}

impl Cli {
    pub fn new() -> Self {
        let matches = Self::command().get_matches();
        match matches.subcommand() {
            Some(("save", sub_matches)) => Self {
                command: Commands::Save(SaveArgs {
                    local: sub_matches.get_flag("local"),
                    username: sub_matches.get_one::<String>("username").cloned(),
                    datetime: sub_matches.get_one::<String>("datetime").cloned(),
                }),
            },
            Some(("list", _)) => Self {
                command: Commands::List,
            },
            Some(("delete", sub_matches)) => Self {
                command: Commands::Delete(DeleteArgs {
                    branch: sub_matches.get_one::<String>("branch").cloned(),
                    all: sub_matches.get_flag("all"),
                    force: sub_matches.get_flag("force"),
                    local: sub_matches.get_flag("local"),
                }),
            },
            Some(("restore", sub_matches)) => Self {
                command: Commands::Restore(RestoreArgs {
                    branch: sub_matches.get_one::<String>("branch").cloned(),
                    force: sub_matches.get_flag("force"),
                    autostash: sub_matches.get_flag("autostash"),
                }),
            },
            _ => unreachable!(),
        }
    }
}
