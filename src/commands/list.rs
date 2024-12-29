use crate::utils::{git_username_with_git, Git, GitCommand};
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
    list_wip_branches_with_git(&git).await
}

/// Implementation that accepts a Git instance for better testability
pub async fn list_wip_branches_with_git(git: &impl Git) -> Result<()> {
    let username = git_username_with_git(git).await?;
    let wip_branches = git.get_user_wip_branches(&username).await?;

    if wip_branches.is_empty() {
        println!("No WIP branches found for the user: {}", username);
    } else {
        println!("WIP branches for user '{}':", username);
        for branch in &wip_branches {
            // Get commit message for each branch
            let commit_msg = git
                .execute(vec![
                    "log".to_string(),
                    "-1".to_string(),
                    "--pretty=%B".to_string(),
                    branch.clone(),
                ])
                .await?;
            println!("- {} ({})", branch, commit_msg.lines().next().unwrap_or(""));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::MockGit;

    #[tokio::test]
    async fn test_list_wip_branches() -> Result<()> {
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

        // Mock getting commit messages for each branch
        for branch in ["wip/test-user/branch1", "wip/test-user/branch2"] {
            mock_git
                .expect_execute()
                .with(mockall::predicate::eq(vec![
                    "log".to_string(),
                    "-1".to_string(),
                    "--pretty=%B".to_string(),
                    branch.to_string(),
                ]))
                .returning(|_| Ok("WIP: Saved state from branch 'main'".to_string()));
        }

        list_wip_branches_with_git(&mock_git).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_list_wip_branches_empty() -> Result<()> {
        let mut mock_git = MockGit::new();

        // Mock username lookup
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "config".to_string(),
                "user.name".to_string(),
            ]))
            .returning(|_| Ok("test-user".to_string()));

        // Mock empty WIP branches list
        mock_git
            .expect_get_user_wip_branches()
            .with(mockall::predicate::eq("test-user"))
            .returning(|_| Ok(Vec::new()));

        list_wip_branches_with_git(&mock_git).await?;
        Ok(())
    }
}
