mod common;

use assert_cmd::Command;
use common::{
    get_wip_branch_name, normalize_text, setup_git_repo, setup_git_repo_with_remote, t_with_args,
};
use predicates::function::function;
use std::fs;

#[tokio::test]
async fn test_save_and_list() {
    for locale in ["en", "fr", "de"] {
        let temp_dir = setup_git_repo();

        // Create a change
        fs::write(temp_dir.path().join("test.txt"), "modified content").unwrap();

        // Test save command
        let mut cmd = Command::cargo_bin("git-wippy").unwrap();
        cmd.current_dir(&temp_dir)
            .env("LANG", locale)
            .arg("save")
            .arg("--local") // Don't try to push to remote
            .assert()
            .success()
            .stdout(function(|output: &str| {
                normalize_text(output).contains(&normalize_text(&t_with_args(
                    "saving-wip",
                    &[],
                    locale,
                )))
            }))
            .stdout(function(|output: &str| {
                normalize_text(output).contains(&normalize_text(&t_with_args(
                    "staged-all-changes",
                    &[],
                    locale,
                )))
            }))
            .stdout(function(|output: &str| {
                normalize_text(output).contains(&normalize_text(&t_with_args(
                    "committed-changes",
                    &[],
                    locale,
                )))
            }));

        // Test list command
        let mut cmd = Command::cargo_bin("git-wippy").unwrap();
        cmd.current_dir(&temp_dir)
            .env("LANG", locale)
            .arg("list")
            .assert()
            .success()
            .stdout(function(|output: &str| {
                normalize_text(output).contains(&normalize_text(&t_with_args(
                    "found-wip-branches",
                    &[],
                    locale,
                )))
            }))
            .stdout(predicates::str::contains("wip/test.user/"));
    }
}

#[tokio::test]
async fn test_delete_wip() {
    for locale in ["en", "fr", "de"] {
        let temp_dir = setup_git_repo();

        // Create and save a WIP
        fs::write(temp_dir.path().join("test.txt"), "content to delete").unwrap();

        let mut cmd = Command::cargo_bin("git-wippy").unwrap();
        cmd.current_dir(&temp_dir)
            .env("LANG", locale)
            .arg("save")
            .arg("--local")
            .assert()
            .success();

        // Get the branch name
        let branch_name = get_wip_branch_name(&temp_dir);

        // Delete the WIP
        let mut cmd = Command::cargo_bin("git-wippy").unwrap();
        cmd.current_dir(&temp_dir)
            .env("LANG", locale)
            .arg("delete")
            .arg("--force") // Skip confirmation
            .arg("--local") // Only delete local branch
            .arg(&branch_name)
            .assert()
            .success()
            .stdout(function(|output: &str| {
                normalize_text(output).contains(&normalize_text(&t_with_args(
                    "delete-complete",
                    &[],
                    locale,
                )))
            }));

        // Verify it's gone from the list
        let mut cmd = Command::cargo_bin("git-wippy").unwrap();
        cmd.current_dir(&temp_dir)
            .env("LANG", locale)
            .arg("list")
            .assert()
            .success()
            .stdout(function(|output: &str| {
                normalize_text(output).contains(&normalize_text(&t_with_args(
                    "no-wip-branches",
                    &[("username", "test.user")],
                    locale,
                )))
            }));
    }
}

