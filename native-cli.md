# VoxVerse Native CLI

> Status: CLI, JSONL/MCP stdio agent-control slice, read-only character/session/profile inspection, structured session export, audited local write tools, guarded network send-message, retry-last recovery, and model-backed coaching report implemented on 2026-04-28.

The native CLI is exposed through `pnpm cli -- ...` and the Rust binary `voxverse-cli`. Legacy database, audit, and Keyring names are still discovered for older installs. It is the safe automation and agent-control surface for VoxVerse: most commands are read-only, emit stable JSON, and never read or print API keys. Profile switching, session start, and local message append are controlled write operations and require explicit `--yes`. Model-backed send-message, retry-last, and coaching reports additionally require `--allow-network`.

## Commands

```bash
pnpm --silent cli -- status --json
pnpm --silent cli -- config profiles list --json
pnpm --silent cli -- config profiles switch <profile-id> --json --yes
pnpm --silent cli -- character list --json
pnpm --silent cli -- session list --json
pnpm --silent cli -- session start --character <character-id> --title "Practice" --mode roleplay --scenario interview --json --yes
pnpm --silent cli -- session append-message --session <session-id> --role user --content "Hello" --json --yes
pnpm --silent cli -- session send-message --session <session-id> --content "Hello" --json --yes --allow-network
pnpm --silent cli -- session retry-last --session <session-id> --json --yes --allow-network
pnpm --silent cli -- session coach-report --session <session-id> --json --allow-network
pnpm --silent cli -- session export --session <id> --json
pnpm --silent cli -- audit list --json --operation practice.message.send --actor voxverse-cli --result success
pnpm --silent cli -- audit export --json --session <id>
pnpm --silent cli -- diagnostics run --json
pnpm --silent cli -- agent tools list --json
pnpm --silent cli -- agent tools list --json --allow-network
pnpm --silent cli -- agent tools list --json --allow-writes
pnpm --silent cli -- agent tools list --json --allow-writes --allow-tool append_session_message
pnpm --silent cli -- agent tools list --json --allow-writes --allow-network --allow-tool send_message --allow-tool retry_last_message
```

Equivalent direct Cargo form:

```bash
cargo run --quiet --manifest-path src-tauri/Cargo.toml --bin voxverse-cli -- diagnostics run --json
```

## Database Discovery

The CLI searches for the local database in this order:

1. `--db <path>`
2. `VOXVERSE_DB`
3. `SPEAKMATE_DB` (legacy)
4. `%LOCALAPPDATA%\com.voxverse.desktop\voxverse.db`
5. `%LOCALAPPDATA%\com.voxverse.app\voxverse.db` (legacy)
6. `%LOCALAPPDATA%\com.ai-speaking.desktop\voxverse.db` (legacy)
7. `%LOCALAPPDATA%\com.ai-speaking.desktop\speakmate.db` (legacy)
8. `%LOCALAPPDATA%\com.ai-speaking.app\speakmate.db` (legacy)
9. `%LOCALAPPDATA%\SpeakMate\speakmate.db` (legacy)

The legacy `com.ai-speaking.app` path is kept so users with data created before the Tauri identifier change can still inspect their local database.

## Safety Rules

- CLI query commands are read-only.
- `config profiles switch`, `session start`, `session append-message`, `session send-message`, and `session retry-last` are write operations and require `--yes`.
- `session send-message`, `session retry-last`, `session coach-report`, agent `send_message`, agent `retry_last_message`, and agent `generate_session_coaching_report` are network operations and require `--allow-network`.
- Write operations append an audit event to `voxverse-audit.jsonl` next to `voxverse.db`, with legacy `speakmate-audit.jsonl` still readable for older data.
- `session coach-report` and agent `generate_session_coaching_report` are network read-only operations: they generate Markdown coaching advice in memory and do not write the database or audit log.
- `session coach-report` and agent `generate_session_coaching_report` send the selected session transcript to the configured active model endpoint when `--allow-network` is present. Use them only for transcripts you are comfortable sharing with that endpoint.
- `agent serve --allow-writes --yes` can be narrowed with repeated `--allow-tool <tool>` flags.
- Query/profile commands do not read API keys. Network send-message, retry-last, and coaching report generation read the active profile key from Keyring when required, but never print it.
- Profile output reports `keychain_status: "not_read"`.
- JSON output includes warnings instead of trying to silently repair data.
- Database access uses SQLite read-only mode.

