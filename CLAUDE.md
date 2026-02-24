# Project: wkspace

Rust CLI tool for managing Git worktrees with lifecycle scripts.

## After making changes

Always run the full CI check locally before considering work complete:

```sh
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

If `cargo fmt --check` fails, run `cargo fmt` to fix formatting.
