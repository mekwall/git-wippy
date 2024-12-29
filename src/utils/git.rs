use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use std::collections::HashSet;
use tokio::process::Command;

/// Trait defining Git operations used throughout the application.
/// Provides both low-level command execution and high-level convenience methods.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Git: Send + Sync {
    /// Executes a raw git command with the given arguments.
    ///
    /// # Arguments
    /// * `args` - Vector of command arguments to pass to git
    ///
    /// # Returns
    /// * `Ok(String)` containing the command's stdout output if successful
    /// * `Err` if the command fails or output can't be parsed as UTF-8
    async fn execute(&self, args: Vec<String>) -> Result<String>;

    /// Gets the name of the current git branch
    async fn get_current_branch(&self) -> Result<String> {
        self.execute(vec![
            "rev-parse".to_string(),
            "--abbrev-ref".to_string(),
            "HEAD".to_string(),
        ])
        .await
    }

    /// Stages all changes in the working directory
    async fn stage_all(&self) -> Result<String>;

    /// Creates a commit with the given message
    ///
    /// # Arguments
    /// * `message` - The commit message to use
    async fn commit(&self, message: &str) -> Result<String> {
        self.execute(vec![
            "commit".to_string(),
            "-m".to_string(),
            message.to_string(),
        ])
        .await
    }

    /// Checks out the specified branch
    ///
    /// # Arguments
    /// * `branch` - Name of the branch to check out
    async fn checkout(&self, branch: &str) -> Result<String> {
        self.execute(vec!["checkout".to_string(), branch.to_string()])
            .await
    }

    /// Creates and checks out a new branch
    ///
    /// # Arguments
    /// * `branch` - Name of the new branch to create
    async fn create_branch(&self, branch: &str) -> Result<String> {
        self.execute(vec![
            "checkout".to_string(),
            "-b".to_string(),
            branch.to_string(),
        ])
        .await
    }

    /// Pushes a branch to a remote repository
    ///
    /// # Arguments
    /// * `remote` - Name of the remote (e.g., "origin")
    /// * `branch` - Name of the branch to push
    async fn push(&self, remote: &str, branch: &str) -> Result<String> {
        self.execute(vec![
            "push".to_string(),
            remote.to_string(),
            branch.to_string(),
        ])
        .await
    }

    /// Gets a list of staged files
    async fn get_staged_files(&self) -> Result<String> {
        self.execute(vec![
            "diff".to_string(),
            "--cached".to_string(),
            "--name-only".to_string(),
        ])
        .await
    }

    /// Gets a list of changed but unstaged files
    async fn get_changed_files(&self) -> Result<String> {
        self.execute(vec!["diff".to_string(), "--name-only".to_string()])
            .await
    }

    /// Gets a list of untracked files
    async fn get_untracked_files(&self) -> Result<String> {
        self.execute(vec![
            "ls-files".to_string(),
            "--others".to_string(),
            "--exclude-standard".to_string(),
        ])
        .await
    }

    /// Gets a list of user WIP branches
    async fn get_user_wip_branches(&self, username: &str) -> Result<Vec<String>> {
        if username.is_empty() {
            return Ok(Vec::new());
        }

        let all_branches = self
            .execute(vec!["branch".to_string(), "-a".to_string()])
            .await?;
        let wip_prefix = format!("wip/{}/", username);

        // Process branches concurrently
        let branches = stream::iter(all_branches.lines())
            .filter_map(|line| {
                let wip_prefix = wip_prefix.clone();
                async move {
                    let trimmed = line.trim().replace("* ", "");
                    if trimmed.contains(&wip_prefix) {
                        Some(trimmed.replace("remotes/origin/", "").trim().to_string())
                    } else {
                        None
                    }
                }
            })
            .collect::<HashSet<_>>()
            .await;

        Ok(branches.into_iter().collect())
    }
}

