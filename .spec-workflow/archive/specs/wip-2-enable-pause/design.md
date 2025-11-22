# Design Document

## Architecture Overview

The pause/resume feature augments three core layers: the loop engine, the audio command channel, and the UI state observers. The keyboard input handler already delegates to `LoopEngine::handle_space`; this function will orchestrate Pause/Resume. We introduce a `PauseAll` audio command that the audio thread consumes to halt sinks atomically. `LoopEngine` records the precise offsets needed to resume playback/recording without drift, while `AudioBus` propagates the pause command to the audio thread. UI remains a passive observer, reacting to state changes in `AppState`.

## Module-Level Changes

### `src/audio.rs`
- Extend `AudioCommand` enum with `PauseAll` variant carrying no payload.
- Update the audio processing loop to handle `PauseAll`: iterate sinks atomically, pausing each stream and pruning finished ones.
- Ensure pause handling is thread-safe; commands should be processed in the same channel as other audio commands to preserve ordering.

### `src/audio/bus.rs` (or equivalent)
- Add `pause_all()` helper that enqueues the `PauseAll` command.
- Ensure existing senders derive from `SenderAudioBus` (or similar) to minimize duplicated code.
- Update mocks/test harnesses to recognize the new command.

### `src/loop_engine.rs`
- Enhance `handle_space` to branch by current `LoopState`.
    - When `Playing`, store playback cycle offset, emit `PauseAll`, and transition to `LoopState::Paused { saved_offset, loop_length, was_recording: false }`.
    - When `Recording`, store both playback offset and overdub start offset, stop scheduling additional overdub events, emit `PauseAll`, and transition to `Paused` capturing recording metadata.
    - When `Paused`, compute `cycle_start` from saved offset, restore overdub timing if `was_recording`, send resume commands (likely `StartAll`/existing playback scheduling), and transition back to `Playing`/`Recording`.
- Introduce helper structs to capture pause snapshot (e.g., `PauseSnapshot { playback_offset_ms, overdub_offset_ms, was_recording }`).
- Maintain idempotency: repeated pause presses in `Paused` should be ignored.

### `src/input.rs`
- Confirm `handle_space` path integrates with new pause logic (may only require ensuring event dispatch still occurs).
- Add guard so non-pause flows continue unaffected.

### `src/ui.rs` / App state observers
- Reflect paused state visually (e.g., highlight or status text) leveraging existing state watchers.

## Data Structures & State Management

- `LoopState::Paused` variant should carry stored offsets and whether recording was active, enabling resume logic.
- App state may require new fields for pause metadata (e.g., `pause_snapshot: Option<PauseSnapshot>`).
- Ensure state transitions maintain invariants: `Paused` only entered from `Playing` or `Recording`.

## Thread & Concurrency Considerations

- `PauseAll` command must execute on the audio thread to avoid cross-thread race conditions.
- Pause/resume should reuse existing channels; ensure no blocking operations inside audio loop.
- If `PauseAll` arrives while sinks already stopped, the handler should no-op safely.

## Testing Strategy

### Unit Tests
- `audio` module: verify `PauseAll` pauses active sinks and leaves queue clean.
- `loop_engine` tests: simulate playing/recording transitions, ensure offsets captured and restored.
- `input` tests: confirm non-pause interactions do not emit `PauseAll`.

### Integration Tests
- Scenario: start playback, pause via space, assert engine state `Paused` and offsets stored.
- Resume scenario: after pause, pressing space resumes playback aligned within ≤1 ms.
- Regression scenario: simulate other keyboard flows ensuring no pause triggered.

### Manual/End-to-End Tests
- With sample set, run actual terminal session, trigger pause/resume, confirm audio artifacts absent.

## Rollout & Monitoring

- Add logging around pause/resume state transitions (debug-level) for troubleshooting.
- Document new behavior in user docs once feature ships.
- Monitor test suite for new coverage ensuring pause/resume cannot regress silently.

## Decision Log

- 2025-10-11: Finalized pause/resume implementation. `LoopEngine::handle_space` now persists `saved_offset`/`was_recording`, emits `PauseAll` on pause, and realigns tracks on resume. Added `AudioCommand::PauseAll` and `ResumeAll` so the audio thread pauses and restarts sinks deterministically; UI summary banner highlights the paused state.
