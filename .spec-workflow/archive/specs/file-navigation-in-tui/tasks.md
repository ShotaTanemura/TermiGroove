# Tasks: File Navigation in TUI (MVP-1)

Traceability: `requirements.md`, `design.md`; Notion story: https://www.notion.so/26f4150965e48031947af81bce71c4f3

Conventions:
- [ ] Pending, [-] In-progress, [x] Completed
- Each task contains a _Prompt with instructions to mark the task in-progress before starting and complete after passing tests.

## 0) Setup & Baseline (pre-TDD)
- [x] 0.1 Setup crates and compile baseline
  - Files: `Cargo.toml`, `src/main.rs`, `src/app_state.rs`, `src/ui.rs`, `src/input.rs`
  - Add crates: `ratatui`, `crossterm`, `ratatui-explorer`, `tui-big-text` (or alternative)
  - Create module skeletons; compile succeeds (no behavior yet)
  - _Leverage: design.md (State Model), examples from crates_
  - _Requirements: none (bootstrap)_
  - _Prompt: Implement the task for spec file-navigation-in-tui, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust TUI Engineer | Task: Add deps and empty module skeletons that compile | Restrictions: No feature logic | _Leverage: ratatui, crossterm examples | _Requirements: bootstrap | Success: cargo build passes. Before starting, mark this task [-] in tasks.md; when done, mark [x]._

## 1) Initial Render (Layout shell)
- [x] 1.1 Red — E2E: initial render shell
  - File: `tests/e2e/initial-render.test.ts`
  - Assert: header text, 75/25 split panes present, footer present, focus starts Left
  - _Leverage: requirements (Layout & Visuals), `tui-test.config.ts`_
  - _Prompt: Implement the task for spec file-navigation-in-tui... Role: QA (TUI) | Task: Write failing test that drives layout shell | Restrictions: No implementation changes | _Leverage: @microsoft/tui-test | _Requirements: Layout & Visuals | Success: test fails for missing layout. Mark [-] before, [x] after._

- [x] 1.2 Green — Minimal layout shell (tui-big-text header)
  - Files: `src/ui.rs`, `src/main.rs`
  - Render header “WELCOME TO TERMIGROOVE” using `tui-big-text`, 75/25 split, right title `Selected (Enter = To Pads)`, footer status `Ready`
  - _Leverage: design (Rendering) | _Requirements: Layout & Visuals_
  - _Prompt: ... Role: Rust UI Engineer | Task: Implement minimal layout to pass 1.1 | Restrictions: No business logic; stub focus Left | Success: 1.1 passes. Mark [-] then [x]._

- [x] 1.3 Refactor — Layout code tidy
  - Files: `src/ui.rs`
  - Extract helpers; no behavior change
  - _Prompt: ... Role: Clean-code Refactorer | Task: Tidy without behavior change | Success: tests stay green. Mark [-] then [x]._

## 2) Focus Toggle (Tab)
- [x] 2.1 Red — E2E: Tab toggles focus visuals
  - File: `tests/e2e/focus-toggle.test.ts`
  - Assert: Right focused -> reversed+bold; unfocused -> bold only
  - _Prompt: ... Role: QA (TUI) | Task: Write failing test per visuals | Success: fails until implemented. Mark [-]/[x]._

- [x] 2.2 Green — Implement focus toggle
  - Files: `src/app_state.rs`, `src/input.rs`, `src/ui.rs`
  - Add `FocusPane`; Tab toggles; render visuals accordingly
  - _Prompt: ... Role: Input Handling Engineer | Task: Implement Tab and visuals | Restrictions: Pure state update + render only | Success: 2.1 passes. Mark [-]/[x]._

- [x] 2.3 Refactor — Focus helpers
  - Files: `src/app_state.rs`, `src/ui.rs`
  - Extract focus style helpers
  - _Prompt: ... Role: Clean-code Refactorer | Success: tests green. Mark [-]/[x]._

## 3) Explorer Integration (Left traversal)
- [x] 3.1 Red — E2E: traversal keys forwarded (smoke)
  - File: `tests/e2e/explorer-forwarding.test.ts`
  - Act: send Up/Down; Assert: app remains stable; help line visible
  - _Prompt: ... Role: QA (TUI) | Task: Failing smoke to ensure no crash & UI stable | Success: fails for missing forwarding. Mark [-]/[x]._

- [x] 3.2 Green — Wire `ratatui_explorer`
  - Files: `src/app_state.rs`, `src/input.rs`, `src/ui.rs`
  - Instantiate `ratatui_explorer::FileExplorer`; render the explorer in the left pane; forward unhandled keys when Left focused and sync current path/dir
  - _Prompt: ... Role: Rust TUI Engineer | Task: Wire explorer & key forwarding | Success: 3.1 passes. Mark [-]/[x]._

- [x] 3.3 Refactor — Explorer plumbing
  - Files: `src/input.rs`
  - Encapsulate forwarding; no behavior change
  - _Prompt: ... Role: Clean-code Refactorer | Success: tests green. Mark [-]/[x]._

