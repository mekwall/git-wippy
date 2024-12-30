use crate::utils::{Color, ColorConfig};
use anyhow::Result;

/// A formatter for terminal output with color support.
///
/// This struct handles the formatting and display of messages, providing:
/// - Consistent color formatting based on Git's color settings
/// - Support for highlighting specific parts of messages
/// - Standard output types (info, warning, error)
///
/// # Examples
///
/// ```no_run
/// use git_wippy::Output;
/// use anyhow::Result;
///
/// async fn example() -> Result<()> {
///     let output = Output::new().await?;
///     output.info("Operation completed successfully")?;
///     output.error("Something went wrong")?;
///     Ok(())
/// }
/// ```
pub(crate) struct Output {
    color: ColorConfig,
}

impl Output {
    /// Creates a new Output instance with color settings determined from Git config.
    pub async fn new() -> Result<Self> {
        Ok(Self {
            color: ColorConfig::new().await,
        })
    }

    /// Normalize text by removing bidirectional control characters
    fn normalize_text(&self, text: &str) -> String {
        text.replace('\u{2068}', "").replace('\u{2069}', "")
    }

    /// Prints an informational message in green.
    pub fn info(&self, message: &str) -> Result<()> {
        if !message.is_empty() {
            print!("{}\n", self.normalize_text(message));
        }
        Ok(())
    }

    /// Prints a warning message in yellow.
    #[allow(dead_code)]
    pub fn warning(&self, message: &str) -> Result<()> {
        if !message.is_empty() {
            print!(
                "{}\n",
                self.color
                    .colorize(&self.normalize_text(message), Color::Yellow)
            );
        }
        Ok(())
    }

    /// Prints an error message in red.
    pub fn error(&self, message: &str) -> Result<()> {
        if !message.is_empty() {
            eprint!(
                "{}\n",
                self.color
                    .colorize(&self.normalize_text(message), Color::Red)
            );
        }
        Ok(())
    }

    /// Prints a debug message in gray, only in debug builds.
    /// In release builds, this is a no-op.
    pub fn debug(&self, message: &str) -> Result<()> {
        #[cfg(debug_assertions)]
        if !message.is_empty() {
            let debug_msg = format!("[DEBUG] {}", message);
            eprint!(
                "{}\n",
                self.color
                    .colorize(&self.normalize_text(&debug_msg), Color::Gray)
            );
        }
        Ok(())
    }

    /// Highlights a piece of text in yellow, useful for branch names and values.
    pub fn highlight(&self, text: &str) -> String {
        self.color
            .colorize(&self.normalize_text(text), Color::Yellow)
    }

    /// Formats a message with highlighted parts.
    ///
    /// # Arguments
    ///
    /// * `message` - The full message
    /// * `highlights` - Parts of the message to highlight in yellow
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use git_wippy::Output;
    /// # use anyhow::Result;
    /// # fn example() -> Result<()> {
    /// let output = Output::new();
    /// let msg = output.format_with_highlights(
    ///     "Switched to branch 'main'",
    ///     &["'main'"]
    /// );
    /// output.info(&msg)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn format_with_highlights(&self, message: &str, highlights: &[&str]) -> String {
        let mut result = message.to_string();
        for highlight in highlights {
            result = result.replace(highlight, &self.highlight(highlight));
        }
        self.normalize_text(&result)
    }

    /// Prints a warning message in yellow.
    #[allow(dead_code)]
    pub fn warn(&self, message: &str) -> Result<()> {
        if !message.is_empty() {
            eprint!(
                "{}\n",
                self.color
                    .colorize(&self.normalize_text(message), Color::Yellow)
            );
        }
        Ok(())
    }
}
