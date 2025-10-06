# Tasks – mvp-2-record-loop-track

- [x] 1. Behavior matrix for loop lifecycle and timing invariants
  - File: docs/tdd/loop_engine/behavior-matrix.md
  - Capture exhaustive scenarios for Idle → Ready → Recording → Playing, metronome timing, cancellation, and BPM/bars changes. Include invariants for latency and command dispatch ordering. Identify non-goals (UI timelines, overdubbing) and open questions (e.g., audio thread failure handling).
  - Purpose: Define TDD acceptance surface and guard against regressions when LoopEngine evolves.
  - _Leverage: requirements.md, design.md, docs/tdd/_shared/invariants.md (create if missing)_
  - _Requirements: 1, 2, 3, 4, 5, 6_
  - _Prompt: Role: TDD Analyst for real-time audio systems | Task: Enumerate positive, negative, and edge cases for loop lifecycle, metronome count-in, recording cancellation, and playback scheduling; state invariants for latency and command ordering; document non-goals and open questions | Restrictions: Do not specify implementation details or data structures; keep cases testable and traceable to requirements | Success: Matrix covers Ready/Recording/Playing transitions, metronome behavior, cancellation semantics, and BPM/bars change handling; stakeholders sign off as the definitive test list_

- [x] 2. Red #1 — Happy path loop capture
  - File: tests/loop_engine/loop_happy_path.rs
  - Write a unit test driving LoopEngine through Ready → Recording → Playing with deterministic FakeClock and AudioBusMock; assert metronome commands, event capture offsets, and playback scheduling for a single cycle.
  - Purpose: Establish first failing test for minimal end-to-end lifecycle.
  - _Leverage: docs/tdd/loop_engine/behavior-matrix.md_
  - _Requirements: 1, 2, 3, 4_
  - _Prompt: Role: Rust TDD Engineer | Task: Implement a failing test for the loop happy path using FakeClock/AudioBusMock, asserting metronome command count, captured events, and playback scheduling | Restrictions: Do not implement production code; no reliance on real time or audio thread | Success: Test fails because LoopEngine is not implemented; failure message highlights missing lifecycle transitions_

- [x] 3. Green #1 — Minimal LoopEngine to satisfy happy path
  - File: src/state/loop_engine.rs (new)
  - Implement LoopEngine struct, state enum, and minimal logic to pass happy path test: handle space presses, count-in ticks, event capture, automatic playback scheduling for one cycle (no cancellation yet). Introduce Clock and AudioBus traits.
  - Purpose: Deliver minimal functionality for the primary loop recording scenario.
  - _Leverage: tests/loop_engine/loop_happy_path.rs, requirements.md, design.md_
  - _Requirements: 1, 2, 3, 4_
  - _Prompt: Role: Implementation-focused Rust developer | Task: Implement LoopEngine basics (state transitions, count-in, event capture, playback scheduling) to satisfy the happy path test; define Clock/AudioBus abstractions | Restrictions: Keep implementation minimal; no persistence; no manual cancellation or BPM change handling yet | Success: Happy path test passes; code remains under ~300 lines with clear separation of concerns_

- [x] 4. Red #2 — Cancel during Ready or Recording
  - File: tests/loop_engine/loop_cancel.rs
  - Add tests covering space press during Ready (aborts count-in) and during Recording (clears events, returns to Idle). Assert AudioBus receives cancellation logs only via state change (no extra commands). Ensure FakeClock determinism.
  - Purpose: Drive cancellation semantics per requirements.
  - _Leverage: docs/tdd/loop_engine/behavior-matrix.md_
  - _Requirements: 1, 2, 5_
  - _Prompt: Role: Rust TDD Engineer | Task: Implement failing tests for cancellation during Ready/Recording, asserting state transitions and cleared events | Restrictions: Isolate tests; no production code changes | Success: Tests fail due to missing cancellation handling_

- [x] 5. Green #2 — Implement cancellation handling
  - File: src/state/loop_engine.rs
  - Extend LoopEngine to handle cancellation paths (Ready → Idle, Recording → Idle with event clear) and update state getters. Ensure metronome generation stops on cancel.
  - Purpose: Fulfill manual stop behavior.
  - _Leverage: tests/loop_engine/loop_cancel.rs, design.md_
  - _Requirements: 1, 2, 5_
  - _Prompt: Role: Implementation-focused Rust developer | Task: Update LoopEngine to satisfy cancellation tests, clearing events and halting metronome on cancel | Restrictions: Avoid refactoring existing passing paths; maintain minimal changes | Success: Cancel tests pass alongside previous happy path test_

