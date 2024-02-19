use crate::utils::{get_user_wip_branches, git_username};
use anyhow::Result;

pub async fn list_wip_branches() -> Result<()> {
    let username = git_username().await?;
    let wip_branches = get_user_wip_branches(&username).await?;

    if wip_branches.is_empty() {
        println!("No WIP branches found for the user: {}", username);
    } else {
        println!("WIP branches for user '{}':", username);
        for branch in wip_branches {
            println!("- {}", branch);
        }
    }

    Ok(())
}