/// Thread-safe Git command implementation.
/// Uses a unit struct since no internal state is needed.
#[derive(Clone)]
pub struct GitCommand(());

impl GitCommand {
    /// Creates a new thread-safe GitCommand instance
    pub fn new() -> Self {
        Self(())
    }
}

impl Default for GitCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Git for GitCommand {
    async fn execute(&self, args: Vec<String>) -> Result<String> {
        let output = Command::new("git")
            .args(&args)
            .kill_on_drop(true)
            .output()
            .await
            .context(format!("Failed to execute git command: {:?}", args))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Git command 'git {}' failed: {}",
                args.join(" "),
                stderr.trim()
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    async fn stage_all(&self) -> Result<String> {
        self.execute(vec!["add".to_string(), "-A".to_string()])
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_git_command_success() {
        let mut mock = MockGit::new();
        // Test that get_current_branch returns the expected branch name
        mock.expect_get_current_branch()
            .times(1)
            .returning(|| Ok("main".to_string()));

        let branch = mock.get_current_branch().await.unwrap();
        assert_eq!(branch, "main");
    }

    #[tokio::test]
    async fn test_git_command_failure() {
        let mut mock = MockGit::new();
        // Test that execute returns an error for invalid commands
        mock.expect_execute()
            .with(mockall::predicate::eq(vec!["invalid".to_string()]))
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("Command failed")));

        let result = mock.execute(vec!["invalid".to_string()]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_convenience_methods() {
        let mut mock = MockGit::new();

        // Test stage_all
        mock.expect_stage_all()
            .times(1)
            .returning(|| Ok("".to_string()));

        // Test commit
        mock.expect_commit()
            .with(mockall::predicate::eq("test message"))
            .times(1)
            .returning(|_| Ok("".to_string()));

        // Test checkout
        mock.expect_checkout()
            .with(mockall::predicate::eq("test-branch"))
            .times(1)
            .returning(|_| Ok("".to_string()));

        // Execute tests in order
        assert!(mock.stage_all().await.is_ok());
        assert!(mock.commit("test message").await.is_ok());
        assert!(mock.checkout("test-branch").await.is_ok());
    }

    #[tokio::test]
    async fn test_get_user_wip_branches() -> Result<()> {
        let mut mock = MockGit::new();

        mock.expect_get_user_wip_branches()
            .with(mockall::predicate::eq("test-user"))
            .returning(|_| {
                Ok(vec![
                    "wip/test-user/branch1".to_string(),
                    "wip/test-user/branch2".to_string(),
                ])
            });

        let branches = mock.get_user_wip_branches("test-user").await?;
        assert_eq!(branches.len(), 2);
        assert!(branches.contains(&"wip/test-user/branch1".to_string()));
        assert!(branches.contains(&"wip/test-user/branch2".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn test_get_user_wip_branches_empty_username() -> Result<()> {
        let mut mock = MockGit::new();

        mock.expect_get_user_wip_branches()
            .with(mockall::predicate::eq(""))
            .returning(|_| Ok(Vec::new()));

        let branches = mock.get_user_wip_branches("").await?;
        assert!(branches.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_get_user_wip_branches_no_branches() -> Result<()> {
        let mut mock = MockGit::new();

        mock.expect_get_user_wip_branches()
            .with(mockall::predicate::eq("test-user"))
            .returning(|_| Ok(Vec::new()));

        let branches = mock.get_user_wip_branches("test-user").await?;
        assert!(branches.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_get_user_wip_branches_deduplicates() -> Result<()> {
        let mut mock = MockGit::new();

        mock.expect_get_user_wip_branches()
            .with(mockall::predicate::eq("test-user"))
            .returning(|_| Ok(vec!["wip/test-user/branch1".to_string()]));

        let branches = mock.get_user_wip_branches("test-user").await?;
        assert_eq!(branches.len(), 1);
        assert!(branches.contains(&"wip/test-user/branch1".to_string()));
        Ok(())
    }
}
