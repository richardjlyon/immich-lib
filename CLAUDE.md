# Project Rules for immich-lib

## Rust Quality Gates

**ALWAYS run before completing any plan/task:**

```bash
cargo clippy -- -D warnings
```

- Treat clippy warnings as errors
- Fix all lints before committing
- Run clippy as part of verification, not just `cargo build`
