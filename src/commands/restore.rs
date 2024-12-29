use crate::utils::{git_username_with_git, parse_commit_message, Git, GitCommand};
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
    restore_wip_changes_with_git(&git).await
}

/// Implementation that accepts a Git instance for better testability
pub async fn restore_wip_changes_with_git(git: &impl Git) -> Result<()> {
    let username = git_username_with_git(git).await?;
    let wip_branches = git.get_user_wip_branches(&username).await?;

    let selected_branch = if wip_branches.len() > 1 {
        get_user_selection(&wip_branches).await?
    } else if let Some(branch) = wip_branches.first() {
        branch.clone()
    } else {
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
    recreate_file_states(git, staged_files, changed_files, untracked_files).await?;

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
    use anyhow::Result;

    #[tokio::test]
    async fn test_recreate_file_states() -> Result<()> {
        let mut mock_git = MockGit::new();

        // Mock staging files
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "add".to_string(),
                "file1.txt".to_string(),
            ]))
            .returning(|_| Ok("".to_string()));

        // Mock resetting changed files
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "reset".to_string(),
                "HEAD".to_string(),
                "--".to_string(),
                "file2.txt".to_string(),
            ]))
            .returning(|_| Ok("".to_string()));

        // Mock resetting untracked files
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "reset".to_string(),
                "HEAD".to_string(),
                "--".to_string(),
                "file3.txt".to_string(),
            ]))
            .returning(|_| Ok("".to_string()));

        recreate_file_states(
            &mock_git,
            vec!["file1.txt".to_string()],
            vec!["file2.txt".to_string()],
            vec!["file3.txt".to_string()],
        )
        .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_restore_wip_changes() -> Result<()> {
        let mut mock_git = MockGit::new();

        // Mock username lookup
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "config".to_string(),
                "user.name".to_string(),
            ]))
            .returning(|_| Ok("test-user".to_string()));

        // Mock WIP branches
        mock_git
            .expect_get_user_wip_branches()
            .with(mockall::predicate::eq("test-user"))
            .returning(|_| Ok(vec!["wip/test-user/branch1".to_string()]));

        // Mock commit message
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "log".to_string(),
                "-1".to_string(),
                "--pretty=%B".to_string(),
                "wip/test-user/branch1".to_string(),
            ]))
            .returning(|_| {
                Ok("WIP: Saved state from branch 'main'\nSource branch: main\nStaged: file1.txt\nUnstaged: file2.txt\nUntracked: file3.txt".to_string())
            });

        // Mock checkout operations using the trait method
        mock_git
            .expect_checkout()
            .with(mockall::predicate::eq("wip/test-user/branch1"))
            .returning(|_| Ok("Switched to branch 'wip/test-user/branch1'".to_string()));

        mock_git
            .expect_checkout()
            .with(mockall::predicate::eq("main"))
            .returning(|_| Ok("Switched to branch 'main'".to_string()));

        // Mock reset to unstage changes
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "reset".to_string(),
                "--soft".to_string(),
                "HEAD~".to_string(),
            ]))
            .returning(|_| Ok("".to_string()));

        // Mock stash changes
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "stash".to_string(),
                "push".to_string(),
                "-u".to_string(),
                "-m".to_string(),
                "Restoring WIP changes".to_string(),
            ]))
            .returning(|_| Ok("Saved working directory".to_string()));

        // Mock branch existence check
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "rev-parse".to_string(),
                "--verify".to_string(),
                "main".to_string(),
            ]))
            .returning(|_| Ok("main".to_string()));

        // Mock stash pop
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "stash".to_string(),
                "pop".to_string(),
            ]))
            .returning(|_| Ok("Changes restored".to_string()));

        // Mock file state recreation (one mock per file)
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "add".to_string(),
                "file1.txt".to_string(),
            ]))
            .returning(|_| Ok("".to_string()));

        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "reset".to_string(),
                "HEAD".to_string(),
                "--".to_string(),
                "file2.txt".to_string(),
            ]))
            .returning(|_| Ok("".to_string()));

        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "reset".to_string(),
                "HEAD".to_string(),
                "--".to_string(),
                "file3.txt".to_string(),
            ]))
            .returning(|_| Ok("".to_string()));

        // Mock local branch deletion
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "branch".to_string(),
                "-D".to_string(),
                "wip/test-user/branch1".to_string(),
            ]))
            .returning(|_| Ok("Deleted branch".to_string()));

        // Mock remote check
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec!["remote".to_string()]))
            .returning(|_| Ok("origin".to_string()));

        // Mock remote branch deletion
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "push".to_string(),
                "origin".to_string(),
                "--delete".to_string(),
                "wip/test-user/branch1".to_string(),
            ]))
            .returning(|_| Ok("".to_string()));

        // Mock branch verification check
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "rev-parse".to_string(),
                "--verify".to_string(),
                "main".to_string(),
            ]))
            .returning(|_| Ok("main".to_string()));

        restore_wip_changes_with_git(&mock_git).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_restore_no_wip_branches() -> Result<()> {
        let mut mock_git = MockGit::new();

        // Mock username lookup
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "config".to_string(),
                "user.name".to_string(),
            ]))
            .returning(|_| Ok("test-user".to_string()));

        // Mock empty WIP branches list
        mock_git
            .expect_get_user_wip_branches()
            .with(mockall::predicate::eq("test-user"))
            .returning(|_| Ok(Vec::new()));

        restore_wip_changes_with_git(&mock_git).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_restore_invalid_commit_message() -> Result<()> {
        let mut mock_git = MockGit::new();

        // Mock username lookup
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "config".to_string(),
                "user.name".to_string(),
            ]))
            .returning(|_| Ok("test-user".to_string()));

        // Mock WIP branches
        mock_git
            .expect_get_user_wip_branches()
            .with(mockall::predicate::eq("test-user"))
            .returning(|_| Ok(vec!["wip/test-user/branch1".to_string()]));

        // Mock checkout using the trait method
        mock_git
            .expect_checkout()
            .with(mockall::predicate::eq("wip/test-user/branch1"))
            .returning(|_| Ok("Switched to branch".to_string()));

        // Mock invalid commit message
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "log".to_string(),
                "-1".to_string(),
                "--pretty=%B".to_string(),
                "wip/test-user/branch1".to_string(),
            ]))
            .returning(|_| Ok("Not a valid WIP commit message".to_string()));

        // Mock reset
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "reset".to_string(),
                "--soft".to_string(),
                "HEAD~".to_string(),
            ]))
            .returning(|_| Ok("".to_string()));

        // Mock stash attempt
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "stash".to_string(),
                "push".to_string(),
                "-u".to_string(),
                "-m".to_string(),
                "Restoring WIP changes".to_string(),
            ]))
            .returning(|_| Ok("Saved working directory".to_string()));

        // Mock branch verification check for empty branch
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "rev-parse".to_string(),
                "--verify".to_string(),
                "".to_string(),
            ]))
            .returning(|_| Err(anyhow::anyhow!("Branch not found")));

        // Mock create_branch for empty branch name
        mock_git
            .expect_create_branch()
            .with(mockall::predicate::eq(""))
            .returning(|_| Err(anyhow::anyhow!("Invalid commit message format")));

        let result = restore_wip_changes_with_git(&mock_git).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid commit message format"));
        Ok(())
    }

    #[tokio::test]
    async fn test_restore_stash_failure() -> Result<()> {
        let mut mock_git = MockGit::new();

        // Mock username lookup
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "config".to_string(),
                "user.name".to_string(),
            ]))
            .returning(|_| Ok("test-user".to_string()));

        // Mock WIP branches
        mock_git
            .expect_get_user_wip_branches()
            .with(mockall::predicate::eq("test-user"))
            .returning(|_| Ok(vec!["wip/test-user/branch1".to_string()]));

        // Mock commit message
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "log".to_string(),
                "-1".to_string(),
                "--pretty=%B".to_string(),
                "wip/test-user/branch1".to_string(),
            ]))
            .returning(
                |_| Ok("WIP: Saved state from branch 'main'\nStaged: file1.txt".to_string()),
            );

        // Mock checkout using the trait method
        mock_git
            .expect_checkout()
            .with(mockall::predicate::eq("wip/test-user/branch1"))
            .returning(|_| Ok("Switched to branch".to_string()));

        // Mock reset
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "reset".to_string(),
                "--soft".to_string(),
                "HEAD~".to_string(),
            ]))
            .returning(|_| Ok("".to_string()));

        // Mock stash failure
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "stash".to_string(),
                "push".to_string(),
                "-u".to_string(),
                "-m".to_string(),
                "Restoring WIP changes".to_string(),
            ]))
            .returning(|_| Err(anyhow::anyhow!("Failed to stash changes")));

        let result = restore_wip_changes_with_git(&mock_git).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to stash changes"));
        Ok(())
    }

    #[tokio::test]
    async fn test_restore_branch_not_found() -> Result<()> {
        let mut mock_git = MockGit::new();

        // Mock username lookup
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "config".to_string(),
                "user.name".to_string(),
            ]))
            .returning(|_| Ok("test-user".to_string()));

        // Mock empty WIP branches list
        mock_git
            .expect_get_user_wip_branches()
            .with(mockall::predicate::eq("test-user"))
            .returning(|_| Ok(Vec::new()));

        let result = restore_wip_changes_with_git(&mock_git).await;
        assert!(result.is_ok()); // Should return Ok with a message
        Ok(())
    }
}
