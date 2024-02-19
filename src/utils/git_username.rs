use super::execute_git_command;
use anyhow::Result;

/// Fetches the Git username asynchronously
pub async fn git_username() -> Result<String> {
    let username = execute_git_command(&["config", "user.name"]).await?;
    Ok(username.replace(' ', "-").to_lowercase())
}
