use crate::utils::{get_user_wip_branches, git_username, GitCommand};
use anyhow::Result;

/// Lists all WIP branches for the current Git user.
///
/// # Details
/// * Retrieves the current Git username
/// * Finds all branches matching the pattern "wip/{username}/*"
/// * Displays the branches in a formatted list
/// * Shows a message if no WIP branches are found
///
/// # Returns
/// * `Ok(())` if the operation succeeds
/// * `Err` if username retrieval or branch listing fails
pub async fn list_wip_branches() -> Result<()> {
    let git = GitCommand::new();
    let username = git_username().await?;
    let wip_branches = get_user_wip_branches(&username, &git).await?;

    if wip_branches.is_empty() {
        println!("No WIP branches found for the user: {}", username);
    } else {
        println!("WIP branches for user '{}':", username);
        for branch in wip_branches {
            println!("- {}", branch);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::MockGit;

    /// Tests listing WIP branches when branches exist
    #[tokio::test]
    async fn test_list_wip_branches() -> Result<()> {
        let mut mock_git = MockGit::new();

        // Mock the git username command
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "config".to_string(),
                "user.name".to_string(),
            ]))
            .returning(|_| Ok("test-user".to_string()));

        // Mock the branch listing command
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "branch".to_string(),
                "-a".to_string(),
            ]))
            .returning(|_| {
                Ok(
                    "* main\nwip/test-user/branch1\nwip/test-user/branch2\nother-branch"
                        .to_string(),
                )
            });

        list_wip_branches().await?;
        Ok(())
    }

    /// Tests listing WIP branches when no branches exist
    #[tokio::test]
    async fn test_list_wip_branches_empty() -> Result<()> {
        let mut mock_git = MockGit::new();

        // Mock the git username command
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "config".to_string(),
                "user.name".to_string(),
            ]))
            .returning(|_| Ok("test-user".to_string()));

        // Mock the branch listing command with no WIP branches
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "branch".to_string(),
                "-a".to_string(),
            ]))
            .returning(|_| Ok("* main\nother-branch".to_string()));

        list_wip_branches().await?;
        Ok(())
    }
}