#[tokio::test]
async fn test_help_text_localization() {
    // Test help text in all locales
    for locale in ["en-US", "en-GB", "fr-FR", "de-DE"] {
        let mut cmd = Command::cargo_bin("git-wippy").unwrap();
        cmd.env("LANG", locale)
            .arg("--help")
            .assert()
            .success()
            .stdout(predicates::str::contains("Usage:"))
            .stdout(function(|output: &str| {
                normalize_text(output).contains(&normalize_text(&t_with_args(
                    "save-command-about",
                    &[],
                    locale,
                )))
            }))
            .stdout(function(|output: &str| {
                normalize_text(output).contains(&normalize_text(&t_with_args(
                    "list-command-about",
                    &[],
                    locale,
                )))
            }))
            .stdout(function(|output: &str| {
                normalize_text(output).contains(&normalize_text(&t_with_args(
                    "delete-command-about",
                    &[],
                    locale,
                )))
            }))
            .stdout(function(|output: &str| {
                normalize_text(output).contains(&normalize_text(&t_with_args(
                    "restore-command-about",
                    &[],
                    locale,
                )))
            }));
    }

    // Test with region codes and UTF-8 encoding
    for locale in ["en_US.UTF-8", "en_GB.UTF-8", "fr_FR.UTF-8", "de_DE.UTF-8"] {
        let mut cmd = Command::cargo_bin("git-wippy").unwrap();
        let base_locale = match locale.split('_').next().unwrap() {
            "en" => "en-US", // Default to en-US unless GB is specified
            lang => lang,
        };
        let region = locale.split('_').nth(1).unwrap().split('.').next().unwrap();
        let ietf_locale = format!("{}-{}", base_locale, region);

        cmd.env("LANG", locale)
            .arg("--help")
            .assert()
            .success()
            .stdout(predicates::str::contains("Usage:"))
            .stdout(function(|output: &str| {
                normalize_text(output).contains(&normalize_text(&t_with_args(
                    "save-command-about",
                    &[],
                    &ietf_locale,
                )))
            }))
            .stdout(function(|output: &str| {
                normalize_text(output).contains(&normalize_text(&t_with_args(
                    "list-command-about",
                    &[],
                    &ietf_locale,
                )))
            }))
            .stdout(function(|output: &str| {
                normalize_text(output).contains(&normalize_text(&t_with_args(
                    "delete-command-about",
                    &[],
                    &ietf_locale,
                )))
            }))
            .stdout(function(|output: &str| {
                normalize_text(output).contains(&normalize_text(&t_with_args(
                    "restore-command-about",
                    &[],
                    &ietf_locale,
                )))
            }));
    }

    // Test with no locale set (should default to en-US)
    let mut cmd = Command::cargo_bin("git-wippy").unwrap();
    cmd.env_remove("LANG")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Usage:"))
        .stdout(function(|output: &str| {
            normalize_text(output).contains(&normalize_text(&t_with_args(
                "save-command-about",
                &[],
                "en-US",
            )))
        }))
        .stdout(function(|output: &str| {
            normalize_text(output).contains(&normalize_text(&t_with_args(
                "list-command-about",
                &[],
                "en-US",
            )))
        }))
        .stdout(function(|output: &str| {
            normalize_text(output).contains(&normalize_text(&t_with_args(
                "delete-command-about",
                &[],
                "en-US",
            )))
        }))
        .stdout(function(|output: &str| {
            normalize_text(output).contains(&normalize_text(&t_with_args(
                "restore-command-about",
                &[],
                "en-US",
            )))
        }));
}

#[tokio::test]
async fn test_restore_wip() {
    for locale in ["en", "fr", "de"] {
        let temp_dir = setup_git_repo();

        // Configure Git for the test
        Command::new("git")
            .current_dir(&temp_dir)
            .args(["config", "user.name", "test.user"])
            .output()
            .unwrap();
        Command::new("git")
            .current_dir(&temp_dir)
            .args(["config", "user.email", "test.user@example.com"])
            .output()
            .unwrap();

        // Create and commit initial tracked files
        fs::write(temp_dir.path().join("tracked.txt"), "initial content").unwrap();
        Command::new("git")
            .current_dir(&temp_dir)
            .args(["add", "tracked.txt"])
            .output()
            .unwrap();
        Command::new("git")
            .current_dir(&temp_dir)
            .args(["commit", "-m", "Initial commit"])
            .output()
            .unwrap();

        // Create changes to save as WIP:
        // 1. Modify tracked file
        fs::write(temp_dir.path().join("tracked.txt"), "modified tracked").unwrap();
        // 2. Create new untracked file
        fs::write(temp_dir.path().join("untracked.txt"), "new untracked").unwrap();

        // Save the WIP
        let mut cmd = Command::cargo_bin("git-wippy").unwrap();
        cmd.current_dir(&temp_dir)
            .env("LANG", locale)
            .arg("save")
            .arg("--local")
            .assert()
            .success();

        // Get the branch name
        let branch_name = get_wip_branch_name(&temp_dir);

        // Restore the WIP
        let mut cmd = Command::cargo_bin("git-wippy").unwrap();
        cmd.current_dir(&temp_dir)
            .env("LANG", locale)
            .arg("restore")
            .arg("-y") // Skip confirmation
            .arg(&branch_name)
            .assert()
            .success()
            .stdout(function(|output: &str| {
                let normalized = normalize_text(output);
                normalized.contains(&normalize_text(&t_with_args(
                    "restoring-wip",
                    &[("name", &branch_name)],
                    locale,
                ))) && normalized.contains(&normalize_text(&t_with_args(
                    "restore-complete",
                    &[("name", &branch_name)],
                    locale,
                )))
            }));

        // Verify the WIP changes were restored:
        // 1. Check tracked file
        let tracked_content = fs::read_to_string(temp_dir.path().join("tracked.txt")).unwrap();
        assert_eq!(tracked_content, "modified tracked");
        // 2. Check untracked file
        let untracked_content = fs::read_to_string(temp_dir.path().join("untracked.txt")).unwrap();
        assert_eq!(untracked_content, "new untracked");
    }
}

