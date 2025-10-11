# Pause/Resume Behavior Matrix

| ID | Requirement(s) | Given | When | Then | Notes |
| --- | --- | --- | --- | --- | --- |
| BM-001 Happy pause from playback | 1 | Loop is actively playing a recorded loop (no active overdub); audio sinks streaming scheduled events | Performer presses space once | Loop transitions to `LoopState::Paused` within the same tick; current playback cycle offset is recorded; `PauseAll` audio command is enqueued and drains active sinks within ≤1 ms without clicks | Verifies Requirement 1 AC1 & AC3; confirm capture of `saved_offset` and audio halt timing |
| BM-002 Pause while recording overdub | 1 | Loop is recording an overdub layer atop existing playback (metronome running, overdub events scheduled) | Performer presses space | Engine stores playback cycle offset AND active overdub start offset; engine transitions to `LoopState::Paused` with metadata `was_recording = true`; `PauseAll` emitted | Covers Requirement 1 AC2 plus AC1/AC3 interactions with recording |
| BM-003 Resume to playback | 2 | Loop is paused from prior BM-001 scenario with stored playback offset; no overdub in progress | Performer presses space | Engine computes `cycle_start = now - saved_offset`, transitions back to `LoopState::Playing`, and scheduled events resume within ≤1 ms drift; monitoring resumes | Requirement 2 AC1 & AC3 |
| BM-004 Resume to recording | 2 | Loop is paused from BM-002 with `was_recording = true` and stored overdub offset | Performer presses space | Engine restores overdub timers using saved offsets, resumes recording pipeline, transitions to `LoopState::Recording`, monitoring resumes immediately | Requirement 2 AC2 & AC3 |
| BM-005 Repeated pause press while paused | 1,3 | Loop is already `LoopState::Paused`; no new inputs | Performer presses space again quickly | System ignores redundant pause, maintaining stored snapshot without emitting extra `PauseAll` | Guards idempotency; prevents toggling churn |
| BM-006 Non-pause command during pause window | 3 | Loop is playing or recording; performer uses other keyboard command (e.g., cancel, navigation) | Performer triggers non-space shortcut | System executes original command; `PauseAll` is NOT emitted; loop state remains unaffected | Requirement 3 AC1 |
| BM-007 Regression: random key noise | 3 | Automated test fires random non-space keys during playback/recording | Noise input occurs | Loop never transitions to `LoopState::Paused`; tests assert lack of `PauseAll` dispatch | Requirement 3 AC2 |
| BM-008 Regression: UI feedback | 2,3 | Loop transitions into paused state (BM-001/BM-002) | Pause occurs | UI reflects paused state within next render frame; resume clears indicator | Requirement 2 AC3 + usability clause |
| BM-009 Resume timing tolerance | 2 | Pause snapshot stored; system idle for arbitrary duration; system clock advanced | Resume triggered after variable delay | Resumed playback/recording aligns with stored offset, verifying ≤1 ms drift tolerance | Explicit timing guard |
| BM-010 Multiple pause/resume cycles | 1,2 | Performer alternates pause/resume several times in succession | Sequence of space presses | Each pause captures fresh offsets; each resume realigns to latest snapshot without accumulating drift | Ensures cumulative integrity |
| BM-011 Audio sink already silent | 1 | Loop enters pause but some audio sinks already idle | Pause triggered | `PauseAll` handling no-ops safely, preserving handles and preventing panic | Reliability clause from requirements |
| BM-012 Post-resume overdub new events | 2 | Loop paused during recording, resumed, and performer records new events | Performer records after resume | Newly recorded events align with resumed cycle_start; previous layers remain aligned | Confirms state restoration completeness |
| BM-013 End-to-end manual control | 1,2,3 | Live terminal session with base loop + overdub | Performer pauses, waits, resumes, triggers other commands | No audio artifacts, UI feedback correct, no unintended commands triggered | Manual validation scenario |

## Regression Guards Summary

- Non-space inputs must never dispatch `PauseAll` or set `LoopState::Paused`.
- Redundant space presses while already paused should be no-ops.
- Audio command queue must maintain ordering; `PauseAll` cannot overtake pending resume commands.

## Open Questions

1. Should the UI display a distinct indicator when pausing during recording versus playback?
2. Do we need to cancel scheduled future events explicitly when pausing, or does `PauseAll` inherently halt them via sink handles?
3. What is the expected behavior if the performer pauses during the count-in before any recording starts?
4. Should metronome clicks stop immediately on pause or continue until resume?


