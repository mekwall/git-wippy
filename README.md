# git-wippy

A Git subcommand for managing work-in-progress changes across branches. Stash with super powers! 🦸‍♂️

## Features

- 🔄 Save WIP changes to a dedicated branch
- 📋 List all your WIP branches
- ⚡ Restore WIP changes back to their original branches
- 🔒 Thread-safe and async operations
- 🌟 Maintains file states (staged, unstaged, untracked)
- 🔍 Smart branch naming with username and timestamps
- 🔄 Interactive branch selection for restore operations
- 🌐 Supports both local and remote operations

## Installation

### From Package Managers

Coming soon!

### Build from Source

1. Install [Rust and cargo](https://www.rust-lang.org/tools/install)
2. Clone this repository
3. Build and install:

```bash
cargo install --path .
```

## Usage

When `git-wippy` is located somewhere in your executable paths it can be used as a `git` subcommand.

### Commands

```bash
# Save your WIP changes (pushes to remote by default)
git wippy save [--message "Your message"]
git wippy save --local  # Save locally only

# List all your WIP branches
git wippy list
git wippy list --all    # Show all users' WIP branches

# Restore changes from a WIP branch
git wippy restore                # Interactive selection
git wippy restore <branch-name>  # Direct restore
```

### Examples

```bash
# Save changes with a custom message
git wippy save -m "Feature work in progress"

# List only your WIP branches with details
git wippy list

# Restore specific WIP changes
git wippy restore wip/username/2024-03-21-175930
```

## How It Works

1. **Saving Changes**:

   - Creates a WIP branch named `wip/{username}/{timestamp}`
   - Preserves the state of all files (staged, unstaged, untracked)
   - Stores metadata about the source branch
   - Optionally pushes to remote

2. **Listing Changes**:

   - Shows all WIP branches for the current user
   - Displays branch creation time and source branch
   - Supports filtering and detailed views
   - Color-coded output for better readability

3. **Restoring Changes**:
   - Interactive branch selection with preview
   - Smart conflict resolution
   - Recreates original file states
   - Automatic cleanup of restored WIP branches

## Development

### Requirements

- Git 2.0+
- Rust 1.70+

### Testing

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test <test_name>
```

### Contributing

1. Fork the project
2. Create a new branch
3. Make your changes and commit them
4. Push your changes to your fork
5. Create a pull request

Please ensure your PR:

- Includes tests for new functionality
- Updates documentation as needed
- Follows the existing code style

## Technical Details

Built with:

- 🦀 Rust for performance and safety
- 🔄 [tokio](https://tokio.rs/) for async operations
- 📎 [clap](https://clap.rs/) for CLI argument parsing
- 🎨 [owo-colors](https://docs.rs/owo-colors/) for terminal coloring
- 🔍 [dialoguer](https://docs.rs/dialoguer/) for interactive prompts

## License

This project is licensed under the [MIT License](./LICENSE).
