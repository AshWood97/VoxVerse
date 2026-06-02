//! Session orchestrator for VoxVerse V1.
//!
//! Coordinates the flow between STT, LLM, TTS, memory, and tool routing
//! during a realtime session. This is the "LlmOrchestrator" described in
//! the upgrade plan.

use crate::protocol::session_event::SessionEvent;
use crate::providers::llm_provider::{ChatMessage, CompletionConfig};
use crate::services::tool_router::ToolRouter;

/// Configuration for creating a new orchestrated session.
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Character's system prompt.
    pub system_prompt: String,
    /// Character ID for memory isolation.
    pub character_id: String,
    /// Session ID.
    pub session_id: String,
    /// LLM model to use.
    pub model: String,
    /// Additional prompt suffix (practice mode, scenario).
    pub prompt_suffix: String,
}

/// The session orchestrator manages a single conversation session.
///
/// It is responsible for:
/// - Building the system prompt with memory injection
/// - Sending messages to the LLM provider
/// - Parsing and routing tool calls
/// - Emitting session events
///
/// In V1, this orchestrator works synchronously with Tauri IPC.
/// In V1-C, it will be adapted to work with WebRTC/LiveKit streams.
pub struct SessionOrchestrator {
    config: SessionConfig,
    tool_router: ToolRouter,
}

impl SessionOrchestrator {
    /// Create a new session orchestrator.
    pub fn new(config: SessionConfig) -> Self {
        let mut tool_router = ToolRouter::new();
        // Register built-in tools
        tool_router.register(Box::new(super::tool_router::SaveMemoryTool));

        Self {
            config,
            tool_router,
        }
    }

    /// Build the full system prompt with memory context.
    ///
    /// In the full implementation, this will:
    /// 1. Load relationship state for the character
    /// 2. Retrieve relevant memory facts
    /// 3. Format them into a memory injection block
    /// 4. Append tool definitions
    /// 5. Combine with the character's base system prompt
    pub fn build_system_prompt(&self, _memory_context: Option<&str>) -> String {
        let mut prompt = self.config.system_prompt.clone();

        if !self.config.prompt_suffix.is_empty() {
            prompt.push('\n');
            prompt.push_str(&self.config.prompt_suffix);
        }

        // Memory context injection placeholder
        // In the full implementation, memory facts will be injected here
        if let Some(memory) = _memory_context {
            prompt.push_str("\n\n[Memory Context]\n");
            prompt.push_str(memory);
        }

        prompt
    }

    /// Build the messages array for the LLM, including system prompt.
    pub fn build_messages(
        &self,
        history: &[ChatMessage],
        memory_context: Option<&str>,
    ) -> Vec<ChatMessage> {
        let system_prompt = self.build_system_prompt(memory_context);

        let mut messages = vec![ChatMessage {
            role: "system".into(),
            content: system_prompt,
        }];

        messages.extend_from_slice(history);
        messages
    }

    /// Get tool definitions for function-calling.
    pub fn tool_definitions(&self) -> Vec<super::tool_router::ToolDefinition> {
        self.tool_router.tool_definitions()
    }

    /// Create a session.started event.
    pub fn session_started_event(&self) -> SessionEvent {
        SessionEvent::SessionStarted {
            session_id: self.config.session_id.clone(),
            character_id: self.config.character_id.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Access the session config.
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> SessionConfig {
        SessionConfig {
            system_prompt: "You are a helpful assistant.".into(),
            character_id: "char-1".into(),
            session_id: "sess-1".into(),
            model: "gpt-4o-mini".into(),
            prompt_suffix: "Be concise.".into(),
        }
    }

    #[test]
    fn builds_system_prompt_with_suffix() {
        let orch = SessionOrchestrator::new(test_config());
        let prompt = orch.build_system_prompt(None);
        assert!(prompt.contains("You are a helpful assistant."));
        assert!(prompt.contains("Be concise."));
    }

    #[test]
    fn builds_system_prompt_with_memory_context() {
        let orch = SessionOrchestrator::new(test_config());
        let prompt = orch.build_system_prompt(Some("User likes coffee."));
        assert!(prompt.contains("[Memory Context]"));
        assert!(prompt.contains("User likes coffee."));
    }

    #[test]
    fn builds_messages_with_system_and_history() {
        let orch = SessionOrchestrator::new(test_config());
        let history = vec![
            ChatMessage {
                role: "user".into(),
                content: "Hi".into(),
            },
            ChatMessage {
                role: "assistant".into(),
                content: "Hello!".into(),
            },
        ];
        let messages = orch.build_messages(&history, None);
        assert_eq!(messages.len(), 3); // system + 2 history
        assert_eq!(messages[0].role, "system");
    }

    #[test]
    fn session_started_event_has_correct_ids() {
        let orch = SessionOrchestrator::new(test_config());
        let event = orch.session_started_event();
        match event {
            SessionEvent::SessionStarted {
                session_id,
                character_id,
                ..
            } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(character_id, "char-1");
            }
            _ => panic!("Expected SessionStarted event"),
        }
    }

    #[test]
    fn orchestrator_has_built_in_tools() {
        let orch = SessionOrchestrator::new(test_config());
        let defs = orch.tool_definitions();
        assert!(!defs.is_empty());
        assert!(defs.iter().any(|d| d.name == "save_memory"));
    }
}
