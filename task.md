# SpeakMate 路线图与状态总表

> 最后更新: 2026-04-28
> 当前产品阶段: V0.1 Release Candidate 查缺补漏阶段
> 本文件是升级计划的唯一事实源

## 状态说明

- `已完成`: 功能有代码落地，且已经被纳入正式产品阶段
- `已实现，待稳定`: 主要代码已存在，但仍需构建修复、联调或端到端验证
- `待人工验收`: 构建和自动化检查已恢复为绿色，但仍需要真实外部服务和桌面交互验证
- `原型阶段`: 已经出现局部实现或 UI 原型，但还不应算作完整里程碑
- `计划中`: 还没有进入当前交付范围

## 里程碑总览

| Level | 名称 | 当前状态 | 说明 |
| --- | --- | --- | --- |
| Lv.0 | 基础对话 | 已完成 | Tauri + Vue 基础桌面聊天链路、流式 LLM、设置面板已经落地 |
| Lv.1 | 语音交互 | 已完成 | 语音输入/输出交互已接入，且后续已被更高阶语音方案部分替换 |
| Lv.2 | 角色系统 | 已完成 | 角色 SQLite 持久化、创建编辑、导入导出、预设角色已落地 |
| Lv.3 | 学习反馈系统 | 已完成 | 会话历史、反馈侧栏、场景、生词本、总结报告已经进入主流程 |
| Lv.4 | 高级语音引擎 | 待人工验收 | Keyring、Whisper STT、Edge TTS、Ollama 兼容、运行诊断和语音回退已落地，仍需真实端点验收 |
| Lv.5 | 完整平台化能力 | 原型阶段 | 统计、成就、角色发现、多语言等已出现局部实现，但不应视为里程碑完成 |

## 已完成里程碑

### Lv.0 基础对话

- Tauri 2 + Vue 3 桌面应用骨架
- OpenAI-compatible 流式聊天链路
- `SettingsModal` 配置入口
- Rust `send_message` command 与前端 Channel 流式消费

### Lv.1 语音交互

- 语音录入与语音播报体验已进入产品
- `MessageInput` 麦克风入口、录音状态反馈、历史消息回放能力已接入
- 后续语音实现已从纯 Web Speech 逐步演进为 Tauri 后端主导方案

### Lv.2 角色系统

- SQLite `characters` 表与 Rust CRUD
- `useCharacter.ts` 角色管理组合式逻辑
- `SideBar.vue` 角色切换和操作入口
- `CharacterModal.vue` 创建/编辑能力
- JSON 导入导出与预设角色初始化

### Lv.3 学习反馈系统

- `chat_sessions` / `messages` / `vocabulary` / `corrections` / `scenarios` 数据模型
- `SessionHistory.vue` 历史会话视图
- `FeedbackPanel.vue`、`DiffView.vue`、反馈命令与总结报告
- 场景系统、生词本、学习报告生成

## 当前交付重点

### V0.1 Release Candidate

当前目标是把现有能力收束为 V0.1 RC，而不是继续扩展 Lv.5。V0.1 的查缺补漏列表、停止边界和人工测试顺序详见 `v0.1-gap-closure-plan.md`。进入 V0.1 补全阶段后，只做该清单内事项；Practice Modes、Character Memory、统计/成就、Marketplace、Realtime Voice、自动更新等新功能全部延后到 V0.2 或更后版本。

### Lv.4 高级语音引擎

当前判断是“主干实现和自动化预检已经存在，但还没有完成真实外部端点的人工验收”。

截至 2026-04-26，`pnpm validate:lv4:full` 已经通过，Tauri identifier 警告也已清除；Lv.4 剩余风险集中在真实外部服务和桌面交互验收。

已经落地的内容：

- 使用系统 Keyring 保存 API Key，而不是继续把密钥放进 SQLite
- Rust 侧 STT command 和 Whisper 请求链路
- Rust 侧 TTS command、Edge TTS voice 列表和音频合成链路
- 前端录音逻辑已经切换到 `MediaRecorder` + Tauri IPC
- Ollama 兼容逻辑已经进入设置和后端调用路径
- 设置页已经提供 provider-aware 配置、运行诊断、TTS preview 和诊断报告复制
- Edge TTS 失败时可以回退到浏览器 `speechSynthesis`
- Whisper STT 不可用或失败时可以尝试浏览器 `SpeechRecognition`
- `lv4-validation-checklist.md`、`pnpm validate:lv4` 和 `pnpm validate:lv4:full` 已经用于发布前验证留痕

仍需完成的收口工作：

1. 使用真实 OpenAI-compatible endpoint 验证聊天、Keyring 和 Whisper STT
2. 在安装 Ollama 的机器上验证 Ollama chat compatibility
3. 在目标桌面环境中验证 Edge TTS、Browser TTS fallback、Browser STT fallback
4. 将诊断报告复制结果和人工结论填入 `lv4-validation-checklist.md`

### Lv.5 升级借鉴

