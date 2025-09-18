# Design: File Navigation in TUI (MVP-1)

Source of truth: [Notion – [MVP-1] File Navigation in TUI](https://www.notion.so/26f4150965e48031947af81bce71c4f3)
Related: `.spec-workflow/specs/file-navigation-in-tui/requirements.md`

## Architectural Overview
- Language/runtime: Rust.
- TUI framework: `ratatui`.
- File explorer: `ratatui_explorer::FileExplorer` for delegating directory navigation and key handling for tree traversal.
- Asynchronous split-loop architecture:
  - UI loop (MVU-style): deterministic input handling and pure rendering; owns `AppState` and `crossterm` event pump.
  - Real-time audio thread: device callbacks, transport, scheduling, and mixing (pads mode lifecycle); communicates via lock-free channels.
  - I/O/decoding pool: file access, decoding, and waveform precomputation off the UI thread; communicates progress/results back to UI.

## Screen Layout
- Header (big text), Body split (75% Left explorer / 25% Right selected list), Footer (status).
- Focus management between Left/Right panes driven by a `focus` enum.

## State Model
```rust
pub enum Mode { Browse, Pads }

pub enum FocusPane { LeftExplorer, RightSelected }

pub struct AppState {
    pub mode: Mode,
    pub focus: FocusPane,
    pub status_message: String,
    pub selected_files: Vec<std::path::PathBuf>, // preserves insertion order; names rendered only
    pub right_idx: usize, // cursor index for Right pane; clamped to bounds
    pub explorer: ratatui_explorer::FileExplorer, // provides directory navigation
}
```

- `selected_files` holds unique files only; toggling removes existing.
- `right_idx` always within `[0, selected_files.len().saturating_sub(1)]`.

## Key Handling
- Global:
  - `Tab`: toggle focus `LeftExplorer` ⇄ `RightSelected`.
  - `Enter`:
    - If `selected_files.is_empty()` → stay `Browse`, set status `Select at least one file first`.
    - Else switch mode to `Pads`, set status `[Pads] Press Esc to go back. Press Q/W/…/< to trigger.`
  - `q` in `Browse` requests quit.

- Left (Explorer):
  - Forward unhandled keys to `explorer.on_key(event)` to allow navigation.
  - `Space`:
    - If cursor on file: toggle selection.
      - If newly added: append; set status `Added {name}`; set `right_idx` to last.
      - If removed (already existed): remove; set status `Removed {name}`; adjust `right_idx` to in-bounds.
    - If cursor on directory: do nothing and set status `Only files can be selected`.

- Right (Selected list, only when focused):
  - `Up`/`Down`: move `right_idx` within bounds; no wrap.
  - `Space`/`Delete`/`d`: remove item at `right_idx`; set status `Removed {name}`; clamp `right_idx`.
  - If list is empty: `Up`/`Down`/`Space`/`Delete`/`d` are no-ops; no status change.

## Rendering
- Use `ratatui` layout constraints to split body 75/25.
- Header: `tui_big_text` or equivalent big text style for “WELCOME TO TERMIGROOVE”.
- Left explorer: bordered block, green theme, highlight symbol `> `.
- Right selected: bordered block titled `Selected (Enter = To Pads)`, green file names.
- Footer: centered status message.
- Focus visualization:
  - Right focused: highlighted row reversed + bold.
  - Right unfocused: bold only.

## Data Structures & Algorithms
- Selection uniqueness: maintain a `HashSet<PathBuf>` alongside `selected_files` to check existence in O(1). Keep `Vec<PathBuf>` for order. Remove by index or by path with retain; both operations are O(n) but lists are small.
- Cursor clamping helper:
```rust
fn clamp_right_idx(state: &mut AppState) {
    if state.selected_files.is_empty() { state.right_idx = 0; return; }
    let last = state.selected_files.len() - 1;
    if state.right_idx > last { state.right_idx = last; }
}
```

## Resize Handling
- On `CrosstermEvent::Resize(..)`, recompute layout. Keep `AppState` unchanged; rerender.

## Errors & Robustness
- No panics on empty lists or out-of-range indices (always clamp).
- Ensure terminal raw mode is properly enabled/disabled on enter/exit; restore on quit.

## Testing Strategy
- Unit tests: cover state transitions (focus toggle, selection add/remove, cursor clamping), mode switching logic, and message/status generation using pure functions.
- E2E tests using `@microsoft/tui-test` under `tests/e2e/` with `tui-test.config.ts`.
- Scenarios mirror Acceptance Criteria (initial render, focus toggle, selection, remove, bounds, enter to pads, quit).
- Use a temporary test directory with files/dirs to simulate explorer interactions; mock/adapter for `ratatui_explorer` if needed.

## Open Points
- Starting directory for explorer (cwd vs. home) – default to process cwd.
- File filters (wav/mp3 only) – optional for MVP; selection UI shows all, filtering can be applied when loading audio in later specs.

## Implementation Notes
- Add crates: `ratatui`, `crossterm`, `ratatui-explorer`, `tui-big-text` (or alternative) in `Cargo.toml`.
- Structure code into modules: `app_state`, `ui`, `input`, `main`.
- Keep rendering and input handling pure functions where possible for testability.
