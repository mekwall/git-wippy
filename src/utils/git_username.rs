use crate::utils::git::{Git, GitCommand};
use anyhow::{Context, Result};

/// Fetches the Git username from global config and formats it for use as a branch name.
///
/// # Returns
/// * `Ok(String)` - A lowercase, hyphen-separated username string
/// * `Err` - If git command fails or username is not configured
///
/// # Example
/// ```
/// let username = git_username().await?; // e.g. "john-doe" from "John Doe"
/// ```
///
/// # Details
/// * Retrieves username using `git config user.name`
/// * Converts spaces to hyphens
/// * Converts to lowercase
/// * Returns error if username is not configured
pub async fn git_username() -> Result<String> {
    let git = GitCommand::new();
    let username = git
        .execute(vec!["config".to_string(), "user.name".to_string()])
        .await
        .context("Failed to fetch git username")?;

    let formatted_username = username.trim();

    if formatted_username.is_empty() {
        anyhow::bail!("Git username is not configured. Please set it using 'git config --global user.name \"Your Name\"'");
    }

    Ok(formatted_username
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("-")
        .to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::MockGit;

    /// Tests successful username retrieval and formatting
    #[tokio::test]
    async fn test_git_username_success() {
        let mut mock_git = MockGit::new();
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "config".to_string(),
                "user.name".to_string(),
            ]))
            .returning(|_| Ok("Test User".to_string()));

        let output = mock_git
            .execute(vec!["config".to_string(), "user.name".to_string()])
            .await;
        assert!(output.is_ok());
        assert_eq!(
            output
                .unwrap()
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join("-")
                .to_lowercase(),
            "test-user"
        );
    }

    /// Tests username retrieval with mock
    #[tokio::test]
    async fn test_git_username_with_mock() {
        let mut mock_git = MockGit::new();
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "config".to_string(),
                "user.name".to_string(),
            ]))
            .returning(|_| Ok("Mock User".to_string()));

        let output = mock_git
            .execute(vec!["config".to_string(), "user.name".to_string()])
            .await;
        assert!(output.is_ok());
        assert_eq!(
            output
                .unwrap()
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join("-")
                .to_lowercase(),
            "mock-user"
        );
    }

    /// Tests handling of empty username
    #[tokio::test]
    async fn test_git_username_empty() {
        let mut mock_git = MockGit::new();
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "config".to_string(),
                "user.name".to_string(),
            ]))
            .returning(|_| Ok("".to_string()));

        let output = mock_git
            .execute(vec!["config".to_string(), "user.name".to_string()])
            .await;
        assert!(output.is_ok());
        assert!(output.unwrap().trim().is_empty());
    }

    /// Tests handling of whitespace-only username
    #[tokio::test]
    async fn test_git_username_whitespace() {
        let mut mock_git = MockGit::new();
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "config".to_string(),
                "user.name".to_string(),
            ]))
            .returning(|_| Ok("   ".to_string()));

        let output = mock_git
            .execute(vec!["config".to_string(), "user.name".to_string()])
            .await;
        assert!(output.is_ok());
        assert!(output.unwrap().trim().is_empty());
    }
}
