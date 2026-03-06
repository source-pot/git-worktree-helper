# wkspace

A CLI tool to manage Git worktrees with lifecycle scripts.

## Why

Git worktrees let you work on multiple branches simultaneously without stashing or cloning. But managing them by hand gets tedious — you have to create branches, set up directories, install dependencies, and remember to clean everything up when you're done.

wkspace automates this. Define setup and teardown scripts once, and every worktree gets the same consistent environment. When you're done, `wkspace rm` tears it all down cleanly.

## Features

- Create worktrees with a single command — branch, directory, and shell session included
- Create worktrees from existing local or remote branches with `wkspace from`
- Interactive prompt for worktree name when not provided as an argument
- Interactive picker to select branches or worktrees
- Re-run setup scripts in an existing worktree with `wkspace setup`
- Run setup scripts automatically after creating a worktree (e.g. `npm install`, `cp .env.example .env`)
- Run teardown scripts before removal (e.g. `docker compose down`)
- Allocate random available ports and expose them as environment variables to scripts and shell
- Auto-creates `.wkspace.toml` config on first use
- Adds the worktrees directory to `.gitignore` automatically
- Lists only wkspace-managed worktrees, not all git worktrees

## Installation

Requires [Rust](https://www.rust-lang.org/tools/install) (cargo).

```sh
cargo install --path .
```

## Platform Support

wkspace is developed and tested on macOS. Pre-built binaries for Linux and Windows are provided as-is, with no guarantees of functionality. If you encounter platform-specific issues, bug reports are welcome.

## Quick Start

```sh
# Initialize config (optional — created automatically on first command)
wkspace init

# Create a worktree called "my-feature" and drop into a shell
wkspace new my-feature

# Or create a worktree from an existing branch
wkspace from feat/login

# ... work on your feature, then exit the shell ...

# Re-run setup scripts (e.g. after pulling changes)
wkspace setup

# Back in your main tree — list active worktrees
wkspace list

# Re-open the worktree shell later
wkspace open my-feature

# Clean up when done
wkspace rm my-feature
```

## Commands

### `wkspace init`

Creates `.wkspace.toml` with default configuration and adds `.worktrees` to `.gitignore`.

This is optional — running any other command will auto-create the config if it doesn't exist.

### `wkspace new [name]`

1. Creates a new branch `<name>` from the configured base branch
2. Creates a worktree at `.worktrees/<name>` (or whatever `directory` is configured)
3. Allocates any configured ports and prints the assignments
4. Runs all `setup` scripts in the worktree directory (with port env vars available)
5. Opens an interactive shell in the worktree (with port env vars available)

If `name` is omitted, an interactive prompt asks for one.

Use `--no-scripts` to skip running setup scripts. Use `--no-shell` to skip opening the interactive shell.

Fails if the branch or worktree already exists.

### `wkspace from [branch]`

Creates a worktree from an existing local or remote branch.

1. Fetches the latest state from the remote
2. Checks out the branch into `.worktrees/<name>` (branch slashes are replaced with dashes, e.g. `feat/login` → `feat-login`)
3. Allocates any configured ports
4. Runs all `setup` scripts
5. Opens an interactive shell

If `branch` is omitted, an interactive picker shows all available branches (excluding those already attached to a worktree).

Use `--no-scripts` to skip running setup scripts.

Selecting the base branch (e.g. `main`) is not allowed — use `wkspace new` instead to create a fresh branch.

### `wkspace setup`

Re-runs setup scripts in the current worktree. Useful after pulling changes or resetting your environment.

1. Validates you're inside a git worktree (not the main working tree)
2. Loads `.wkspace.toml` from the main repository root (must already exist)
3. Allocates fresh ports and prints the assignments
4. Runs all `setup` scripts in the current directory (with port and `$WORKTREE_NAME` env vars)

Must be run from inside a worktree created by `wkspace new`. Does not spawn a shell — you're already in one.

### `wkspace open <name>`

Opens an interactive shell in an existing worktree. Does not re-run setup scripts.

Uses `$SHELL` (falls back to `/bin/sh`).

### `wkspace list`

Lists wkspace-managed worktrees with their name, branch, and path. Only shows worktrees inside the configured directory — not all git worktrees in the repo.

### `wkspace rm [name]`

1. Runs all `teardown` scripts in the worktree directory
2. Removes the worktree directory
3. Prunes stale worktree references (`git worktree prune`)
4. Force-deletes the branch (`git branch -D`)

If `name` is omitted, an interactive arrow-key picker is shown to select from active worktrees.

Use `--no-scripts` to skip running teardown scripts.

Fails if the worktree doesn't exist. Teardown script failure stops the removal.

## Configuration

wkspace is configured via `.wkspace.toml` at the repository root:

```toml
[worktree]
# Branch that new worktrees are based on
base_branch = "main"

# Directory (relative to repo root) where worktrees are stored
directory = ".worktrees"

[scripts]
# Commands to run after creating a worktree (runs in worktree directory)
setup = []

# Commands to run before removing a worktree (runs in worktree directory)
teardown = []

# [ports]
# Allocate random available ports and expose as env vars to scripts and shell
# Format: label = "ENV_VAR_NAME"
# frontend_port = "FRONTEND_PORT"
# backend_port = "BACKEND_PORT"
```

### Example with scripts and ports

```toml
[worktree]
base_branch = "main"
directory = ".worktrees"

[scripts]
setup = [
    "cp .env.example .env",
    "npm install",
]
teardown = [
    "docker compose down",
]

[ports]
frontend = "FRONTEND_PORT"
backend = "BACKEND_PORT"
```

Each port is randomly allocated from the range 10000–11000 and guaranteed to be available at the time of allocation. The environment variables are injected into both setup scripts and the interactive shell.

Scripts run sequentially via `sh -c` and stop on the first failure.

## How It Works

Under the hood, wkspace wraps standard git commands:

- **`new`** runs `git worktree add .worktrees/<name> -b <name> <base_branch>`
- **`from`** runs `git worktree add .worktrees/<name> <existing-branch>` (no `-b`)
- **`rm`** removes the directory, runs `git worktree prune`, then `git branch -D <name>`
- **`list`** parses `git worktree list --porcelain` and filters to managed worktrees

All worktrees live under a single directory (`.worktrees/` by default) which is automatically added to `.gitignore`. Each worktree gets its own branch with the same name as the worktree.

## License

MIT.