## Implemented Surface

- `pnpm cli -- status --json`: local database discovery, active profile summary, profile/session/message/character counts.
- `pnpm cli -- config profiles list --json`: provider profile list, with legacy `app_config` fallback.
- `pnpm cli -- config profiles switch <profile-id> --json --yes`: set active provider profile and append an audit event.
- `pnpm cli -- character list --json`: local character catalog for automation, excluding full prompts.
- `pnpm cli -- session list --json`: recent session summaries with character name and message count.
- `pnpm cli -- session start --character <id> --mode <mode-id> --scenario <scenario-id> --json --yes`: create a context-aware practice session for a character and append an audit event. `--mode` defaults to `free_talk`; `--scenario` is optional.
- `pnpm cli -- session append-message --session <id> --role <user|assistant> --content <text> --json --yes`: append a local message and audit event without calling an LLM.
- `pnpm cli -- session send-message --session <id> --content <text> --json --yes --allow-network`: append a user message, call the active OpenAI-compatible profile, append the assistant response, and audit the run.
- `pnpm cli -- session retry-last --session <id> --json --yes --allow-network`: retry the last user message in a session, append only the assistant response, and audit the recovery run.
- `pnpm cli -- session coach-report --session <id> --json --allow-network`: export a session locally, call the active profile for Markdown coaching advice, and return the report without DB or audit writes.
- `pnpm cli -- session export --session <id> --json`: one session plus ordered messages, mode/scenario context, local summary counts, and Markdown transcript.
- `pnpm cli -- audit list --json`: recent audit events from `voxverse-audit.jsonl`, with legacy `speakmate-audit.jsonl` fallback, optionally filtered by `--operation`, `--actor`, `--result`, `--profile`, `--session`, `--message`, or `--character`.
- `pnpm cli -- audit export --json`: all matching audit events from `voxverse-audit.jsonl`, with legacy `speakmate-audit.jsonl` fallback and the same filter flags as `audit list`.
- `pnpm cli -- diagnostics run --json`: schema presence, active profile, candidate database paths, read-only runtime hints.
- `pnpm cli -- agent tools list --json`: list read-only tools available to agent callers.
- `pnpm cli -- agent tools list --json --allow-network`: include network read-only coaching report generation in the advertised tool list.
- `pnpm cli -- agent tools list --json --allow-writes`: include guarded write tools in the advertised tool list.
- `pnpm cli -- agent tools list --json --allow-writes --allow-tool <tool>`: preview a narrowed write-tool allowlist.
- `pnpm cli -- agent tools list --json --allow-writes --allow-network --allow-tool send_message --allow-tool retry_last_message`: preview model-backed send and retry tools.
- `pnpm cli -- agent serve --stdio --read-only`: JSONL agent-control server for read-only tool calls, with MCP-style JSON-RPC 2.0 compatibility when requests include `jsonrpc: "2.0"`.
- `pnpm cli -- agent serve --stdio --read-only --allow-network`: enables network read-only `generate_session_coaching_report` while keeping DB writes disabled.
- `pnpm cli -- agent serve --stdio --allow-writes --yes`: enables guarded agent write tools. Currently this exposes `start_practice_session`, `append_session_message`, and `switch_provider_profile`.
- `pnpm cli -- agent serve --stdio --allow-writes --yes --allow-tool append_session_message`: enables only the named write tool while keeping all read-only tools available.
- `pnpm cli -- agent serve --stdio --allow-writes --yes --allow-network --allow-tool send_message --allow-tool retry_last_message`: enables model-backed send-message and retry-last recovery, keeping both behind write and network gates.
- `pnpm validate:lv4`: includes CLI diagnostics, character-list, session-list, structured session export, agent-tool, legacy agent-stdio, MCP stdio, profile-switch, filtered audit-log, agent-write guard, local mock send-message smoke, retry-last recovery, and model-backed coaching report checks. Write smoke checks use temporary database copies.

