# 原生 CLI 与 Agent Control Layer 调研

> 调研时间: 2026-04-27
> 目的: 判断 SpeakMate 是否需要补充原生 CLI 与 agent 全程操控能力
> 结论: 调研时项目没有正式原生 CLI，也没有 MCP/JSON-RPC/stdio agent 控制面；截至 2026-04-27 已完成 `speakmate` CLI、JSONL/MCP stdio agent server、受控 profile switch、agent `switch_provider_profile` 和 audit log 查询/导出。

## 实现进展

截至 2026-04-27，第一阶段已新增独立 Rust binary `speakmate`，当前只读命令包括:

1. `speakmate status --json`
2. `speakmate config profiles list --json`
3. `speakmate config profiles switch <id> --json --yes`
4. `speakmate session list --json`
5. `speakmate session export --session <id> --json`
6. `speakmate audit list/export --json`
7. `speakmate diagnostics run --json`
8. `speakmate agent tools list --json`
9. `speakmate agent serve --stdio --read-only`（支持 legacy JSONL 与 MCP-style JSON-RPC 2.0）
10. `speakmate agent serve --stdio --allow-writes --yes`（仅暴露受控 `switch_provider_profile` 写工具）

当前 CLI 会自动查找新旧 Tauri 数据目录里的 `speakmate.db`，支持 `SPEAKMATE_DB` 与 `--db <path>` 覆盖，输出稳定 JSON，并明确不读取 Keyring / API key。profile switch 必须显式 `--yes`，并追加写入 `speakmate-audit.jsonl`；audit list/export 可回读这些事件。agent server 使用一行请求一行响应的 stdio 模式，兼容 legacy JSONL RPC 与带 `jsonrpc: "2.0"` 的 MCP-style 请求，支持 `initialize`、`notifications/initialized`、`tools/list`、`tools/call` 和 `shutdown`。默认 `--read-only` 纯只读；只有 `--allow-writes --yes` 模式才会暴露 `switch_provider_profile`。使用说明见 `native-cli.md`。

## 本项目现状判断

当前 SpeakMate 已有:

- Tauri GUI 应用
- 前端通过 Tauri IPC 调用 Rust commands
- `pnpm validate:lv4` / `pnpm validate:lv4:full` 本地验证脚本
- Provider Profiles、语音运行态、诊断报告等应用内能力

调研时 SpeakMate 没有:

- 面向最终用户的 `speakmate` 原生 CLI
- `speakmate --json` 这类可脚本化输出
- `speakmate agent serve --stdio` 或 MCP server
- HTTP/WebSocket/JSON-RPC 本地控制面
- 让 agent 创建会话、切换角色、运行诊断、发起练习、导出报告的工具 schema
- agent 操作审计日志、权限确认、只读模式或危险操作 allowlist

因此，“原生 CLI + 原生 agent 全程操控”应作为新增升级类别。当前已经完成 CLI 查询、受控 profile switch、session export、JSONL/MCP stdio agent server、agent `switch_provider_profile` 和 audit log 查看/导出。后续重点是更多写工具的 allowlist、审计和用户确认策略。

## GitHub 借鉴项目

