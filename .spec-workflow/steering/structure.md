# Project Structure

## Directory Organization

```
termigroove/
├── src/                      # Rust source code (binary entrypoint in main.rs)
├── tests/
│   └── e2e/                  # End-to-end TUI tests (JS/TS via @microsoft/tui-test)
├── target/                   # Cargo build outputs (debug/release)
├── .spec-workflow/           # Specs & steering docs/templates
│   ├── templates/
│   └── steering/
├── .cursor/                  # Cursor MCP/rules and commands
├── tui-traces/               # Test traces from tui-test
├── package.json              # Dev tooling (tui-test, ts)
├── tui-test.config.ts        # TUI test runner config (binary path, env, size)
├── Cargo.toml                # Rust package manifest
├── rust-toolchain            # Pinned Rust toolchain
└── README.md                 # Project readme
```

## Naming Conventions

### Files
- **Rust source**: `snake_case.rs` (e.g., `main.rs`)
- **Tests (E2E)**: `*.test.ts` / `*.spec.ts` under `tests/e2e/`
- **Docs**: Markdown under `.spec-workflow/`

### Code
- **Rust types/structs/enums**: PascalCase
- **Rust functions/variables**: snake_case
- **Constants**: UPPER_SNAKE_CASE
- **TS test variables/functions**: camelCase

## Import Patterns

### Rust
1. Std lib imports
2. External crates
3. Internal modules

### TypeScript (tests)
1. External packages (`@microsoft/tui-test`)
2. Local helpers (if added)

## Code Structure Patterns

### Rust module organization
1. `use` imports
2. consts/config
3. types
4. implementation
5. helpers

### Test organization (tui-test)
1. Configure program path and env in `tui-test.config.ts`
2. Assert visible text and flows via `terminal.getByText`

## Code Organization Principles
1. Single Responsibility per file
2. Modularity for future TUI modules (views, input, audio)
3. Testability via binary-level E2E tests
4. Consistency with Rust & TS idioms

## Module Boundaries
- Core binary (`src/`) vs. test harness (`tests/e2e`)
- Future: TUI (ratatui) views vs. engine modules; clear one-way deps from UI → core

## Code Size Guidelines
- Keep functions small (<50 lines where practical)
- Prefer splitting into modules when files exceed ~300 lines

## Documentation Standards
- Public modules/functions: rustdoc where applicable
- Complex logic: brief comments above non-obvious sections
- Steering/spec docs live in `.spec-workflow/`


