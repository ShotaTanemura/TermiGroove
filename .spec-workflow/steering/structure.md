# Project Structure

## Directory Organization

```
TermiGroove/
├── src/                      # Rust source (state, audio, UI, input, selection modules)
│   ├── app_state.rs          # Global application state & focus models
│   ├── audio.rs              # Audio engine commands, thread management, CPAL integration
│   ├── input.rs              # Keyboard and event handling, focus routing
│   ├── selection.rs          # File selection modeling for pads workflow
│   ├── ui.rs                 # ratatui rendering for explorer, pads, summary/popup
│   ├── main.rs               # Binary entrypoint wiring terminal loop
│   └── state/                # Supporting state models (if present)
├── tests/                    # Rust integration/unit tests and TUI e2e harness
│   ├── app_state_tests.rs    # Unit tests for AppState behavior
│   ├── input_handling_tests.rs
│   ├── integration_*.rs      # Scenario tests for file navigation, pads, etc.
│   ├── unit/                 # Additional fine-grained tests
│   └── e2e/                  # @microsoft/tui-test scripts (.test.ts)
├── samples/                  # Example audio files for manual testing
├── termigroove/              # (Optional) workspace or generated artifacts
├── .serena/                  # Serena MCP agent data (memories, configs) supporting AI workflows
├── Cargo.toml                # Rust crate manifest
├── Cargo.lock                # Locked dependencies
├── package.json              # Node scripts for e2e testing
├── package-lock.json         # Node dependency lockfile
├── tui-test.config.ts        # Configuration for tui-test harness
├── .spec-workflow/           # Spec and steering docs, decision logs
└── .cursor/                  # Automation rules and commands
```

> Note: `.serena/` captures AI-assistant memory/configuration per the Serena MCP tooling (see https://github.com/oraios/serena). These files should be preserved for agentic development workflows.

## Naming Conventions

### Files
- Rust modules use `snake_case` (`app_state.rs`, `input_handling_tests.rs`).
- Directories also use `snake_case` or lowercase (`state`, `tests`, `samples`).
- Test files mirror module names with `_tests` suffix or scenario descriptors.

### Code
- **Enums/Structs**: `PascalCase` (`AppState`, `PopupFocus`).
- **Functions/Methods**: `snake_case` (`handle_event`, `enter_pads`).
- **Constants**: `UPPER_SNAKE_CASE` (`HELP_LINE`, `BPM_MIN`).
- **Variables**: `snake_case` (`current_left_item`, `draft_bpm`).

## Import Patterns

### Import Order
1. External crates (standard library, third-party).
2. Internal modules via `crate::` or module path.
3. No style imports (TUI styling handled inline).

### Module/Package Organization
- Prefer absolute crate paths (`crate::audio::AudioCommand`).
- Cross-module access done through public methods on `AppState` rather than exposing inner fields.
- Tests import modules with `use termigroove::*` or direct module references.

## Code Structure Patterns

### Module Organization
1. Imports and use statements.
2. Constants and helper functions.
3. Struct/enum definitions.
4. Core implementations (`impl` blocks) with public API first.
5. Private helpers at bottom where needed.

### Function Organization
- Validate input/state early (e.g., selection checks in `enter_pads`).
- Update state via dedicated setters to maintain invariants (BPM, bars clamp).
- Emit audio/UI commands after state mutation.
- Return `Result` with context via `anyhow` when operations may fail.

### File Organization Principles
- One primary component per file (state, input, UI, audio separated).
- Keep UI rendering logic in `ui.rs`; avoid mixing with state transitions.
- Tests colocated by concern (AppState tests interact with state behaviors only).

## Code Organization Principles
1. **Single Responsibility**: Each module owns a distinct concern (state vs UI vs audio).
2. **Separation of Concerns**: Input handlers mutate `AppState`; UI reads from state.
3. **Testability**: Public getters/setters enable unit and integration testing without UI coupling.
4. **Predictability**: Clamping and validation centralized to avoid duplicated logic.

## Module Boundaries
- `AppState` is the central source of truth; other modules depend on it via public APIs.
- `audio` module only receives commands, never directly reads UI state.
- `ui` module renders based on read-only state access; no side effects.
- `input` owns translating events into state mutations and audio commands.
- Tests may mock or simulate audio via channel senders, keeping coupling low.

## Code Size Guidelines
- Aim to keep modules under ~400 lines; split when adding major features.
- Functions ideally ≤50 lines; refactor lengthy logic into helpers.
- Limit enum variants to what is necessary; prefer compositional state over deep nesting.

## Dashboard/Monitoring Structure (if applicable)
- TUI dashboard components live in `ui.rs`; summary boxes and popups use dedicated rendering functions.
- No separate dashboard subsystem yet; future remote/analytics components would reside under `src/ui/` submodules or dedicated directories.

## Documentation Standards
- Public-facing enums and structs documented when behavior is non-obvious.
- Complex focus or input flows warrant inline comments near state transitions.
- Steering/spec documents maintained under `.spec-workflow/` with decision logs updated per feature.
