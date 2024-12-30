mod color;
mod formatted_datetime;
mod git;
mod git_username;
mod parse_commit_message;

pub use color::{Color, ColorConfig};
pub use formatted_datetime::formatted_datetime;

#[cfg(test)]
pub use git::MockGit;
pub use git::{Git, GitCommand};
pub use git_username::git_username_with_git;
pub use parse_commit_message::parse_commit_message;
