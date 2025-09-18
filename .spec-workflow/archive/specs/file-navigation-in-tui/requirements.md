# Requirements: File Navigation in TUI (MVP-1)

Source of truth: [Notion – [MVP-1] File Navigation in TUI](https://www.notion.so/26f4150965e48031947af81bce71c4f3)

## Context
- Product framing: see `.spec-workflow/steering/product.md` for audience, principles, and success metrics.
- TUI app built in Rust with `ratatui`; target is keyboard-only UX on macOS/Linux terminals.

## User Story
As a user, I want to navigate my filesystem using the keyboard (arrow keys or WASD) within the TUI so that I can select audio files without leaving the application.

## Definitions
- Browse mode: default application mode focused on selecting files.
- Pads mode: subsequent mode entered after selecting ≥1 file, used for triggering samples (out of scope for this doc beyond entry conditions).
- Explorer (Left pane): file/directory tree powered by `ratatui_explorer::FileExplorer`.
- Selected list (Right pane): list of chosen file names; insert-order preserved; no duplicates.

## Acceptance Criteria
### Layout & Visuals
- On launch the app starts in Browse mode with status `Ready`.
- Header shows centered big text “WELCOME TO TERMIGROOVE” and centered subtitle “Load your samples...”.
- Body split 75%/25%:
  - Left: directory/file explorer (green theme, rounded borders, highlight symbol `> `).
  - Right: boxed list titled `Selected (Enter = To Pads)`.
- Help line is rendered via explorer theme title bottom: “Enter: to pads Space: select Tab: switch pane d/Delete: remove q: quit”.
- Footer shows the current status centered.
- Focus starts Left; Tab toggles focus between Left/Right.
  - When Right is focused, highlight is reversed + bold; unfocused is bold only.

### Navigation & Input (Left)
- Unhandled keys are forwarded to `ratatui_explorer::FileExplorer`; file navigation works.
- Space on a file toggles selection (see Selection rules).
- Space on a directory does not select and sets status `Only files can be selected`.

### Selection List (Right)
- Initially empty; shows file names (not paths) in green; no duplicates; preserves insertion order.
- Right list has cursor `right_idx`:
  - With Right focus, Up/Down moves cursor; never out of bounds.
  - With empty list, Up/Down/Space/Delete/d are no-ops; no status change; no panics.
- Removing with Right focus:
  - Space/Delete/d removes the item at cursor; status `Removed {name}`.
  - After removing last item, cursor moves to previous if any; else none selected.
- Toggling a file already in list from Left removes it; status `Removed {name}`.

### Selection Rules (Left – Space)
- Selecting a file appends to Right list, moves `right_idx` to that new item, status `Added {name}`.
- Toggling selected file again removes it and updates cursor as above.
- Directories cannot be selected (status `Only files can be selected`).

### Mode Switching & Quit
- Enter from either pane:
  - If no files selected: remain in Browse; status `Select at least one file first`.
  - If ≥1 file selected: switch to Pads; status `[Pads] Press Esc to go back. Press Q/W/…/< to trigger.`
- `q` in Browse requests app quit; event loop ends cleanly; terminal restored.

### Resizing & Robustness
- On terminal resize, layout recalculates without corruption or panics; status preserved.
- Repeated add/remove, rapid focus switching, and boundary navigation must not panic; indices remain valid.

### Key Scenarios (Gherkin)
- Initial render: mode Browse; status `Ready`; header/subtitle visible; Left/Right panes shown; focus Left.
- Focus toggle: Tab switches focus Left ⇄ Right; Right focused highlight reversed + bold.
- Select a file: Space on unselected file appends to Right; cursor points to it; status `Added {name}`.
- Attempt to select a directory: Space shows `Only files can be selected`.
- Toggle remove from Left: Space on selected file removes; status `Removed {name}`.
- Remove from Right: Space/Delete/d removes at cursor; cursor stays in-bounds; status `Removed {name}`.
- Right list bounds: Up at index 0 stays 0; Down at last index stays last.
- Enter requires selection: empty `Selected` keeps Browse; status `Select at least one file first`.
- Enter with selection: switches to Pads; status `[Pads] ...`.

## Constraints & Non-Goals
- Keyboard-only; mouse interactions out of scope.
- No audio playback behavior specified here; Pads mode behavior beyond entry message is out of scope.
- Paths vs names: `Selected` list shows names only; underlying paths retained in state.

## Technical Notes (from Notion)
- ratatui: TUI framework
- tui-widgets: big_text for header
- ratatui-explorer: file explorer integration

### Minor Enhancements (MVP-1 polish)
1) Move help line into explorer title bottom
- Render help via `ExplorerTheme.with_title_bottom`; no separate paragraph.
2) Remove standard-size header title
- Only big-text header remains; no duplicate plain title.
3) Resize and center header contents
- Ensure big-text is fully visible and header content is centered.
4) Right selected pane navigation and actions
- With Right focus:
  - Up/Down move the `right_idx` cursor and never go out of bounds.
  - Space/Delete/d remove the item at `right_idx`; status `Removed {name}`.
  - After removal, cursor moves to previous item if any; none if list becomes empty.
- With Left focus: Right-pane Up/Down/Space/Delete/d are no-ops.
- Tab toggles focus Left ⇄ Right.
5) Center-align footer
- Footer status is centered horizontally across resizes.

## Open Questions
- Which root directory should the explorer start in? (e.g., current working directory)
- Maximum size of `Selected` list?

## Decision Log
- 2025-09-16: Created requirements from Notion story and product context; awaiting approval.



