use crate::utils::git::Git;
use anyhow::Result;
use futures::stream::{self, StreamExt};
use std::collections::HashSet;

/// Retrieves a list of WIP branches for a specific user.
///
/// # Arguments
/// * `username` - The username to filter WIP branches for
/// * `git` - Implementation of the Git trait for executing commands
///
/// # Returns
/// * `Ok(Vec<String>)` - List of WIP branch names for the user
/// * `Err` - If git command fails or output can't be parsed
///
/// # Details
/// * Returns empty Vec if username is empty
/// * Filters branches starting with "wip/{username}/"
/// * Removes duplicates and remote branch prefixes
pub async fn get_user_wip_branches(username: &str, git: &impl Git) -> Result<Vec<String>> {
    if username.is_empty() {
        return Ok(Vec::new());
    }

    let all_branches = git
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::MockGit;

    /// Tests that WIP branches are correctly filtered and returned
    #[tokio::test]
    async fn test_get_user_wip_branches() -> Result<()> {
        let mut mock_git = MockGit::new();
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "branch".to_string(),
                "-a".to_string(),
            ]))
            .returning(|_| Ok("* main\nwip/test-user/branch1\nwip/test-user/branch2".to_string()));

        let branches = get_user_wip_branches("test-user", &mock_git).await?;
        assert_eq!(branches.len(), 2);
        assert!(branches.contains(&"wip/test-user/branch1".to_string()));
        assert!(branches.contains(&"wip/test-user/branch2".to_string()));
        Ok(())
    }

    /// Tests that empty username returns empty list
    #[tokio::test]
    async fn test_empty_username() -> Result<()> {
        let mock_git = MockGit::new();
        let branches = get_user_wip_branches("", &mock_git).await?;
        assert!(branches.is_empty());
        Ok(())
    }
}
