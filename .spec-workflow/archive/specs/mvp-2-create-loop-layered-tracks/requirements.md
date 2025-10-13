# Requirements – mvp-2-create-loop-layered-tracks

## Introduction

Enable performers on the Pads screen to overdub fresh material onto an already playing loop so that each pass through the loop accumulates layered tracks without stopping the performance. The feature extends the existing loop capture workflow with track-level separation, transport controls for pause/clear, real-time monitoring, and immediate drop-in overdubs that do not replay the metronome.

## Alignment with Product Vision

- Advances **Keyboard Mastery** by keeping overdub, pause, resume, and clear actions bound to Space and Control+Space with no mouse interaction.
- Reinforces **Retro-Futuristic Clarity** by tracking each overdub as an explicit layer that the UI can surface in transport and future mix widgets.
- Upholds **Reliability Through TDD** and the Tempo & Timing Control pillar by requiring deterministic count-in, sample scheduling, and latency budgets under 10 ms during overdub sessions.
- Supports the business goal of collaborative live looping by letting a performer (or duo) build up arrangements in real time from the terminal.

## Requirements

### Requirement 1 – Immediate overdub entry

**User Story:** As a Pads performer listening to an existing loop, I want to drop into an overdub just by playing a pad so layering stays in time without an extra metronome count-in.

#### Acceptance Criteria

1. WHEN the loop is playing with valid BPM and measures AND the performer presses any mapped pad key while no overdub is active, THEN the loop engine SHALL immediately enter `Recording`, capture the event at its precise offset within the ongoing cycle (relative to the existing loop start), and SHALL NOT emit metronome ticks.
2. IF the performer does not trigger any pad during the armed window (i.e., stays hands-off), THEN the engine SHALL remain in `Playing` with the existing tracks; no metronome or state churn occurs.
3. WHEN an overdub is initiated by the first pad press, THEN the loop engine SHALL preserve the current loop timeline (no restart), ensuring the captured event remains at its actual in-cycle offset and subsequent playback of all tracks stays phase-aligned.

### Requirement 2 – Real-time overdub capture

**User Story:** As a performer layering parts, I want every pad I trigger during an overdub to monitor in my headphones and store precise offsets so future cycles replay what I just played.

#### Acceptance Criteria

1. WHEN `Recording` is active AND the performer presses any mapped pad key, THEN the system SHALL play the sample immediately for monitoring AND store an event containing `(pad_key, velocity if available, offset_ms_from_overdub_start)`.
2. WHEN additional overdub events are captured while prior tracks already exist, THEN the engine SHALL buffer the new events in a distinct track structure without mutating existing track data.
3. WHEN multiple events share the same millisecond offset, THEN the engine SHALL preserve their arrival order in the track data so playback matches the performer’s timing.

### Requirement 3 – Track creation and layered playback

**User Story:** As a performer, I want each overdub take to become its own named track so that the loop automatically plays every layer once per cycle with tight timing.

#### Acceptance Criteria

1. WHEN an overdub cycle ends because the loop length elapses, THEN the engine SHALL seal the captured events into a new track, append it to the loop’s track list, and expose the track metadata (name/id, event count, duration) to the UI layer.
2. WHEN playback advances through subsequent cycles, THEN every track in the track list SHALL emit its stored events exactly once per cycle at their recorded offsets with jitter ≤2 ms.
3. WHEN no events exist in a given track during a cycle, THEN the engine SHALL avoid emitting redundant play commands to preserve audio headroom.

### Requirement 4 – Transport controls during layered looping

**User Story:** As a performer juggling layers, I want to pause, resume, or clear the loop using Space so I can manage the arrangement mid-performance without touching the file navigator.

#### Acceptance Criteria

1. WHEN `Recording` is active AND Space is pressed, THEN the engine SHALL stop recording immediately, preserve the captured events up to that moment as a track, and continue playback in `Playing` state.
2. WHEN the loop is playing AND Space is pressed, THEN the engine SHALL pause playback, maintain the current track list, and expose `Paused` state to the UI.
3. WHEN the loop is paused AND Space is pressed, THEN the engine SHALL resume playback from the next cycle boundary using the existing tracks.
4. WHEN the loop is playing (or paused) AND Control+Space is pressed, THEN the engine SHALL clear all tracks, stop playback, and reset to `Idle` with an empty track list.

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle:** Track layering logic lives in the loop engine module; UI and input layers interact via explicit commands/events.
- **Modular Design:** Track storage structures remain isolated so future features (mute, delete) can manipulate individual tracks without touching core scheduling.
- **Dependency Management:** Overdub logic reuses the existing clock, audio bus, and AppState messaging without introducing cross-module cyclic dependencies.
- **Clear Interfaces:** Extend the loop engine API with explicit methods for `start_overdub`, `commit_overdub`, `pause_loop`, and `clear_loop`, each returning deterministic state snapshots for testing.

### Performance
- Maintain ≤10 ms additional latency between pad press and monitored audio during overdubs.
- Ensure layered playback scheduling exhibits ≤2 ms jitter per event over at least 30 consecutive cycles.
- Guarantee overdub playback commands do not starve the audio callback thread, and no metronome ticks are enqueued during layered recording.

### Security
- Restrict overdub operations to already loaded samples; never load new file paths during recording.
- Validate Control+Space input to avoid accidental clears triggered by unrelated modifiers (e.g., require explicit key chord recognition).

### Reliability
- Provide deterministic state transitions (`Idle ↔ Ready ↔ Recording ↔ Playing ↔ Paused`) with comprehensive unit and integration coverage.
- Clearing a loop MUST release buffered audio commands and deallocate track memory to prevent stale playback on subsequent sessions.
- Loop mode MUST avoid XRuns under the defined layering workflow on macOS 14 reference hardware.

### Usability
- Surface transport state (`Ready`, `Recording`, `Playing`, `Paused`) within one render frame so performers receive immediate feedback.
- Present per-track counts (e.g., total layered tracks) in the transport strip or status area for quick glanceability.
- Provide audio-only confirmation (metronome ticks, sample monitoring) to maintain keyboard-only operation.

## Dependencies & Interfaces
- Builds on the loop capture foundation from `mvp-2-record-loop-track` and BPM/bars configuration from `mvp-2-setup-bpm-and-loop-bars`.
- Extends the loop engine housed under `src/state/loop_engine.rs`, which already exposes clocked scheduling and state snapshots.
- Requires input routing in `src/input.rs` to recognize Control+Space and to signal overdub start/stop events.
- UI updates occur through `src/ui.rs` transport components and should display aggregated track state without blocking the render loop.

## Decision Log
- 2025-09-28 — Adopt a track list structure (`Vec<LoopTrack>`) within the loop engine to preserve each overdub separately, enabling future mute/solo features.
- 2025-09-28 — Treat Control+Space as a destructive clear that resets the loop state entirely, paired with UI confirmation cues, to keep live performance semantics explicit.
- 2025-09-28 — Align overdub entry with reviewer feedback: the first pad press drops straight into recording with no metronome count-in or timeline reset, ensuring seamless layering over existing loops.

## References
- Notion user story: https://www.notion.so/27c4150965e48001ac5cf80d68e63dbf
- Parent epic: https://www.notion.so/2704150965e480b0ab9ce85f5b5c7254
- Steering documents: `.spec-workflow/steering/product.md`, `.spec-workflow/steering/tech.md`, `.spec-workflow/steering/structure.md`

