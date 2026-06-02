# GitHub 同类项目借鉴清单

> 调研时间: 2026-04-26
> 目的: 为 SpeakMate Lv.4 验收后的升级路线提供产品和架构参考
> 原则: 借鉴产品模式、架构边界和验证方式，不直接复制代码

## 筛选标准

- 与语言练习、语音对话、AI 角色、桌面 AI 客户端或本地优先 AI 体验相关
- 已经体现 OpenAI-compatible、Ollama、本地模型、STT、TTS、角色/persona、学习反馈中的至少一项
- 优先选择仍有维护迹象、README 信息清晰、可从产品形态中抽象经验的项目

## 高价值参考项目

| 项目 | 类型 | 当前热度 | 可借鉴点 | 对 SpeakMate 的启发 |
| --- | --- | --- | --- | --- |
| [open-webui/open-webui](https://github.com/open-webui/open-webui) | 自托管 AI 平台 | 约 134k stars | Ollama/OpenAI-compatible、多 STT/TTS provider、语音/视频通话、RAG、管理后台 | Provider 抽象和诊断可以继续做深，但 SpeakMate 不应过早膨胀成平台 |
| [open-webui/desktop](https://github.com/open-webui/desktop) | 桌面 AI 客户端 | 约 1.5k stars | 桌面端一键启动、本地/远程连接切换、系统级 push-to-talk、浮动 chatbar、自动更新 | Lv.5 可考虑“桌面原生体验”：全局快捷键、浮窗、自动更新、连接配置 |
| [janhq/jan](https://github.com/janhq/jan) | 本地优先桌面 AI 客户端 | 约 42k stars | 离线优先、本地模型、OpenAI-compatible local API、隐私叙事清晰 | SpeakMate 的 Ollama/local-first 路线可以更明确地包装成隐私卖点 |
| [ChatGPTNextWeb/NextChat](https://github.com/ChatGPTNextWeb/NextChat) | 跨平台 AI 助手 | 约 87k stars | 多端、模型/端点配置、prompt 模板、分享、插件、Tauri fetch 安全增强 | 设置页可以演进为“连接配置 + prompt 模板 + 导出分享”的组合 |
| [bigsk1/voice-chat-ai](https://github.com/bigsk1/voice-chat-ai) | 语音 AI 对话 | 约 400+ stars | OpenAI/Ollama/Anthropic/xAI、多 TTS provider、Realtime voice、游戏和故事模式 | Lv.5 最值得借鉴的是“练习模式产品化”，不是单纯再加 provider |
| [Purple-Horizons/openclaw-voice](https://github.com/Purple-Horizons/openclaw-voice) | 浏览器语音助手 | 约 100 stars | Whisper STT、ElevenLabs TTS、WebSocket、自托管、OpenAI/Claude/custom agents | 可参考“语音管线状态机”和实时连接状态展示 |
| [ShayneP/local-voice-ai](https://github.com/ShayneP/local-voice-ai) | 本地语音 AI | 约 480 stars | Ollama、Kokoro、OpenAI-compatible STT、LiveKit Agents | 如果未来做低延迟连续对话，可先研究 LiveKit/Realtime 管线 |
| [AndraxDev/speak-gpt](https://github.com/AndraxDev/speak-gpt) | 移动语音助手 | 约 430 stars | 多 provider、Whisper、语音助手体验、API key 本地保存说明 | API key 安全和隐私说明应该进入 README/设置页，而不只在代码里实现 |
| [PeterBlenessy/TeamAI](https://github.com/PeterBlenessy/TeamAI) | Tauri + Vue AI personas | 低星但技术相近 | Tauri、Vue 3、OpenAI/Ollama、persona、会话历史 | 技术栈接近，可参考 persona 管理和 OpenAI/Ollama 并存的 UI 组织 |
| [0xAdafang/PersonAi](https://github.com/0xAdafang/PersonAi) | Tauri 本地角色聊天 | 低星但方向相近 | 本地优先、角色、persona、持久历史、计划加入语音 | 和 SpeakMate 的角色系统相似，可作为角色记忆/会话恢复的轻量参考 |
| [playztag/ai-language-tutor](https://github.com/playztag/ai-language-tutor) | AI 语言导师 | 低星但产品贴近 | 多语言、文本/语音输入输出、native speaker 模拟、Amazon Polly | 语言学习产品的核心是练习场景和反馈闭环，不只是聊天窗口 |
| [vbookshelf/E-Bot-English-Practice-Chatbot](https://github.com/vbookshelf/E-Bot-English-Practice-Chatbot) | 英语练习机器人 | 小型项目 | 语法纠错、翻译、口语响应、版本中加入 voice detection | 可借鉴“每轮对话自动纠错 + 母语解释”的学习体验 |
| [rahulsamant37/english-speaking-practice-app](https://github.com/rahulsamant37/english-speaking-practice-app) | IELTS 口语练习 GUI | 小型项目 | 录音、转写、AI 反馈、IELTS 评分、会话保存 | 可把 Lv.5 的学习报告拆成具体考试/场景模式，例如 IELTS speaking |

## 对 SpeakMate 的升级建议

### Lv.4 验收后优先补强

1. Provider profiles: 支持保存多套 OpenAI-compatible/Ollama/Custom 配置，而不是只有当前一套设置。
2. Voice session status: 把录音、转写、LLM、TTS、fallback 每一步做成可见状态，方便用户知道卡在哪。
3. Manual diagnostics archive: 允许把设置页诊断报告保存到本地验收记录，和 `validation-reports/` 形成闭环。
4. Privacy copy: 在设置页和 README 明确说明 API key 走系统 Keyring，本地数据如何保存。

### Lv.5 可以正式化的方向

1. Speaking practice modes: 增加 Free Talk、IELTS、Roleplay、Scenario Drill 等模式入口。
2. Structured feedback: 每轮输出拆成自然回复、纠错、替代表达、生词、得分/建议。
3. Desktop-native layer: 评估全局快捷键、浮动输入条、系统级 push-to-talk、自动更新。
4. Local-first track: 把 Ollama、本地 STT/TTS、离线数据存储包装成一个明确路线，而不是隐藏在设置里。
5. Character memory: 角色不仅有 prompt，还应有会话摘要、偏好、学习目标和长期记忆边界。

## 暂不建议立即追的方向

- 不建议现在复制 Open WebUI 的平台化复杂度，例如完整管理后台、RAG 插件市场、多租户权限。
- 不建议在 Lv.4 未完成人工验收前引入 Realtime/WebRTC/LiveKit；这会把调试面扩大很多。
- 不建议把所有 provider 同时做全，先把 OpenAI-compatible、Ollama、Custom 三类抽象打磨稳定。
- 不建议把角色市场后端化提前做成主线，除非角色导入、导出、校验和本地预览先稳定。

## 推荐下一步

1. 先完成 `lv4-validation-checklist.md` 的真实端点验收。
2. 验收通过后，把 Lv.5 拆成三个可交付包：Practice Modes、Provider Profiles、Desktop Native Layer。
3. 每个包都要求有自动构建验证、人工验收清单和构建日志，不再只写“计划中”。

