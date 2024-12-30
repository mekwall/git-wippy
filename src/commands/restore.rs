use crate::i18n::t_with_args;
use crate::output::Output;
use crate::utils::{git_username_with_git, parse_commit_message, Git, GitCommand};
use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Select};

pub struct RestoreOptions {
    pub branch_name: Option<String>,
    #[allow(dead_code)]
    pub force: bool,
    pub autostash: bool,
}

/// Restores changes from a WIP branch back to its original source branch.
///
/// # Arguments
/// * `options` - Configuration for the restore operation
///   - `branch_name`: Optional name of the branch to restore
///   - `force`: Skip confirmation prompts
///   - `autostash`: Automatically stash and reapply local changes
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
pub async fn restore_wip_changes(options: RestoreOptions) -> Result<()> {
    let git = GitCommand::new();
    restore_wip_changes_with_git(&git, options).await
}

/// Implementation that accepts a Git instance for better testability
pub async fn restore_wip_changes_with_git(git: &impl Git, options: RestoreOptions) -> Result<()> {
    let output = Output::new().await?;
    let username = git_username_with_git(git).await?;
    let wip_branches = git.get_user_wip_branches(&username).await?;

    let selected_branch = if let Some(branch) = options.branch_name {
        if !wip_branches.contains(&branch) {
            let message = t_with_args("branch-not-found", &[("name", &branch)]);
            output.info(&output.format_with_highlights(&message, &[&format!("'{}'", branch)]))?;
            return Ok(());
        }
        branch
    } else if wip_branches.len() > 1 {
        get_user_selection(&wip_branches).await?
    } else if let Some(branch) = wip_branches.first() {
        branch.clone()
    } else {
        let message = t_with_args("no-wip-branches", &[("username", &username)]);
        output.info(&output.format_with_highlights(&message, &[&username]))?;
        return Ok(());
    };

    // Get the last commit message from the WIP branch
    let commit_message = git.get_commit_message(&selected_branch).await?;
    let (source_branch, staged_files, changed_files, untracked_files) =
        parse_commit_message(&commit_message);

    let message = t_with_args("restoring-wip", &[("name", &selected_branch)]);
    output.info(&output.format_with_highlights(&message, &[&format!("'{}'", selected_branch)]))?;

    // Check for local changes
    let has_changes = !git.get_staged_files().await?.is_empty()
        || !git.get_changed_files().await?.is_empty()
        || !git.get_untracked_files().await?.is_empty();

    if has_changes && !options.autostash {
        return Err(anyhow::anyhow!(
            "You have local changes. Please commit or stash them, or use --autostash"
        ));
    }

    // Stash any existing changes if autostash is enabled
    if has_changes && options.autostash {
        output.info(&t_with_args("stashing-existing-changes", &[]))?;
        // Create a unique stash name for the local changes
        let stash_name = format!("git-wippy-autostash-{}", source_branch);
        git.execute(vec![
            "stash".to_string(),
            "push".to_string(),
            "--include-untracked".to_string(),
            "-m".to_string(),
            stash_name.clone(),
        ])
        .await
        .context("Failed to stash changes")?;
    }

    // Determine if the source branch exists, create it if not
    if git.branch_exists(&source_branch).await? {
        git.checkout(&source_branch).await?;
        let message = t_with_args("checked-out-branch", &[("name", &source_branch)]);
        output
            .info(&output.format_with_highlights(&message, &[&format!("'{}'", source_branch)]))?;
    } else {
        git.create_branch(&source_branch).await?;
        let message = t_with_args("created-branch", &[("name", &source_branch)]);
        output
            .info(&output.format_with_highlights(&message, &[&format!("'{}'", source_branch)]))?;
    }

    // Get the list of files in the WIP branch
    let files_output = git
        .execute(vec![
            "ls-tree".to_string(),
            "-r".to_string(),
            "--name-only".to_string(),
            selected_branch.clone(),
        ])
        .await?;
    let files: Vec<String> = files_output.lines().map(|s| s.to_string()).collect();

    // For each file in the WIP branch, get its contents and write it
    for file in files {
        let _content = git
            .execute(vec![
                "show".to_string(),
                format!("{}:{}", selected_branch, file),
            ])
            .await?;
        git.execute(vec![
            "checkout".to_string(),
            selected_branch.clone(),
            "--".to_string(),
            file.clone(),
        ])
        .await?;
    }
    output.info(&t_with_args("applied-changes", &[]))?;

    // Recreate the original state of files based on the parsed commit message
    recreate_file_states(git, staged_files, changed_files, untracked_files).await?;
    output.info(&t_with_args("recreated-file-states", &[]))?;

    // Pop any previously stashed changes if autostash was used
    if has_changes && options.autostash {
        output.info(&t_with_args("restoring-existing-changes", &[]))?;
        let stash_name = format!("git-wippy-autostash-{}", source_branch);

        // Try to find the stash index by listing all stashes and searching for our name
        let stash_list = git
            .execute(vec!["stash".to_string(), "list".to_string()])
            .await
            .context("Failed to list stashes")?;

        // Find the stash by looking for the message in the stash list
        // The stash list format is: stash@{n}: WIP on branch: message
        let stash_index = stash_list
            .lines()
            .position(|line| line.contains(&format!(": {}", stash_name)))
            .ok_or_else(|| anyhow::anyhow!("Could not find stash with name: {}", stash_name))?;
        let stash_ref = format!("stash@{{{}}}", stash_index);

        // Create a temporary branch from the current state
        let temp_branch = format!("git-wippy-temp-{}", source_branch);
        git.execute(vec![
            "checkout".to_string(),
            "-b".to_string(),
            temp_branch.clone(),
        ])
        .await
        .context("Failed to create temporary branch")?;

        // Apply the stash to the temporary branch
        let apply_result = git
            .execute(vec![
                "stash".to_string(),
                "apply".to_string(),
                stash_ref.clone(),
            ])
            .await;

        // Switch back to the target branch
        git.execute(vec!["checkout".to_string(), source_branch.clone()])
            .await
            .context("Failed to switch back to source branch")?;

        match apply_result {
            Ok(_) => {
                // Try to merge the temporary branch
                let merge_result = git
                    .execute(vec![
                        "merge".to_string(),
                        "--no-commit".to_string(), // Don't create a merge commit
                        "--no-ff".to_string(),     // Always create a merge to handle conflicts
                        temp_branch.clone(),
                    ])
                    .await;

                // Clean up the temporary branch
                git.execute(vec![
                    "branch".to_string(),
                    "-D".to_string(),
                    temp_branch.clone(),
                ])
                .await
                .context("Failed to delete temporary branch")?;

                match merge_result {
                    Ok(_) => {
                        // Drop the stash if we successfully applied it
                        git.execute(vec![
                            "stash".to_string(),
                            "drop".to_string(),
                            stash_ref.clone(),
                        ])
                        .await
                        .context("Failed to drop stash")?;
                        output.info(&t_with_args("applied-stash", &[]))?;
                    }
                    Err(e) => {
                        // Don't fail on conflicts, let the user handle them
                        if !e.to_string().contains("conflict") {
                            return Err(anyhow::anyhow!(
                                "Failed to restore existing changes: {}",
                                e
                            ));
                        }
                    }
                }
            }
            Err(e) => {
                // Clean up the temporary branch
                git.execute(vec![
                    "branch".to_string(),
                    "-D".to_string(),
                    temp_branch.clone(),
                ])
                .await
                .context("Failed to delete temporary branch")?;
                return Err(anyhow::anyhow!("Failed to apply stashed changes: {}", e));
            }
        }
    }

    // Now that we've successfully applied all changes, we can delete the WIP branch
    git.delete_branch(&selected_branch, true).await?;
    let message = t_with_args("deleted-local-branch", &[("name", &selected_branch)]);
    output.info(&output.format_with_highlights(&message, &[&format!("'{}'", selected_branch)]))?;

    // Delete the remote branch if it exists
    let remotes = git.get_remotes().await?;
    if remotes.contains(&"origin".to_string()) {
        git.delete_remote_branch("origin", &selected_branch).await?;
        let message = t_with_args("deleted-remote-branch", &[("name", &selected_branch)]);
        output
            .info(&output.format_with_highlights(&message, &[&format!("'{}'", selected_branch)]))?;
    }

    let message = t_with_args("restore-complete", &[("name", &selected_branch)]);
    output.info(&output.format_with_highlights(&message, &[&format!("'{}'", selected_branch)]))?;

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
    // Stage and unstage files using the Git trait methods
    git.stage_files(&staged_files).await?;
    git.unstage_files(&changed_files).await?;
    git.unstage_files(&untracked_files).await?;

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
            .expect_stage_files()
            .with(mockall::predicate::eq(vec![
                "staged1.txt".to_string(),
                "staged2.txt".to_string(),
            ]))
            .returning(|_| Ok(()));

        // Mock unstaging changed files
        mock_git
            .expect_unstage_files()
            .with(mockall::predicate::eq(vec![
                "changed1.txt".to_string(),
                "changed2.txt".to_string(),
            ]))
            .returning(|_| Ok(()));

        // Mock unstaging untracked files
        mock_git
            .expect_unstage_files()
            .with(mockall::predicate::eq(vec![
                "untracked1.txt".to_string(),
                "untracked2.txt".to_string(),
            ]))
            .returning(|_| Ok(()));

        // Test the function with multiple files in each category
        recreate_file_states(
            &mock_git,
            vec!["staged1.txt".to_string(), "staged2.txt".to_string()],
            vec!["changed1.txt".to_string(), "changed2.txt".to_string()],
            vec!["untracked1.txt".to_string(), "untracked2.txt".to_string()],
        )
        .await?;

        Ok(())
    }
}