#[tokio::test]
async fn test_restore_wip_with_autostash() {
    for locale in ["en", "fr", "de"] {
        let temp_dir = setup_git_repo();

        // Configure Git for the test
        Command::new("git")
            .current_dir(&temp_dir)
            .args(["config", "user.name", "test.user"])
            .output()
            .unwrap();
        Command::new("git")
            .current_dir(&temp_dir)
            .args(["config", "user.email", "test.user@example.com"])
            .output()
            .unwrap();

        // Create and commit initial tracked files
        fs::write(temp_dir.path().join("tracked.txt"), "initial content").unwrap();
        fs::write(temp_dir.path().join("other.txt"), "initial other").unwrap();
        Command::new("git")
            .current_dir(&temp_dir)
            .args(["add", "tracked.txt", "other.txt"])
            .output()
            .unwrap();
        Command::new("git")
            .current_dir(&temp_dir)
            .args(["commit", "-m", "Initial commit"])
            .output()
            .unwrap();

        // Create changes to save as WIP:
        // 1. Modify tracked file
        fs::write(temp_dir.path().join("tracked.txt"), "modified tracked").unwrap();
        // 2. Create new untracked file
        fs::write(temp_dir.path().join("untracked.txt"), "new untracked").unwrap();

        // Save the WIP
        let mut cmd = Command::cargo_bin("git-wippy").unwrap();
        cmd.current_dir(&temp_dir)
            .env("LANG", locale)
            .arg("save")
            .arg("--local")
            .assert()
            .success();

        // Get the branch name
        let branch_name = get_wip_branch_name(&temp_dir);

        // Reset to initial state
        Command::new("git")
            .current_dir(&temp_dir)
            .args(["reset", "--hard", "HEAD"])
            .output()
            .unwrap();
        Command::new("git")
            .current_dir(&temp_dir)
            .args(["clean", "-fd"])
            .output()
            .unwrap();

        // Create new changes that should be preserved (in different files):
        // 1. Modify other tracked file
        fs::write(temp_dir.path().join("other.txt"), "local other changes").unwrap();
        // 2. Create new untracked file
        fs::write(
            temp_dir.path().join("local-untracked.txt"),
            "local untracked",
        )
        .unwrap();

        // Restore the WIP with autostash
        let mut cmd = Command::cargo_bin("git-wippy").unwrap();
        cmd.current_dir(&temp_dir)
            .env("LANG", locale)
            .arg("restore")
            .arg("-y") // Skip confirmation
            .arg("--autostash") // Automatically handle local changes
            .arg(&branch_name)
            .assert()
            .success()
            .stdout(function(|output: &str| {
                let normalized = normalize_text(output);
                normalized.contains(&normalize_text(&t_with_args(
                    "restoring-wip",
                    &[("name", &branch_name)],
                    locale,
                ))) && normalized.contains(&normalize_text(&t_with_args(
                    "restore-complete",
                    &[("name", &branch_name)],
                    locale,
                )))
            }));

        // Verify the WIP changes were restored:
        // 1. Check tracked file has WIP content
        let tracked_content = fs::read_to_string(temp_dir.path().join("tracked.txt")).unwrap();
        assert_eq!(tracked_content, "modified tracked");
        // 2. Check WIP untracked file exists
        let untracked_content = fs::read_to_string(temp_dir.path().join("untracked.txt")).unwrap();
        assert_eq!(untracked_content, "new untracked");
        // 3. Check local changes were preserved
        let other_content = fs::read_to_string(temp_dir.path().join("other.txt")).unwrap();
        assert_eq!(other_content, "local other changes");
        let local_untracked_content =
            fs::read_to_string(temp_dir.path().join("local-untracked.txt")).unwrap();
        assert_eq!(local_untracked_content, "local untracked");
    }
}

