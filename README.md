# VoxVerse

VoxVerse is a voice-first character universe for immersive roleplay and learning. Built on a Tauri 2 + Vue 3 desktop stack, it features pluggable AI providers, multi-layered session memory, interactive relationship dynamics, and contextual learning feedback.

## Current Status

VoxVerse is in the V1 Alpha stage. The baseline brand transition from SpeakMate has been completed, along with core architecture upgrades (V1-A & V1-B):
- **Realtime Session Event Protocol**: Structured Tauri IPC events (`SessionEvent`) connecting front-end audio/transcript UI with the backend session coordinator.
- **Backend Provider Abstractions**: General traits (`SttProvider`, `TtsProvider`, `LlmProvider`) with reference implementations wrapping Whisper, Edge TTS, and OpenAI-compatible completions.
- **Four-layer Memory & Relationship Engine**: SQLite persistence of long-term facts, intimacy/trust state indicators, and tool routing auditing logs.
- **Frontend Management Panels**: Cognitive Memory and Relationship dynamics overlays with interactive editing options.

## Documentation Map

These files are the current documentation entry points:

- `task.md`: Authoritative roadmap and level/milestone status
- `implementation_plan.md`: VoxVerse V1 implementation plan details
- `walkthrough.md`: Developer/Maintainer snapshot and architectural overview
- `native-cli.md`: Read-only native CLI usage and diagnostic instructions
- `项目构建日志/README.md`: Iteration indices and historical milestones

## Main Capabilities

- **Streaming Roleplay**: Low-latency conversational loops powered by OpenAI-compatible completion layers.
- **Custom Character Definition**: Character profiles including names, avatars, greeting styles, personality guidelines, and TTS configurations.
- **Pluggable Audio Engine**: Edge-TTS integration alongside transcription support for Whisper.
- **Cognitive Memory Panel**: Visualize, toggle, delete, or manually inject facts stored inside SQLite.
- **Interactive Relationship Card**: View and adjust intimacy and trust levels, story phases, commitments, and user boundaries.

## Quick Start

1. Install dependencies:

   ```bash
   pnpm install
   ```

2. Start the desktop app in development mode:

   ```bash
   pnpm tauri dev
   ```

3. Run Rust unit & integration tests:

   ```bash
   cargo test --manifest-path src-tauri/Cargo.toml
   ```

4. **Double-click Launcher (Windows)**:
   - Double-click [LaunchVoxVerse.exe](file:///C:/AI_Coding/VoxVerse/LaunchVoxVerse.exe) to instantly run the newest compiled version of the app (release or debug build) silently.
   - Alternatively, you can run or inspect [LaunchVoxVerse.bat](file:///C:/AI_Coding/VoxVerse/LaunchVoxVerse.bat).

## Recommended Tooling

- VS Code
- Vue - Official (Volar)
- Tauri VS Code extension
- rust-analyzer
