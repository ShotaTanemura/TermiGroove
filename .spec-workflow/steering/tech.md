# Technology Stack

## Project Type
Terminal User Interface (TUI) music app (sampler/player) written in Rust with end-to-end TUI tests driven from Node.js tooling.

## Core Technologies

### Primary Language(s)
- **Language**: Rust 2024 edition; TypeScript for E2E test harness
- **Runtime/Compiler**: rustc (via toolchain file), Node.js for test runner
- **Language-specific tools**: cargo for Rust builds; npm for JS tooling

### Key Dependencies/Libraries
- **Rust**: (TBD as features are added; currently no runtime crates specified)
- **Dev tooling**: `cargo-husky` for git hooks (developer ergonomics)
- **JS/TS (dev)**: `@microsoft/tui-test` for terminal UI E2E tests; `typescript` for typings/transforms
 - **CI**: GitHub Actions for continuous integration

### Application Architecture
Standalone monolith binary (`termigroove`). The TUI orchestrates input handling, playback control, and view rendering. E2E tests spawn the compiled binary and assert terminal output.

### Data Storage (if applicable)
- **Primary storage**: In-memory for alpha
- **Data formats**: CLI args, stdout text; future sample metadata likely as simple files (TBD)

### External Integrations (if applicable)
- None in alpha (offline, co-located usage). Networked/remote features are out of scope.

### Monitoring & Dashboard Technologies (if applicable)
- **Dashboard Framework**: Terminal UI
- **Real-time Communication**: Local event loop within the process
- **Visualization Libraries**: Terminal text rendering (TBD specific crate)
- **State Management**: In-memory application state

## Development Environment

### Build & Development Tools
- **Build System**: cargo
- **Package Management**: cargo (Rust), npm (dev tooling)
- **Development workflow**: Build with cargo; run E2E with `tui-test`

### Code Quality Tools
- **Testing Framework**: `@microsoft/tui-test` for E2E
- **Formatting/Lints**: rustfmt/clippy (recommended), TypeScript compiler for test code
 - **CI**: GitHub Actions workflows for build and test automation

### Version Control & Collaboration
- **VCS**: Git
- **Branching Strategy**: Feature branches with PR reviews (per repo conventions)

## Deployment & Distribution (if applicable)
- **Target Platform(s)**: Desktop (macOS first-class), Linux/Windows TBD
- **Distribution Method**: Built binaries (cargo build/release)

## Technical Requirements & Constraints

### Performance Requirements
- Fast startup, responsive keyboard interactions in TUI

### Compatibility Requirements  
- **Platform Support**: macOS 14+ (validated in CI locally), others TBD
- **Dependency Versions**: Rust toolchain pinned via `rust-toolchain`

### Security & Compliance
- Local, offline app; minimal data handling in alpha

### Scalability & Reliability
- Single-process local app; reliability via robust input handling and graceful errors

## Technical Decisions & Rationale
1. **Rust for performance and portability**: Low-latency input/render loop is a good fit.
2. **TUI-first UX**: Keyboard-driven workflows match the product’s simplicity and flow principles.
3. **E2E via `@microsoft/tui-test`**: High-confidence testing against compiled binary behavior.

### Planned Decisions (not yet applied in codebase)
- **ratatui**: TUI framework to build terminal interfaces (`https://github.com/ratatui/ratatui`).
- **ratatui-explorer**: File selection UI for choosing sample files (`https://github.com/tatounee/ratatui-explorer`).
- **ratatui-widgets**: Additional widgets to enhance the TUI (`https://github.com/joshka/ratatui-widgets`).

## Known Limitations
- No audio engine or TUI framework integrated yet (skeleton). Crate selection pending.
- Networked/remote collaboration is out of scope for alpha.

---

### References
- `Cargo.toml`
- `src/main.rs`
- `package.json`
- `tui-test.config.ts`