## UI Feedback vs CLI Coach Report

- The in-app feedback panel is a per-message learning helper. It asks the model for structured JSON so the UI can render grammar correction, translation, polish, score breakdown, next drills, and summary cards. If the model returns prose instead of JSON, VoxVerse shows the raw model response as a fallback instead of failing the panel, and successful corrections are saved for stats/report context.
- `session coach-report` is a CLI/agent report path. It exports a local session transcript, sends that transcript to the active model endpoint only when `--allow-network` is present, and returns Markdown coaching advice. It does not write DB rows or audit events because it is intentionally network read-only.

## Agent Protocols

The agent-control server supports two line-delimited stdio request styles:

- Legacy VoxVerse JSONL RPC: `{ "id": "...", "tool": "get_app_status", "arguments": {} }`
- MCP-style JSON-RPC 2.0: `{ "jsonrpc": "2.0", "id": "...", "method": "tools/list" }`

Start the server:

```bash
pnpm --silent cli -- agent serve --stdio --read-only
```

Start network read-only coaching mode:

```bash
pnpm --silent cli -- agent serve --stdio --read-only --allow-network
```

Start guarded write mode:

```bash
pnpm --silent cli -- agent serve --stdio --allow-writes --yes
```

Start guarded write mode with a narrow allowlist:

```bash
pnpm --silent cli -- agent serve --stdio --allow-writes --yes --allow-tool append_session_message
```

Start model-backed send-message mode:

```bash
pnpm --silent cli -- agent serve --stdio --allow-writes --yes --allow-network --allow-tool send_message
```

Example requests:

```jsonl
{"id":"init","method":"initialize"}
{"id":"tools","method":"tools/list"}
{"id":"sessions","method":"tools/call","params":{"name":"list_sessions","arguments":{"limit":5}}}
{"id":"stop","method":"shutdown"}
```

MCP-style example:

```jsonl
{"jsonrpc":"2.0","id":"init","method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"client","version":"0.0.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":"tools","method":"tools/list"}
{"jsonrpc":"2.0","id":"status","method":"tools/call","params":{"name":"get_app_status","arguments":{}}}
```

Supported read-only tools:

- `get_app_status`
- `list_provider_profiles`
- `list_characters`
- `run_runtime_diagnostics`
- `list_sessions`
- `export_session`
- `list_audit_events`

`list_audit_events` accepts optional `operation`, `actor`, `result`, `profile_id`, `session_id`, `message_id`, and `character_id` filters.

Network read-only tool, only available with `--allow-network`:

- `generate_session_coaching_report`

Guarded write tools, only available with `--allow-writes --yes`:

- `start_practice_session`
- `append_session_message`
- `switch_provider_profile`

Guarded network write tools, only available with `--allow-writes --yes --allow-network`:

- `send_message`
- `retry_last_message`

Supported `--allow-tool` values are `start_practice_session`, `append_session_message`, `switch_provider_profile`, `send_message`, `retry_last_message`, and `all`. The flag can be repeated or passed as a comma-separated list. If no `--allow-tool` is provided, all current guarded write tools remain available in write mode; network write tools still require `--allow-network`.

## Next Slices

- Validate model-backed `send_message` against real OpenAI-compatible and Ollama endpoints. Local mock endpoint validation is already automated.
- Validate model-backed `session coach-report` and agent `generate_session_coaching_report` against real OpenAI-compatible and Ollama endpoints. Local mock endpoint validation is already automated.
- Expand failure recovery from retry-last into richer UI/agent guidance if real endpoint validation exposes recurring failures.
- Expand MCP compatibility with resources/prompts only if VoxVerse needs them.
