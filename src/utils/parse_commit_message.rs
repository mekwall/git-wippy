use std::collections::HashSet;

/// Parses a WIP commit message to extract branch and file information.
///
/// # Arguments
/// * `message` - The commit message to parse
///
/// # Returns
/// A tuple containing:
/// * source_branch: The original branch name
/// * staged_files: List of files that were staged
/// * changed_files: List of files that were changed but not staged
/// * untracked_files: List of untracked files
///
/// # Format
/// Expected commit message format:
/// ```text
/// chore: saving work in progress
///
/// Source branch: main
/// Staged changes:
///     file1.txt
///     file2.txt
/// Changes:
///     file3.txt
/// Untracked:
///     file4.txt
/// ```
pub fn parse_commit_message(message: &str) -> (String, Vec<String>, Vec<String>, Vec<String>) {
    let mut source_branch = String::new();
    let mut staged_files = HashSet::new();
    let mut changed_files = HashSet::new();
    let mut untracked_files = HashSet::new();

    let mut current_section = None;

    for line in message.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("Source branch:") {
            source_branch = trimmed
                .trim_start_matches("Source branch:")
                .trim()
                .to_string();
            continue;
        }

        match trimmed {
            "Staged changes:" => {
                current_section = Some("staged");
                continue;
            }
            "Changes:" => {
                current_section = Some("changed");
                continue;
            }
            "Untracked:" => {
                current_section = Some("untracked");
                continue;
            }
            "" => continue,
            _ => {}
        }

        if let Some(section) = current_section {
            let file = trimmed.to_string();
            match section {
                "staged" => staged_files.insert(file),
                "changed" => changed_files.insert(file),
                "untracked" => untracked_files.insert(file),
                _ => false,
            };
        }
    }

    (
        source_branch,
        staged_files.into_iter().collect(),
        changed_files.into_iter().collect(),
        untracked_files.into_iter().collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests parsing a complete commit message with all sections
    #[test]
    fn test_parse_commit_message() {
        let message = r#"chore: saving work in progress

Source branch: main
Staged changes:
    staged1.txt
    staged2.txt
Changes:
    changed1.txt
    changed2.txt
Untracked:
    untracked1.txt
    untracked2.txt"#;

        let (branch, staged, changed, untracked) = parse_commit_message(message);

        assert_eq!(branch, "main");
        assert_eq!(staged.len(), 2);
        assert_eq!(changed.len(), 2);
        assert_eq!(untracked.len(), 2);
    }

    /// Tests parsing an empty commit message
    #[test]
    fn test_empty_message() {
        let (branch, staged, changed, untracked) = parse_commit_message("");

        assert!(branch.is_empty());
        assert!(staged.is_empty());
        assert!(changed.is_empty());
        assert!(untracked.is_empty());
    }

    /// Tests parsing a malformed commit message
    #[test]
    fn test_malformed_message() {
        let message = "Some random text\nwithout any expected sections";

        let (branch, staged, changed, untracked) = parse_commit_message(message);

        assert!(branch.is_empty());
        assert!(staged.is_empty());
        assert!(changed.is_empty());
        assert!(untracked.is_empty());
    }
}
