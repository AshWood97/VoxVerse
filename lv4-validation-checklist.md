# V0.1 Runtime Validation Checklist

> Date: 2026-04-28
> Scope: V0.1 release-candidate runtime validation
> Goal: record real runtime results for provider switching, diagnostics, STT/TTS, CLI network gates, and installer startup.

This checklist is for manual verification after the automated build checks pass. It intentionally avoids storing API keys or secrets.

## Environment

- OS:
- App build:
- Tester:
- Date/time:
- Network:
- OpenAI-compatible endpoint:
- Ollama base URL/version/model:
- Notes:

## Preflight

- [ ] `pnpm build` passes
- [ ] `cargo check` passes
- [ ] `pnpm validate:v0.1` passes and report path is recorded
- [ ] `pnpm validate:v0.1:full` passes and report path is recorded
- [ ] App opens from the generated installer or release executable
- [ ] Settings modal opens without console-visible UI breakage

## Configuration Persistence

| Check | Steps | Expected | Result | Notes |
| --- | --- | --- | --- | --- |
| OpenAI config save | Select OpenAI, enter base URL/model/key, save | Settings remain saved after reopening modal | Pending | |
| Ollama config save | Select Ollama, keep key empty, save | Base URL/model persist and key is optional | Pending | |
| Custom config save | Select Custom, enter endpoint/model, save | Values persist after reopening modal | Pending | |
| Keyring behavior | Save a key, reopen settings | UI says key is saved, key value is not shown | Pending | |
| Profile validation | Try saving an empty base URL or model | Save is blocked with a clear error | Pending | |
| Duplicate profile name | Create or rename to an existing profile name | UI/backend blocks the duplicate | Pending | |
| Rename profile | Rename a non-default profile | Profile list updates and active profile remains usable | Pending | |
| Delete profile | Delete a non-default profile with a saved key | Profile is removed, active profile falls back safely, per-profile key is cleaned | Pending | |

## Runtime Diagnostics

| Provider | Steps | Expected | Result | Copied Report Location |
| --- | --- | --- | --- | --- |
| OpenAI | Save OpenAI settings, run diagnostics, copy report | Chat is healthy; STT ready or credential issue is explicit; TTS catalog check returns a clear result | Pending | |
| Ollama | Save Ollama settings, run diagnostics, copy report | Chat succeeds if Ollama is running; Whisper shows warning; TTS catalog check returns a clear result | Pending | |
| Custom | Save custom compatible endpoint, run diagnostics, copy report | Chat result reflects real endpoint behavior; STT auth uncertainty is explicit when key is absent | Pending | |
| Privacy copy | Copy diagnostics report | Report includes profile/base URL/model/key saved status, but never the API key value | Pending | |

## Speech Output

| Check | Steps | Expected | Result | Notes |
| --- | --- | --- | --- | --- |
| Edge TTS preview | Open Settings, select voice, click Preview Voice | Audio plays and current engine shows Edge TTS | Pending | |
| Browser TTS fallback | Temporarily make Edge TTS unavailable, click Preview Voice | Browser Speech fallback is shown, audio plays if supported | Pending | |
| Chat auto-play | Enable auto-play, send a message | Assistant response is spoken or a visible fallback/error appears | Pending | |
| Manual replay | Click play on an assistant message | Message is spoken or a visible fallback/error appears | Pending | |
| Stop playback | Start playback, click Stop | Audio stops promptly | Pending | |

## Speech Input

| Check | Steps | Expected | Result | Notes |
| --- | --- | --- | --- | --- |
| Whisper STT | Use OpenAI-compatible speech endpoint with key, click mic, speak, stop | Transcript appears in input field | Pending | |
| Browser STT fallback from Ollama | Select Ollama, click mic | Browser STT starts if WebView supports it; input shows Browser STT | Pending | |
| Browser STT fallback from missing key | Select OpenAI without saved key, click mic | Browser STT starts if supported; missing key does not silently fail | Pending | |
| Whisper failure retry | Force Whisper request failure, click mic and stop | App suggests browser STT retry; next mic click uses Browser STT | Pending | |
| Unsupported STT | Test on a WebView without browser STT | App shows a clear unsupported message | Pending | |

## Chat, Feedback, and Sessions

| Check | Steps | Expected | Result | Notes |
| --- | --- | --- | --- | --- |
| Startup without history pollution | Open app, switch characters without sending | No new blank sessions are created automatically | Pending | |
| Lazy first session | Select a character with no history and send first message | A session is created only at send time | Pending | |
| Retry last assistant response | Force chat endpoint failure after a user message is saved, then click Retry last | App retries assistant response without duplicating the user message | Pending | |
| Feedback JSON fallback | Use a test endpoint that returns prose instead of JSON for feedback | Feedback panel shows raw model response instead of failing | Pending | |

## CLI / Agent Real Endpoint

| Check | Steps | Expected | Result | Notes |
| --- | --- | --- | --- | --- |
| CLI send-message real endpoint | Run `pnpm --silent cli -- session send-message --session <id> --content "Hello" --json --yes --allow-network` | User and assistant messages are appended, audit event is written | Pending | |
| CLI retry-last real endpoint | Run `pnpm --silent cli -- session retry-last --session <id> --json --yes --allow-network` | Assistant response is appended without duplicating user message | Pending | |
| CLI coach-report privacy | Run `pnpm --silent cli -- session coach-report --session <id> --json --allow-network` | Markdown coaching report returns; transcript sharing is understood; no DB/audit write occurs | Pending | |

## Release Decision

- [ ] OpenAI chat verified
- [ ] Ollama chat verified
- [ ] Keyring persistence verified
- [ ] Edge TTS verified
- [ ] Browser TTS fallback verified or documented as unavailable in the test environment
- [ ] Whisper STT verified or blocked by documented endpoint limitation
- [ ] Browser STT fallback verified or documented as unavailable in the test environment
- [ ] Diagnostics report copied for each tested provider
- [ ] Provider profile rename/delete verified
- [ ] Chat retry-last UI verified
- [ ] CLI real endpoint send-message, retry-last, and coach-report verified or explicitly skipped with reason
- [ ] Installer startup verified from MSI or NSIS

## Final Notes

- Decision:
- Blockers:
- Follow-up tasks:
