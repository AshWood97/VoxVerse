# VoxVerse

VoxVerse is a voice-first character universe for immersive roleplay and learning. Built on a Tauri 2 + Vue 3 desktop stack, it features pluggable AI providers, multi-layered session memory, interactive relationship dynamics, and contextual learning feedback.

## Current Status

VoxVerse is on the V0.3 upgrade baseline. The active upgrade scope is:
- **Provider Profiles**: Multiple OpenAI-compatible, Ollama, or custom profiles with per-profile OS Keychain storage for API keys.
- **Practice Modes**: Free Talk, Roleplay, Scenario Drill, IELTS Speaking, and Interview Practice are seeded into SQLite and saved on sessions.
- **Voice Runtime UX**: The UI exposes recording, transcription, thinking, speaking, fallback, and error states for STT/TTS flows.
- **Structured Feedback Loop**: Each successful user turn can enter correction/vocabulary/session feedback with score breakdowns and next-drill suggestions.
- **Character Memory & Relationship Context**: Visible long-term facts and relationship state are injected into chat prompts per character.
- **Native CLI & Agent Control**: `pnpm cli -- ...` exposes read-only diagnostics/export plus guarded write/network tools.
- **Local-first Privacy Controls**: Automatic per-message correction can be disabled while keeping manual correction, translation, polish, and report actions available.

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
- **Practice Sessions**: Mode and scenario context are persisted with each conversation and restored from history.
- **Cognitive Memory Panel**: Visualize, toggle, delete, or manually inject facts stored inside SQLite.
- **Interactive Relationship Card**: View and adjust intimacy and trust levels, story phases, commitments, and user boundaries.

## Local-first & Privacy

- Conversation data, characters, memory facts, vocabulary, corrections, and audit logs are stored in the local SQLite database.
- API keys are stored in the OS Keychain and are never returned by diagnostics, profile listing, or session export.
- Settings can clear the active profile's saved API key without deleting local learning data.
- Network operations send content only to the active configured provider. CLI `coach-report` and agent coaching tools send the selected transcript only when `--allow-network` is present.
- Legacy `speakmate.db`, `speakmate-audit.jsonl`, and old Keyring names remain as compatibility paths for older installs and scripts.

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

4. Run the product validation baseline:

   ```bash
   pnpm exec vue-tsc --noEmit
   pnpm build
   pnpm validate:v0.1
   pnpm validate:lv4
   ```

5. Use the native CLI:

   ```bash
   pnpm cli -- status --json
   pnpm cli -- diagnostics run --json
   pnpm cli -- session export --session <id> --json
   ```

6. **Double-click Launcher (Windows)**:
   - Double-click [LaunchVoxVerse.exe](file:///C:/AI_Coding/VoxVerse/LaunchVoxVerse.exe) to instantly run the newest compiled version of the app (release or debug build) silently.
   - Alternatively, you can run or inspect [LaunchVoxVerse.bat](file:///C:/AI_Coding/VoxVerse/LaunchVoxVerse.bat).

## Recommended Tooling

- VS Code
- Vue - Official (Volar)
- Tauri VS Code extension
- rust-analyzer
