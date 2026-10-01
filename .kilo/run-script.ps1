# Run script for Agent Manager.
# Runs in the selected worktree (or repo root for local).
# Serves the UI for the basic fixture so worktree reviews are browsable
# immediately (E15). NOTE: UI assets are embedded into the exe at build time --
# after editing crates/ui/assets/** you MUST rebuild before the change shows.

cargo build --bin tool
target\debug\tool.exe ui tests\fixtures\basic
