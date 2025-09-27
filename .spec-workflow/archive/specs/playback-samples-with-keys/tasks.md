# Tasks — playback-samples-with-keys (TDD)

- [x] 1. Pads state and mode wiring
  - Files: `src/app_state.rs`, `src/selection.rs`
  - Purpose: Add `Mode::Pads` and `PadsState { key_to_slot, active_keys, last_press_ms }`. Build mapping from selected files to pad keys on transition; validate `.wav` only and show error otherwise.
  - _Leverage: src/app_state.rs, src/selection.rs_
  - _Requirements: R1, R2_
  - _Prompt: Role: Rust TUI state engineer | Task: Add Mode::Pads and PadsState, populate mapping on transition, enforce .wav validation; no audio yet | Restrictions: Keep functions small, follow existing module/import style | _Leverage: src/app_state.rs, src/selection.rs | _Requirements: R1,R2 | Success: transition to Pads populates state or shows error without crash._

- [x] 2. Audio thread skeleton with commands
  - Files: `src/lib.rs` (or new `src/audio.rs`), `src/app_state.rs`
  - Purpose: Define `AudioCommand { Preload{key,path}, Play{key}, StopAll }`; spawn audio thread and expose `mpsc::Sender<AudioCommand>` in `AppState`.
  - _Leverage: std::sync::mpsc, design.md_
  - _Requirements: R2_
  - _Prompt: Role: Systems engineer (Rust concurrency) | Task: Create AudioCommand enum and background audio thread with mpsc; wire sender into AppState; log commands (no rodio yet) | Restrictions: No blocking on UI thread; graceful thread shutdown not required yet | _Leverage: std::sync::mpsc | _Requirements: R2 | Success: thread runs; commands enqueue without panic._

- [x] 3. Rodio engine with cache-by-key
  - Files: `src/audio.rs`
  - Purpose: Integrate `rodio`; on `Preload`, decode `.wav` once and cache by key; on `Play`, create `Sink` from cached buffer; support polyphony and re-trigger.
  - _Leverage: rodio crate, design.md_
  - _Requirements: R2_
  - _Prompt: Role: Audio engineer (Rust/rodio) | Task: Use rodio to decode wav once per key and play from cache; multiple sinks mix; same-key re-trigger overlaps | Restrictions: Keep API surface minimal; handle decode errors without crash | _Leverage: rodio | _Requirements: R2 | Success: unit tests assert preload→cache and play spawns sinks._

- [x] 4. UI: Pads rendering and highlight
  - Files: `src/ui.rs`
  - Purpose: Render pads grid with key and file name; highlight dark green while active.
  - _Leverage: src/ui.rs_
  - _Requirements: R1, R2_
  - _Prompt: Role: Rust TUI UI engineer | Task: Draw pads with keys and filenames; highlight on active press | Restrictions: Keep layout stable on resize; follow existing style | _Leverage: src/ui.rs | _Requirements: R1,R2 | Success: visible grid with labels; highlight toggles on key press._

- [x] 5. Input handling and debounce
  - Files: `src/input.rs`, `src/app_state.rs`
  - Purpose: On key press: suppress auto-repeat within 100ms using `last_press_ms`; set active, send `Play(key)`; `Esc` returns to navigation.
  - _Leverage: src/input.rs, design.md_
  - _Requirements: R2_
  - _Prompt: Role: Input handling engineer (Rust TUI) | Task: Debounce auto-repeat (100ms), update active state, send Play(key) over channel; handle Esc to go back | Restrictions: Avoid blocking calls; keep branching shallow | _Leverage: src/input.rs | _Requirements: R2 | Success: single fire per physical press; Esc switches modes._

- [x] 6. E2E: Pads happy flow
  - Files: `tests/e2e/pads-flow.test.ts`
  - Purpose: Test transition to Pads, visible pads, key press highlight, Esc back.
  - _Leverage: @microsoft/tui-test, `tui-test.config.ts`_
  - _Requirements: R1, R2_
  - _Prompt: Role: QA (TUI E2E) | Task: Write TUI tests covering pads flow and visuals | Restrictions: Use stable selectors/text; add retries if needed | _Leverage: @microsoft/tui-test | _Requirements: R1,R2 | Success: tests pass reliably under retries._

- [ ] 7. E2E: Polyphony and re-trigger
  - Files: `tests/e2e/pads-polyphony.test.ts`
  - Purpose: Assert overlapping key presses highlight multiple pads; repeated same key re-triggers highlight appropriately.
  - _Leverage: @microsoft/tui-test_
  - _Requirements: R2_
  - _Prompt: Role: QA (TUI E2E) | Task: Add tests for multiple keys and same-key re-trigger | Restrictions: Don’t rely on timing too tightly; assert UI state | _Leverage: @microsoft/tui-test | _Requirements: R2 | Success: tests pass and are stable._