#[tokio::test]
async fn test_restore_wip_fails_with_local_changes() {
    for locale in ["en", "fr", "de"] {
        let temp_dir = setup_git_repo();

        // Configure Git for the test
        Command::new("git")
            .current_dir(&temp_dir)
            .args(["config", "user.name", "test.user"])
            .output()
            .unwrap();
        Command::new("git")
            .current_dir(&temp_dir)
            .args(["config", "user.email", "test.user@example.com"])
            .output()
            .unwrap();

        // Create and commit initial tracked files
        fs::write(temp_dir.path().join("tracked.txt"), "initial content").unwrap();
        Command::new("git")
            .current_dir(&temp_dir)
            .args(["add", "tracked.txt"])
            .output()
            .unwrap();
        Command::new("git")
            .current_dir(&temp_dir)
            .args(["commit", "-m", "Initial commit"])
            .output()
            .unwrap();

        // Create changes to save as WIP:
        // 1. Modify tracked file
        fs::write(temp_dir.path().join("tracked.txt"), "modified tracked").unwrap();
        // 2. Create new untracked file
        fs::write(temp_dir.path().join("untracked.txt"), "new untracked").unwrap();

        // Save the WIP
        let mut cmd = Command::cargo_bin("git-wippy").unwrap();
        cmd.current_dir(&temp_dir)
            .env("LANG", locale)
            .arg("save")
            .arg("--local")
            .assert()
            .success();

        // Get the branch name
        let branch_name = get_wip_branch_name(&temp_dir);

        // Create new changes that should cause restore to fail:
        // 1. Modify tracked file
        fs::write(temp_dir.path().join("tracked.txt"), "local tracked changes").unwrap();
        // 2. Create new untracked file
        fs::write(
            temp_dir.path().join("local-untracked.txt"),
            "local untracked",
        )
        .unwrap();

        // Restore should fail due to local changes
        let mut cmd = Command::cargo_bin("git-wippy").unwrap();
        cmd.current_dir(&temp_dir)
            .env("LANG", locale)
            .arg("restore")
            .arg("-y") // Skip confirmation
            .arg(&branch_name)
            .assert()
            .failure();

        // Verify the local changes are still there:
        // 1. Check tracked file still has local changes
        let tracked_content = fs::read_to_string(temp_dir.path().join("tracked.txt")).unwrap();
        assert_eq!(tracked_content, "local tracked changes");
        // 2. Check local untracked file still exists
        let local_untracked_content =
            fs::read_to_string(temp_dir.path().join("local-untracked.txt")).unwrap();
        assert_eq!(local_untracked_content, "local untracked");
    }
}

