use crate::utils::{formatted_datetime, git_username_with_git, Git};
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
        None => git_username_with_git(git).await?,
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

    #[tokio::test]
    async fn test_save_wip_changes_local() -> Result<()> {
        let mut mock_git = MockGit::new();

        // Mock git repo check
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "rev-parse".to_string(),
                "--is-inside-work-tree".to_string(),
            ]))
            .returning(|_| Ok("true".to_string()));

        // Mock username lookup
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "config".to_string(),
                "user.name".to_string(),
            ]))
            .returning(|_| Ok("test-user".to_string()));

        // Mock get_current_branch using the trait method
        mock_git
            .expect_get_current_branch()
            .returning(|| Ok("main".to_string()));

        // Mock getting staged files
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "diff".to_string(),
                "--cached".to_string(),
                "--name-only".to_string(),
            ]))
            .returning(|_| Ok("file1.txt".to_string()));

        // Mock getting changed files
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "diff".to_string(),
                "--name-only".to_string(),
            ]))
            .returning(|_| Ok("file2.txt".to_string()));

        // Mock getting untracked files
        mock_git
            .expect_execute()
            .with(mockall::predicate::eq(vec![
                "ls-files".to_string(),
                "--others".to_string(),
                "--exclude-standard".to_string(),
            ]))
            .returning(|_| Ok("file3.txt".to_string()));

        // Mock create_branch using the trait method
        mock_git
            .expect_create_branch()
            .with(mockall::predicate::function(|branch: &str| {
                branch.starts_with("wip/test-user/")
            }))
            .returning(|_| Ok("Created branch".to_string()));

        // Mock commit using the trait method
        mock_git
            .expect_commit()
            .with(mockall::predicate::function(|msg: &str| {
                msg.contains("Source branch: main")
                    && msg.contains("Staged changes:\n\tfile1.txt")
                    && msg.contains("Changes:\n\tfile2.txt")
                    && msg.contains("Untracked:\n\tfile3.txt")
            }))
            .returning(|_| Ok("Created commit".to_string()));

        // Mock Git trait methods
        mock_git
            .expect_get_staged_files()
            .returning(|| Ok("file1.txt".to_string()));

        mock_git
            .expect_get_changed_files()
            .returning(|| Ok("file2.txt".to_string()));

        mock_git
            .expect_get_untracked_files()
            .returning(|| Ok("file3.txt".to_string()));

        // Mock stage_all using the trait method
        mock_git
            .expect_stage_all()
            .returning(|| Ok("Changes staged".to_string()));

        // Mock checkout back to original branch
        mock_git
            .expect_checkout()
            .with(mockall::predicate::eq("main"))
            .returning(|_| Ok("Switched back to branch 'main'".to_string()));

        save_wip_changes(&mock_git, true, None, None).await?;
        Ok(())
    }
}
