# General messages
usage-prefix = Usage:
commands-intro = Commands:
commands-category = Available commands:
help-command = Display help information
see-also = See also:

# Command descriptions
save-command-about = Save current changes as a WIP branch
list-command-about = List all WIP branches
delete-command-about = Delete a WIP branch
restore-command-about = Restore changes from a WIP branch

# Operation messages
saving-wip = Saving WIP changes...
created-branch = Created branch '{ $name }'
staged-all-changes = Staged all changes
committed-changes = Changes committed
pushed-changes = Changes pushed to remote
skipped-push-no-remote = No remote repository configured, skipping push
switched-back = Switched back to branch '{ $name }'
delete-complete = WIP branch deleted successfully
no-wip-branches = No WIP branches found for user '{ $username }'
restoring-wip = Restoring changes from branch '{ $name }'...
checked-out-branch = Checked out branch '{ $name }'
unstaged-changes = Unstaged changes
stashed-changes = Stashed changes
applied-stash = Applied stashed changes
recreated-file-states = Recreated original file states
deleted-local-branch = Deleted local branch '{ $name }'
deleted-remote-branch = Deleted remote branch '{ $name }'
restore-complete = Successfully restored changes from '{ $name }'
operation-cancelled = Operation cancelled
branch-not-found = Branch '{ $name }' not found
branch-name = { $name }
wip-branch-created = Created WIP branch '{ $name }'
wip-branch-deleted = Deleted WIP branch '{ $name }' { $remote ->
    [true] (local and remote)
    *[false] (local only)
}

# Dialogue prompts
delete-branch-prompt = Delete this branch?
delete-all-prompt = Delete all { $count } WIP branches?
delete-remote-prompt = Also delete { $count } remote branches?
select-branches-to-delete = Select branches to delete:
selection-instructions = Use space to select/deselect, press enter to confirm
no-branches-selected = No branches selected
selected-branches = Selected branches:
found-wip-branch = Found WIP branch:
found-wip-branches = Found WIP branches:

# Error messages
remote-delete-failed = Failed to delete remote branch '{ $name }': { $error }

# Help messages
save-local-help = Do not push changes to remote repository
save-username-help = Specify a custom username
save-datetime-help = Specify a custom date and time
delete-branch-help = Name of the branch to delete
delete-all-help = Delete all WIP branches
delete-force-help = Skip confirmation prompt
delete-local-help = Only delete local branches
restore-branch-help = Name of the branch to restore
restore-force-help = Skip confirmation prompt
restore-autostash-help = Automatically stash and reapply local changes

# Stashing messages
stashing-existing-changes = Stashing existing changes...
restoring-existing-changes = Restoring existing changes...
