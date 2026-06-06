import { invoke } from '@tauri-apps/api/core';

export interface SaveCorrectionInput {
  sessionId?: string | null;
  originalText: string;
  correctedText: string;
  explanation?: string | null;
  betterExpression?: string | null;
  score?: number | null;
  scoreBreakdown?: Record<string, number> | null;
  pronunciationNote?: string | null;
  nextPromptSuggestion?: string | null;
  createdAt?: string | null;
}

/**
 * Analyze user text for grammar, word choice, naturalness.
 * Returns raw JSON string from LLM (must be parsed by caller).
 */
export async function analyzeText(text: string): Promise<string> {
  return invoke<string>('analyze_text', { text });
}

export async function saveCorrection(correction: SaveCorrectionInput): Promise<void> {
  await invoke('save_correction', { correction });
}

/**
 * Translate text to a target language.
 * Returns raw JSON string from LLM.
 */
export async function translateText(text: string, targetLang: string): Promise<string> {
  return invoke<string>('translate_text', { text, targetLang });
}

/**
 * Polish/improve user text to sound more native.
 * Returns raw JSON string from LLM.
 */
export async function polishText(text: string): Promise<string> {
  return invoke<string>('polish_text', { text });
}

/**
 * Generate a learning summary report for a conversation.
 * Takes an array of [role, content] tuples.
 * Returns raw JSON string from LLM.
 */
export async function generateSummary(messages: [string, string][], sessionId?: string | null): Promise<string> {
  return invoke<string>('generate_summary', { messages, sessionId: sessionId || null });
}
