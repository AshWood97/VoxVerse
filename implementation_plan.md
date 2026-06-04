# VoxVerse 当前实施计划

> 更新时间: 2026-06-04
> 范围: V0.3 升级收口 + 人工测试准备
> 对齐文件: `task.md`、`upgrade-plan.md`

当前实施计划按 `升级计划v0.3.md` 收口。`pnpm build`、`cargo test` 和 Lv.4 自动化 smoke test 是可信基线；接下来的工作重点是完成真实环境验收、保持命名/迁移兼容，并持续补齐练习模式、结构化反馈、隐私控制和性能验证。

## 当前目标

把项目从“持续升级中”收束为“可安装、可测试、可记录问题的 v0.3 baseline”。

## V0.3 收口规则

- v0.3 范围以 `升级计划v0.3.md` 为准。
- 保持构建、Rust 测试、CLI smoke 和核心 UI 路径可验证。
- 真实端点、语音 fallback、数据迁移和隐私设置都要有人工验收记录。

## 工作流 1: Lv.4 人工验收

### 任务

- 按 `lv4-validation-checklist.md` 验证 OpenAI-compatible 聊天、Keyring 和 Whisper STT
- 在安装 Ollama 的环境中验证 Ollama chat compatibility
- 验证 Edge TTS、Browser TTS fallback、Browser STT fallback
- 保存 Settings diagnostics 复制报告
- 通过后把 `task.md` 中 Lv.4 状态提升为 `已完成`

### 完成标准

- `pnpm validate:lv4:full` 通过并保留报告
- checklist 中关键链路有人工结论
- 阻塞项只剩明确外部环境限制，且已记录

## 工作流 2: 第一批 Lv.5 升级包

### Voice Runtime UX

先把语音状态可视化，降低后续语音功能调试成本。第一切片已经接入 recording、transcribing、thinking、speaking、fallback、error 的可见提示，完成中英文运行态文案，并抽出 `RuntimeStatusBadge.vue` 作为复用组件；下一步是在真实桌面交互中验收这些状态。

### Provider Profiles

把单套 provider 设置升级成多套 profile，支持 OpenAI-compatible、Ollama、Custom 端点保存、切换、诊断和 Keyring 隔离。第一切片已经完成表结构、默认迁移、profile 列表/切换/另存为新 profile；删除和重命名后续补齐。

### Practice Modes

把现有聊天、角色、场景、反馈能力组合为 Free Talk、Roleplay、Scenario Drill、IELTS Speaking、Interview Practice 等练习入口。

### Native CLI & Agent Control

第一版 CLI、JSONL/MCP stdio agent server、受控 profile switch、受控 session start、受控 message append、结构化 session export、filtered audit inspection、agent 写工具 allowlist、网络闸门版 `send_message`、`retry-last` 恢复路径和模型化 `coach-report` 已完成，当前可通过 `pnpm --silent cli -- diagnostics run --json`、`pnpm --silent cli -- status --json`、`pnpm --silent cli -- config profiles list --json`、`pnpm --silent cli -- config profiles switch <id> --json --yes`、`pnpm --silent cli -- character list --json`、`pnpm --silent cli -- session list/start/append-message/send-message/retry-last/coach-report/export --json`、`pnpm --silent cli -- audit list/export --json` 查询或切换/创建/追加本地状态；`session export` 会返回 ordered messages、summary counts 和 Markdown transcript，`session coach-report` 可基于本地 export 调用 active profile 生成 Markdown 学习建议且不写 DB/audit，`audit list/export` 可按 operation、actor、result 和 target id 过滤。agent 可用 `pnpm --silent cli -- agent serve --stdio --read-only` 调用只读工具（包含 `list_characters`、结构化 `export_session` 与可过滤 `list_audit_events`），也可用 `pnpm --silent cli -- agent serve --stdio --read-only --allow-network` 启用网络只读 `generate_session_coaching_report`，或用 `pnpm --silent cli -- agent serve --stdio --allow-writes --yes [--allow-network] [--allow-tool <tool>]` 启用受控 `start_practice_session`、`append_session_message`、`switch_provider_profile`、`send_message` 与 `retry_last_message`，并按工具收窄写权限。下一步是对 `send_message`、`retry-last` 和 `coach-report` 做真实 OpenAI-compatible/Ollama 端点验收，再优化学习报告质量和更完整的失败恢复提示。

## 工作流 3: 第二批产品化能力

- Structured Feedback: 每轮对话输出自然回复、纠错、替代表达、生词、总分、分项评分和下一步建议
- Character Memory: 为角色增加长期记忆摘要、学习目标、记忆查看和清除
- Local-first & Privacy: 明确本地数据、Keyring、安全配置、导出和清除

## 工作流 4: 暂缓能力

- Desktop Native Layer: 全局快捷键、托盘、浮动输入条、自动更新
- Character Marketplace: 等角色导入导出和本地校验稳定后再做
- Realtime Voice: 等 Lv.4 和 Voice Runtime UX 稳定后再研究

## 每个升级包的交付要求

- 更新 `task.md` 状态
- 新增或更新构建日志
- 保持 `pnpm build` 和 `cargo check` 通过
- 涉及桌面打包时运行 `pnpm validate:lv4:full`
- 涉及语音/provider 时更新 `lv4-validation-checklist.md` 或新增对应验收清单

## 当前结论

下一步最稳的工程顺序是: 按 `升级计划v0.3.md` 完成收口 -> `pnpm build` -> `cargo test` -> `pnpm validate:lv4` -> 人工测试真实端点和语音 fallback -> 只修 v0.3 清单内阻塞问题。
