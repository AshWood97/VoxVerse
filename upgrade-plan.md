# SpeakMate 后续升级计划

> 更新时间: 2026-04-28
> 范围: Lv.4 验收后的产品化与平台化升级
> 输入依据: `task.md`、`github-reference-projects.md`、当前代码结构

本计划把后续升级拆成可交付类别。原则是先把现有语音和配置链路验收稳定，再把 SpeakMate 从“聊天工具”推进为“可持续练习的语言学习产品”。

## V0.1 冻结边界

当前项目进入 V0.1 Release Candidate 查缺补漏。V0.1 补全范围和人工测试顺序以 `v0.1-gap-closure-plan.md` 为准。V0.1 测试完成前，本计划中的 P1/P2/P3 只作为后续升级池，不再继续展开新功能。

V0.1 测试前允许处理的内容仅限:

1. `pnpm validate:v0.1` 或 `pnpm validate:v0.1:full` 暴露的构建/打包问题
2. 真实 OpenAI-compatible、Whisper、Ollama、TTS/STT fallback 验收阻塞
3. Provider Profiles、Voice Runtime UX、Native CLI/Agent 已落地切片中的主流程缺陷
4. 数据、Keyring、audit、网络闸门相关安全问题
5. 测试文档和已知问题记录错误

## 升级原则

1. 先验收，再升级: Lv.4 真实端点验收没有完成前，不引入 Realtime/WebRTC/LiveKit 这类高调试成本能力。
2. 小包交付: 每个升级包都必须有代码范围、验收标准、构建验证和构建日志。
3. 产品优先: 先做练习模式、反馈闭环、语音状态，而不是复制大型平台项目的管理后台。
4. 本地优先: Ollama、Keyring、本地数据库、离线数据导出应成为清晰产品卖点。
5. 可回退: 新 provider、新语音链路、新桌面能力都必须保留明确 fallback 或禁用路径。

## 总体优先级

| 优先级 | 升级类别 | 目标状态 | 主要价值 |
| --- | --- | --- | --- |
| P0 | Lv.4 验收收口 | `待人工验收` -> `已完成` | 让当前高级语音引擎成为可信基线 |
| P1 | Provider Profiles | 已开始 | 多端点/多模型配置可保存、切换、诊断 |
| P1 | Practice Modes | 正式功能 | 把聊天转成明确练习场景 |
| P1 | Voice Runtime UX | 已开始 | 让录音、转写、思考、播报、回退状态可见 |
| P1 | Structured Feedback | 正式功能 | 把学习反馈做成每轮闭环 |
| P2 | Character Memory | 正式功能 | 角色具备长期上下文和学习目标 |
| P2 | Local-first & Privacy | 正式能力 | 把本地优先、安全凭据、数据导出包装清楚 |
| P2 | Native CLI & Agent Control | 新增计划 | 让脚本和 agent 安全操控配置、诊断、会话和练习流程 |
| P2 | Desktop Native Layer | 正式能力 | 全局快捷键、托盘、浮窗、自动更新 |
| P3 | Stats & Achievements Productization | 正式功能 | 让已有统计和成就从原型进入主流程 |
| P3 | Character Marketplace | 暂缓/实验 | 角色分发、导入校验、市场后端化 |
| P3 | Realtime Voice | 暂缓/研究 | 低延迟连续语音，等 Lv.4/P1 稳定后再评估 |

## P0: Lv.4 验收收口

### 目标

把高级语音引擎从“自动化绿色”推进到“真实环境可用”。

### 范围

- OpenAI-compatible 聊天、Keyring、Whisper STT
- Ollama chat compatibility
- Edge TTS、Browser TTS fallback
- Browser STT fallback
- Settings diagnostics 和复制报告

### 验收标准

- `pnpm validate:lv4:full` 通过
- `lv4-validation-checklist.md` 填完真实端点结论
- 每个 provider 至少有一份诊断报告记录
- `task.md` 中 Lv.4 状态可提升为 `已完成`

## P1-A: Provider Profiles

### 目标

把当前单套设置升级成多套 provider profile，让用户可以保存和切换多个 OpenAI-compatible、Ollama、Custom 配置。

### 代码范围

- Rust: `src-tauri/src/state/mod.rs`
- Rust: `src-tauri/src/repositories/config.rs`
- Rust: `src-tauri/src/commands/chat.rs`
- Frontend: `src/components/domain/SettingsModal.vue`
- Frontend: `src/services/tauri/chat.ts`
- Types: `src/types/api.ts`

