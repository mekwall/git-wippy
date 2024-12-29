use crate::utils::{get_user_wip_branches, git_username, parse_commit_message, Git, GitCommand};
use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Select};
use futures::future::try_join_all;

/// Restores changes from a WIP branch back to its original source branch.
///
/// # Details
/// * Retrieves WIP branches for the current user
/// * If multiple WIP branches exist, prompts user to select one
/// * Extracts source branch and file states from the WIP commit message
/// * Recreates the original file states (staged, changed, untracked)
/// * Deletes the WIP branch both locally and remotely
///
/// # Flow
/// 1. Get WIP branches and select one
/// 2. Extract information from commit message
/// 3. Checkout WIP branch and unstage changes
/// 4. Stash changes
/// 5. Switch to source branch (create if needed)
/// 6. Apply stashed changes
/// 7. Recreate original file states
/// 8. Clean up WIP branch
///
/// # Returns
/// * `Ok(())` if restoration succeeds
/// * `Err` if any step fails
pub async fn restore_wip_changes() -> Result<()> {
    let git = GitCommand::new();
    let username = git_username().await.context("Failed to get Git username")?;
    let wip_branches = get_user_wip_branches(&username, &git).await?;

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
    let commit_message: String = git
        .execute(vec![
            "log".to_string(),
            "-1".to_string(),
            "--pretty=%B".to_string(),
            selected_branch.clone(),
        ])
        .await?;
    let (source_branch, staged_files, changed_files, untracked_files) =
        parse_commit_message(&commit_message);

    // Checkout the WIP branch and revert the last commit to unstage changes
    git.checkout(&selected_branch).await?;
    git.execute(vec![
        "reset".to_string(),
        "--soft".to_string(),
        "HEAD~".to_string(),
    ])
    .await?;

    // Stash all changes
    git.execute(vec![
        "stash".to_string(),
        "push".to_string(),
        "-u".to_string(),
        "-m".to_string(),
        "Restoring WIP changes".to_string(),
    ])
    .await?;

    // Determine if the source branch exists, create it if not
    let branch_exists = git
        .execute(vec![
            "rev-parse".to_string(),
            "--verify".to_string(),
            source_branch.clone(),
        ])
        .await
        .is_ok();

    if branch_exists {
        git.checkout(&source_branch).await?;
    } else {
        git.create_branch(&source_branch).await?;
    }

    // Apply the stash
    git.execute(vec!["stash".to_string(), "pop".to_string()])
        .await?;

    // Recreate the original state of files based on the parsed commit message
    recreate_file_states(&git, staged_files, changed_files, untracked_files).await?;

    // Delete the WIP branch locally and remotely
    git.execute(vec![
        "branch".to_string(),
        "-D".to_string(),
        selected_branch.clone(),
    ])
    .await?;

    // Only attempt to delete the remote branch if the "origin" remote exists
    let remotes = git.execute(vec!["remote".to_string()]).await?;
    if remotes.contains("origin") {
        git.execute(vec![
            "push".to_string(),
            "origin".to_string(),
            "--delete".to_string(),
            selected_branch.clone(),
        ])
        .await?;
    }

    println!("Successfully restored changes from '{}'", selected_branch);
    Ok(())
}

/// Prompts the user to select a WIP branch from a list.
///
/// # Arguments
/// * `options` - List of branch names to choose from
///
/// # Returns
/// * `Ok(String)` - The selected branch name
/// * `Err` if user interaction fails
async fn get_user_selection(options: &[String]) -> Result<String> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a WIP branch to restore")
        .items(&options)
        .default(0)
        .interact()
        .context("Failed to select a WIP branch")?;

    Ok(options[selection].clone())
}

/// Recreates the original state of files in the working directory.
///
/// # Arguments
/// * `git` - Git implementation to use for commands
/// * `staged_files` - Files that should be staged
/// * `changed_files` - Files that should be changed but unstaged
/// * `untracked_files` - Files that should be untracked
///
/// # Details
/// * Stages files using `git add`
/// * Unstages files using `git reset HEAD`
/// * Ensures correct tracking status for each file
async fn recreate_file_states(
    git: &impl Git,
    staged_files: Vec<String>,
    changed_files: Vec<String>,
    untracked_files: Vec<String>,
) -> Result<()> {
    // Process file operations concurrently
    let stage_futures = staged_files
        .into_iter()
        .map(|file| git.execute(vec!["add".to_string(), file]));

    let change_futures = changed_files.into_iter().map(|file| {
        git.execute(vec![
            "reset".to_string(),
            "HEAD".to_string(),
            "--".to_string(),
            file,
        ])
    });

    let untrack_futures = untracked_files.into_iter().map(|file| {
        git.execute(vec![
            "reset".to_string(),
            "HEAD".to_string(),
            "--".to_string(),
            file,
        ])
    });

    // Run all operations concurrently
    try_join_all(stage_futures).await?;
    try_join_all(change_futures).await?;
    try_join_all(untrack_futures).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::MockGit;

    /// Tests that file states are correctly recreated
    #[tokio::test]
    async fn test_recreate_file_states() -> Result<()> {
        let mut mock_git = MockGit::new();

        let staged_files = vec!["staged.txt".to_string()];
        let changed_files = vec!["changed.txt".to_string()];
        let untracked_files = vec!["untracked.txt".to_string()];

        // Expect add command for staged files
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "add".to_string(),
                "staged.txt".to_string(),
            ]))
            .returning(|_| Ok(String::new()));

        // Expect reset command for changed files
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "reset".to_string(),
                "HEAD".to_string(),
                "--".to_string(),
                "changed.txt".to_string(),
            ]))
            .returning(|_| Ok(String::new()));

        // Expect reset command for untracked files
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "reset".to_string(),
                "HEAD".to_string(),
                "--".to_string(),
                "untracked.txt".to_string(),
            ]))
            .returning(|_| Ok(String::new()));

        recreate_file_states(&mock_git, staged_files, changed_files, untracked_files).await?;
        Ok(())
    }
}
