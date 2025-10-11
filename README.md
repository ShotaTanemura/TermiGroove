# TermiGroove

TermiGroove is a Rust TUI groovebox that lets you trigger pads and capture loops directly from the terminal.

## Build & Run

```
cargo build --release
./target/release/termigroove
```

For end-to-end tests (via `@microsoft/tui-test`):

```
npm install
npx tui-test tests/e2e/loop_capture.test.ts
```

## Loop Recording Quickstart

1. Launch TermiGroove and navigate the file explorer (left pane).
2. Press `Space` to select at least one `.wav` sample; the selection appears in the right pane.
3. Press `Enter` to switch into Pads mode.
4. Press `Space` to start the metronome count-in (4 ticks at the current BPM).
5. When recording begins, hit pad keys (e.g. `Q`, `W`, etc.) to capture events.
6. After the loop length elapses, playback repeats automatically.
7. Press `Space` again to stop the loop and return to Idle; `Esc` exits Pads mode.

## Pause & Resume Controls

- Press `Space` while playback or recording is running to pause immediately. The loop transitions to `LoopState::Paused`, sends `PauseAll` to halt audio sinks without pops, and the summary banner shows `PAUSED` in yellow.
- Press `Space` again to resume. The loop realigns using the stored offset so playback and overdubs restart within ≤1 ms drift, and the UI status clears the paused indicator.
- Other shortcuts (e.g., `Ctrl+Space` to clear, navigation keys in Browse mode) continue to behave normally during the pause feature and never emit pause commands.

## Metronome & Timing Notes

- Timing is driven by a `Clock` abstraction so tests can inject deterministic time.
- The metronome synthesizes short beeps instead of loading external samples.
- The engine drains any overdue metronome ticks in a single `update` call to avoid lag.

## Testing Strategy

- Unit tests cover `LoopEngine` happy paths, cancellation, and tempo resets.
- Integration tests (`tests/app_state_loop.rs`) verify AppState and metronome behavior.
- The E2E script (`tests/e2e/loop_capture.test.ts`) drives the TUI count-in → record → playback flow.

## Known Limitations

- Loop contents are kept in memory only; restarting the app clears recorded loops.
- Overdubbing and multi-layer loops are out of scope for MVP-2.
- The UI shows textual hints; no timeline visualization is present yet.
