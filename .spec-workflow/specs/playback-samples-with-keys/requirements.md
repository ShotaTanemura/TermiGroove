# Requirements Document

## Introduction

This spec defines the requirements to enable playback of user-selected samples via keyboard keys within the TermiGroove TUI. It delivers a minimal, responsive pad UI mapped to selected audio samples, allowing immediate playback through key presses and intuitive navigation between file selection and playback screens.

## Alignment with Product Vision

This feature supports the product principles of Simplicity & Flow and Accessibility over Gear by providing a keyboard-first, low-friction playback experience that gets users creating quickly without external hardware.

## Requirements

### Requirement 1: Load selected samples into pads view

**User Story:** As a user, I want to playback samples selected using the keyboard within the TUI so that I can play music improvised simply.

#### Acceptance Criteria

1. WHEN samples (wav files) are selected AND the user moves from the file navigation screen to the pads screen, THEN the application SHALL load those samples into memory.
2. WHEN samples are selected AND the user moves to the pads screen, THEN the user SHALL see square pads arranged with equal spacing, each labeled with a key and its associated file name.
3. IF a selected file is not a `.wav`, THEN the system SHALL present an error popup and remain on the file navigation screen when the user attempts to move to the pads screen.

### Requirement 2: Immediate playback and visual feedback

**User Story:** As a user, I want mapped keys to play their samples immediately so that the instrument feels responsive for improvisation.

#### Acceptance Criteria

1. WHEN a user presses a key displayed on a pad in the pads screen AND the corresponding sample is loaded, THEN the application SHALL start playing the sample immediately.
2. WHEN a user presses a key displayed on a pad in the pads screen, THEN the pad SHALL highlight with dark green while active and return to normal when released/finished.
3. WHEN a user presses `Esc` on the pads screen, THEN the application SHALL navigate back to the file navigation screen.
4. WHEN two or more samples are loaded AND the user presses multiple mapped keys at the same time, THEN the samples SHALL play simultaneously in overlapping layers.
5. WHEN a sample is already playing AND the user presses its mapped key again before playback completes, THEN a new instance SHALL start immediately, overlapping partially with the currently playing instance.

## Non-Functional Requirements

### Code Architecture and Modularity
- Single Responsibility Principle: Each file should have a single, well-defined purpose
- Modular Design: Components, utilities, and services should be isolated and reusable
- Dependency Management: Minimize interdependencies between modules
- Clear Interfaces: Define clean contracts between components and layers

### Performance
- Startup and screen transitions under 200 ms on macOS release build
- Key press to audible start targeted under 30 ms (subject to audio backend selection)

### Security
- No network I/O; local-only processing of files

### Reliability
- Graceful handling of unsupported file types with clear error messaging
- No crashes when pressing keys with no sample mapped

### Usability
- Pads arranged at regular intervals; keys arranged around 60% of terminal height
- Each pad shows the mapped key (center) and file name (below)

---

References: Notion User Story: https://www.notion.so/2714150965e480ef9966ceed46cf47b1


