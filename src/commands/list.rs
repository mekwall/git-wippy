use crate::i18n::t_with_args;
use crate::output::Output;
use crate::utils::{git_username_with_git, Git, GitCommand};
use anyhow::Result;

pub async fn list_wip_branches() -> Result<()> {
    let git = GitCommand::new();
    list_wip_branches_with_git(&git).await
}

pub async fn list_wip_branches_with_git(git: &impl Git) -> Result<()> {
    let output = Output::new().await?;
    let username = git_username_with_git(git).await?;
    let wip_branches = git.get_user_wip_branches(&username).await?;

    if wip_branches.is_empty() {
        let message = t_with_args("no-wip-branches", &[("username", &username)]);
        output.info(&output.format_with_highlights(&message, &[&username]))?;
        return Ok(());
    }

    output.info(&t_with_args("found-wip-branches", &[]))?;
    for branch in wip_branches {
        output.info(&output.format_with_highlights(
            &t_with_args("branch-name", &[("name", &branch)]),
            &[&branch],
        ))?;
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
            .returning(|_| Ok(vec!["wip/test-user/branch1".to_string()]));

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

        // Mock WIP branches (empty)
        mock_git
            .expect_get_user_wip_branches()
            .with(mockall::predicate::eq("test-user"))
            .returning(|_| Ok(vec![]));

        list_wip_branches_with_git(&mock_git).await?;
        Ok(())
    }
}
