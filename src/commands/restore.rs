use crate::utils::{
    execute_git_command, get_user_wip_branches, git_username, parse_commit_message,
};
use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Select};

pub async fn restore_wip_changes() -> Result<()> {
    let username = git_username().await.context("Failed to get Git username")?;
    let mut wip_branches = get_user_wip_branches(&username)
        .await
        .context("Failed to fetch WIP branches")?;

    // Clean up the branch names by removing any leading asterisks
    for branch in &mut wip_branches {
        *branch = branch.trim_start_matches("* ").to_string();
    }

    let selected_branch = if wip_branches.len() > 1 {
        // Present the user with a selection if more than one WIP branch exists
        get_user_selection(&wip_branches).await?
    } else if let Some(branch) = wip_branches.first() {
        // Automatically select the branch if only one exists
        branch.clone()
    } else {
        // No WIP branches found
        println!("No WIP branches found for the user: {}", username);
        return Ok(());
    };

    // Fetch the last commit message from the WIP branch
    let commit_message =
        execute_git_command(&["log", "-1", "--pretty=%B", &selected_branch]).await?;
    let (source_branch, staged_files, changed_files, untracked_files) =
        parse_commit_message(&commit_message);

    // Checkout the WIP branch and revert the last commit to unstage changes
    execute_git_command(&["checkout", &selected_branch]).await?;
    execute_git_command(&["reset", "--soft", "HEAD~"]).await?;

    // Stash all changes
    execute_git_command(&["stash", "push", "-u", "-m", "Restoring WIP changes"]).await?;

    // Determine if the source branch exists, create it if not
    let branch_exists = execute_git_command(&["rev-parse", "--verify", &source_branch])
        .await
        .is_ok();
    if branch_exists {
        execute_git_command(&["checkout", &source_branch]).await?;
    } else {
        execute_git_command(&["checkout", "-b", &source_branch]).await?;
    }

    // Apply the stash
    execute_git_command(&["stash", "pop"]).await?;

    // Recreate the original state of files based on the parsed commit message
    recreate_file_states(staged_files, changed_files, untracked_files).await?;

    // Delete the WIP branch locally and remotely
    execute_git_command(&["branch", "-D", &selected_branch]).await?;

    // Only attempt to delete the remote branch if the "origin" remote exists
    let remotes = execute_git_command(&["remote"]).await?;
    if remotes.contains("origin") {
        execute_git_command(&["push", "origin", "--delete", &selected_branch]).await?;
    }

    println!("Successfully restored changes from '{}'", selected_branch);
    Ok(())
}

async fn get_user_selection(options: &[String]) -> Result<String> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a WIP branch to restore")
        .items(&options)
        .default(0)
        .interact()
        .context("Failed to select a WIP branch")?;

    Ok(options[selection].clone())
}

async fn recreate_file_states(
    staged_files: Vec<String>,
    changed_files: Vec<String>,
    untracked_files: Vec<String>,
) -> Result<()> {
    // Example logic to recreate the original state of files
    // You'll need to adjust based on your specific needs and Git's capabilities

    // Restage staged files
    for file in staged_files {
        execute_git_command(&["add", &file]).await?;
    }

    for file in changed_files {
        execute_git_command(&["reset", "HEAD", "--", &file]).await?;
    }

    for file in untracked_files {
        execute_git_command(&["reset", "HEAD", "--", &file]).await?;
    }

    Ok(())
}
