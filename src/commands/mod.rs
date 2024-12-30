//! Command implementations for git-wippy.
//!
//! This module contains the core functionality for managing WIP (Work In Progress) branches:
//!
//! - `save`: Creates a WIP branch with the current changes
//! - `list`: Shows all WIP branches for the current user
//! - `restore`: Restores changes from a WIP branch back to the original branch
//! - `delete`: Removes WIP branches locally and/or remotely
//!
//! Each command is implemented in its own submodule and follows a pattern of having
//! both a public interface function and a testable implementation that accepts a
//! Git trait object.

pub mod delete;
pub mod list;
pub mod restore;
pub mod save;
