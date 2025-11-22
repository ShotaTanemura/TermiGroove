# Technology Stack

## Project Type
Terminal-based music workstation and live looping instrument built as a cross-platform CLI/TUI application.

## Core Technologies

### Primary Language(s)
- **Language**: Rust 2024 Edition
- **Runtime/Compiler**: rustc 1.89.0 (rust-toolchain pinned)
- **Language-specific tools**: Cargo for build/test, rustfmt, clippy

### Key Dependencies/Libraries
- **ratatui 0.29**: Terminal UI rendering framework
- **crossterm 0.27**: Cross-platform terminal input/output backend
- **tui-popup 0.6**: Modal dialog rendering within ratatui
- **tui-input 0.14**: Text input field handling for the popup editor
- **ratatui-explorer (git)**: File explorer widget for browsing sample directories
- **rodio 0.18 / CPAL**: Audio playback stack for mixing and streaming samples
- **anyhow 1**: Error handling with context
- **@microsoft/tui-test**: Node-based end-to-end testing harness

### Application Architecture
Modular monolith following Domain-Driven Design (DDD) and Ports & Adapters (Hexagonal Architecture) patterns:

- **Domain Layer** (`src/domain/`): Defines port traits (interfaces) that domain logic requires. Contains no implementations, only trait definitions (e.g., `Clock`, `AudioBus`).
- **State Layer** (`src/state/`): Contains domain logic implementations (e.g., `LoopEngine`) that depend on domain ports via trait bounds.
- **Infrastructure Layer** (`src/audio.rs`, etc.): Provides concrete implementations of domain ports (adapters). Implements traits defined in domain layer.

Dedicated modules for state management (`app_state`), audio engine (`audio`), user input handling (`input`), selection/file models (`selection`), and rendering (`ui`). Follows an event-driven loop pulling terminal events, updating state, and re-rendering frames.

**Dependency Rule**: Domain layer has no dependencies on infrastructure or state layers. Infrastructure and state layers depend on domain layer (implement/use port traits).

### Data Storage (if applicable)
- **Primary storage**: In-memory structures; audio files read from local filesystem on demand
- **Caching**: None beyond process memory
- **Data formats**: WAV/MP3 sample files; internal structs and enums

### External Integrations (if applicable)
- **APIs**: None
- **Protocols**: Terminal I/O (stdin/stdout), audio device access via CPAL
- **Authentication**: Not applicable

### Monitoring & Dashboard Technologies (if applicable)
- **Dashboard Framework**: ratatui widgets for on-terminal dashboards
- **Real-time Communication**: Local render loop; potential future websocket bridge
- **Visualization Libraries**: ratatui built-ins, custom widgets, ASCII meters
- **State Management**: Central `AppState` struct

## Development Environment

### Build & Development Tools
- **Build System**: Cargo
- **Package Management**: Cargo (Rust), npm (Node e2e tools)
- **Development workflow**: Tight TDD loop with specs; manual cargo runs for interactive testing

### Code Quality Tools
- **Static Analysis**: `cargo clippy`
- **Formatting**: `cargo fmt`
- **Testing Framework**: `cargo test` (unit/integration), `npm run test:e2e` with @microsoft/tui-test
- **Documentation**: `cargo doc` (on demand)

### Version Control & Collaboration
- **VCS**: Git
- **Branching Strategy**: Feature branches with PRs to mainline (per workflow notes)
- **Code Review Process**: Spec-driven approvals; documented decision logs, GitHub PRs

### Dashboard Development (if applicable)
- **Live Reload**: Terminal redraw via event loop; no hot reload
- **Port Management**: Not applicable
- **Multi-Instance Support**: Multiple terminals possible; no shared state

## Deployment & Distribution (if applicable)
- **Target Platform(s)**: macOS primary, Linux and Windows supported via CPAL/crossterm
- **Distribution Method**: Source build via Cargo; future binaries possible
- **Installation Requirements**: Rust toolchain 1.89+, Node 18+ for e2e tests
- **Update Mechanism**: Git pull and rebuild

## Technical Requirements & Constraints

### Performance Requirements
- Sub-10ms input-to-audio latency for pad triggering
- Stable audio playback with zero pops/clicks during BPM changes
- Terminal rendering responsive at 20–60 FPS target

### Compatibility Requirements  
- **Platform Support**: macOS 14 (primary), Linux, Windows (targeted via CPAL)
- **Dependency Versions**: ratatui 0.29, crossterm 0.27, rodio 0.18, tui-input 0.14, @microsoft/tui-test ^0.0.1-rc.5
- **Standards Compliance**: Follow Rust edition idioms, CPAL audio interfaces

### Security & Compliance
- **Security Requirements**: Local-only app; ensure safe file handling and avoid executing untrusted content
- **Compliance Standards**: None formal; respect user privacy by avoiding telemetry
- **Threat Model**: Minimal attack surface; guard against malformed audio files causing crashes

### Scalability & Reliability
- **Expected Load**: Single user sessions, 8 concurrent tracks
- **Availability Requirements**: High reliability during live sets; deterministic behavior under stress
- **Growth Projections**: Future remote collaboration may require networking layer

## Technical Decisions & Rationale
1. **Rust + ratatui for TUI**: Provides performance, safety, and expressive terminal rendering; alternatives like Python curses lacked performance.
2. **CPAL/Rodio audio stack**: Cross-platform audio with low-level control, enabling precise looping and mixing.
3. **Strict TDD workflow**: Ensures confidence for live performance features and aligns with Notion-driven spec process.
4. **Domain-Driven Design with Ports & Adapters**: Introduces domain layer (`src/domain/ports.rs`) to define port traits (interfaces) that domain logic depends on. This enables:
   - **Dependency Inversion**: Domain logic doesn't depend on infrastructure; infrastructure implements domain interfaces.
   - **Testability**: Port traits enable dependency injection, allowing test doubles (e.g., `FakeClock`, `AudioBusMock`) to isolate domain logic.
   - **Maintainability**: Clear separation between business logic (domain) and technical concerns (infrastructure).
   - **Flexibility**: Infrastructure implementations can be swapped without changing domain logic.

## Known Limitations
- **Headless Audio Dependencies**: Virtual audio setup required for CI; configuration complexity exists.
- **Single-Process Architecture**: No remote collaboration yet; scaling to multi-user requires new infrastructure.
- **File Format Scope**: Currently optimized for WAV/MP3; other formats unsupported until future work.
