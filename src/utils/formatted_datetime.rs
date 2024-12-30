use chrono::Local;

/// Returns the current datetime formatted for use in branch names.
///
/// The datetime is formatted in a Git-friendly format that:
/// - Is sortable chronologically
/// - Contains no special characters that would be invalid in Git branch names
/// - Is human-readable
///
/// # Format
///
/// The format is: `YYYY-MM-DD-HH-mm-SS`
///
/// For example: `2024-03-14-15-30-45`
///
/// # Examples
///
/// ```
/// use git_wippy::formatted_datetime;
///
/// let datetime = formatted_datetime();
/// assert!(datetime.len() == 19); // YYYY-MM-DD-HH-mm-SS
/// ```
///
/// # Usage
///
/// This function is typically used to create unique WIP branch names by combining
/// with the username: `wip/{username}/{formatted_datetime}`
pub fn formatted_datetime() -> String {
    Local::now().format("%Y-%m-%d-%H-%M-%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    /// Tests that the formatted datetime matches expected pattern
    #[tokio::test]
    async fn test_formatted_datetime() {
        let datetime = formatted_datetime();
        let re = Regex::new(r"^\d{4}-\d{2}-\d{2}-\d{2}-\d{2}-\d{2}$").unwrap();
        assert!(re.is_match(&datetime));
    }
}
