use super::execute_git_command;
use anyhow::Result;
use std::collections::HashSet;

pub async fn get_user_wip_branches(username: &str) -> Result<Vec<String>> {
    let all_branches = execute_git_command(&["branch", "-a"]).await?;
    let mut branch_set = HashSet::new();

    all_branches
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim().replace("* ", ""); // Remove any leading asterisks and spaces
            if trimmed.contains(&format!("wip/{}/", username)) {
                Some(
                    trimmed
                        .replace("remotes/origin/", "") // Clean up remote branch names
                        .trim()
                        .to_string(),
                )
            } else {
                None
            }
        })
        .for_each(|branch| {
            branch_set.insert(branch); // HashSet automatically removes duplicates
        });

    // Convert the HashSet back into a Vec<String> for the return value
    Ok(branch_set.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use mockall::{automock, predicate::*};

    #[automock]
    #[async_trait]
    trait Git {
        async fn get_user_wip_branches(&self, user: &str) -> Result<Vec<String>, anyhow::Error>;
    }

    #[tokio::test]
    async fn test_get_user_wip_branches() -> Result<(), anyhow::Error> {
        let mut mock = MockGit::new();

        // Test case 1: No branches
        mock.expect_get_user_wip_branches()
            .with(eq("user1"))
            .return_once(|_| Ok(Vec::<String>::new()));
        let branches = mock.get_user_wip_branches("user1").await?;
        assert_eq!(branches, Vec::<String>::new());

        // Test case 2: Single branch
        mock.expect_get_user_wip_branches()
            .with(eq("user2"))
            .return_once(|_| Ok(vec!["branch1".to_string()]));
        let branches = mock.get_user_wip_branches("user2").await?;
        assert_eq!(branches, vec!["branch1".to_string()]);

        // Test case 3: Multiple branches
        mock.expect_get_user_wip_branches()
            .with(eq("user3"))
            .return_once(|_| Ok(vec!["branch2".to_string(), "branch3".to_string()]));
        let branches = mock.get_user_wip_branches("user3").await?;
        assert_eq!(branches, vec!["branch2".to_string(), "branch3".to_string()]);

        Ok(())
    }
}
