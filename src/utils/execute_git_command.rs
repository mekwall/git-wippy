use anyhow::{Context, Result};
use std::str;
use tokio::process::Command;

/// Asynchronously executes a Git command and returns its standard output as a String.
///
/// # Arguments
///
/// * `args` - A slice of &str representing the Git command and its arguments.
///
/// # Returns
///
/// A Result<String>, which is Ok containing the command's standard output as a string
/// if the command succeeds, or an Err otherwise.
///
/// # Examples
///
/// ```
/// let branch_list = execute_git_command(&["branch", "-a"]).await?;
/// println!("Branches: {}", branch_list);
/// ```
pub async fn execute_git_command(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .await
        .context(format!("Failed to execute git command: {:?}", args))?;

    if !output.status.success() {
        let stderr = str::from_utf8(&output.stderr)
            .context("Failed to read stderr")?
            .trim();

        return Err(anyhow::anyhow!("Git command {:?} failed: {}", args, stderr));
    }

    Ok(str::from_utf8(&output.stdout)?.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_git_command_success() {
        let output = execute_git_command(&["branch", "-a"]).await;
        assert!(output.is_ok());
    }

    #[tokio::test]
    async fn test_execute_git_command_failure() {
        let output = execute_git_command(&["invalid-command"]).await;
        assert!(output.is_err());
    }
}
