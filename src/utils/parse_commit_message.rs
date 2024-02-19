/// Parses a commit message to extract lists of staged, changed, and untracked files.
pub fn parse_commit_message(
    commit_message: &str,
) -> (String, Vec<String>, Vec<String>, Vec<String>) {
    let mut source_branch = String::new();
    let mut staged_files = Vec::new();
    let mut changed_files = Vec::new();
    let mut untracked_files = Vec::new();

    let mut current_section = "";

    for line in commit_message.lines() {
        let trimmed_line = line.trim();
        if trimmed_line.starts_with("Source branch: ") {
            source_branch = trimmed_line["Source branch: ".len()..].to_string();
        } else if trimmed_line.starts_with("Staged changes:") {
            current_section = "staged";
        } else if trimmed_line.starts_with("Changes:") {
            current_section = "changed";
        } else if trimmed_line.starts_with("Untracked:") {
            current_section = "untracked";
        } else if !line.starts_with("\t") {
            // Detect non-indented lines to reset the section
            current_section = "";
        } else {
            // Based on the current section, add the file to the appropriate list
            match current_section {
                "staged" => staged_files.push(trimmed_line.to_string()),
                "changed" => changed_files.push(trimmed_line.to_string()),
                "untracked" => untracked_files.push(trimmed_line.to_string()),
                _ => (),
            }
        }
    }

    (source_branch, staged_files, changed_files, untracked_files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_commit_message() {
        let commit_message = "Source branch: main\n\
Staged changes:\n\
\tfile1.txt\n\
\tfile2.txt\n\
Changes:\n\
\tfile3.txt\n\
\tfile4.txt\n\
Untracked:\n\
\tfile5.txt\n\
\tfile6.txt";

        let (source_branch, staged_files, changed_files, untracked_files) =
            parse_commit_message(commit_message);

        assert_eq!(source_branch, "main");
        assert_eq!(staged_files, vec!["file1.txt", "file2.txt"]);
        assert_eq!(changed_files, vec!["file3.txt", "file4.txt"]);
        assert_eq!(untracked_files, vec!["file5.txt", "file6.txt"]);
    }
}
