# Design Document

## Overview

Enable playback of user-selected `.wav` samples via keyboard in a pads view. The TUI will provide two screens: `FileNavigation` (select samples) and `Pads` (playback). Selected samples are loaded into memory when transitioning to `Pads`. Each pad maps to a key; pressing a key plays the mapped sample immediately, supports overlapping polyphony and re-triggering, and highlights the active pad.

## Steering Document Alignment

### Technical Standards (tech.md)
Follows Rust 2024, modular files, and binary-level E2E testing via `@microsoft/tui-test`. For audio, adopt `rodio` (`https://github.com/RustAudio/rodio`) to load `.wav` files and play samples. The design keeps clear boundaries to integrate a TUI framework (e.g., `ratatui`) later.

### Project Structure (structure.md)
Organize modules for input handling, state, and UI rendering. Keep clear module boundaries and small functions. Tests live under `tests/e2e` and validate user flows.

## Code Reuse Analysis

### Existing Components to Leverage
- `src/app_state.rs`: Central state container; extend to hold selected file paths and a `pads` view state.
- `src/input.rs`: Extend key handling for navigation and pad triggers.
- `src/ui.rs`: Extend to render `Pads` layout and highlight logic.

### Integration Points
- Transition logic from `FileNavigation` → `Pads` uses selection state to populate pads.
- On pads entry, the controller sends Preload(key, path) commands to the audio thread to decode/cache samples once.
- UI/Controller sends Play(key) over a channel to the audio thread; no decoding on key press.
- Audio playback abstraction introduced behind a trait with a `rodio`-backed implementation.

## Architecture

- `AppState` holds mode enum: `FileNavigation | Pads` and selections.
- `PadsState` contains mapping: `key -> SampleSlot` where `SampleSlot { file_name, file_path }`.
- `AudioEngine` runs on a dedicated audio thread and encapsulates play semantics (polyphony, re-trigger). Concrete implementation `RodioAudioEngine` wraps `rodio` types.

```mermaid
flowchart LR
    Input[input.rs] -->|keys| Controller
    Controller[app_state.rs] -->|mode switch| UI[ui.rs]
    Controller -->|Play(key)| Chan((mpsc))
    Chan --> AudioThread[AudioRuntime (rodio)]
    UI -->|render pads| Terminal
```

### Modular Design Principles
- Single File Responsibility
- Component Isolation
- Service Layer Separation (AudioEngine)
- Utility Modularity

## Components and Interfaces

### `AudioEngine` (trait)
- Purpose: Abstract audio playback on a background thread.
- Interfaces:
  - `send(&self, cmd: AudioCommand) -> Result<()>` enqueues a command to the audio thread.
  - `AudioCommand` includes:
    - `Preload { key: char, path: PathBuf }`
    - `Play { key: char }`
    - `StopAll`
  - Supports concurrent plays (polyphony) and re-trigger overlap.

### `RodioAudioEngine` (concrete)
- Purpose: Implement `AudioEngine` using `rodio`.
- Approach:
  - Maintain a long-lived `rodio::OutputStream`/`OutputStreamHandle`.
  - Audio thread owns an `mpsc::Receiver<AudioCommand>` and a cache: `BTreeMap<char, Arc<DecodedSample>>` keyed by pad key.
  - On `Preload { key, path }`, decode the `.wav` once (e.g., via `rodio::Decoder` → `SamplesBuffer`) and store as `DecodedSample` in the cache under `key`.
  - On `Play { key }`, create a new `rodio::Sink` and append a cloned source from the cached `DecodedSample` (no per-press decode).
  - Polyphony: multiple `Sink`s mix concurrently.
  - Re-trigger: pressing the same key spawns a new `Sink`, overlapping with the previous one.

### `AppState`
- Purpose: Orchestrate modes and selected samples.
- Interfaces: `enter_pads()`, `leave_pads()`, `handle_key(Key)`.

### `PadsView`
- Purpose: Render pads grid with labels and highlight active pads.
- Interfaces: `render(frame, area, pads_state)`.

## Data Models

### `PadsState`
```
struct PadsState {
    key_to_slot: BTreeMap<char, SampleSlot>,
    active_keys: HashSet<char>,
    last_press_ms: BTreeMap<char, u128>,
}

struct SampleSlot {
    key: char,
    file_name: String,
}
```
Note: Original file paths are provided to the audio thread only during `Preload` and are not retained in UI state; UI displays `file_name` only.

### Key Auto-Repeat Suppression
- Purpose: Fire playback once per physical press.
- Approach: Track `last_press_ms[key]`; ignore subsequent press events for the same key if `now_ms - last_press_ms[key] < 100`.
- Rationale: Terminal key auto-repeat can generate rapid repeat events; this prevents unintended multiple triggers while preserving explicit re-triggers.

## Error Handling

### Error Scenarios
1. Unsupported file type on transition
   - Handling: Validate selected files are `.wav`; show error and stay on FileNavigation.
   - User Impact: Error popup, no crash.
2. Missing file at playback
   - Handling: Log and skip play; pad still renders; no crash.
3. Unsupported/failed decode
   - Handling: Surface a user-friendly error; do not crash; continue running.
4. Cache miss (unexpected)
   - Handling: Fall back to decode-at-play with warning; repopulate cache when feasible.
5. Channel send/recv failure
   - Handling: Attempt graceful restart of audio thread; surface error state to UI; do not crash.

## Assumptions & Non-goals
- Reuse the existing `FileNavigation` UI and selection logic; do not re-implement file navigation in this feature.
- Out of scope: choosing and wiring a concrete TUI framework (kept behind simple render functions for now).

## Testing Strategy

### Unit Testing
- Map building from selection to pads.
- Active pad highlight toggling on key press/release.

### Integration Testing
- Mode transitions with/without invalid files.
- AudioEngine polyphony and re-trigger semantics (mocked).

### End-to-End Testing
- Initial render: pads layout and labels.
- Press multiple keys simultaneously → overlapping indications.
- Press same key repeatedly → remains responsive, highlight persists appropriately.
- Esc navigates back to file navigation.

---

References:
- Requirements: `.spec-workflow/specs/playback-samples-with-keys/requirements.md`
- Notion: https://www.notion.so/2714150965e480ef9966ceed46cf47b1