- [x] 6. Red #3 — BPM/bars change reset behavior
  - File: tests/loop_engine/loop_bpm_reset.rs
  - Add test verifying that invoking `loop_engine.reset_for_new_tempo(bpm, bars)` (to be called by AppState) clears events and returns state to Idle. Include guard that playback does not emit stale commands post-reset.
  - Purpose: Enforce loop invalidation when tempo context changes.
  - _Leverage: docs/tdd/loop_engine/behavior-matrix.md_
  - _Requirements: 5_
  - _Prompt: Role: Rust TDD Engineer | Task: Add failing test for BPM/bars change forcing loop reset | Restrictions: Use FakeClock; no production updates | Success: Test fails indicating missing reset behavior_

- [x] 7. Green #3 — Implement tempo reset hook
  - File: src/state/loop_engine.rs
  - Add public method to reset loop data, clear events, and stop playback scheduling when tempo changes. Integrate with LoopEngine update cycle to ensure no lingering commands.
  - Purpose: Support tempo change requirement.
  - _Leverage: tests/loop_engine/loop_bpm_reset.rs_
  - _Requirements: 5_
  - _Prompt: Role: Implementation-focused Rust developer | Task: Implement tempo reset logic, ensuring no playback occurs after reset | Restrictions: Keep method side-effect scoped; maintain existing functionality | Success: Tempo reset test passes_

- [x] 8. Integrate LoopEngine into AppState
  - File: src/app_state.rs, src/input.rs, src/audio.rs
  - Wire LoopEngine into AppState: instantiate with real Clock/AudioBus, route Space key handling, forward pad presses to `LoopEngine::record_event`, call update per tick. Extend AudioCommand enum with metronome and scheduled playback variants produced by LoopEngine.
  - Purpose: Connect engine to application runtime.
  - _Leverage: design.md, existing AppState patterns_
  - _Requirements: 1, 2, 3, 4, 5, 6_
  - _Prompt: Role: Application integrator | Task: Integrate LoopEngine into existing AppState/input/audio modules, ensuring command dispatch aligns with design | Restrictions: Preserve existing pad behavior; ensure new commands don’t block audio thread | Success: Unit/integration tests compile; manual smoke (if run) shows state transitions via status text_

- [x] 9. Audio thread enhancements for scheduling & metronome
  - File: src/audio.rs
  - Handle new AudioCommand variants: synthesized metronome beep, scheduled pad playback using offsets and rodio sinks. Ensure non-blocking behavior and cleanup of finished sinks.
  - Purpose: Provide audio-side support for loop engine.
  - _Leverage: design.md, existing rodio setup_
  - _Requirements: 2, 3, 4, 6_
  - _Prompt: Role: Audio systems developer | Task: Extend audio thread to support metronome beep generation and scheduled playback triggered by LoopEngine commands | Restrictions: Avoid heap churn inside callback; maintain existing immediate play path | Success: Unit tests with mocked channel verify commands; manual smoke shows metronome tick and loop playback_

- [x] 10. Integration tests for AppState + LoopEngine
  - File: tests/app_state_loop.rs
  - Write integration tests covering full workflow: selection → enter pads → space ready/record/play with FakeClock and audio channel spy. Assert AppState status messages and command dispatch order.
  - Purpose: Validate wiring across modules.
  - _Leverage: requirements.md, design.md_
  - _Requirements: 1, 2, 3, 4, 5, 6_
  - _Prompt: Role: Rust integration tester | Task: Implement integration tests exercising AppState with LoopEngine and mocked audio sender | Restrictions: Use deterministic clock and stub metronome audio | Success: Tests confirm transitions, command dispatch order, and event capture_

- [x] 11. E2E script for loop capture via tui-test
  - File: tests/e2e/loop_capture.test.ts
  - Add @microsoft/tui-test scenario: assign sample to pad, enter pads, trigger recording, play pad, wait for auto playback. Verify console output/state via snapshots.
  - Purpose: End-to-end validation of user story.
  - _Leverage: requirements.md, design.md, existing tui-test config_
  - _Requirements: 1, 2, 3, 4, 5, 6_
  - _Prompt: Role: QA Automation Engineer (tui-test) | Task: Implement e2e test driving loop capture happy path with deterministic timing configuration | Restrictions: Use available fixtures; ensure test stable across runs | Success: Test passes after implementation; fails before audio enhancements are complete_

- [x] 12. Documentation & cleanup
  - File: docs/tdd/loop_engine/behavior-matrix.md, README updates if needed
  - Update documentation with final behavior matrix adjustments, usage instructions for loop recording, and known limitations. Ensure code comments reflect final architecture.
  - Purpose: Wrap up spec deliverables.
  - _Leverage: completed implementation and tests_
  - _Requirements: 6_
  - _Prompt: Role: Documentation steward | Task: Update docs and inline comments to reflect final loop recording behavior and testing strategy | Restrictions: Keep docs concise and aligned with spec | Success: Docs match shipped behavior; no outstanding TODOs in code_
