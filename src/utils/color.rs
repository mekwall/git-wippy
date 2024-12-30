use crate::utils::{Git, GitCommand};
use std::env;
use std::io::IsTerminal;

/// Configuration for terminal color output.
///
/// This struct determines whether and how to colorize output based on:
/// - Git's color.ui configuration
/// - Terminal capabilities
/// - Environment variables
///
/// # Color Detection
///
/// Colors are enabled when:
/// 1. Git's color.ui is set to "always", or
/// 2. Git's color.ui is "auto" (default) and:
///    - Output is to a terminal
///    - NO_COLOR environment variable is not set
///    - TERM is not "dumb"
pub struct ColorConfig {
    enabled: bool,
}

impl ColorConfig {
    /// Creates a new ColorConfig instance asynchronously with settings determined from the environment.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use git_wippy::ColorConfig;
    ///
    /// async {
    ///     let config = ColorConfig::new().await;
    /// };
    /// ```
    pub async fn new() -> Self {
        let git = GitCommand::new();
        Self::new_with_git(&git).await
    }

    /// Creates a new ColorConfig instance with a specific Git implementation.
    pub(crate) async fn new_with_git(git: &impl Git) -> Self {
        let mut config = Self { enabled: false };
        config.init(git).await;
        config
    }

    /// Initializes color settings based on Git configuration and environment.
    async fn init(&mut self, git: &impl Git) {
        let auto_color = std::io::stdout().is_terminal()
            && env::var("NO_COLOR").is_err()
            && env::var("TERM").map(|t| t != "dumb").unwrap_or(true);

        if let Ok(Some(value)) = git.get_config_value("color.ui").await {
            match value.as_str() {
                "always" => self.enabled = true,
                "never" => self.enabled = false,
                "auto" | "" => self.enabled = auto_color,
                _ => self.enabled = false,
            }
        } else {
            // If no color configuration is found, use auto behavior
            self.enabled = auto_color;
        }
    }

    /// Colorizes text with the specified color if colors are enabled.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to colorize
    /// * `color` - The color to apply
    ///
    /// # Returns
    ///
    /// The text with ANSI color codes if colors are enabled, otherwise the original text.
    pub fn colorize(&self, text: &str, color: Color) -> String {
        if self.enabled {
            format!("{}{}{}", color.ansi_code(), text, "\x1b[0m")
        } else {
            text.to_string()
        }
    }
}

/// ANSI colors available for terminal output.
///
/// These colors are used to highlight different types of messages:
/// - Red: Errors and warnings
/// - Green: Success and info messages
/// - Yellow: Branch names and important values
pub enum Color {
    /// Red color for errors and warnings
    Red,
    /// Green color for success and info messages
    #[allow(dead_code)]
    Green,
    /// Yellow color for branch names and important values
    Yellow,
    /// Gray color for branch names and important values
    Gray,
}

impl Color {
    /// Returns the ANSI escape code for the color.
    fn ansi_code(&self) -> &str {
        match self {
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Yellow => "\x1b[33m",
            Color::Gray => "\x1b[90m",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::MockGit;

    #[tokio::test]
    async fn test_color_config_always() {
        let mut mock_git = MockGit::new();
        mock_git
            .expect_get_config_value()
            .with(mockall::predicate::eq("color.ui"))
            .returning(|_| Ok(Some("always".to_string())));

        let config = ColorConfig::new_with_git(&mock_git).await;
        assert!(config.enabled);
    }

    #[tokio::test]
    async fn test_color_config_never() {
        let mut mock_git = MockGit::new();
        mock_git
            .expect_get_config_value()
            .with(mockall::predicate::eq("color.ui"))
            .returning(|_| Ok(Some("never".to_string())));

        let config = ColorConfig::new_with_git(&mock_git).await;
        assert!(!config.enabled);
    }

    #[tokio::test]
    async fn test_color_config_auto() {
        let mut mock_git = MockGit::new();
        mock_git
            .expect_get_config_value()
            .with(mockall::predicate::eq("color.ui"))
            .returning(|_| Ok(Some("auto".to_string())));

        let config = ColorConfig::new_with_git(&mock_git).await;
        assert!(
            config.enabled == std::io::stdout().is_terminal()
                && env::var("NO_COLOR").is_err()
                && env::var("TERM").map(|t| t != "dumb").unwrap_or(true)
        );
    }

    #[tokio::test]
    async fn test_color_config_empty() {
        let mut mock_git = MockGit::new();
        mock_git
            .expect_get_config_value()
            .with(mockall::predicate::eq("color.ui"))
            .returning(|_| Ok(Some("".to_string())));

        let config = ColorConfig::new_with_git(&mock_git).await;
        assert!(
            config.enabled == std::io::stdout().is_terminal()
                && env::var("NO_COLOR").is_err()
                && env::var("TERM").map(|t| t != "dumb").unwrap_or(true)
        );
    }

    #[tokio::test]
    async fn test_color_config_invalid() {
        let mut mock_git = MockGit::new();
        mock_git
            .expect_get_config_value()
            .with(mockall::predicate::eq("color.ui"))
            .returning(|_| Ok(Some("invalid".to_string())));

        let config = ColorConfig::new_with_git(&mock_git).await;
        assert!(!config.enabled);
    }

    #[tokio::test]
    async fn test_color_config_not_found() {
        let mut mock_git = MockGit::new();
        mock_git
            .expect_get_config_value()
            .with(mockall::predicate::eq("color.ui"))
            .returning(|_| Ok(None));

        let config = ColorConfig::new_with_git(&mock_git).await;
        assert!(
            config.enabled == std::io::stdout().is_terminal()
                && env::var("NO_COLOR").is_err()
                && env::var("TERM").map(|t| t != "dumb").unwrap_or(true)
        );
    }

    #[test]
    fn test_colorize() {
        let config = ColorConfig { enabled: true };
        let text = "test";
        let colored = config.colorize(text, Color::Red);
        assert!(colored.starts_with("\x1b[31m"));
        assert!(colored.ends_with("\x1b[0m"));
        assert!(colored.contains(text));
    }

    #[test]
    fn test_colorize_disabled() {
        let config = ColorConfig { enabled: false };
        let text = "test";
        let colored = config.colorize(text, Color::Red);
        assert_eq!(colored, text);
    }
}
