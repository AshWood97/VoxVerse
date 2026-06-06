# VoxVerse

VoxVerse is a voice-first character universe for immersive roleplay and learning. Built on a Tauri 2 + Vue 3 desktop stack, it features pluggable AI providers, multi-layered session memory, interactive relationship dynamics, and contextual learning feedback.

## Current Status

VoxVerse is on the v0.6.0 productization baseline. The release number is ahead of the product-roadmap "V0.3 baseline" language in `task.md`: v0.6.0 packages the already-built Lv.5 slices into a tighter desktop learning product while keeping external voice/provider paths marked for manual validation.

The active v0.6 scope is:
- **Provider Profiles**: Multiple OpenAI-compatible, Ollama, or custom profiles with per-profile OS Keychain storage for API keys.
- **Practice Modes**: Free Talk, Roleplay, Scenario Drill, IELTS Speaking, and Interview Practice are seeded into SQLite and saved on sessions.
- **Voice Runtime UX**: The UI exposes recording, transcription, thinking, speaking, fallback, and error states for STT/TTS flows.
- **Structured Feedback Loop**: Each successful user turn can enter correction/vocabulary/session feedback with quick scores, score breakdowns, pronunciation notes, and next prompt suggestions.
- **Character Memory & Relationship Context**: Visible long-term facts, relationship state, and per-character learning goals are injected into chat prompts; the Memory panel can clear the active character's memory facts.
- **Native CLI & Agent Control**: `pnpm cli -- ...` exposes read-only diagnostics/export plus guarded write/network tools.
- **Local-first Privacy Controls**: Automatic per-message correction can be disabled while keeping manual correction, translation, polish, and report actions available.
- **Stats, Achievements, and Local Samples**: Learning stats and local achievement toasts are productized, and character discovery is intentionally limited to local sample imports in v0.6.

## Documentation Map

These files are the current documentation entry points:

- `task.md`: Authoritative roadmap and level/milestone status
- `implementation_plan.md`: VoxVerse V1 implementation plan details
- `walkthrough.md`: Developer/Maintainer snapshot and architectural overview
- `native-cli.md`: Read-only native CLI usage and diagnostic instructions
- `升级计划v0.6.md`: Current v0.6 productization execution plan

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
- The Memory panel can clear only the active character's memory facts without deleting conversations, vocabulary, or saved corrections.
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

## Platform Targets

- **Windows**: The supported Windows target is Intel/AMD 64-bit. Windows 32-bit (`i686-pc-windows-msvc`) is not part of the v0.5 release scope.
- **macOS**: The v0.5 macOS target is Apple Silicon. The release target is a locally buildable `.app`; Developer ID signing, notarization, auto-update, and DMG distribution are not part of this pass.

## macOS Apple Silicon

1. Install Rust, Node.js, and pnpm on the Mac:

   ```bash
   brew install node pnpm
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Install dependencies and start development mode:

   ```bash
   pnpm install
   pnpm tauri dev
   ```

3. Build the frontend and package the desktop app:

   ```bash
   pnpm build
   pnpm tauri build
   ```

   The local `.app` bundle is generated under `src-tauri/target/release/bundle/macos/VoxVerse.app`.

4. Runtime notes:
   - API keys are stored in the macOS Keychain.
   - The SQLite database is discovered at `~/Library/Application Support/com.voxverse.desktop/voxverse.db`.
   - TTS first tries the existing Edge TTS path. If that fails, the frontend falls back to browser `speechSynthesis`.

## Recommended Tooling

- VS Code
- Vue - Official (Volar)
- Tauri VS Code extension
- rust-analyzer
