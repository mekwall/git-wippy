# git-wippy

A Git utility for managing work-in-progress changes across branches with super powers! 🦸‍♂️

## Features

- 🔄 Save WIP changes to a dedicated branch
- 📋 List all your WIP branches
- ⚡ Restore WIP changes back to their original branches
- 🔒 Thread-safe and async operations
- 🌟 Maintains file states (staged, unstaged, untracked)
- 🔍 Smart branch naming with username and timestamps
- 🌐 Supports both local and remote operations

## Installation

Binary releases and automatic publishing to package repositories are coming soon!

## Build and install from sources

This project is built with [Rust](https://www.rust-lang.org/), using [cargo](https://doc.rust-lang.org/cargo/) as a build system and package manager.

To install, follow these steps:

1. Install [Rust and cargo](https://www.rust-lang.org/tools/install).
2. Clone this repository.
3. Navigate to the project directory and run `cargo build -r`.
4. Copy `git-wippy` binary from `target/release` to somewhere in your executable paths.

## Usage

When `git-wippy` is located somewhere in your executable paths it can be used as a `git` subcommand.

### Commands

```bash
# Save your WIP changes (pushes to remote by default)
git wippy save

# Save changes locally only
git wippy save --local

# List all your WIP branches
git wippy list

# Restore changes from a WIP branch
git wippy restore
```

## How It Works

1. **Saving Changes**:

   - Creates a WIP branch named `wip/{username}/{timestamp}`
   - Preserves the state of all files (staged, unstaged, untracked)
   - Stores metadata about the source branch
   - Optionally pushes to remote

2. **Listing Changes**:

   - Shows all WIP branches for the current user
   - Filters branches matching `wip/{username}/*`
   - Handles both local and remote branches

3. **Restoring Changes**:
   - Retrieves changes from the WIP branch
   - Recreates the original file states
   - Returns to the source branch
   - Cleans up the WIP branch

## Development

### Requirements

- Git 2.0+
- Rust 1.70+

### Contributing

If you want to contribute to this project, please follow these steps:

1. Fork the project.
2. Create a new branch.
3. Make your changes and commit them.
4. Push your changes to your fork.
5. Create a pull request.

```bash
# Run tests
cargo test

# Build release version
cargo build -r

# Install locally
cargo install --path .
```

## Acknowledgments

- Built with Rust 🦀
- Uses [tokio](https://tokio.rs/) for async operations
- Uses [clap](https://clap.rs/) for CLI argument parsing

## License

This project is licensed under the [MIT License](./LICENSE).