### 功能切片

1. Profile 数据模型: id、name、provider、base_url、model、key_saved、supports_stt、created_at、updated_at。
2. Profile CRUD: 创建、复制、重命名、删除、设为默认。
3. Keyring 命名隔离: 每个 profile 的 API key 独立保存，不互相覆盖。
4. 诊断绑定 profile: diagnostics 报告注明当前 profile。
5. 安全迁移: 现有单套配置自动迁移为 `Default` profile。

### 验收标准

- 可以保存至少两个 OpenAI-compatible profiles 并切换聊天
- Ollama profile 不要求 API key
- 删除 profile 不会泄漏或展示 key 明文
- 运行 `pnpm build`、`cargo check`、`pnpm validate:lv4`

### 已落地切片

- 已新增 `provider_profiles` SQLite 表，并从旧 `app_config` 自动生成 `Default` profile
- 运行时配置已带上 `profile_id` 和 `profile_name`
- API key 已改为按 profile 使用独立 Keyring 槽位，默认 profile 仍兼容旧 `api-key`
- Settings 已支持 profile 列表、切换和另存为新 profile
- 已新增 `list_config_profiles`、`switch_config_profile`、`create_config_profile` Tauri commands
- `pnpm validate:lv4` 已通过并生成 smoke report

## P1-B: Practice Modes

### 目标

把 SpeakMate 从普通聊天窗口升级成有明确学习任务的练习产品。

### 第一批模式

- Free Talk: 当前自由聊天能力的正式化入口
- Roleplay: 结合现有角色系统做情境对话
- Scenario Drill: 使用现有 `ScenarioSelector` 进行目标场景练习
- IELTS Speaking: 固定 Part 1/2/3 结构、计时、评分建议
- Interview Practice: 面试场景、追问、表达建议

### 代码范围

- Frontend: `src/components/domain/ScenarioSelector.vue`
- Frontend: `src/components/domain/ChatPanel.vue`
- Frontend: `src/composables/useChat.ts`
- Frontend: `src/composables/useScenario.ts`
- Rust: `src-tauri/src/commands/scenario.rs`
- Rust: `src-tauri/src/services/feedback.rs`

### 验收标准

- 用户可选择练习模式并看到模式说明
- 每个模式能生成不同 system prompt 或 feedback rubric
- 模式信息保存进 session
- 至少 Free Talk、Roleplay、Scenario Drill 完成端到端验证

## P1-C: Voice Runtime UX

### 目标

让语音链路变得透明，用户能看懂当前处在录音、转写、LLM、播报、回退还是错误状态。

### 状态模型

- idle
- recording
- transcribing
- thinking
- speaking
- fallback
- error

### 代码范围

- Frontend: `src/composables/useSpeechRecognition.ts`
- Frontend: `src/composables/useSpeechSynthesis.ts`
- Frontend: `src/composables/useChat.ts`
- Frontend: `src/components/base/MessageInput.vue`
- Frontend: `src/components/domain/ChatPanel.vue`
- Frontend: `src/components/domain/SettingsModal.vue`

### 验收标准

- Whisper 与 Browser STT 的切换状态可见
- Edge TTS 与 Browser TTS fallback 状态可见
- 失败时给出下一步建议，而不是只显示异常字符串
- `pnpm build` 通过

### 已落地切片

- `src/types/voice.ts` 已定义 STT/TTS 运行态类型
- `MessageInput.vue` 已显示 recording、transcribing、fallback、error
- `ChatPanel.vue` 已显示 thinking、speaking、fallback、error
- 运行态 UI 文案已接入 `en.json` / `zh.json`
- 运行态提示已抽成 `RuntimeStatusBadge.vue`，可复用于后续 provider/profile/mode 状态
- Browser TTS fallback 的 header 状态已调整为“运行中可见，结束后保留 notice”
- `pnpm validate:lv4` 已通过并生成 smoke report

## P1-D: Structured Feedback

### 目标

把反馈从“事后面板”升级成“每轮学习闭环”。

### 输出结构

- Natural reply
- Correction
- Better expression
- Vocabulary
- Pronunciation note
- Score or confidence
- Next prompt suggestion

### 代码范围

