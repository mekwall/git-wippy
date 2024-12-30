use crate::i18n::t_with_args;
use crate::output::Output;
use crate::utils::{git_username_with_git, Git, GitCommand};
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
/// * `options` - Configuration for the delete operation
///
/// # Features
/// * Interactive branch selection if no branch specified
/// * Confirmation prompt (unless force flag used)
/// * Handles both local and remote deletion
/// * Can delete all user's WIP branches
pub async fn delete_wip_branches(options: DeleteOptions) -> Result<()> {
    let git = GitCommand::new();
    delete_wip_branches_with_git(&git, options).await
}

pub async fn delete_wip_branches_with_git(git: &impl Git, options: DeleteOptions) -> Result<()> {
    let output = Output::new().await?;
    let username = git_username_with_git(git).await?;
    let wip_branches = git.get_user_wip_branches(&username).await?;

    if wip_branches.is_empty() {
        let message = t_with_args("no-wip-branches", &[("username", &username)]);
        output.info(&output.format_with_highlights(&message, &[&username]))?;
        return Ok(());
    }

    let branches_to_delete = if options.all {
        if !options.force {
            let message = t_with_args(
                "delete-all-prompt",
                &[("count", &wip_branches.len().to_string())],
            );
            let confirm = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(message)
                .interact()?;

            if !confirm {
                output.info(&t_with_args("operation-cancelled", &[]))?;
                return Ok(());
            }
        }
        wip_branches
    } else if let Some(branch) = options.branch_name {
        if !wip_branches.contains(&branch) {
            let message = t_with_args("branch-not-found", &[("name", &branch)]);
            output.info(&output.format_with_highlights(&message, &[&format!("'{}'", branch)]))?;
            return Ok(());
        }
        if !options.force {
            let confirm = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(t_with_args("delete-branch-prompt", &[]))
                .interact()?;

            if !confirm {
                output.info(&t_with_args("operation-cancelled", &[]))?;
                return Ok(());
            }
        }
        vec![branch]
    } else if wip_branches.len() == 1 {
        // For a single branch, use a simple confirm dialog
        let branch = &wip_branches[0];
        output.info(&t_with_args("found-wip-branch", &[]))?;
        output.info(
            &output.format_with_highlights(
                &t_with_args("branch-name", &[("name", branch)]),
                &[branch],
            ),
        )?;

        if !options.force {
            let confirm = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(t_with_args("delete-branch-prompt", &[]))
                .interact()?;

            if !confirm {
                output.info(&t_with_args("operation-cancelled", &[]))?;
                return Ok(());
            }
        }
        wip_branches
    } else {
        // Multiple branches - use multi-select
        output.info(&t_with_args("select-branches-to-delete", &[]))?;
        output.info(&t_with_args("selection-instructions", &[]))?;

        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("WIP branches")
            .items(&wip_branches)
            .defaults(&vec![false; wip_branches.len()])
            .interact()?;

        if selections.is_empty() {
            output.info(&t_with_args("no-branches-selected", &[]))?;
            return Ok(());
        }

        // Show what's selected before confirmation
        let selected_branches: Vec<_> = selections
            .iter()
            .map(|&i| wip_branches[i].clone())
            .collect();

        output.info(&t_with_args("selected-branches", &[]))?;
        for branch in &selected_branches {
            output.info(&output.format_with_highlights(
                &t_with_args("branch-name", &[("name", branch)]),
                &[branch],
            ))?;
        }

        selected_branches
    };

    // Ask about remote deletion if not specified
    let delete_remote = if !options.local_only {
        let remotes = git.get_remotes().await?;
        if remotes.contains(&"origin".to_string()) {
            if options.force {
                true
            } else {
                let count = branches_to_delete.len().to_string();
                Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(t_with_args("delete-remote-prompt", &[("count", &count)]))
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
        git.delete_branch(branch, true)
            .await
            .context(format!("Failed to delete local branch '{}'", branch))?;

        // Delete remote branch if requested
        if delete_remote {
            match git.delete_remote_branch("origin", branch).await {
                Ok(_) => {}
                Err(e) => {
                    let message = t_with_args(
                        "remote-delete-failed",
                        &[("name", branch), ("error", &e.to_string())],
                    );
                    output.error(
                        &output.format_with_highlights(&message, &[&format!("'{}'", branch)]),
                    )?;
                }
            }
        }

        let message = t_with_args(
            "wip-branch-deleted",
            &[
                ("name", branch),
                ("remote", if delete_remote { "true" } else { "false" }),
            ],
        );
        output.info(&output.format_with_highlights(&message, &[&format!("'{}'", branch)]))?;
    }

    let message = t_with_args(
        "delete-complete",
        &[
            ("count", &branches_to_delete.len().to_string()),
            ("remote", if delete_remote { "true" } else { "false" }),
        ],
    );
    output.info(&message)?;
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
            .expect_delete_branch()
            .with(
                mockall::predicate::eq("wip/test-user/branch1"),
                mockall::predicate::eq(true),
            )
            .returning(|_, _| Ok("Deleted branch".to_string()));

        // Mock remote check
        mock_git
            .expect_get_remotes()
            .returning(|| Ok(vec!["origin".to_string()]));

        // Mock remote branch deletion
        mock_git
            .expect_delete_remote_branch()
            .with(
                mockall::predicate::eq("origin"),
                mockall::predicate::eq("wip/test-user/branch1"),
            )
            .returning(|_, _| Ok("".to_string()));

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
                .expect_delete_branch()
                .with(mockall::predicate::eq(branch), mockall::predicate::eq(true))
                .returning(move |_, _| Ok(format!("Deleted branch '{}'", branch)));
        }

        // Mock remote check
        mock_git
            .expect_get_remotes()
            .returning(|| Ok(vec!["origin".to_string()]));

        // Mock remote branch deletions
        for branch in ["wip/test-user/branch1", "wip/test-user/branch2"] {
            mock_git
                .expect_delete_remote_branch()
                .with(
                    mockall::predicate::eq("origin"),
                    mockall::predicate::eq(branch),
                )
                .returning(|_, _| Ok("".to_string()));
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

        delete_wip_branches_with_git(&mock_git, options).await?;
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
            .expect_delete_branch()
            .with(
                mockall::predicate::eq("wip/test-user/branch1"),
                mockall::predicate::eq(true),
            )
            .returning(|_, _| Ok("Deleted branch".to_string()));

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
            .expect_delete_branch()
            .with(
                mockall::predicate::eq("wip/test-user/branch1"),
                mockall::predicate::eq(true),
            )
            .returning(|_, _| Ok("Deleted branch".to_string()));

        // Mock remote check
        mock_git
            .expect_get_remotes()
            .returning(|| Ok(vec!["origin".to_string()]));

        // Mock remote branch deletion
        mock_git
            .expect_delete_remote_branch()
            .with(
                mockall::predicate::eq("origin"),
                mockall::predicate::eq("wip/test-user/branch1"),
            )
            .returning(|_, _| Ok("".to_string()));

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
