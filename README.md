# git-wippy

## Description

Git stash with super powers! A dev tool to simplify working with work in progress changes that acts as a sub command for git.

When saving your work it creates a temporary WIP branch (pushes to remote by default) that stores your changes and allows you to restore them later. It even keeps track of which branch you were working on and which files were staged and untracked.

## Installation

Coming soon!

Download the latest binary for your platform under releases and put it in your executable paths.

## Build and install from sources

This project is built with [Rust](https://www.rust-lang.org/), using [cargo](https://doc.rust-lang.org/cargo/) as a build system and package manager.

To install, follow these steps:

1. Install [Rust and cargo](https://www.rust-lang.org/tools/install).
2. Clone this repository.
3. Navigate to the project directory and run `cargo build -r`.
4. Copy `git-wippy` binary from `target/release` to somehwere that is in your executable paths.

## Usage

When `git-wippy` is located somewhere in your executable paths it can be used as a `git` subcommand.

Executed with `git wippy <command>`:

- `restore`: Restore your WIP changes
- `save`: Save your WIP changes (use `--local` flag to only save locally)
- `list`: List all WIP branches

## Contributing

If you want to contribute to this project, please follow these steps:

1. Fork the project.
2. Create a new branch.
3. Make your changes and commit them.
4. Push your changes to your fork.
5. Create a pull request.

## License

This project is licensed under [MIT](./LICENSE).