- Frontend: `src/components/domain/FeedbackPanel.vue`
- Frontend: `src/components/base\DiffView.vue`
- Frontend: `src/composables/useFeedback.ts`
- Rust: `src-tauri/src/commands/feedback.rs`
- Rust: `src-tauri/src/services/feedback.rs`

### 验收标准

- 用户每轮对话后可以看到结构化反馈
- 反馈能写入现有 corrections/vocabulary/session 数据
- 总结报告能引用这些结构化数据

## P2-A: Character Memory

### 目标

让角色从静态 prompt 升级为有长期上下文边界的学习伙伴。

### 范围

- 角色长期记忆摘要
- 用户学习目标
- 角色偏好和禁用主题
- 会话摘要写回
- 手动清除/导出记忆

### 验收标准

- 角色切换时记忆不会混用
- 用户能查看和清除角色记忆
- 记忆进入 prompt 前有长度限制和可解释来源

## P2-B: Local-first & Privacy

### 目标

把已有 Keyring、本地 SQLite、Ollama 兼容包装成清晰的本地优先能力。

### 范围

- README 和设置页隐私说明
- 本地数据导出/清除
- API key 存储位置说明
- Ollama/local profile 引导
- 诊断报告脱敏规则

### 验收标准

- 用户能理解哪些数据在本地，哪些会发给外部 endpoint
- 导出文件不包含 API key
- 设置页能一键清除敏感配置

## P2-C: Native CLI & Agent Control

### 目标

为 SpeakMate 增加原生 CLI 和 agent 控制面，让外部脚本或 AI agent 可以安全、可审计地操作应用能力。

### 当前判断

项目已新增原生 CLI、JSONL/MCP stdio agent 控制层、受控 profile switch、受控 session start、受控 message append、结构化 session export、filtered audit inspection、agent 写工具 allowlist、网络闸门版 `send_message`、`retry-last` 恢复路径和模型化 `coach-report`: Rust binary `speakmate`。它支持 `status --json`、`config profiles list/switch --json`、`character list --json`、`session list/start/append-message/send-message/retry-last/coach-report/export --json`、`audit list/export --json`、`diagnostics run --json`、`agent tools list --json`、`agent serve --stdio --read-only [--allow-network]` 和 `agent serve --stdio --allow-writes --yes [--allow-network] [--allow-tool <tool>]`。查询和默认 agent server 仍保持只读；`session export` 会返回 ordered messages、summary counts 和 Markdown transcript，`session coach-report` 会基于本地 export 调用 active profile 生成 Markdown 学习建议且不写 DB/audit；`audit list/export` 支持按 operation、actor、result 和 target id 过滤；profile switch、session start、message append、send-message 与 retry-last 必须显式 `--yes`，并会写入 `speakmate-audit.jsonl`。agent 写模式目前暴露 `start_practice_session`、`append_session_message` 与 `switch_provider_profile`，并可通过 `--allow-tool` 收窄；`send_message`、`retry_last_message` 和网络只读 `generate_session_coaching_report` 还必须显式开启 `--allow-network`。

### 范围

- `speakmate status --json`（已完成只读切片）
- `speakmate diagnostics run --json`（已完成只读切片）
- `speakmate config profiles list/switch --json`（已完成；switch 必须 `--yes` 并写 audit log）
- `speakmate character list --json`（已完成只读切片；供 agent 选择练习角色）
- `speakmate session list/start/append-message/send-message/retry-last/coach-report/export --json`（list/export 只读；export 返回 summary 和 Markdown transcript；coach-report 网络只读且不写 DB/audit；start/append 必须 `--yes` 并写 audit log；send-message/retry-last 还必须 `--allow-network`）
- `speakmate audit list/export --json`（已完成只读切片，支持 operation/actor/result/target id 过滤）
- `speakmate agent serve --stdio --read-only [--allow-network]`（已完成 JSONL 与 MCP-style JSON-RPC 2.0 只读切片；`--allow-network` 下额外暴露网络只读 coaching report）
- `speakmate agent serve --stdio --allow-writes --yes [--allow-network] [--allow-tool <tool>]`（已完成受控 `start_practice_session`、`append_session_message`、`switch_provider_profile`、网络闸门版 `send_message` 与 `retry_last_message`，并支持精确 allowlist）
- 后续继续做真实端点验收、报告质量优化和 UI/agent guidance 对接

### 借鉴

详见 `agent-cli-reference-projects.md`。关键参考包括 Commander、GitHub MCP Server、tauri-pilot、OpenHands、Aider、Open Interpreter 和 Agent-S。

