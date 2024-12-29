use crate::utils::{git_username_with_git, Git};
use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};

pub struct DeleteOptions {
    pub branch_name: Option<String>,
    pub all: bool,
    pub force: bool,
    pub local_only: bool,
}

/// Deletes one or more WIP branches.
///
/// # Arguments
/// * `git` - Git implementation to use for operations
/// * `options` - Configuration for the delete operation
///
/// # Features
/// * Interactive branch selection if no branch specified
/// * Confirmation prompt (unless force flag used)
/// * Handles both local and remote deletion
/// * Can delete all user's WIP branches
pub async fn delete_wip_branches(git: &impl Git, options: DeleteOptions) -> Result<()> {
    delete_wip_branches_with_git(git, options).await
}

pub async fn delete_wip_branches_with_git(git: &impl Git, options: DeleteOptions) -> Result<()> {
    let username = git_username_with_git(git).await?;
    let wip_branches = git.get_user_wip_branches(&username).await?;

    if wip_branches.is_empty() {
        println!("No WIP branches found for the user: {}", username);
        return Ok(());
    }

    let branches_to_delete = if options.all {
        if !options.force {
            let confirm = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("Delete all {} WIP branches?", wip_branches.len()))
                .interact()?;

            if !confirm {
                println!("Operation cancelled");
                return Ok(());
            }
        }
        wip_branches
    } else if let Some(branch) = options.branch_name {
        if !wip_branches.contains(&branch) {
            anyhow::bail!("Branch '{}' not found", branch);
        }
        if !options.force {
            let confirm = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Delete this branch?")
                .interact()?;

            if !confirm {
                println!("Operation cancelled");
                return Ok(());
            }
        }
        vec![branch]
    } else if wip_branches.len() == 1 {
        // For a single branch, use a simple confirm dialog
        let branch = &wip_branches[0];
        println!("\nFound WIP branch:");
        println!("  • {}\n", branch);

        if !options.force {
            let confirm = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Delete this branch?")
                .interact()?;

            if !confirm {
                println!("Operation cancelled");
                return Ok(());
            }
        }
        wip_branches
    } else {
        // Multiple branches - use multi-select
        println!("\nSelect WIP branches to delete:");
        println!("‣ Space to select/unselect branches");
        println!("‣ Enter to confirm selection");
        println!("‣ Esc to cancel\n");

        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("WIP branches")
            .items(&wip_branches)
            .defaults(&vec![false; wip_branches.len()])
            .interact()?;

        if selections.is_empty() {
            println!("No branches selected, operation cancelled");
            return Ok(());
        }

        // Show what's selected before confirmation
        let selected_branches: Vec<_> = selections
            .iter()
            .map(|&i| wip_branches[i].clone())
            .collect();

        println!("\nSelected branches:");
        for branch in &selected_branches {
            println!("  • {}", branch);
        }
        println!();

        selected_branches
    };

    // Ask about remote deletion if not specified
    let delete_remote = if !options.local_only {
        let remotes = git.execute(vec!["remote".to_string()]).await?;
        if remotes.contains("origin") {
            if options.force {
                true
            } else {
                Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(format!(
                        "Also delete {} branch(es) from remote?",
                        branches_to_delete.len()
                    ))
                    .interact()?
            }
        } else {
            false
        }
    } else {
        false
    };

    // Delete branches
    for branch in &branches_to_delete {
        // Delete local branch
        git.execute(vec!["branch".to_string(), "-D".to_string(), branch.clone()])
            .await
            .context(format!("Failed to delete local branch '{}'", branch))?;

        // Delete remote branch if requested
        if delete_remote {
            match git
                .execute(vec![
                    "push".to_string(),
                    "origin".to_string(),
                    "--delete".to_string(),
                    branch.clone(),
                ])
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    // Log the error but continue execution
                    eprintln!(
                        "Warning: Failed to delete remote branch '{}' (it may not exist): {}",
                        branch, e
                    );
                }
            }
        }

        println!(
            "Deleted branch '{}'{}",
            branch,
            if delete_remote {
                " (local and remote)"
            } else {
                " (local only)"
            }
        );
    }

    println!(
        "Successfully deleted {} branch(es){}",
        branches_to_delete.len(),
        if delete_remote {
            " from local and remote"
        } else {
            " from local"
        }
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::MockGit;
    use std::sync::Once;

    // Setup to disable terminal UI during tests
    static INIT: Once = Once::new();
    fn setup() {
        INIT.call_once(|| {
            dialoguer::console::set_colors_enabled(false);
        });
    }

    #[tokio::test]
    async fn test_delete_single_branch() -> Result<()> {
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

        let options = DeleteOptions {
            branch_name: Some("wip/test-user/branch1".to_string()),
            all: false,
            force: true,
            local_only: false,
        };

        delete_wip_branches_with_git(&mock_git, options).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_all_branches() -> Result<()> {
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
            .returning(|_| {
                Ok(vec![
                    "wip/test-user/branch1".to_string(),
                    "wip/test-user/branch2".to_string(),
                ])
            });

        // Mock local branch deletions
        for branch in ["wip/test-user/branch1", "wip/test-user/branch2"] {
            mock_git
                .expect_execute()
                .with(mockall::predicate::eq(vec![
                    "branch".to_string(),
                    "-D".to_string(),
                    branch.to_string(),
                ]))
                .returning(move |_| Ok(format!("Deleted branch '{}'", branch)));
        }

        // Mock remote check
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec!["remote".to_string()]))
            .returning(|_| Ok("origin".to_string()));

        // Mock remote branch deletions
        for branch in ["wip/test-user/branch1", "wip/test-user/branch2"] {
            mock_git
                .expect_execute()
                .with(mockall::predicate::eq(vec![
                    "push".to_string(),
                    "origin".to_string(),
                    "--delete".to_string(),
                    branch.to_string(),
                ]))
                .returning(|_| Ok("".to_string()));
        }

        let options = DeleteOptions {
            branch_name: None,
            all: true,
            force: true,
            local_only: false,
        };

        delete_wip_branches_with_git(&mock_git, options).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_nonexistent_branch() -> Result<()> {
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
            .returning(|_| Ok(vec!["wip/test-user/existing-branch".to_string()]));

        let options = DeleteOptions {
            branch_name: Some("wip/test-user/nonexistent".to_string()),
            all: false,
            force: true,
            local_only: false,
        };

        let result = delete_wip_branches_with_git(&mock_git, options).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
        Ok(())
    }

    #[tokio::test]
    async fn test_local_only_delete() -> Result<()> {
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

        // Mock only local branch deletion
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "branch".to_string(),
                "-D".to_string(),
                "wip/test-user/branch1".to_string(),
            ]))
            .returning(|_| Ok("Deleted branch".to_string()));

        let options = DeleteOptions {
            branch_name: Some("wip/test-user/branch1".to_string()),
            all: false,
            force: true,
            local_only: true,
        };

        delete_wip_branches_with_git(&mock_git, options).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_interactive_delete() -> Result<()> {
        setup();
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

        let options = DeleteOptions {
            branch_name: None,
            all: false,
            force: true,
            local_only: false,
        };

        delete_wip_branches_with_git(&mock_git, options).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_interactive_delete_with_mock_dialoguer() -> Result<()> {
        setup();
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

        let options = DeleteOptions {
            branch_name: None,
            all: false,
            force: true,
            local_only: false,
        };

        delete_wip_branches_with_git(&mock_git, options).await?;
        Ok(())
    }
}
