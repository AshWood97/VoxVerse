# SpeakMate 维护者速览

> 更新时间: 2026-04-23
> 首先阅读: `task.md`

这份文档给维护者和下一位接手者使用。它不负责定义路线图的权威状态，而是负责快速解释项目现在长什么样、核心模块在哪里、应该先做什么。

## 一句话概览

SpeakMate 是一个基于 Tauri + Vue 的 AI 口语训练桌面应用，已经具备角色聊天、语音输入输出、学习反馈、会话历史、场景和生词本等能力，并正在把更高阶的语音链路和平台化特性收口成稳定版本。

## 文档分工

- `README.md`: 项目总览和入口说明
- `task.md`: 路线图唯一事实源
- `implementation_plan.md`: 近期执行计划
- `项目构建日志/*.md`: 每次重要迭代的历史记录

## 代码结构

### 前端

- `src/App.vue`
  - 应用主编排层
  - 负责角色、会话、反馈、统计、场景和弹窗状态联动
- `src/components/`
  - `base/`: 基础 UI 组件
  - `domain/`: 业务组件，如 `ChatPanel`、`FeedbackPanel`、`SessionHistory`
- `src/composables/`
  - 主要业务状态入口，如 `useChat`、`useCharacter`、`useSession`、`useFeedback`
- `src/services/tauri/`
  - 前端对 Tauri command 的 IPC 封装
- `src/i18n.ts` + `src/locales/`
  - 多语言基础设施，目前也是构建修复点之一

### 后端

- `src-tauri/src/lib.rs`
  - Tauri 应用入口和 command 注册中心
- `src-tauri/src/commands/`
  - 前后端桥接命令
- `src-tauri/src/repositories/`
  - SQLite 读写和配置持久化
- `src-tauri/src/services/`
  - LLM、反馈、STT、TTS 等服务实现
- `src-tauri/src/state/`
  - 运行时状态和 Keyring 相关配置

## 里程碑现状

- Lv.0-Lv.3: 已完成并进入主功能路径
- Lv.4: 已有主要实现，但仍需构建修复和端到端验证
- Lv.5: 有若干原型能力，不应误判为完整平台发布

## 当前风险

### 1. 文档状态曾经不一致

此前 `task.md`、`implementation_plan.md`、`walkthrough.md` 和日志索引对当前阶段的描述存在冲突。本轮已经按“`task.md` 为唯一事实源”的原则进行收敛。

### 2. 构建仍然不是绿色

当前已知问题：

- `src/App.vue` 缺少 `useI18n` 导入
- `src/i18n.ts` locale 类型不匹配

因此在没有修复这些问题前，不要把文档里的“已实现”直接等同于“已发布可用”。

### 3. Lv.4 需要真实验证

虽然 Keyring、Whisper STT、Edge TTS、Ollama 兼容代码都已经存在，但还需要一次完整的功能走查和验证记录。

## 建议接手顺序

1. 先看 `task.md`，确认阶段定义
2. 再看 `implementation_plan.md`，确认最近要做什么
3. 然后看 `src/App.vue` 和 `src-tauri/src/lib.rs`，理解前后端编排
4. 最后查 `项目构建日志/` 了解每次里程碑改动

## 下一步建议

如果下一位接手者要继续推进，建议优先做下面三件事：

1. 修复 `pnpm build`
2. 验证 Lv.4 语音与配置链路
3. 判断 Lv.5 原型中哪些保留，哪些回退到 backlog
