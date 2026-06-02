//! VoxVerse realtime session event protocol.
//!
//! This module defines the unified event types exchanged between the backend
//! session orchestrator and the frontend UI. These events will be transported
//! via Tauri IPC channels in V1, and later via WebRTC data channels in V1-C.

use serde::{Deserialize, Serialize};

/// Reason for a turn interruption.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InterruptReason {
    /// User started speaking while assistant was still outputting.
    UserBarged,
    /// System detected silence or explicit stop from user.
    UserStopped,
    /// System-initiated interrupt (e.g., error recovery).
    SystemAbort,
}

/// A tool call request from the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    /// Unique ID for this tool call.
    pub id: String,
    /// Tool name (e.g., "generate_image", "save_memory").
    pub name: String,
    /// JSON-encoded arguments for the tool.
    pub arguments: serde_json::Value,
}

/// Result of a tool call execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    /// The tool call ID this result corresponds to.
    pub tool_id: String,
    /// Whether the tool call succeeded.
    pub success: bool,
    /// Result payload (tool-specific).
    pub output: Option<serde_json::Value>,
    /// Error message if the tool call failed.
    pub error: Option<String>,
}

/// Unified session event protocol for VoxVerse V1.
///
/// Each variant corresponds to a realtime event in the session lifecycle.
/// The `event` field is used as the discriminant in the serialized form.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data")]
pub enum SessionEvent {
    /// Emitted when a realtime session is successfully created.
    #[serde(rename = "session.started")]
    SessionStarted {
        session_id: String,
        character_id: String,
        timestamp: String,
    },

    /// Emitted when the client begins or continues audio input.
    #[serde(rename = "audio.input")]
    AudioInput {
        session_id: String,
        is_active: bool,
    },

    /// Emitted for partial (in-progress) STT transcripts.
    #[serde(rename = "transcript.partial")]
    TranscriptPartial {
        session_id: String,
        text: String,
        is_final: bool,
    },

    /// Emitted when the STT produces a final, committed transcript.
    #[serde(rename = "transcript.final")]
    TranscriptFinal {
        session_id: String,
        text: String,
        confidence: f32,
    },

    /// Emitted when a turn is interrupted.
    #[serde(rename = "turn.interrupted")]
    TurnInterrupted {
        session_id: String,
        reason: InterruptReason,
    },

    /// Emitted for incremental assistant output (text and/or audio).
    #[serde(rename = "assistant.delta")]
    AssistantDelta {
        session_id: String,
        text: Option<String>,
        audio_chunk: Option<Vec<u8>>,
    },

    /// Emitted when the assistant finishes its current turn.
    #[serde(rename = "assistant.done")]
    AssistantDone {
        session_id: String,
        full_text: String,
        tool_calls: Vec<ToolCallRequest>,
    },

    /// Emitted when the LLM requests a tool call.
    #[serde(rename = "tool.requested")]
    ToolRequested {
        session_id: String,
        tool_call: ToolCallRequest,
    },

    /// Emitted when a tool call finishes execution.
    #[serde(rename = "tool.completed")]
    ToolCompleted {
        session_id: String,
        tool_id: String,
        result: ToolCallResult,
    },

    /// Emitted when a runtime error occurs in the session.
    #[serde(rename = "runtime.error")]
    RuntimeError {
        session_id: String,
        code: String,
        message: String,
        recoverable: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_event_round_trips_through_json() {
        let events = vec![
            SessionEvent::SessionStarted {
                session_id: "s1".into(),
                character_id: "c1".into(),
                timestamp: "2026-06-01T00:00:00Z".into(),
            },
            SessionEvent::TranscriptPartial {
                session_id: "s1".into(),
                text: "hello".into(),
                is_final: false,
            },
            SessionEvent::TranscriptFinal {
                session_id: "s1".into(),
                text: "hello world".into(),
                confidence: 0.95,
            },
            SessionEvent::TurnInterrupted {
                session_id: "s1".into(),
                reason: InterruptReason::UserBarged,
            },
            SessionEvent::AssistantDelta {
                session_id: "s1".into(),
                text: Some("Hi".into()),
                audio_chunk: None,
            },
            SessionEvent::AssistantDone {
                session_id: "s1".into(),
                full_text: "Hi there!".into(),
                tool_calls: vec![],
            },
            SessionEvent::ToolRequested {
                session_id: "s1".into(),
                tool_call: ToolCallRequest {
                    id: "tc1".into(),
                    name: "save_memory".into(),
                    arguments: serde_json::json!({"content": "likes coffee"}),
                },
            },
            SessionEvent::RuntimeError {
                session_id: "s1".into(),
                code: "stt_timeout".into(),
                message: "STT timed out".into(),
                recoverable: true,
            },
        ];

        for event in &events {
            let json = serde_json::to_string(event).expect("serialize");
            let parsed: SessionEvent = serde_json::from_str(&json).expect("deserialize");
            let re_json = serde_json::to_string(&parsed).expect("re-serialize");
            assert_eq!(json, re_json, "round-trip failed for event");
        }
    }

    #[test]
    fn session_started_has_correct_tag() {
        let event = SessionEvent::SessionStarted {
            session_id: "s1".into(),
            character_id: "c1".into(),
            timestamp: "2026-06-01T00:00:00Z".into(),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["event"], "session.started");
    }

    #[test]
    fn interrupt_reason_serializes_as_snake_case() {
        let json = serde_json::to_string(&InterruptReason::UserBarged).unwrap();
        assert!(json.contains("user_barged"));
    }
}
