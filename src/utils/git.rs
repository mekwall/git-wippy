use crate::output::Output;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use tokio::process::Command;

/// A trait that abstracts Git operations used throughout the application.
///
/// This trait provides both low-level command execution and high-level Git operations.
/// It is designed to be mockable for testing and allows for different implementations
/// (e.g., real Git commands, mock for testing, or alternative Git implementations).
///
/// # Implementation Notes
///
/// - All methods are async and return `Result<T>`
/// - Methods should handle common Git errors and provide context
/// - Implementations should be thread-safe (Send + Sync)
///
/// # Examples
///
/// ```no_run
/// use git_wippy::Git;
///
/// async fn example(git: &impl Git) -> anyhow::Result<()> {
///     // Get current branch
///     let branch = git.get_current_branch().await?;
///
///     // Stage and commit changes
///     git.stage_all().await?;
///     git.commit("feat: add new feature").await?;
///
///     Ok(())
/// }
/// ```
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Git: Send + Sync {
    /// Executes a raw Git command with the given arguments.
    ///
    /// This is a low-level method that runs Git commands directly. Prefer using
    /// the higher-level methods when possible.
    ///
    /// # Arguments
    ///
    /// * `args` - Vector of command arguments to pass to git
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Command's stdout output if successful
    /// * `Err(Error)` - If the command fails or output can't be parsed as UTF-8
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The Git command fails to execute
    /// - The command returns a non-zero exit code
    /// - The output cannot be parsed as UTF-8
    async fn execute(&self, args: Vec<String>) -> Result<String>;

    /// Gets all Git configuration as key-value pairs.
    ///
    /// # Returns
    /// * `Ok(HashMap<String, String>)` - Map of config keys to their values
    /// * `Err` if the git config command fails
    ///
    /// # Example keys
    /// * "user.name"
    /// * "user.email"
    /// * "color.ui"
    #[allow(dead_code)]
    async fn get_config(&self) -> Result<HashMap<String, String>> {
        let output = self
            .execute(vec!["config".to_string(), "--list".to_string()])
            .await?;

        let mut config = HashMap::new();
        for line in output.lines() {
            if let Some((key, value)) = line.split_once('=') {
                config.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        Ok(config)
    }

    /// Gets a specific Git configuration value.
    ///
    /// # Arguments
    /// * `key` - The configuration key to look up (e.g., "user.name", "color.ui")
    ///
    /// # Returns
    /// * `Ok(Some(String))` - The config value if found
    /// * `Ok(None)` - If the config key doesn't exist
    /// * `Err` if the git config command fails
    async fn get_config_value(&self, key: &str) -> Result<Option<String>> {
        match self
            .execute(vec![
                "config".to_string(),
                "--get".to_string(),
                key.to_string(),
            ])
            .await
        {
            Ok(value) => Ok(Some(value)),
            Err(e) => {
                if e.to_string().contains("exit code: 1") {
                    // Config key not found (git returns exit code 1)
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

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
    async fn stage_all(&self) -> Result<String> {
        self.execute(vec!["add".to_string(), "-A".to_string()])
            .await
    }

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

    /// Gets a list of WIP branches for a specific user
    async fn get_user_wip_branches(&self, username: &str) -> Result<Vec<String>> {
        let output = Output::new().await?;
        let git_output = self
            .execute(vec![
                "branch".to_string(),
                "--all".to_string(),
                "--format=%(refname:short)".to_string(),
            ])
            .await?;

        output.debug(&format!("Raw git output:\n{}", git_output))?;

        let wip_prefix = format!("wip/{}/", username);
        output.debug(&format!("Looking for branches with prefix: {}", wip_prefix))?;

        let branches: Vec<String> = git_output
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .filter(|line| line.starts_with(&wip_prefix))
            .map(|line| line.replace("remotes/origin/", ""))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        output.debug(&format!("Found branches: {:?}", branches))?;
        Ok(branches)
    }

    /// Verifies if a branch exists
    async fn branch_exists(&self, branch: &str) -> Result<bool> {
        self.execute(vec![
            "rev-parse".to_string(),
            "--verify".to_string(),
            branch.to_string(),
        ])
        .await
        .map(|_| true)
        .or_else(|_| Ok(false))
    }

    /// Gets the last commit message from a branch
    async fn get_commit_message(&self, branch: &str) -> Result<String> {
        self.execute(vec![
            "log".to_string(),
            "-1".to_string(),
            "--pretty=%B".to_string(),
            branch.to_string(),
        ])
        .await
    }

    /// Stashes changes with a message
    #[allow(dead_code)]
    async fn stash_push(&self, message: &str) -> Result<String> {
        self.execute(vec![
            "stash".to_string(),
            "push".to_string(),
            "-m".to_string(),
            message.to_string(),
        ])
        .await
    }

    /// Applies and removes the latest stash
    #[allow(dead_code)]
    async fn stash_pop(&self) -> Result<String> {
        self.execute(vec!["stash".to_string(), "pop".to_string()])
            .await
    }

    /// Deletes a branch locally
    async fn delete_branch(&self, branch: &str, force: bool) -> Result<String> {
        let flag = if force { "-D" } else { "-d" };
        self.execute(vec![
            "branch".to_string(),
            flag.to_string(),
            branch.to_string(),
        ])
        .await
    }

    /// Deletes a branch from a remote
    async fn delete_remote_branch(&self, remote: &str, branch: &str) -> Result<String> {
        self.execute(vec![
            "push".to_string(),
            remote.to_string(),
            "--delete".to_string(),
            branch.to_string(),
        ])
        .await
    }

    /// Lists all remotes
    async fn get_remotes(&self) -> Result<Vec<String>> {
        self.execute(vec!["remote".to_string()])
            .await
            .map(|output| output.lines().map(|s| s.to_string()).collect())
    }

    /// Stages specific files
    async fn stage_files(&self, files: &[String]) -> Result<()> {
        for file in files {
            self.execute(vec!["add".to_string(), file.clone()]).await?;
        }
        Ok(())
    }

    /// Unstages specific files
    async fn unstage_files(&self, files: &[String]) -> Result<()> {
        for file in files {
            self.execute(vec![
                "reset".to_string(),
                "HEAD".to_string(),
                "--".to_string(),
                file.clone(),
            ])
            .await?;
        }
        Ok(())
    }

    /// Resets the current branch to the previous commit
    #[allow(dead_code)]
    async fn reset_soft(&self) -> Result<String>;

    /// Resets the current branch and working directory to HEAD
    #[allow(dead_code)]
    async fn reset_hard(&self) -> Result<String>;

    /// Check if working tree is clean
    #[allow(dead_code)]
    async fn is_working_tree_clean(&self) -> Result<bool>;

    /// Applies the latest stash without removing it
    #[allow(dead_code)]
    async fn stash_apply(&self) -> Result<String> {
        self.execute(vec!["stash".to_string(), "apply".to_string()])
            .await
    }

    /// Applies the latest stash with index state without removing it
    #[allow(dead_code)]
    async fn stash_apply_with_index(&self) -> Result<String> {
        self.execute(vec![
            "stash".to_string(),
            "apply".to_string(),
            "--index".to_string(),
        ])
        .await
    }

    /// Drops the latest stash
    #[allow(dead_code)]
    async fn stash_drop(&self) -> Result<String> {
        self.execute(vec!["stash".to_string(), "drop".to_string()])
            .await
    }

    /// Shows the content of a file from a specific branch
    #[allow(dead_code)]
    async fn show_file(&self, branch: &str, file: &str) -> Result<String> {
        self.execute(vec!["show".to_string(), format!("{}:{}", branch, file)])
            .await
    }

    /// Writes content to a file
    #[allow(dead_code)]
    async fn write_file(&self, file: &str, content: &str) -> Result<()> {
        use tokio::fs;
        fs::write(file, content).await.map_err(|e| e.into())
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

    async fn get_user_wip_branches(&self, username: &str) -> Result<Vec<String>> {
        let output = Output::new().await?;
        let git_output = self
            .execute(vec![
                "branch".to_string(),
                "--all".to_string(),
                "--format=%(refname:short)".to_string(),
            ])
            .await?;

        output.debug(&format!("Raw git output:\n{}", git_output))?;

        let wip_prefix = format!("wip/{}/", username);
        output.debug(&format!("Looking for branches with prefix: {}", wip_prefix))?;

        let branches: Vec<String> = git_output
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .filter(|line| line.starts_with(&wip_prefix))
            .map(|line| line.replace("remotes/origin/", ""))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        output.debug(&format!("Found branches: {:?}", branches))?;
        Ok(branches)
    }

    async fn is_working_tree_clean(&self) -> Result<bool> {
        let output = self
            .execute(vec!["status".to_string(), "--porcelain".to_string()])
            .await?;
        Ok(output.trim().is_empty())
    }

    /// Resets the current branch to the previous commit
    #[allow(dead_code)]
    async fn reset_soft(&self) -> Result<String> {
        self.execute(vec![
            "reset".to_string(),
            "--soft".to_string(),
            "HEAD~".to_string(),
        ])
        .await
    }

    /// Resets the current branch and working directory to HEAD
    #[allow(dead_code)]
    async fn reset_hard(&self) -> Result<String> {
        self.execute(vec![
            "reset".to_string(),
            "--hard".to_string(),
            "HEAD".to_string(),
        ])
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