## 4) Selection Rules — Left Space add/remove
- [x] 4.1 Red — Unit tests: add/remove + statuses + cursor
  - File: `tests/unit/app_state_tests.rs`
  - Cover: add appends + status `Added {name}` + cursor to last; toggle removes + status `Removed {name}`; dir -> status `Only files can be selected`; clamp index
  - _Prompt: ... Role: Rust Test Engineer | Task: Write failing unit tests for pure state | Success: fails until logic. Mark [-]/[x]._

- [x] 4.2 Green — Implement Left Space toggle
  - Files: `src/app_state.rs`, `src/input.rs`
  - Maintain `Vec<PathBuf>` + `HashSet` uniqueness; implement toggle + statuses; clamp right_idx
  - _Prompt: ... Role: Input Handling Engineer | Task: Implement minimal logic to pass 4.1 | Success: 4.1 passes. Mark [-]/[x]._

- [x] 4.3 Refactor — Extract clamp helper
  - Files: `src/app_state.rs`
  - Add `clamp_right_idx` helper; no behavior change
  - _Prompt: ... Role: Clean-code Refactorer | Success: tests green. Mark [-]/[x]._

## 5) Right Pane — Navigation & Removal
- [x] 5.1 Red — Unit tests: Up/Down bounds + removal keys
  - File: `tests/unit/app_state_tests.rs`
  - Cover: Up at 0 stays 0; Down at last stays last; Space/Delete/d remove at cursor; empty list no-ops
  - _Prompt: ... Role: Rust Test Engineer | Task: Add failing tests for right pane | Success: fails pre-impl. Mark [-]/[x]._

- [x] 5.2 Green — Implement right navigation/removal
  - Files: `src/input.rs`, `src/app_state.rs`
  - Implement Up/Down clamp; removal keys with status; maintain bounds
  - _Prompt: ... Role: Input Handling Engineer | Task: Minimal logic to pass 5.1 | Success: tests pass. Mark [-]/[x]._

- [x] 5.3 Refactor — Naming/cleanup
  - Files: `src/app_state.rs`
  - Tidy function names and small extractions
  - _Prompt: ... Role: Clean-code Refactorer | Success: tests green. Mark [-]/[x]._

## 6) Mode Switching & Quit
- [x] 6.1 Red — E2E: Enter + q behavior
  - File: `tests/e2e/mode-and-quit.test.ts`
  - Assert: Enter with empty -> status `Select at least one file first`; with ≥1 -> Pads + pads message; `q` quits cleanly
  - _Prompt: ... Role: QA (TUI) | Task: Write failing tests | Success: fails pre-impl. Mark [-]/[x]._

- [x] 6.2 Green — Implement Enter & q
  - Files: `src/input.rs`, `src/main.rs`
  - Implement behaviors per design; ensure terminal restored on quit
  - _Prompt: ... Role: Rust App Engineer | Task: Minimal logic to pass 6.1 | Success: tests pass. Mark [-]/[x]._

- [x] 6.3 Refactor — Shutdown paths
  - Files: `src/main.rs`
  - Extract shutdown helpers; no behavior change
  - _Prompt: ... Role: Clean-code Refactorer | Success: tests green. Mark [-]/[x]._

## 7) Resizing & Focus Visuals polish
- [x] 7.1 Red — E2E: resize stability + visuals
  - File: `tests/e2e/resize-visuals.test.ts`
  - Assert: No corruption/panics after resize; focus visuals remain correct
  - _Prompt: ... Role: QA (TUI) | Task: Write failing resize test | Success: fails pre-impl. Mark [-]/[x]._

- [x] 7.2 Green — Implement resize handling
  - Files: `src/ui.rs`
  - Recompute layout on resize; preserve state
  - _Prompt: ... Role: Rust UI Engineer | Task: Minimal logic to pass 7.1 | Success: tests pass. Mark [-]/[x]._

- [x] 7.3 Refactor — UI code cleanup
  - Files: `src/ui.rs`
  - Extract layout builder functions
  - _Prompt: ... Role: Clean-code Refactorer | Success: tests green. Mark [-]/[x]._

## 8) E2E Coverage Consolidation
- [x] 8.1 Red — E2E: Consolidated AC scenarios
  - File: `tests/e2e/acceptance.test.ts`
  - Combine scenarios: initial, focus toggle, select/remove, bounds, enter requires selection, enter to pads, quit
  - _Prompt: ... Role: QA (TUI) | Task: Write failing umbrella test | Success: fails pre-impl. Mark [-]/[x]._

- [x] 8.2 Green — Fix gaps to pass umbrella
  - Files: `src/**`
  - Address any gaps uncovered by umbrella test
  - _Prompt: ... Role: Implementation-focused TDD Developer | Task: Minimal fixes to pass 8.1 | Success: all E2E green. Mark [-]/[x]._

- [x] 8.3 Refactor — Final tidy
  - Files: `src/**`, `tests/**`
  - Remove duplication; improve names
  - _Prompt: ... Role: Clean-code Refactorer | Success: all tests green. Mark [-]/[x]._
