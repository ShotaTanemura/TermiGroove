# Tasks – mvp-2-create-loop-layered-tracks

- [x] 1. Extend loop engine behavior matrix for layered overdubs
  - File: docs/tdd/loop_engine/behavior-matrix.md
  - Document happy paths, pause/resume, clear, backpressure, and invalid state inputs for layered tracks
  - Capture open questions about track naming, memory limits, and audio-channel backpressure handling
  - _Leverage: docs/tdd/loop_engine/behavior-matrix.md, requirements.md §Requirements, design.md §Components and Interfaces_
  - _Requirements: Requirement 1, Requirement 2, Requirement 3, Requirement 4_
  - _Prompt: Role: Loop Engine TDD Analyst | Task: Enumerate layered overdub scenarios (happy, pause/resume, clear, error) in the behavior matrix with Gherkin-style entries | Restrictions: No code edits; remain implementation-agnostic | _Leverage: docs/tdd/loop_engine/behavior-matrix.md_ | _Requirements: Requirement 1-4_
  - _Success: Matrix covers all state transitions, overdub edge cases, and ties to requirements so testers agree on Done criteria_

- [x] 2. Red #1 — unit: overdub happy path scheduling
  - File: tests/loop_engine/loop_overdub_layers.rs
  - Add unit test verifying first pad press during playback starts overdub, captures precise offset, and schedules playback next cycle without metronome
  - _Leverage: tests/loop_engine/loop_happy_path.rs, requirements.md Requirement 1-3_
  - _Requirements: Requirement 1, Requirement 2, Requirement 3_
  - _Prompt: Role: Loop Engine TDD Engineer | Task: Implement a failing unit test for layered overdub scheduling using fake clock | Restrictions: Assert on events and state only; reuse existing helpers | _Leverage: tests/loop_engine/loop_happy_path.rs_ | _Requirements: Requirement 1-3_
  - _Success: Test fails because overdub layering not implemented; explains expectations in comments_

- [x] 3. Green #1 — loop engine minimal overdub support
  - File: src/state/loop_engine.rs
  - Implement minimal logic to pass loop_overdub_layers.rs: start overdub on first pad press, store events, playback once per cycle, no metronome
  - Update LoopSnapshot with track count metadata per design
  - _Leverage: requirements.md Requirement 1-3, design.md LoopEngine section_
  - _Requirements: Requirement 1, Requirement 2, Requirement 3_
  - _Prompt: Role: Loop Engine Implementer | Task: Modify loop_engine to satisfy overdub happy path with minimal changes | Restrictions: Keep changes localized; maintain existing state machine semantics | _Leverage: tests/loop_engine/loop_overdub_layers.rs_
  - _Success: New test passes without breaking existing loop tests; snapshot includes track count_

- [x] 4. Red #2 — unit: pause/resume layered loop
  - File: tests/loop_engine/loop_pause_resume.rs
  - Add unit test covering Space pause/resume behavior with layered tracks, ensuring playback stops and restarts on cycle boundary
  - _Leverage: requirements.md Requirement 4, design.md LoopEngine/AppState_
  - _Requirements: Requirement 4_
  - _Prompt: Role: Loop Engine TDD Engineer | Task: Write failing unit test verifying pause/resume semantics without affecting stored tracks | Restrictions: Use existing fake clock; assert state transitions | _Leverage: tests/loop_engine/loop_cancel.rs_
  - _Success: Test fails due to missing pause/resume wiring_

- [x] 5. Green #2 — implement pause/resume
  - File: src/state/loop_engine.rs, src/app_state.rs
  - Add pause/resume handling with snapshot flags and ensure scheduling deactivates during pause
  - _Leverage: design.md LoopEngine/AppState, tests/loop_engine/loop_pause_resume.rs_
  - _Requirements: Requirement 4_
  - _Prompt: Role: Loop Engine Implementer | Task: Add pause/resume logic in loop engine and propagate snapshot state | Restrictions: Keep changes minimal; avoid UI work | _Leverage: tests/loop_engine/loop_pause_resume.rs_
  - _Success: Pause/resume test passes; prior tests stay green_

