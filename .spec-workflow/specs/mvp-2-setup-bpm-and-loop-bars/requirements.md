# Requirements – mvp-2-setup-bpm-and-loop-bars

## Introduction
Enable users on the pads screen to configure the global tempo (BPM) and loop length (bars) using keyboard-only workflows in a retro‑futuristic TUI. This unlocks creating loops at the desired time base and ensures consistent playback timing across pads.

## Alignment with Product Vision
- Aligns with simplicity & flow: keyboard‑first, minimal UI with clear focus/selection semantics.
- Supports accessible, social music creation by making tempo/loop base easy to set without leaving the creative flow.

## Requirements

### Requirement 1 – Show and focus the Tempo & Loop Summary Box
**User Story:** As a user, I want a visible bpm/bars box at the top‑left so I can quickly access global tempo and loop length.

#### Acceptance Criteria
1. WHEN the user is on the pads screen AND presses any arrow key THEN the bpm/bars box appears at the top‑left with default values and becomes focusable with a visible outline.
2. WHEN the box is focused via arrow keys THEN a white focus ring is shown (read‑only state until Enter is pressed).

### Requirement 2 – Enter configuration via Enter key to open center popup
**User Story:** As a user, I want to press Enter on the bpm/bars box to edit values in a modal popup.

#### Acceptance Criteria
1. WHEN focus is on the bpm/bars box AND the user presses Enter THEN the box becomes selected (active feel) AND a center popup appears.
2. WHEN the popup appears THEN the bpm input is focused with the caret, and both bpm and bars inputs display their current values as defaults.

### Requirement 3 – Edit bpm/bars like simple text inputs
**User Story:** As a user, I want the bpm and bars fields to behave like standard text inputs so editing is predictable.

#### Acceptance Criteria
1. WHEN editing bpm (or bars) THEN arrow keys ←/→ move the caret and numeric typing overwrites/appends characters.
2. WHEN navigating within the popup THEN ↑/↓ moves focus vertically among bpm → bars → OK (looping), and OK ↔ Cancel horizontally (looping).
3. WHEN focus is on OK and the user presses Enter THEN changes are confirmed; WHEN focus is on Cancel and the user presses Enter or presses Esc THEN changes are discarded.

### Requirement 4 – Apply or discard changes and update UI/state
**User Story:** As a user, I want confirming to update global bpm/bars and cancelling to keep previous values, with the UI reflecting the result.

#### Acceptance Criteria
1. WHEN OK is activated THEN the application updates global BPM and bars AND the pads UI reflects the new values immediately.
2. WHEN Cancel or Esc is activated THEN no state changes occur AND the UI remains as before entering configuration.

## Non-Functional Requirements

### Code Architecture and Modularity
- Single Responsibility: input handling, UI rendering (summary box, popup), and state management are clearly separated.
- Modular Design: popup and input widgets encapsulated; summary box rendering isolated.
- Clear Interfaces: app state exposes bpm and bars; UI consumes state via read‑only getters and mutation APIs.

### Performance
- Popup open/close, focus changes, and input edits must render within one frame (no visible lag) on macOS 14 on typical laptops.

### Reliability
- Graceful handling of non‑numeric input: ignore or constrain to numeric; clamp bpm and bars to sane ranges.

### Usability
- Consistent accent language: white = focus, dark green = active, black = input surface, green text for labels.
- Right‑aligned numeric values for readability.

## References
- Notion story: [MVP-2: Enable user to setup BPM and bars](https://www.notion.so/2754150965e48027807cf50a77a9fa84)
- UI behavior inspiration: HTML textarea semantics (editing model), adapted for numeric fields
- Planned widgets: `tui-popup` and `tui-input` per Notion Technical info
