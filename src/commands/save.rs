use crate::utils::{formatted_datetime, git_username, Git};
use anyhow::Result;

pub async fn save_wip_changes(
    git: &impl Git,
    local: bool,
    username: Option<String>,
    datetime: Option<String>,
) -> Result<()> {
    // Use provided values or get them from functions
    let username = match username {
        Some(u) => u,
        None => git_username().await?,
    };
    let datetime = match datetime {
        Some(d) => d,
        None => formatted_datetime(),
    };

    let branch_name = format!("wip/{}/{}", username, datetime);

    // Ensure we're in a git repository
    git.execute(vec![
        "rev-parse".to_string(),
        "--is-inside-work-tree".to_string(),
    ])
    .await?;

    // Store the current branch name before switching
    let original_branch = git.get_current_branch().await?;

    // Generate the detailed commit message
    let commit_message = generate_commit_message(git).await?;

    // Create and switch to the new branch
    git.create_branch(&branch_name).await?;
    git.stage_all().await?;
    git.commit(&commit_message).await?;

    if !local {
        git.push("origin", &branch_name).await?;
    }

    git.checkout(&original_branch).await?;

    println!(
        "WIP changes saved to branch '{}' and returned to '{}'",
        branch_name, original_branch
    );
    Ok(())
}

async fn generate_commit_message(git: &impl Git) -> Result<String> {
    let staged = git.get_staged_files().await?;
    let changed = git.get_changed_files().await?;
    let untracked = git.get_untracked_files().await?;
    let source_branch = git.get_current_branch().await?;

    let staged_section = if !staged.is_empty() {
        format!("\nStaged changes:\n\t{}", staged.replace("\n", "\n\t"))
    } else {
        String::new()
    };

    let changed_section = if !changed.is_empty() {
        format!("\nChanges:\n\t{}", changed.replace("\n", "\n\t"))
    } else {
        String::new()
    };

    let untracked_section = if !untracked.is_empty() {
        format!("\nUntracked:\n\t{}", untracked.replace("\n", "\n\t"))
    } else {
        String::new()
    };

    let message = format!(
        "chore: saving work in progress\n\nSource branch: {}{}{}{}",
        source_branch, staged_section, changed_section, untracked_section
    );

    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::MockGit;
    use anyhow::Result;

    #[tokio::test]
    async fn test_save_wip_changes_local() -> Result<()> {
        let mut mock_git = Box::new(MockGit::new());
        let test_username = "test-user".to_string();
        let test_datetime = "2024-01-01-12-00-00".to_string();

        mock_git
            .expect_get_staged_files()
            .returning(|| Ok("staged.txt".to_string()));

        mock_git
            .expect_get_changed_files()
            .returning(|| Ok("modified.txt".to_string()));

        mock_git
            .expect_get_untracked_files()
            .returning(|| Ok("untracked.txt".to_string()));

        mock_git
            .expect_execute()
            .withf(|args: &Vec<String>| {
                args == &vec!["rev-parse".to_string(), "--is-inside-work-tree".to_string()]
            })
            .returning(|_| Ok(String::new()));

        mock_git
            .expect_get_current_branch()
            .times(2)
            .returning(|| Ok("main".to_string()));

        mock_git
            .expect_create_branch()
            .with(mockall::predicate::eq("wip/test-user/2024-01-01-12-00-00"))
            .returning(|_| Ok(String::new()));

        mock_git.expect_stage_all().returning(|| Ok(String::new()));

        mock_git
            .expect_commit()
            .with(mockall::predicate::eq(
                "chore: saving work in progress\n\nSource branch: main\nStaged changes:\n\tstaged.txt\nChanges:\n\tmodified.txt\nUntracked:\n\tuntracked.txt"
            ))
            .returning(|_| Ok(String::new()));

        mock_git
            .expect_checkout()
            .with(mockall::predicate::eq("main"))
            .returning(|_| Ok(String::new()));

        save_wip_changes(&*mock_git, true, Some(test_username), Some(test_datetime)).await?;
        Ok(())
    }
}
