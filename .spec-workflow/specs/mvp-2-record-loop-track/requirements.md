# Requirements – mvp-2-record-loop-track

## Introduction
Enable performers on the Pads screen to capture a one-loop performance from live pad triggers, driven by the configured BPM and bar count, so that the captured pattern replays automatically while they continue playing. The feature must feel natural during live jams: the performer presses Space, hears a four-beat count-in, records one loop length, and the captured pattern repeats seamlessly without extra effort.

## Alignment with Product Vision
- Reinforces **Keyboard Mastery** by mapping the entire record/stop workflow to Space and metronome-only feedback.
- Supports **Retro-Futuristic Clarity** with simple, glanceable recording states across the loop timeline and transport strip.
- Honors **Reliability Through TDD** by defining observable state transitions (Idle → Ready → Recording → Playing) that can be asserted in tests before shipping.
- Advances the **Tempo & Timing Control** objective in `.spec-workflow/steering/product.md` by pairing configurable BPM/bars with loop capture.

## Requirements

### Requirement 1 – Recording lifecycle and state machine
**User Story:** As a Pads performer, I want a deterministic Idle → Ready → Recording → Playing lifecycle triggered from Space so that I can capture loops without manual bookkeeping.

#### Acceptance Criteria
1. GIVEN Pads mode AND no recording in progress, WHEN Space is pressed, THEN the loop engine enters `Ready` and emits a `Ready` audio status (before any recording begins).
2. GIVEN state `Ready`, WHEN the fourth metronome tick finishes, THEN the loop engine automatically transitions to `Recording` and marks the loop start (`t=0`).
3. GIVEN state `Recording`, WHEN the loop duration elapses, THEN the loop engine transitions to `Playing` and seals the captured events for loop playback.
4. GIVEN state `Recording`, WHEN Space is pressed, THEN recording is canceled immediately, captured events for the in-flight take are cleared, and the engine returns to `Idle`.
5. GIVEN state `Playing`, WHEN Space is pressed, THEN the lifecycle restarts at `Ready` (four-beat count-in) using the current BPM/bars configuration.

### Requirement 2 – Four-beat metronome count-in
**User Story:** As a performer, I want exactly four metronome ticks before recording starts so that I have time to prepare and stay on tempo.

#### Acceptance Criteria
1. GIVEN Pads mode AND Space is pressed, THEN exactly four metronome ticks play at the configured BPM before recording starts; tempo changes immediately affect tick spacing.
2. GIVEN the count-in is active, THEN pressing Space cancels the pending recording, halts the metronome ticks, and returns the engine to `Idle` with no events captured.

### Requirement 3 – Event capture with precise offsets
**User Story:** As a performer, I want every pad trigger during recording to be stored with precise offsets so playback matches my timing.

#### Acceptance Criteria
1. GIVEN state `Recording`, WHEN any mapped pad key is pressed, THEN its sample is played immediately (monitoring) AND the engine stores `(pad_id, offset_ms)` relative to the loop start.
2. GIVEN multiple events arrive within the same millisecond bucket, THEN they are preserved in arrival order without loss.
3. GIVEN recording stops (auto or manual), THEN the engine seals the loop pattern so subsequent cycles reuse the captured offsets without mutation.

### Requirement 4 – Loop playback and continuity
**User Story:** As a performer, I want the captured loop to replay automatically each cycle so I can layer additional parts live.

#### Acceptance Criteria
1. GIVEN captured events exist, WHEN the playhead reaches a stored offset in `Playing`, THEN the original sample fires exactly once per cycle at that offset.
2. GIVEN the loop repeats, THEN playback timing remains locked to BPM/bars with no drift across at least 30 consecutive cycles.
3. GIVEN the performer triggers new pads while the loop is playing but not recording, THEN those pads play in real time without affecting the stored loop pattern.

### Requirement 5 – Manual stop, auto-stop, and readiness to re-record
**User Story:** As a performer, I want to end a recording manually or let it auto-complete after one loop so that I control the captured content without extra UI steps.

#### Acceptance Criteria
1. GIVEN state `Recording`, WHEN elapsed time reaches `bars * 4 * 60 / BPM` seconds, THEN recording auto-stops, transitions to `Playing`, and no additional events are captured beyond the loop length.
2. GIVEN state `Recording`, WHEN Space is pressed, THEN recording cancels immediately, transitions to `Idle`, and clears any events captured during that take.
3. GIVEN state `Playing`, WHEN BPM or bars values change, THEN the current loop playback finishes its cycle, discards the outdated pattern, and a new recording must be captured (documented in Decision Log).

### Requirement 6 – Transport & UI feedback
**User Story:** As a performer, I want unambiguous UI feedback for Ready/Recording/Playing so that I can perform without guesswork.

#### Acceptance Criteria
1. GIVEN any lifecycle state, THEN the transport strip shows the active state (`Ready`, `Recording`, `Playing`) with color and text cues consistent with existing steering guidelines.
2. GIVEN state transitions occur, THEN the Pads screen updates within one frame (≤50 ms) to reflect the new state.
3. GIVEN the metronome count-in or playback is active, THEN there is no overlapping or stale UI indicator lingering after the state changes.

## Non-Functional Requirements
- **Performance:** Recording and playback must introduce ≤10 ms additional latency beyond current pad triggering, and metronome ticks must schedule with jitter <2 ms relative to BPM spacing.
- **Reliability:** Avoid XRuns by pre-allocating buffers for loop events; recording should never drop events even under rapid pad input; loop data persists across app restarts unless explicitly cleared.
- **Usability:** Provide clear keyboard-only affordances; never require mouse interaction. Count-in feedback is audio-only in this scope.
- **Testability:** Expose deterministic hooks (e.g., mockable clock, state snapshots) so automated tests can assert lifecycle transitions without real audio hardware.
- **Telemetry/Logging (Optional):** Log lifecycle transitions at debug level for troubleshooting without flooding release builds.

## Dependencies & Interfaces
- Depends on the existing BPM/bars configuration feature (spec: `mvp-2-setup-bpm-and-loop-bars`) to provide tempo context.
- Extends `AppState` and audio command pipelines to support `MetronomeTick` and scheduled playback of recorded events.
- Coordinates with the planned `LoopEngine` (introduce if not yet implemented) to centralize state transitions and timing logic.

## Open Questions & Risks
- Confirm audio channel budget supports metronome tick overlay without starving sample playback.
- Determine whether the loop visualization (timeline) is needed in this scope or deferred to later stories.
- Clarify persistence expectations: is loop content ephemeral per session or should it survive app restarts?

## Decision Log
- 2025-09-28 — Adopt a dedicated LoopEngine inside `AppState` to manage lifecycle transitions, mirroring the Notion design proposal, so UI/input layers delegate to a single authority.
- 2025-09-28 — Reset captured loop when BPM/bars change during playback to avoid resampling artifacts; performers must re-record under the new tempo.
- 2025-09-28 — Provide mockable clock/timer abstraction to keep TDD feasible for timing-sensitive acceptance criteria.

## References
- Notion user story: [https://www.notion.so/2794150965e480199b7fddf8a666cdd8](https://www.notion.so/2794150965e480199b7fddf8a666cdd8)
- Parent epic: [https://www.notion.so/2704150965e480b0ab9ce85f5b5c7254](https://www.notion.so/2704150965e480b0ab9ce85f5b5c7254)
- Steering docs: `.spec-workflow/steering/product.md`, `.spec-workflow/steering/tech.md`, `.spec-workflow/steering/structure.md`
