# VoxVerse 路线图与状态总表

> 最后更新: 2026-06-04
> 当前产品阶段: v0.6.0 产品化收口阶段
> 本文件是升级计划的唯一事实源

## 状态说明

- `已完成`: 功能有代码落地，且已经被纳入正式产品阶段
- `已实现，待稳定`: 主要代码已存在，但仍需构建修复、联调或端到端验证
- `待人工验收`: 构建和自动化检查已恢复为绿色，但仍需要真实外部服务和桌面交互验证
- `原型阶段`: 已经出现局部实现或 UI 原型，但还不应算作完整里程碑
- `部分产品化`: 原型或切片已进入主流程，但仍有外部端点、人工验收或后续平台化边界
- `计划中`: 还没有进入当前交付范围

## 里程碑总览

| Level | 名称 | 当前状态 | 说明 |
| --- | --- | --- | --- |
| Lv.0 | 基础对话 | 已完成 | Tauri + Vue 基础桌面聊天链路、流式 LLM、设置面板已经落地 |
| Lv.1 | 语音交互 | 已完成 | 语音输入/输出交互已接入，且后续已被更高阶语音方案部分替换 |
| Lv.2 | 角色系统 | 已完成 | 角色 SQLite 持久化、创建编辑、导入导出、预设角色已落地 |
| Lv.3 | 学习反馈系统 | 已完成 | 会话历史、反馈侧栏、场景、生词本、总结报告已经进入主流程 |
| Lv.4 | 高级语音引擎 | 待人工验收 | Keyring、Whisper STT、Edge TTS、Ollama 兼容、运行诊断和语音回退已落地，仍需真实端点验收 |
| Lv.5 | 完整平台化能力 | 部分产品化 | 统计、成就、i18n、本地角色示例、结构化反馈元数据和记忆管理已进入 v0.6 收口；真实市场、自动更新、Realtime Voice 仍不在本阶段 |

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

### v0.6.0 产品化收口

当前目标是按 `升级计划v0.6.md` 把已有 Lv.5 原型和切片收束为可验证的产品化版本。范围包括结构化反馈元数据、角色记忆清除、角色学习目标、本地优先隐私说明、统计和成就 i18n、本地示例角色导入、版本号和发布文档统一。

注意: v0.6.0 是工程发布版本号；本文件早期保留的 “V0.3 baseline” 语言描述的是产品路线图基线，不代表 npm/Cargo/Tauri 版本号。

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

std::io agent server、受控 profile switch、受控 session start、受控 message append、结构化 session export、filtered audit inspection、agent 写工具 allowlist、网络闸门版 `send_message`、`retry-last` 恢复路径和模型化 `coach-report` 已落地：`pnpm --silent cli -- status --json`、`pnpm --silent cli -- config profiles list/switch --json`、`pnpm --silent cli -- character list --json`、`pnpm --silent cli -- session list/start/append-message/send-message/retry-last/coach-report/export --json`、`pnpm --silent cli -- audit list/export --json`、`pnpm --silent cli -- diagnostics run --json`、`pnpm --silent cli -- agent tools list --json`、`pnpm --silent cli -- agent serve --stdio --read-only`、`pnpm --silent cli -- agent serve --stdio --allow-writes --yes` 可读取或受控切换/创建/追加本地 SQLite 状态；`session export` 已返回 ordered messages、summary counts 和 Markdown transcript，`session coach-report` 可基于本地 export 调用 active profile 生成 Markdown 学习建议；`audit list/export` 支持按 operation、actor、result 和 target id 过滤；`--allow-tool <tool>` 可以收窄 agent 写模式暴露的工具，`send_message`、`retry_last_message` 和 `generate_session_coaching_report` 还必须显式 `--allow-network`。`config profiles switch`、`session start`、`session append-message`、agent `switch_provider_profile`、agent `start_practice_session`、agent `append_session_message`、网络 send-message 和 retry-last 都会写入 `voxverse-audit.jsonl`；旧 `speakmate-audit.jsonl` 仍可读取。coach-report 是网络只读路径，不写 DB 或 audit。Agent 只读工具已包含 `list_characters`、可过滤的 `list_audit_events`，以及 `--allow-network` 下的 `generate_session_coaching_report`，写模式可基于该角色目录创建练习会话、追加本地 user/assistant 消息，并在显式联网授权后调用 active profile 或重试最后一条 user 消息；本地 mock OpenAI-compatible send-message、retry-last 与 coaching report 已纳入自动烟测。

## v0.6 已产品化与仍属原型的 Lv.5 功能

这些能力已从原型进入 v0.6 产品化收口：

- 学习统计面板
- 成就提示和学习进度反馈
- 中英文切换和 i18n 基础设施
- 结构化反馈元数据
- 当前角色记忆清除和角色学习目标
- 本地示例角色发现/导入

这些能力仍属于后续原型或暂缓范围：

- 真实角色市场后端、远程分发、账号体系和下载统计
- 自动更新、托盘、全局快捷键、浮动输入条
- Realtime/WebRTC/LiveKit 连续语音

Lv.5 功能的处理原则：

1. 如果功能能通过构建并完成端到端验证，就提升到正式里程碑
2. 如果只是局部 UI 或半成品逻辑，就继续保留在原型阶段，不写成“已完成”

## 当前阻塞项

截至 2026-04-28，当前最直接的交付阻塞项如下：

1. 本机未检测到 `ollama` 命令或可用 Ollama HTTP 服务，Ollama 真实端点验收仍需在对应环境完成
2. Whisper STT 需要真实 OpenAI-compatible speech endpoint 和有效 key 才能完成端到端验收
3. Browser STT 可用性依赖 WebView/OS 能力，仍需人工交互验证

## 近期执行顺序

1. 按 `升级计划v0.6.md` 完成产品化收口和文档统一
2. 运行 `pnpm build`、`cargo test --manifest-path src-tauri/Cargo.toml`、`pnpm validate:lv4`
3. 按 `lv4-validation-checklist.md` 记录真实端点和桌面交互结论
4. 对未完成人工验收的语音/provider 能力保持 `待人工验收`
5. v0.6 发布后再评估自动更新、真实角色市场和 Realtime Voice

## 长期 Backlog

以下仍然属于后续规划，不计入当前已完成里程碑：

- SillyTavern V2 角色卡导入
- 角色市场后端化和真实分发链路
- 更完整的学习数据统计与可视化
- 亮色主题
- 成就系统与打卡激励的产品化
- Tauri 自动更新
- 原生 CLI 与 agent 控制面的写操作、安全审计和 UI 自动化桥接
