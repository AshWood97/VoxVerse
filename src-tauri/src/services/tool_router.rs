//! Tool router for VoxVerse V1.
//!
//! Manages registration, dispatch, and auditing of tool calls
//! requested by the LLM during a session.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::providers::provider_error::ProviderError;
use crate::repositories::db::DbState;
use crate::services::memory_service;

/// Definition of a tool that can be called by the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Unique tool name.
    pub name: String,
    /// Human-readable description for the LLM.
    pub description: String,
    /// JSON Schema for the tool's parameters.
    pub parameters: serde_json::Value,
}

/// Result from executing a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether the tool call succeeded.
    pub success: bool,
    /// Result payload (tool-specific).
    pub output: Option<serde_json::Value>,
    /// Error message if the tool call failed.
    pub error: Option<String>,
}

/// Runtime context available while executing a session tool.
#[derive(Default)]
pub struct ToolExecutionContext<'a> {
    pub db: Option<&'a DbState>,
    pub character_id: Option<&'a str>,
    pub session_id: Option<&'a str>,
    pub source_turn_id: Option<&'a str>,
}

/// Trait for tool handlers that can be registered with the router.
///
/// Tool execution is synchronous; tools that need async I/O should
/// spawn a task and return a pending status.
pub trait ToolHandler: Send + Sync {
    /// Return the tool definition for LLM function-calling.
    fn definition(&self) -> ToolDefinition;

    /// Execute the tool synchronously with the given input arguments.
    fn execute(
        &self,
        input: serde_json::Value,
        context: &ToolExecutionContext<'_>,
    ) -> Result<ToolResult, ProviderError>;
}

/// Routes tool calls to registered handlers.
pub struct ToolRouter {
    tools: HashMap<String, Box<dyn ToolHandler>>,
}

impl ToolRouter {
    /// Create a new empty tool router.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool handler.
    pub fn register(&mut self, handler: Box<dyn ToolHandler>) {
        let name = handler.definition().name.clone();
        self.tools.insert(name, handler);
    }

    /// Get all tool definitions for LLM function-calling.
    pub fn tool_definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|h| h.definition()).collect()
    }

    /// Check if a tool is registered.
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Execute a tool by name.
    pub fn execute(
        &self,
        tool_name: &str,
        input: serde_json::Value,
    ) -> Result<ToolResult, ProviderError> {
        self.execute_with_context(tool_name, input, &ToolExecutionContext::default())
    }

    /// Execute a tool by name with session context.
    pub fn execute_with_context(
        &self,
        tool_name: &str,
        input: serde_json::Value,
        context: &ToolExecutionContext<'_>,
    ) -> Result<ToolResult, ProviderError> {
        let handler = self
            .tools
            .get(tool_name)
            .ok_or_else(|| ProviderError::InvalidInput(format!("Unknown tool: {tool_name}")))?;

        handler.execute(input, context)
    }
}

impl Default for ToolRouter {
    fn default() -> Self {
        Self::new()
    }
}

// ── Built-in tool: save_memory ──

/// A built-in tool that saves a memory fact for the current character.
pub struct SaveMemoryTool;

impl ToolHandler for SaveMemoryTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "save_memory".into(),
            description:
                "Save an important fact about the user or the conversation to long-term memory."
                    .into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "The fact to remember."
                    },
                    "fact_type": {
                        "type": "string",
                        "enum": ["event", "preference", "commitment", "trait", "custom"],
                        "description": "Category of the memory fact."
                    }
                },
                "required": ["content", "fact_type"]
            }),
        }
    }

    fn execute(
        &self,
        input: serde_json::Value,
        context: &ToolExecutionContext<'_>,
    ) -> Result<ToolResult, ProviderError> {
        let content = input
            .get("content")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| ProviderError::InvalidInput("save_memory requires content.".into()))?;

        let fact_type = input
            .get("fact_type")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| ProviderError::InvalidInput("save_memory requires fact_type.".into()))?;

        let db = context.db.ok_or_else(|| {
            ProviderError::Unavailable("save_memory requires database context.".into())
        })?;
        let character_id = context
            .character_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                ProviderError::Unavailable("save_memory requires character context.".into())
            })?;

        let fact = memory_service::create_memory_fact(
            db,
            character_id,
            context.session_id,
            fact_type,
            content,
            context.source_turn_id,
        )
        .map_err(|error| ProviderError::Internal(error.to_string()))?;

        Ok(ToolResult {
            success: true,
            output: Some(serde_json::json!({
                "saved": true,
                "id": fact.id,
                "character_id": fact.character_id,
                "session_id": fact.session_id,
                "fact_type": fact.fact_type,
                "content": fact.content,
            })),
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::db::DbState;
    use rusqlite::Connection;
    use std::sync::Mutex;

    fn setup_memory_db() -> DbState {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE memory_facts (
                id TEXT PRIMARY KEY, character_id TEXT NOT NULL,
                session_id TEXT, fact_type TEXT NOT NULL, content TEXT NOT NULL,
                source_turn_id TEXT, confidence REAL NOT NULL DEFAULT 1.0,
                is_visible INTEGER NOT NULL DEFAULT 1, is_deleted INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );",
        )
        .unwrap();
        DbState {
            conn: Mutex::new(conn),
        }
    }

    #[test]
    fn tool_router_registers_and_finds_tools() {
        let mut router = ToolRouter::new();
        router.register(Box::new(SaveMemoryTool));
        assert!(router.has_tool("save_memory"));
        assert!(!router.has_tool("nonexistent"));
    }

    #[test]
    fn tool_router_lists_definitions() {
        let mut router = ToolRouter::new();
        router.register(Box::new(SaveMemoryTool));
        let defs = router.tool_definitions();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "save_memory");
    }

    #[test]
    fn tool_router_executes_known_tool() {
        let mut router = ToolRouter::new();
        router.register(Box::new(SaveMemoryTool));
        let db = setup_memory_db();
        let result = router
            .execute_with_context(
                "save_memory",
                serde_json::json!({"content": "User likes coffee", "fact_type": "preference"}),
                &ToolExecutionContext {
                    db: Some(&db),
                    character_id: Some("c1"),
                    session_id: Some("s1"),
                    source_turn_id: Some("m1"),
                },
            )
            .unwrap();
        assert!(result.success);
    }

    #[test]
    fn tool_router_rejects_unknown_tool() {
        let router = ToolRouter::new();
        let result = router.execute("nonexistent", serde_json::json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn save_memory_tool_persists_fact() {
        let db = setup_memory_db();
        let tool = SaveMemoryTool;
        let result = tool
            .execute(
                serde_json::json!({
                    "content": "User likes coffee",
                    "fact_type": "preference"
                }),
                &ToolExecutionContext {
                    db: Some(&db),
                    character_id: Some("c1"),
                    session_id: Some("s1"),
                    source_turn_id: Some("m1"),
                },
            )
            .unwrap();
        assert!(result.success);
        assert!(result.output.is_some());

        let conn = db.lock_conn().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM memory_facts", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn save_memory_tool_requires_context() {
        let tool = SaveMemoryTool;
        let result = tool.execute(
            serde_json::json!({
                "content": "User likes coffee",
                "fact_type": "preference"
            }),
            &ToolExecutionContext::default(),
        );
        assert!(result.is_err());
    }
}