| 项目 | 类型 | 可借鉴点 | 对 SpeakMate 的启发 |
| --- | --- | --- | --- |
| [autohandai/commander](https://github.com/autohandai/commander) | Tauri 桌面 agent 编排器 | 原生 Tauri 桌面，编排 Claude Code/Codex/Gemini CLI，多 agent 流式输出，Git worktree 隔离，本地持久化 | 如果未来让 SpeakMate 调度外部 agent，应该用“薄 Tauri command + 本地进程/会话管理”模式 |
| [github/github-mcp-server](https://github.com/github/github-mcp-server) | 官方 MCP server | stdio 启动、toolsets、单工具启用、read-only 模式、环境变量配置 | SpeakMate 的 agent 控制面应优先考虑 MCP stdio，并支持工具分组和只读模式 |
| [tauri-plugin-cli](https://tauri.ubitools.com/plugin/cli/) | Tauri CLI 插件 | 通过 `tauri.conf.json` 定义 CLI，并在 Rust/JS 读取参数 | 适合做轻量启动参数，但 Windows GUI 程序输出到调用终端有额外限制 |
| [tauri-pilot](https://mpiton.github.io/tauri-pilot/) | Tauri UI agent/testing CLI | 面向 LLM 的 UI snapshot、click、fill 命令，解决 Playwright 不适配 Tauri 的问题 | SpeakMate 可借鉴“UI 可观测快照 + 受控交互命令”，用于 agent 测试和验收 |
| [OpenHands/OpenHands](https://github.com/OpenHands/OpenHands) | 软件开发 agent + CLI | CLI、本地 GUI、SDK、agentic runtime | 可借鉴 CLI/SDK/GUI 三层分离，但不应复制其软件工程 agent 复杂度 |
| [Aider-AI/aider](https://github.com/Aider-AI/aider) | 终端 AI 编程工具 | CLI-first、Git 仓库上下文、可脚本化终端工作流 | SpeakMate CLI 应保持简单、可组合、可 JSON 输出 |
| [openinterpreter/open-interpreter](https://github.com/OpenInterpreter/open-interpreter) | 电脑自然语言控制 CLI | 终端交互、权限控制、可配置系统提示 | 对“agent 全程操控”有启发，但 SpeakMate 应先限定工具范围，避免开放任意电脑控制 |
| [simular-ai/Agent-S](https://github.com/simular-ai/Agent-S) | Computer-use agent 框架 | GUI agent、Agent-Computer Interface、规划/执行/观察闭环 | 长期可作为 UI 自动化和多步任务评估参考，短期不建议直接集成 |

## 建议新增升级包: Native CLI & Agent Control

### 目标

让 SpeakMate 不只被人通过 GUI 使用，也能被脚本和 agent 安全、可审计地操控。

### 第一阶段: 原生 CLI

建议新增独立 Rust binary 或 Tauri CLI 子命令，优先支持:

1. `speakmate config profiles list --json`（已完成只读 list）
2. `speakmate config profiles switch <id>`（已完成，要求 `--yes`）
3. `speakmate diagnostics run --json`（已完成）
4. `speakmate chat send --character <id> --message "..."`
5. `speakmate session list --json`（已完成）
6. `speakmate session export --session <id> --json`（已完成）
7. `speakmate export report --session <id>`

第一阶段不建议开放:

- 任意文件读写
- 任意 shell 执行
- 无限制 UI 点击
- 自动删除 profile/key/session

### 第二阶段: Agent Control Server

建议做 MCP stdio 优先，而不是先做 HTTP 服务。

候选命令:

```bash
speakmate agent serve --stdio
speakmate agent serve --stdio --read-only
speakmate agent tools list --json
```

候选工具:

- `get_app_status`
- `list_characters`
- `list_sessions`
- `list_provider_profiles`
- `switch_provider_profile`
- `run_runtime_diagnostics`
- `start_practice_session`
- `send_practice_message`
- `get_learning_report`
- `export_session_summary`

### 第三阶段: UI Agent/Test Bridge

参考 tauri-pilot 的思路，补充:

- UI snapshot
- 按 semantic ref 点击
- 填写输入框
- 读取当前 toast/error/runtime badge
- 禁止 agent 直接通过坐标盲点

这一层更适合自动验收和回归测试，不一定先开放给普通用户。

## 安全边界

Native CLI 和 agent 控制面必须默认保守:

- 所有命令支持 `--json`，方便 agent 解析，避免靠自然语言猜状态。
- 写操作默认要求显式命令，不做隐式自动执行。
- 提供 `--read-only` 模式。
- 不返回 API key 明文。
- 诊断报告继续脱敏。
- 每次 agent 操作写入 audit log。
- 删除 profile、删除会话、清除 key 必须二次确认或 require `--yes`。
- 后续如加入 UI click/fill，优先使用 semantic refs，不使用裸坐标。

## 推荐落地顺序

1. 先完成 Provider Profiles 的重命名/删除/验收。
2. 已完成 `speakmate diagnostics run --json`，这是最小且风险最低的 CLI 切片。
3. 已完成 `speakmate config profiles list --json`。
4. 已完成 `session list/export --json`，为 agent 读取学习历史铺路。
5. 已完成 `speakmate agent serve --stdio --read-only` 的 JSONL 只读切片，只暴露查询和诊断工具。
6. 已完成 `config profiles switch --json --yes`，并写入 audit log。
7. 已完成 audit log 查看/导出。
8. 已完成 MCP-style JSON-RPC 2.0 `initialize` / `tools/list` / `tools/call` 兼容层。
9. 已完成 agent `switch_provider_profile`，仅在 `--allow-writes --yes` 模式启用。
10. 下一步评估更多 agent 写操作: start session、send message。
11. 最后考虑 UI snapshot/click/fill。
