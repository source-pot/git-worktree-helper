# Project: wkspace

Rust CLI tool for managing Git worktrees with lifecycle scripts.

## Workflow

When starting a new task, create a worktree:

```sh
wkspace new <branch-name> --no-shell
```

Then work in `.worktrees/<branch-name>/`. When complete, commit, push, and create a PR. Clean up with `wkspace rm <branch-name>`.

## After changing command behaviour

Update `README.md` to reflect any new flags, changed steps, or removed features.

## After making changes

Always run the full CI check locally before considering work complete:

```sh
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

If `cargo fmt --check` fails, run `cargo fmt` to fix formatting.
