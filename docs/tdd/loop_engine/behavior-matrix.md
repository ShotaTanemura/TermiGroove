# Loop Engine Behavior Matrix

## Legend
- **State abbreviations:** I (Idle), Rdy (Ready), Rec (Recording), Ply (Playing)
- **Inputs:** `Space`, `Pad(key)`, `Tick` (metronome), `Elapsed(loop length)`, `TempoChange`
- **Outputs:** `AudioCommand::Play`, `AudioCommand::PlayMetronome`, scheduled playback, state change notifications
- **Clock:** mockable Instant (FakeClock in tests)

## Happy Path Lifecycle
| ID | Given | When | Then | Invariants |
|----|-------|------|------|------------|
| HP-001 | I, Pads mode | `Space` pressed | State → Rdy, enqueue 4 metronome ticks spaced by BPM interval | No pad events captured; status indicates Ready |
| HP-002 | Rdy after N ticks (<4) | `Tick` fires | Continue Rdy, decrement countdown | Countdown must reach 0 after exactly 4 ticks |
| HP-003 | Rdy with countdown 0 | `Tick` fires | State → Rec, record start timestamp | Loop start timestamp resets; pending metronome stops |
| HP-004 | Rec, pad key `q` pressed at t=10ms | `Pad(q)` | Play sample immediately, store event `(q,10)` | Offset relative to record start; command ordering Play before storage |
| HP-005 | Rec, elapsed reaches loop length | `Elapsed` event | State → Ply, stop recording further events | Subsequent pad presses ignored for storage |
| HP-006 | Ply with stored events | `Clock` advances to offset | Emit scheduled `Play` for each event once per cycle | No drift >2ms per requirements |

## Count-in & Metronome Edge Cases
| ID | Given | When | Then | Notes |
|----|-------|------|------|-------|
| MT-001 | I, BPM=120 | `Space` | Issue exactly 4 metronome commands, each separated by 500ms | Synthesized beep each time |
| MT-002 | Rdy, countdown=2 | `Space` | Cancel count-in, state → I, no further ticks | Metronome queue cleared |
| MT-003 | Rdy, countdown=1 | `Tick` fires late (clock skew) | Still transition to Rec exactly once | Clock abstraction ensures deterministic steps |
| MT-004 | Rdy, countdown>0, main loop stalled | `update` called after multiple overdue ticks | Engine drains all due ticks in one call, arrives in Rec without lag | Guarded by `metronome_count_in_handles_delayed_update_without_lag` |

## Cancellation Scenarios
| ID | Given | When | Then | Invariants |
|----|-------|------|------|------------|
| CN-001 | Rdy (during count-in) | `Space` | State → I, pending ticks cleared | No events stored |
| CN-002 | Rec with events stored | `Space` | State → I, clear recorded events, no playback scheduled | AudioBus receives no additional commands |
| CN-003 | Rec | `TempoChange` triggered | Equivalent to CN-002 | Tempo change handled via explicit reset |
| CN-004 | Ply with loop running | `Space` | State → I, metronome queue stays empty, recorded events cleared | Ensures no metronome retrigger on stop |

## Tempo/Bars Change
| ID | Given | When | Then | Notes |
|----|-------|------|------|-------|
| TB-001 | Ply with events | `TempoChange` (BPM updated) | Engine reset: state → I, events cleared | Requires AppState invoking reset hook |
| TB-002 | Ply with events | `BarsChange` | Same as TB-001 | Loop length recomputed next record |

## Playback Scheduling Checks
| ID | Given | When | Then | Invariants |
|----|-------|------|------|------------|
| PS-001 | Ply, cycle start = t0 | `Clock` hits t0+offset | Emit `Play` once; schedule next cycle for t0+loop_length+offset | No duplicate commands per cycle |
| PS-002 | Ply | `Clock` advances past multiple cycles | Engine wraps and schedules commands per cycle | Start times stay consistent |
| PS-003 | Ply, no events | `Clock` | No commands emitted | Idle playback allowed |

## Reliability / Failure Modes
| ID | Given | When | Then | Notes |
|----|-------|------|------|-------|
| RF-001 | Any state | AudioBus send fails | Log error, state unchanged | Implementation should handle Result on send |
| RF-002 | Ready/Recording | Generated metronome fails | Log warning, continue lifecycle | Covered by synthesized beep fallback |
| RF-003 | Playing | AppState loop update missed frame | Next update catches up due to stored timestamps | Drift minimized by using durations |
| RF-004 | Ready/Recording/Playing | User stops loop via Space | AudioBus receives no new metronome commands post-stop | Covered by integration tests ensuring saturation |

## Non-Goals / Out of Scope
- Timeline visualization or transport UI changes (handled elsewhere)
- Overdubbing multiple layers per requirement scope
- Persisting loop data across runs (explicitly avoided in design)
- Automatic sample selection; user must select at least one pad sample before loop capture

## Coverage & Regression Guards
- `tests/loop_engine/loop_happy_path.rs` – validates end-to-end lifecycle and metronome integrity
- `tests/loop_engine/loop_cancel.rs` – enforces Ready/Recording cancellation behavior
- `tests/loop_engine/loop_bpm_reset.rs` – guards tempo/bars reset semantics
- `tests/app_state_loop.rs` – integration coverage for AppState wiring and metronome stop behavior
- `tests/e2e/loop_capture.test.ts` – user-level verification of loop capture flow in the TUI

## Open Questions
1. Should status text show countdown ticks numerically (requires minor UI work)?
2. How to surface errors from AudioBus on metronome generation failure (status vs. log)?
3. Do we need throttle on cancel spam (Space mashing) or is instantaneous transitions acceptable?
