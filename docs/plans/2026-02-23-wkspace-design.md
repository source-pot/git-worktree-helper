# Design: wkspace

> Date: 2026-02-23
> Status: Approved

## Overview

**wkspace** is a CLI tool that wraps `git worktree` to simplify creating isolated workspaces. It manages the full lifecycle: create a worktree + branch, run setup scripts, open a shell in the worktree, run teardown scripts, and clean up completely on removal.

Built as a standalone Rust binary with no runtime dependencies.

## Commands

```
wkspace init               # Create .wkspace.toml with sensible defaults
wkspace new <name>         # Create worktree + branch, run setup, spawn subshell
wkspace rm <name>          # Run teardown (stop on failure), force-remove dir, delete branch
wkspace list               # Show active worktrees (name, branch, path)
wkspace open <name>        # Spawn subshell in existing worktree
```

## Command Behavior

### wkspace init

- Creates `.wkspace.toml` in the current repo root with commented defaults.
- Fails if not inside a git repo.
- No-op if `.wkspace.toml` already exists (idempotent).

### wkspace new \<name\>

1. Validate: must be in a git repo, name must not already exist.
2. If no `.wkspace.toml` exists, auto-run `init` first (print a message so user knows).
3. `git worktree add .worktrees/<name> -b <name>` based off the configured `base_branch`.
4. Run each command in `[scripts].setup` sequentially (cwd = new worktree). Stop on first failure.
5. Spawn an interactive subshell (`$SHELL`) with cwd = the new worktree.

### wkspace rm \<name\>

1. Run each command in `[scripts].teardown` sequentially (cwd = the worktree). Stop on first failure.
2. Force-remove the worktree directory (`rm -rf .worktrees/<name>`).
3. `git worktree prune` to clean up stale worktree references.
4. `git branch -D <name>` to delete the branch.

Note: `git worktree remove` is insufficient because it fails when untracked files (e.g. `.env`) exist in the worktree. Force-removing the directory + pruning handles this cleanly.

### wkspace list

- Parse `git worktree list --porcelain` and display managed worktrees (those under the configured worktree directory).
- Show: name, branch, path.

### wkspace open \<name\>

- Validate worktree exists.
- Spawn interactive subshell (`$SHELL`) with cwd = the worktree directory.

## Auto-Init Behavior

Any command that needs config (`new`, `rm`, `list`, `open`) will auto-create `.wkspace.toml` with defaults if it's missing. A message is printed: `Created .wkspace.toml with defaults`.

## Config: .wkspace.toml

```toml
[worktree]
base_branch = "main"
directory = ".worktrees"

[scripts]
setup = [
    "npm install",
    "cp .env.example .env",
]
teardown = []
```

- **base_branch**: Branch that new worktrees are always based on.
- **directory**: Relative path (from repo root) where worktrees are stored.
- **scripts.setup**: List of shell commands run sequentially after worktree creation. Each command can be an inline shell command or a path to a script file — a command is a command either way.
- **scripts.teardown**: List of shell commands run sequentially before worktree removal.

## Naming

- Worktree directory name = the `<name>` argument.
- Git branch name = the `<name>` argument (exact match, no prefix).

## Error Handling

| Situation | Behavior |
|-----------|----------|
| Not in a git repo | Clear error message |
| Missing `.wkspace.toml` | Auto-init with defaults |
| Worktree name already exists (`new`) | Error, suggest `wkspace open <name>` |
| Branch already exists (`new`) | Error, suggest a different name |
| Setup script fails (`new`) | Stop, print which command failed, leave worktree in place (user can `wkspace rm` to clean up) |
| Teardown script fails (`rm`) | Stop, print which command failed, leave worktree in place (user fixes and retries) |
| Non-existent worktree (`rm`, `open`) | Clear error message |

## Project Structure

```
wkspace/
  Cargo.toml
  src/
    main.rs           # CLI entry point, clap definitions
    commands/
      mod.rs
      init.rs
      new.rs
      rm.rs
      list.rs
      open.rs
    config.rs         # .wkspace.toml parsing
    git.rs            # Git worktree/branch operations
    scripts.rs        # Setup/teardown script runner
    error.rs          # Error types
```

## Key Crates

- `clap` (derive) — CLI argument parsing
- `serde` + `toml` — config parsing
- `anyhow` — error handling
- `which` — find user's shell

## Decisions & Rationale

1. **Subshell over new terminal window**: Decoupled from platform-specific terminal emulators. The `open` command name leaves room to change this behavior later.
2. **Worktrees nested in repo (.worktrees/)**: Keeps worktrees co-located with the repo. Directory should be added to `.gitignore`.
3. **Force-remove on rm**: `git worktree remove` fails with untracked files. Force-removing the directory then pruning is more reliable for complete cleanup.
4. **Stop on failure for both setup and teardown**: Consistent behavior, easier to debug. User sees exactly which command failed.
5. **Auto-init**: Reduces friction. User doesn't need to remember to run `wkspace init` before first use.
