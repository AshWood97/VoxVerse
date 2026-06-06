export type FactType = 'event' | 'preference' | 'commitment' | 'trait' | 'custom';

export interface MemoryFact {
  id: string;
  character_id: string;
  session_id?: string;
  fact_type: FactType;
  content: string;
  source_turn_id?: string;
  confidence: number;
  is_visible: boolean;
  is_deleted: boolean;
  created_at: string;
  updated_at: string;
}

export interface RelationshipState {
  id: string;
  character_id: string;
  intimacy_level: number;
  trust_level: number;
  plot_stage?: string;
  learningGoal?: string;
  user_preferences?: Record<string, unknown>;
  boundaries?: Record<string, unknown>;
  commitments?: Record<string, unknown>;
  updated_at: string;
}

export interface ToolInvocation {
  id: string;
  session_id: string;
  tool_name: string;
  status: 'pending' | 'running' | 'success' | 'failed';
  input_json?: string;
  output_json?: string;
  error_message?: string;
  started_at: string;
  completed_at?: string;
}