### 验收标准

- 所有 CLI 输出支持稳定 JSON
- 默认不暴露 API key 明文
- 写操作有 `--yes` 或确认机制
- agent server 支持只读模式
- agent 操作有 audit log
- 至少 diagnostics/profile list/profile switch 能被自动化脚本调用

### 已落地切片

- 新增 `src-tauri/src/bin/speakmate.rs` 独立 CLI binary
- 新增 `pnpm cli -- ...`、`pnpm cli:status`、`pnpm cli:diagnostics`
- 已支持 `character list --json` 和 agent `list_characters`
- 已支持 `session list --json`、`session start --character <id> --json --yes`、`session append-message --session <id> --role <role> --content <text> --json --yes`、`session send-message --session <id> --content <text> --json --yes --allow-network`、`session retry-last --session <id> --json --yes --allow-network`、`session coach-report --session <id> --json --allow-network` 与带 summary/Markdown transcript 的 `session export --session <id> --json`
- 已支持 `agent tools list --json` 与 JSONL `agent serve --stdio --read-only`
- 已支持 `config profiles switch <id> --json --yes`，并写入 `speakmate-audit.jsonl`
- 已支持可过滤的 `audit list/export --json` 和 agent `list_audit_events`
- 已支持 MCP-style `initialize`、`notifications/initialized`、`tools/list`、`tools/call`
- 已支持 agent `start_practice_session`、`append_session_message` 与 `switch_provider_profile`，仅在 `--allow-writes --yes` 模式出现并可调用
- 已支持 `--allow-tool <tool>` 对 agent 写工具做精确授权，烟测覆盖被 allowlist 禁用的工具拒绝路径
- 已支持 agent `send_message` 和 `retry_last_message` 的网络闸门：只有 `--allow-writes --yes --allow-network` 且 allowlist 允许时才暴露；已支持 agent `generate_session_coaching_report` 网络只读工具，可在 `--read-only --allow-network` 下调用；自动烟测覆盖未开网络时的拒绝路径、本地 mock OpenAI-compatible 端到端返回、retry-last 恢复路径和 coaching report 生成，真实模型返回仍需人工端点验收
- `pnpm validate:lv4` 已接入 `speakmate cli diagnostics`、character list、session list、structured session export、agent tools、legacy agent stdio、MCP stdio、临时 DB profile switch、filtered audit log、retry-last recovery、coaching report 和 agent write guard smoke
- 新增 `native-cli.md` 记录命令、数据库发现顺序和安全边界

## P2-D: Desktop Native Layer

### 目标

补齐桌面应用应有的原生体验。

### 范围

- 系统托盘
- 全局快捷键
- Push-to-talk
- 浮动输入条
- 自动更新

### 前置条件

- Lv.4 验收完成
- Voice Runtime UX 完成
- Provider Profiles 完成
- Native CLI 的只读诊断能力完成

### 验收标准

- 所有桌面能力都可在设置里关闭
- 快捷键冲突有提示
- 自动更新具备签名和回滚策略后再启用

## P3: 暂缓和研究类

### Character Marketplace

先完成角色导入、导出、校验、本地预览，再考虑市场后端化。

### Realtime Voice

先研究 LiveKit、OpenAI Realtime、WebRTC 方案，但不进入主线实现，除非 Voice Runtime UX 已经稳定。

### Platform Admin

不复制 Open WebUI 的管理后台、多租户、插件市场。SpeakMate 当前核心仍是个人语言练习。

## 建议实施顺序

1. P0: 完成 Lv.4 人工验收并更新 `task.md`
2. P1-C: 做 Voice Runtime UX，降低后续语音升级调试成本
3. P1-A: 做 Provider Profiles，稳定多端点配置基础
4. P1-B: 做 Practice Modes，让产品形态变清楚
5. P1-D: 做 Structured Feedback，形成学习闭环
6. P2-A/P2-B: 做 Character Memory 和 Local-first & Privacy
7. P2-C: 做 Native CLI & Agent Control 的只读切片
8. P2-D: 做桌面原生能力
9. P3: 再评估市场、Realtime、平台化

## 第一批升级包定义

第一批只建议进入三个包:

1. Voice Runtime UX
2. Provider Profiles
3. Practice Modes

这三个包直接复用当前 Lv.4 和 Lv.3 基础，收益高、风险可控，也最接近 GitHub 调研中值得借鉴的方向。
