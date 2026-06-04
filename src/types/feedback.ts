// ── Scenario ──

export interface Scenario {
  id: string;
  name: string;
  description: string;
  icon: string;
  learningGoals?: string;
  isPreset: boolean;
  createdAt: string;
}

// ── Vocabulary ──

export interface VocabularyItem {
  id: string;
  wordOrPhrase: string;
  translation?: string;
  context?: string;
  notes?: string;
  characterId?: string;
  createdAt: string;
}

// ── Correction / Feedback ──

export interface CorrectionResult {
  hasErrors: boolean;
  corrected: string;
  explanation: string;
  betterExpression: string;
  vocabulary: { word: string; meaning: string }[];
}

export interface TranslationResult {
  translation: string;
  notes: string;
}

export interface PolishResult {
  polished: string;
  changes: {
    original: string;
    improved: string;
    reason: string;
  }[];
}

export interface SummaryResult {
  overallScore: number;
  scoreBreakdown?: {
    fluency: number;
    grammar: number;
    vocabulary: number;
    coherence: number;
    pronunciation?: number;
  };
  strengths: string[];
  improvements: string[];
  commonErrors: { error: string; correction: string }[];
  vocabularyUsed: number;
  suggestedTopics: string[];
  nextDrills?: string[];
}

// ── Feedback action type ──

export type FeedbackAction = 'correct' | 'translate' | 'polish';

export interface FeedbackState {
  action: FeedbackAction | null;
  loading: boolean;
  result: CorrectionResult | TranslationResult | PolishResult | null;
  error: string | null;
  sourceText: string;
}
