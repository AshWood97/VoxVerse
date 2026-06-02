export interface ToolCallRequest {
  id: string;
  name: string;
  arguments: string;
}

export type SessionEvent =
  | { event: 'session.started'; data: { session_id: string; character_id: string; timestamp: string } }
  | { event: 'transcript.partial'; data: { session_id: string; text: string; is_final: boolean } }
  | { event: 'transcript.final'; data: { session_id: string; text: string; confidence: number } }
  | { event: 'turn.interrupted'; data: { session_id: string; reason: string } }
  | { event: 'assistant.delta'; data: { session_id: string; text?: string; audio_chunk?: number[] } }
  | { event: 'assistant.done'; data: { session_id: string; full_text: string; tool_calls: ToolCallRequest[] } }
  | { event: 'tool.requested'; data: { session_id: string; tool_call: ToolCallRequest } }
  | { event: 'runtime.error'; data: { session_id: string; code: string; message: string; recoverable: boolean } };
