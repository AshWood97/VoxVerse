//! Tool router for VoxVerse V1.
//!
//! Manages registration, dispatch, and auditing of tool calls
//! requested by the LLM during a session.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::providers::provider_error::ProviderError;

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

/// Trait for tool handlers that can be registered with the router.
///
/// Tool execution is synchronous; tools that need async I/O should
/// spawn a task and return a pending status.
pub trait ToolHandler: Send + Sync {
    /// Return the tool definition for LLM function-calling.
    fn definition(&self) -> ToolDefinition;

    /// Execute the tool synchronously with the given input arguments.
    fn execute(&self, input: serde_json::Value) -> Result<ToolResult, ProviderError>;
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
        let handler = self
            .tools
            .get(tool_name)
            .ok_or_else(|| ProviderError::InvalidInput(format!("Unknown tool: {tool_name}")))?;

        handler.execute(input)
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
            description: "Save an important fact about the user or the conversation to long-term memory.".into(),
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

    fn execute(&self, input: serde_json::Value) -> Result<ToolResult, ProviderError> {
        // In the full implementation, this will call MemoryService to persist the fact.
        // For now, return a success placeholder.
        let content = input
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("(empty)");

        Ok(ToolResult {
            success: true,
            output: Some(serde_json::json!({
                "saved": true,
                "content": content,
            })),
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let result = router
            .execute(
                "save_memory",
                serde_json::json!({"content": "User likes coffee", "fact_type": "preference"}),
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
    fn save_memory_tool_returns_success() {
        let tool = SaveMemoryTool;
        let result = tool
            .execute(serde_json::json!({
                "content": "User likes coffee",
                "fact_type": "preference"
            }))
            .unwrap();
        assert!(result.success);
        assert!(result.output.is_some());
    }
}
