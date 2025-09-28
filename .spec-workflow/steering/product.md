# Product Overview

## Product Purpose
TermiGroove delivers a keyboard-driven terminal music workstation that lets small groups in Japan’s Gen Z scene co-create loops and tracks without expensive hardware. It lowers the cost and complexity of live sample performance while preserving a retro-futuristic vibe that feels intentional and professional.

## Target Users
- 20-somethings in Japan who already gather socially around laptops and music discovery
- Developers, tinkerers, and AI collaborators who prefer command-line tools but still want expressive audio workflows
- Bedroom DJs who need a portable setup for impromptu jams without controllers or DAW licenses

## Key Features
1. **Terminal-First Live Looping**: Map up to eight samples to pads, trigger them in real time, and manage volume/mute/solo entirely from the keyboard.
2. **Tempo & Timing Control**: Central BPM engine with bar-length configuration keeps all tracks in sync during performance.
3. **Integrated Audio Engine**: Rust-based CPAL pipeline handles loading, mixing, and playback with professional-grade reliability.
4. **File Explorer Workflow**: Ratatui Explorer integration streamlines browsing and assigning local audio files inside the TUI.

## Business Objectives
- Validate that a terminal-native music workstation resonates with early adopter groups in Japan.
- Achieve measurable social creation moments that differentiate TermiGroove from traditional DAWs.
- Build a sustainable foundation for future paid features (remote sessions, advanced effects).

## Success Metrics
- Monthly active users: 10,000 MAU within year one among 20-somethings in Japan.
- Week-one retention: 30% of new users continue sessions after onboarding.
- Group co-creation: 50% of active users participate in at least one group jam per month.
- Sharing: 40% of created tracks shared in real time, targeting 20,000 shared creations in first year.
- Community: 500 recurring groups with ≥3 members, sustaining 5% monthly organic growth.

## Product Principles
1. **Keyboard Mastery**: Every core workflow must be executable without a mouse, reinforcing speed and focus.
2. **Retro-Futuristic Clarity**: Visual design embraces neon TUI aesthetics while keeping information dense yet legible.
3. **Reliability Through TDD**: Ship only when specs, tests, and documentation align, preserving confidence in live sets.
4. **Inclusive Collaboration**: Encourage sessions where multiple people (or agents) can participate without specialized gear.

## Monitoring & Visibility (if applicable)
- **Dashboard Type**: Native TUI dashboard showing track states, BPM, resource usage, and session context.
- **Real-time Updates**: Continuous render loop with keyboard-driven refresh; future roadmap includes websocket bridge for remote mirroring.
- **Key Metrics Displayed**: Track activity, BPM, loop bars, volume meters, session status, and system performance indicators.
- **Sharing Capabilities**: Planned export of session state and jam logs; roadmap includes read-only shared views via secure tunnel.

## Future Vision
TermiGroove evolves into a collaborative music surface that spans local sessions, remote co-creation, and AI-assisted arrangement while remaining terminal-native and affordable. It becomes the go-to tool for spontaneous jams, livestream prep, and human+AI performance experiments.

### Potential Enhancements
- **Remote Access**: Secure session sharing over tunnels so off-site collaborators can view or influence a jam.
- **Analytics**: Capture historical BPM, loop, and track usage to inform set improvements and community insights.
- **Collaboration**: Multi-user editing, presence indicators, and shared sample libraries to speed up group creativity.