`github-reference-projects.md` 已记录同类 GitHub 项目的调研结果，`upgrade-plan.md` 已把后续升级拆成可交付类别。当前结论是：Lv.5 不应直接复制大型平台项目，而应优先从 Voice Runtime UX、Provider Profiles、Practice Modes 三个升级包开始。

Voice Runtime UX 已完成第一切片：输入栏可显示录音、转写、STT fallback 和错误状态；聊天头部可显示 thinking、speaking、TTS fallback 和错误状态；相关运行态文案已接入中英文 i18n，并抽出可复用的 `RuntimeStatusBadge.vue`。后续仍需在真实桌面语音链路中人工验收。

Provider Profiles 已完成第一切片：新增 `provider_profiles` 表、默认配置迁移、按 profile 隔离的 Keyring 槽位、Settings profile 列表/切换/另存为新 profile，以及对应 Tauri commands。删除、重命名和更完整的 profile 状态提示后续补齐。

Native CLI & Agent Control 已新增为 Lv.5 后续升级类别。原生 CLI、JSONL/MCP

## Verification & Release Build

- [x] cargo check passes
- [x] cargo clippy -D warnings passes
- [x] cargo test passes (including new tests)
- [x] pnpm build passes
- [x] Application starts with VoxVerse branding
- [x] Double-click launcher (`LaunchVoxVerse.exe` & `LaunchVoxVerse.bat`) prioritizes release build
- [x] Port checking diagnostics added to launcher debug fallback
- [x] Production release installers (MSI, NSIS Setup) generated successfully

std::io agent server、受控 profile switch、受控 session start、受控 message append、结构化 session export、filtered audit inspection、agent 写工具 allowlist、网络闸门版 `send_message`、`retry-last` 恢复路径和模型化 `coach-report` 已落地：`speakmate status --json`、`speakmate config profiles list/switch --json`、`speakmate character list --json`、`speakmate session list/start/append-message/send-message/retry-last/coach-report/export --json`、`speakmate audit list/export --json`、`speakmate diagnostics run --json`、`speakmate agent tools list --json`、`speakmate agent serve --stdio --read-only`、`speakmate agent serve --stdio --allow-writes --yes` 可读取或受控切换/创建/追加本地 SQLite 状态；`session export` 已返回 ordered messages、summary counts 和 Markdown transcript，`session coach-report` 可基于本地 export 调用 active profile 生成 Markdown 学习建议；`audit list/export` 支持按 operation、actor、result 和 target id 过滤；`--allow-tool <tool>` 可以收窄 agent 写模式暴露的工具，`send_message`、`retry_last_message` 和 `generate_session_coaching_report` 还必须显式 `--allow-network`。`config profiles switch`、`session start`、`session append-message`、agent `switch_provider_profile`、agent `start_practice_session`、agent `append_session_message`、网络 send-message 和 retry-last 都会写入 `speakmate-audit.jsonl`；coach-report 是网络只读路径，不写 DB 或 audit。Agent 只读工具已包含 `list_characters`、可过滤的 `list_audit_events`，以及 `--allow-network` 下的 `generate_session_coaching_report`，写模式可基于该角色目录创建练习会话、追加本地 user/assistant 消息，并在显式联网授权后调用 active profile 或重试最后一条 user 消息；本地 mock OpenAI-compatible send-message、retry-last 与 coaching report 已纳入自动烟测。

## 原型阶段功能

这些能力已经在代码里出现，但目前归类为 Lv.5 原型，而不是正式完成：

- 学习统计面板
- 成就提示和学习进度反馈
- 角色发现/角色市场 UI
- 中英文切换和 i18n 基础设施

这些功能的处理原则：

1. 如果功能能通过构建并完成端到端验证，就提升到正式里程碑
2. 如果只是局部 UI 或半成品逻辑，就继续保留在原型阶段，不写成“已完成”

## 当前阻塞项

截至 2026-04-28，当前最直接的交付阻塞项如下：

1. 本机未检测到 `ollama` 命令或可用 Ollama HTTP 服务，Ollama 真实端点验收仍需在对应环境完成
2. Whisper STT 需要真实 OpenAI-compatible speech endpoint 和有效 key 才能完成端到端验收
3. Browser STT 可用性依赖 WebView/OS 能力，仍需人工交互验证

## 近期执行顺序

1. 运行 `pnpm validate:v0.1`，必要时运行 `pnpm validate:v0.1:full` 生成 V0.1 自动化基线
2. 按 `v0.1-gap-closure-plan.md` 的查缺补漏清单逐项修复
3. 按 `lv4-validation-checklist.md` 记录真实端点和桌面交互结论
4. 只修 V0.1 清单内问题，不再新增功能
5. 人工测试完成后再决定是否进入 V0.2

## 长期 Backlog

以下仍然属于后续规划，不计入当前已完成里程碑：

- SillyTavern V2 角色卡导入
- 角色市场后端化和真实分发链路
- 更完整的学习数据统计与可视化
- 亮色主题
- 成就系统与打卡激励的产品化
- Tauri 自动更新
- 原生 CLI 与 agent 控制面的写操作、安全审计和 UI 自动化桥接
