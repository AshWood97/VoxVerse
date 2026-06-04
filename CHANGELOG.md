# Changelog

## v0.5.0 - 2026-06-04

### Added

- Added support for macOS Application Support database discovery to native CLI (`voxverse-cli`).
- Added robust unit tests verifying discovery order and platform overrides (Windows and macOS).
- Added platform target specifications and macOS build setup instructions in `README.md`.
- Added GitHub Actions CI matrix (`.github/workflows/ci.yml`) to automatically build and test the codebase on Windows and macOS.

### Changed

- Updated version configuration to v0.5.0 across npm (`package.json`), Cargo (`Cargo.toml`), and Tauri (`tauri.conf.json`).

### Compatibility

- Database candidates schema remains identical; CLI JSON output structure is preserved.

## v0.4.0 - 2026-06-04

### Added

- Added a full skin preset system with ten selectable UI skins from the skin reference plan.
- Added shared theme normalization and application helpers for startup and Settings.
- Added bilingual Settings labels and descriptions for System Default and all shipped skins.

### Changed

- Changed the default appearance from legacy dark mode to Graphite Minimal.
- Changed legacy `dark` and `light` preferences to migrate to Graphite Minimal and Warm Paper.
- Tokenized hard-coded accent, warning, success, danger, overlay, and progress colors across the main UI.

### Compatibility

- Existing `localStorage.theme` values remain supported: `dark`, `light`, and `system` all resolve safely.
- No backend schema, app data, or API configuration changes are required.

### Validation

- `pnpm build`
- `pnpm validate:lv4`

## v0.3.0 - 2026-06-04

### Added

- Added v0.3 version metadata across npm, Cargo, Tauri, and CLI output.
- Added SQLite indexes for session history, messages, corrections, vocabulary, memory facts, and tool invocations.
- Added structured learning report fields for skill breakdowns and next drills.
- Added a Settings privacy control for automatic per-message correction.
- Added virtualized rendering for the session history modal.

### Changed

- Renamed new CLI/Agent audit actors to `voxverse-cli` and `voxverse-agent`.
- Kept `speakmate-cli` and `speakmate-agent` as audit-filter aliases for old logs and scripts.
- Updated agent protocol metadata to prefer VoxVerse names while still reporting the legacy JSONL protocol alias.
- Updated v0.3 planning, maintainer, CLI, and README docs to match current VoxVerse naming and scope.

### Fixed

- Audit reading now falls back to legacy `speakmate-audit.jsonl` when no `voxverse-audit.jsonl` exists.
- CLI database discovery now prefers `VOXVERSE_DB` and keeps `SPEAKMATE_DB` only as a legacy alias.
- CLI send-message smoke overrides now prefer `VOXVERSE_CLI_SEND_*` and keep `SPEAKMATE_CLI_SEND_*` as legacy aliases.

### Compatibility

- Existing `speakmate.db`, `speakmate-audit.jsonl`, old app data directories, and old Keyring entries remain readable for migration.
- New write paths use `voxverse.db`, `voxverse-audit.jsonl`, `voxverse-cli`, and `voxverse-agent`.

### Validation

- `pnpm build`
- `cargo check`
- `cargo test`
- `pnpm validate:lv4`

### Known Manual Checks

- Ollama chat compatibility still requires a machine with Ollama installed and reachable.
- OpenAI/Whisper and browser STT/TTS fallback still require interactive desktop validation with real credentials and device permissions.