- [x] 6. Red #3 — unit: clear loop via Control+Space
  - File: tests/loop_engine/loop_clear.rs
  - Add unit test ensuring Control+Space clears all tracks and resets state to Idle
  - _Leverage: requirements.md Requirement 4_
  - _Requirements: Requirement 4_
  - _Prompt: Role: Loop Engine TDD Engineer | Task: Create failing test for clear-loop command that asserts track list is emptied | Restrictions: Use deterministic clock; verify audio commands are flushed_
  - _Success: Test fails because clear not implemented_

- [x] 7. Green #3 — implement clear loop
  - File: src/state/loop_engine.rs, src/app_state.rs
  - Implement Control+Space handling, clearing tracks, resetting state, and releasing audio commands if needed
  - _Leverage: design.md LoopEngine/AppState, tests/loop_engine/loop_clear.rs_
  - _Requirements: Requirement 4_
  - _Prompt: Role: Loop Engine Implementer | Task: Add clear loop implementation ensuring state reset and command queue flush | Restrictions: No UI code; update snapshots accordingly_
  - _Success: Clear test passes; ensures Idle state with zero tracks_

- [x] 8. Red #4 — integration: AppState routing
  - File: tests/app_state_loop.rs
  - Add integration test verifying pad press triggers overdub start, Space toggles pause/resume, and Control+Space clears tracks via AppState
  - _Leverage: design.md AppState loop facade, requirements.md Requirement 1-4_
  - _Requirements: Requirement 1, Requirement 4_
  - _Prompt: Role: AppState Integration Tester | Task: Write failing test to ensure AppState issues appropriate LoopCommands for layered features | Restrictions: Use existing AppState harness; assert snapshots and command sequences_
  - _Success: Test fails until AppState wiring updated_

- [x] 9. Green #4 — AppState wiring for layered controls
  - File: src/app_state.rs, src/input.rs, src/state/mod.rs
  - Wire AppState and input handling to new loop commands for overdub, pause/resume, clear; expose snapshots
  - _Leverage: design.md AppState loop facade, tests/app_state_loop.rs_
  - _Requirements: Requirement 1-4_
  - _Prompt: Role: AppState Implementer | Task: Update command routing and loop snapshot exposure for layered controls | Restrictions: No UI rendering changes; maintain ergonomics and test determinism_
  - _Success: AppState integration test passes along with previous unit tests_

- [x] 10. Red #5 — e2e layering flow
- [x] 11. Green #5 — end-to-end wiring & cleanup
  - File: tests/e2e/loop_overdub_layers.test.ts
  - Add e2e scenario: capture base loop, overdub layering via pad presses, pause/resume, clear, ensuring CLI feedback stays consistent
  - _Leverage: tests/e2e/loop_capture.test.ts, requirements.md Requirement 1-4_
  - _Requirements: Requirement 1-4_
  - _Prompt: Role: E2E Automation Engineer | Task: Create failing e2e test covering layered overdub workflow using tui-test harness | Restrictions: Deterministic key sequences; no UI redesign_
  - _Success: New e2e test fails because layering is not yet wired end-to-end_

- [ ] 11. Green #5 — end-to-end wiring & cleanup
  - File: src/app_state.rs, src/input.rs, src/audio.rs, docs/tdd/loop_engine/behavior-matrix.md (updates)
  - Ensure e2e layering test passes; adjust audio scheduling or tracing as needed; update behavior matrix with any discovered cases
  - _Leverage: tests/e2e/loop_overdub_layers.test.ts, design.md Architecture_
  - _Requirements: Requirement 1-4_
  - _Prompt: Role: Full-stack Loop Implementer | Task: Finish wiring so e2e test passes; update documentation | Restrictions: Keep audio thread deterministic; no new UI widgets_
  - _Success: All unit/integration/e2e tests green; behavior matrix reflects final state_

