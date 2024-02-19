use crate::utils::{execute_git_command, formatted_datetime, git_username};
use anyhow::Result;

/// Stages all changes, creates a detailed commit message, commits the changes, and optionally pushes.
pub async fn save_wip_changes(local: bool) -> Result<()> {
    // Ensure we're in a git repository
    execute_git_command(&["rev-parse", "--is-inside-work-tree"]).await?;

    let username = git_username().await?;
    let datetime = formatted_datetime();
    let branch_name = format!("wip/{}/{}", username, datetime);

    // Generate the detailed commit message
    let commit_message = generate_commit_message().await?;

    // Create and switch to the new branch
    execute_git_command(&["checkout", "-b", &branch_name]).await?;

    // Stage all changes, including untracked files
    execute_git_command(&["add", "--all"]).await?;

    // Commit the changes
    execute_git_command(&["commit", "-m", &commit_message]).await?;

    if !local {
        // Push the new branch to the remote repository
        execute_git_command(&["push", "-u", "origin", &branch_name]).await?;
    }

    println!("WIP changes saved to branch '{}'", branch_name);
    Ok(())
}

/// Generates a detailed commit message including staged, changed, and untracked files.
async fn generate_commit_message() -> Result<String> {
    let staged = execute_git_command(&["diff", "--cached", "--name-only"]).await?;
    let changed = execute_git_command(&["diff", "--name-only"]).await?;
    let untracked = execute_git_command(&["ls-files", "--others", "--exclude-standard"]).await?;

    let source_branch = execute_git_command(&["rev-parse", "--abbrev-ref", "HEAD"]).await?;

    let staged_section = if !staged.is_empty() {
        format!("\nStaged changes:\n\t{}", staged.replace("\n", "\n\t"))
    } else {
        String::new()
    };

    let changed_section = if !changed.is_empty() {
        format!("\nChanges:\n\t{}", changed.replace("\n", "\n\t"))
    } else {
        String::new()
    };

    let untracked_section = if !untracked.is_empty() {
        format!("\nUntracked:\n\t{}", untracked.replace("\n", "\n\t"))
    } else {
        String::new()
    };

    let message = format!(
        "chore: saving work in progress\n\nSource branch: {}{}{}{}",
        source_branch, staged_section, changed_section, untracked_section
    );

    Ok(message)
}
