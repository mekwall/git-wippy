use crate::utils::Git;
use anyhow::Result;

pub async fn git_username_with_git(git: &impl Git) -> Result<String> {
    let username = git
        .execute(vec!["config".to_string(), "user.name".to_string()])
        .await?
        .trim()
        .to_string();

    if username.is_empty() {
        anyhow::bail!("Git username not found. Please configure your git user.name");
    }

    Ok(username)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::MockGit;

    #[tokio::test]
    async fn test_git_username_success() -> Result<()> {
        let mut mock_git = MockGit::new();
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "config".to_string(),
                "user.name".to_string(),
            ]))
            .returning(|_| Ok("Test User".to_string()));

        let username = git_username_with_git(&mock_git).await?;
        assert_eq!(username, "Test User");
        Ok(())
    }

    #[tokio::test]
    async fn test_git_username_empty() -> Result<()> {
        let mut mock_git = MockGit::new();
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "config".to_string(),
                "user.name".to_string(),
            ]))
            .returning(|_| Ok("".to_string()));

        let result = git_username_with_git(&mock_git).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
        Ok(())
    }

    #[tokio::test]
    async fn test_git_username_whitespace() -> Result<()> {
        let mut mock_git = MockGit::new();
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "config".to_string(),
                "user.name".to_string(),
            ]))
            .returning(|_| Ok("   ".to_string()));

        let result = git_username_with_git(&mock_git).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
        Ok(())
    }
}
