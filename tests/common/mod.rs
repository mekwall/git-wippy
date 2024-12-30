use assert_cmd::Command;
use fluent::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use std::fs;
use tempfile::TempDir;
use unic_langid::LanguageIdentifier;

pub fn setup_git_repo() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    Command::new("git")
        .args(&["init"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    // Set up git config
    Command::new("git")
        .args(&["config", "--local", "user.name", "test.user"])
        .current_dir(&temp_dir)
        .assert()
        .success();
    Command::new("git")
        .args(&["config", "--local", "user.email", "test@example.com"])
        .current_dir(&temp_dir)
        .assert()
        .success();
    Command::new("git")
        .args(&["config", "--local", "commit.gpgsign", "false"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    // Create and commit a test file
    fs::write(temp_dir.path().join("test.txt"), "initial content").unwrap();
    Command::new("git")
        .args(&["add", "test.txt"])
        .current_dir(&temp_dir)
        .assert()
        .success();
    Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    temp_dir
}

pub fn get_wip_branch_name(temp_dir: &TempDir) -> String {
    let output = Command::new("git")
        .args(&["branch", "--list", "wip/test.user/*"])
        .current_dir(temp_dir)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .next()
        .unwrap()
        .trim_start_matches("* ")
        .trim()
        .to_string()
}

/// Get a translation bundle for a specific locale
fn get_bundle(locale: &str) -> FluentBundle<FluentResource> {
    let lang_id: LanguageIdentifier = locale
        .split('.')
        .next()
        .unwrap_or("en-US")
        .parse()
        .unwrap_or_else(|_| "en-US".parse().unwrap());

    let resource_path = match (
        lang_id.language.as_str(),
        lang_id.region.as_ref().map(|r| r.as_str()),
    ) {
        ("en", Some("GB")) => include_str!("../../locales/en-GB.ftl"),
        ("de", Some("DE") | None) => include_str!("../../locales/de-DE.ftl"),
        ("fr", Some("FR") | None) => include_str!("../../locales/fr-FR.ftl"),
        _ => include_str!("../../locales/en-US.ftl"),
    };

    let res =
        FluentResource::try_new(resource_path.to_string()).expect("Failed to parse FluentResource");

    let mut bundle = FluentBundle::new(vec![lang_id]);
    bundle
        .add_resource(res)
        .expect("Failed to add FluentResource to bundle");

    bundle
}

/// Get a translation for a specific locale and key
#[allow(dead_code)]
pub fn t(key: &str) -> String {
    let locale = std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .unwrap_or_else(|_| String::from("en"));
    t_with_args(key, &[], &locale)
}

/// Get a translation for a specific locale and key with variables
pub fn t_with_args(key: &str, args: &[(&str, &str)], locale: &str) -> String {
    let bundle = get_bundle(locale);
    let msg = bundle.get_message(key).expect("Message not found");
    let pattern = msg.value().expect("No value for message");

    let mut fluent_args = FluentArgs::new();
    for (k, v) in args {
        // Handle both variable formats:
        // - { $username } -> use "username" as key
        // - {name} -> use "name" as key
        let var_name = if k.starts_with('$') { &k[1..] } else { k };
        fluent_args.set(var_name, FluentValue::from(*v));
    }

    let mut errors = vec![];
    let formatted = bundle.format_pattern(pattern, Some(&fluent_args), &mut errors);

    if !errors.is_empty() {
        // Try again with the variable name without $ prefix
        let mut fluent_args = FluentArgs::new();
        for (k, v) in args {
            let k = k.strip_prefix('$').unwrap_or(k);
            fluent_args.set(k, FluentValue::from(*v));
        }
        let formatted = bundle.format_pattern(pattern, Some(&fluent_args), &mut vec![]);
        return formatted.to_string();
    }

    formatted.to_string()
}

/// Normalize text by removing bidirectional control characters
pub fn normalize_text(text: &str) -> String {
    text.replace('\u{2068}', "").replace('\u{2069}', "")
}

/// Set up a Git repository with a remote
pub fn setup_git_repo_with_remote() -> (TempDir, TempDir) {
    // Set up the remote repository
    let remote_dir = TempDir::new().unwrap();
    Command::new("git")
        .args(&["init", "--bare"])
        .current_dir(&remote_dir)
        .assert()
        .success();

    // Set up the local repository
    let local_dir = setup_git_repo();

    // Add the remote
    Command::new("git")
        .args(&[
            "remote",
            "add",
            "origin",
            remote_dir.path().to_str().unwrap(),
        ])
        .current_dir(&local_dir)
        .assert()
        .success();

    // Push initial commit to remote
    Command::new("git")
        .args(&["push", "-u", "origin", "main"])
        .current_dir(&local_dir)
        .assert()
        .success();

    (local_dir, remote_dir)
}