#[tokio::test]
async fn test_restore_wip_with_autostash_conflicts() {
    for locale in ["en", "fr", "de"] {
        let temp_dir = setup_git_repo();

        // Configure Git for the test
        Command::new("git")
            .current_dir(&temp_dir)
            .args(["config", "user.name", "test.user"])
            .output()
            .unwrap();
        Command::new("git")
            .current_dir(&temp_dir)
            .args(["config", "user.email", "test.user@example.com"])
            .output()
            .unwrap();

        // Create and commit initial tracked files
        fs::write(temp_dir.path().join("tracked.txt"), "initial content").unwrap();
        fs::write(temp_dir.path().join("other.txt"), "initial other").unwrap();
        Command::new("git")
            .current_dir(&temp_dir)
            .args(["add", "tracked.txt", "other.txt"])
            .output()
            .unwrap();
        Command::new("git")
            .current_dir(&temp_dir)
            .args(["commit", "-m", "Initial commit"])
            .output()
            .unwrap();

        // Create changes to save as WIP:
        // 1. Modify tracked file
        fs::write(temp_dir.path().join("tracked.txt"), "modified tracked").unwrap();
        // 2. Create new untracked file
        fs::write(temp_dir.path().join("untracked.txt"), "new untracked").unwrap();

        // Save the WIP
        let mut cmd = Command::cargo_bin("git-wippy").unwrap();
        cmd.current_dir(&temp_dir)
            .env("LANG", locale)
            .arg("save")
            .arg("--local")
            .assert()
            .success();

        // Get the branch name
        let branch_name = get_wip_branch_name(&temp_dir);

        // Reset to initial state
        Command::new("git")
            .current_dir(&temp_dir)
            .args(["reset", "--hard", "HEAD"])
            .output()
            .unwrap();
        Command::new("git")
            .current_dir(&temp_dir)
            .args(["clean", "-fd"])
            .output()
            .unwrap();

        // Create changes in different files
        fs::write(temp_dir.path().join("other.txt"), "local changes").unwrap();
        fs::write(
            temp_dir.path().join("local-untracked.txt"),
            "local untracked",
        )
        .unwrap();

        // Restore the WIP with autostash
        let mut cmd = Command::cargo_bin("git-wippy").unwrap();
        cmd.current_dir(&temp_dir)
            .env("LANG", locale)
            .arg("restore")
            .arg("-y") // Skip confirmation
            .arg("--autostash") // Automatically handle local changes
            .arg(&branch_name)
            .assert()
            .success()
            .stdout(function(|output: &str| {
                let normalized = normalize_text(output);
                normalized.contains(&normalize_text(&t_with_args(
                    "restoring-wip",
                    &[("name", &branch_name)],
                    locale,
                ))) && normalized.contains(&normalize_text(&t_with_args(
                    "restore-complete",
                    &[("name", &branch_name)],
                    locale,
                )))
            }));

        // Verify the WIP changes were restored:
        // 1. Check tracked file has WIP content
        let tracked_content = fs::read_to_string(temp_dir.path().join("tracked.txt")).unwrap();
        assert_eq!(tracked_content, "modified tracked");
        // 2. Check WIP untracked file exists
        let untracked_content = fs::read_to_string(temp_dir.path().join("untracked.txt")).unwrap();
        assert_eq!(untracked_content, "new untracked");

        // Verify local changes were preserved:
        // 1. Check other file has local changes
        let other_content = fs::read_to_string(temp_dir.path().join("other.txt")).unwrap();
        assert_eq!(other_content, "local changes");
        // 2. Check local untracked file exists
        let local_untracked_content =
            fs::read_to_string(temp_dir.path().join("local-untracked.txt")).unwrap();
        assert_eq!(local_untracked_content, "local untracked");
    }
}

#[tokio::test]
async fn test_save_with_remote() {
    for locale in ["en", "fr", "de"] {
        let (local_dir, _remote_dir) = setup_git_repo_with_remote();

        // Create a change
        fs::write(local_dir.path().join("test.txt"), "modified content").unwrap();

        // Test save command with remote
        let mut cmd = Command::cargo_bin("git-wippy").unwrap();
        cmd.current_dir(&local_dir)
            .env("LANG", locale)
            .arg("save")
            .assert()
            .success()
            .stdout(function(|output: &str| {
                normalize_text(output).contains(&normalize_text(&t_with_args(
                    "saving-wip",
                    &[],
                    locale,
                )))
            }))
            .stdout(function(|output: &str| {
                normalize_text(output).contains(&normalize_text(&t_with_args(
                    "pushed-changes",
                    &[],
                    locale,
                )))
            }));

        // Verify branch exists on remote
        let branch_name = get_wip_branch_name(&local_dir);
        Command::new("git")
            .current_dir(&local_dir)
            .args(&["ls-remote", "--heads", "origin", &branch_name])
            .assert()
            .success()
            .stdout(predicates::str::contains(&branch_name));
    }
}

#[tokio::test]
async fn test_save_without_remote() {
    for locale in ["en", "fr", "de"] {
        let temp_dir = setup_git_repo();

        // Create a change
        fs::write(temp_dir.path().join("test.txt"), "modified content").unwrap();

        // Test save command without remote
        let mut cmd = Command::cargo_bin("git-wippy").unwrap();
        cmd.current_dir(&temp_dir)
            .env("LANG", locale)
            .arg("save")
            .assert()
            .success()
            .stdout(function(|output: &str| {
                normalize_text(output).contains(&normalize_text(&t_with_args(
                    "saving-wip",
                    &[],
                    locale,
                )))
            }))
            .stdout(function(|output: &str| {
                normalize_text(output).contains(&normalize_text(&t_with_args(
                    "skipped-push-no-remote",
                    &[],
                    locale,
                )))
            }));

        // Verify branch exists locally
        let branch_name = get_wip_branch_name(&temp_dir);
        Command::new("git")
            .current_dir(&temp_dir)
            .args(&["branch", "--list", &branch_name])
            .assert()
            .success()
            .stdout(predicates::str::contains(&branch_name));
    }
}
