# Project Rules for immich-lib

## Rust Quality Gates

**ALWAYS run before completing any plan/task:**

```bash
cargo clippy -- -D warnings
```

- Treat clippy warnings as errors
- Fix all lints before committing
- Run clippy as part of verification, not just `cargo build`

## Rust Idioms

**NEVER use `.unwrap()` or `.expect()` in library code** - use `?` operator instead.

- Prefer `Result<T, E>` for fallible operations, not panics
- Use `thiserror` for library errors, `anyhow` for binaries only
- Use `if let` / `match` for Option handling
- Derive `Debug`, `Clone`, `Serialize` etc. to reduce boilerplate

## Code Inspection

**Use Serena MCP tools for code analysis** instead of basic Read/Edit:

- `mcp__serena__get_symbols_overview` - Get file structure before reading
- `mcp__serena__find_symbol` - Find functions, structs, enums by name
- `mcp__serena__search_for_pattern` - Search code patterns across codebase
- `mcp__serena__find_referencing_symbols` - Find where symbols are used
- `mcp__serena__replace_symbol_body` - Replace function/struct implementations

## Web Browsing

**Use Playwright MCP for web browsing** instead of WebFetch:

- `mcp__playwright__browser_navigate` - Navigate to URLs
- `mcp__playwright__browser_snapshot` - Get page accessibility snapshot
- `mcp__playwright__browser_click` - Interact with elements

Playwright provides full browser automation for API docs, testing, etc.
