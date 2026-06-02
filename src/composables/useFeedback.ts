import { ref, type Ref } from 'vue';
import type {
  CorrectionResult,
  TranslationResult,
  PolishResult,
  SummaryResult,
  FeedbackAction,
} from '../types/feedback';
import { analyzeText, translateText, polishText, generateSummary, saveCorrection } from '../services/tauri/feedback';
import { logError, toErrorMessage } from '../utils/errors';

interface UseFeedbackReturn {
  isOpen: Ref<boolean>;
  isLoading: Ref<boolean>;
  action: Ref<FeedbackAction | null>;
  sourceText: Ref<string>;
  correctionResult: Ref<CorrectionResult | null>;
  translationResult: Ref<TranslationResult | null>;
  polishResult: Ref<PolishResult | null>;
  summaryResult: Ref<SummaryResult | null>;
  rawFallback: Ref<string | null>;
  error: Ref<string | null>;
  runCorrection: (text: string, sessionId?: string | null) => Promise<void>;
  runTranslation: (text: string, targetLang?: string) => Promise<void>;
  runPolish: (text: string) => Promise<void>;
  runSummary: (messages: [string, string][], sessionId?: string | null) => Promise<void>;
  close: () => void;
}

/**
 * Parse LLM JSON response, tolerating markdown fences and prose wrappers.
 */
function parseLLMJson<T>(raw: string): T {
  const candidates = buildJsonCandidates(raw);
  let lastError: unknown = null;

  for (const candidate of candidates) {
    try {
      return JSON.parse(candidate) as T;
    } catch (errorCause) {
      lastError = errorCause;
    }
  }

  throw lastError || new Error('Model response was not valid JSON.');
}

function buildJsonCandidates(raw: string): string[] {
  const trimmed = raw.trim();
  const candidates = new Set<string>();
  candidates.add(trimmed);

  const fenced = trimmed.match(/```(?:json)?\s*([\s\S]*?)```/i);
  if (fenced?.[1]) {
    candidates.add(fenced[1].trim());
  }

  const embedded = extractBalancedJson(trimmed);
  if (embedded) {
    candidates.add(embedded);
  }

  for (const candidate of Array.from(candidates)) {
    candidates.add(candidate.replace(/,\s*([}\]])/g, '$1'));
  }

  return Array.from(candidates).filter(Boolean);
}

function extractBalancedJson(text: string): string | null {
  const start = text.search(/[{\[]/);
  if (start === -1) {
    return null;
  }

  const opening = text[start];
  const stack = [opening === '{' ? '}' : ']'];
  let inString = false;
  let escaped = false;

  for (let i = start + 1; i < text.length; i += 1) {
    const char = text[i];

    if (escaped) {
      escaped = false;
      continue;
    }

    if (char === '\\') {
      escaped = true;
      continue;
    }

    if (char === '"') {
      inString = !inString;
      continue;
    }

    if (inString) {
      continue;
    }

    if (char === '{') {
      stack.push('}');
    } else if (char === '[') {
      stack.push(']');
    } else if (char === stack[stack.length - 1]) {
      stack.pop();
      if (stack.length === 0) {
        return text.slice(start, i + 1);
      }
    }
  }

  return null;
}

function safeParseLLMJson<T>(raw: string, fallback: (text: string) => T): { result: T; rawFallback: string | null } {
  try {
    return { result: parseLLMJson<T>(raw), rawFallback: null };
  } catch {
    const rawText = raw.trim() || 'The model returned an empty response.';
    return { result: fallback(rawText), rawFallback: rawText };
  }
}

function classifyFeedbackError(errorCause: unknown): string {
  const message = toErrorMessage(errorCause);
  const normalized = message.toLowerCase();

  if (normalized.includes('api key') || normalized.includes('401') || normalized.includes('403') || normalized.includes('auth')) {
    return `Authentication issue: ${message}`;
  }

  if (normalized.includes('timed out') || normalized.includes('timeout')) {
    return `Network timeout: ${message}`;
  }

  if (normalized.includes('network') || normalized.includes('dns') || normalized.includes('connection')) {
    return `Network issue: ${message}`;
  }

  if (normalized.includes('api returned') || normalized.includes('model')) {
    return `Model/API issue: ${message}`;
  }

  return message;
}

function correctionFallback(rawText: string, sourceText: string): CorrectionResult {
  return {
    hasErrors: false,
    corrected: sourceText,
    explanation: rawText,
    betterExpression: '',
    vocabulary: [],
  };
}

function translationFallback(rawText: string): TranslationResult {
  return {
    translation: rawText,
    notes: 'The model returned plain text instead of structured JSON.',
  };
}

function polishFallback(rawText: string): PolishResult {
  return {
    polished: rawText,
    changes: [],
  };
}

function summaryFallback(rawText: string): SummaryResult {
  return {
    overallScore: 0,
    strengths: [rawText],
    improvements: [],
    commonErrors: [],
    vocabularyUsed: 0,
    suggestedTopics: [],
  };
}

export function useFeedback(): UseFeedbackReturn {
  const isOpen = ref(false);
  const isLoading = ref(false);
  const action = ref<FeedbackAction | null>(null);
  const sourceText = ref('');
  const correctionResult = ref<CorrectionResult | null>(null);
  const translationResult = ref<TranslationResult | null>(null);
  const polishResult = ref<PolishResult | null>(null);
  const summaryResult = ref<SummaryResult | null>(null);
  const rawFallback = ref<string | null>(null);
  const error = ref<string | null>(null);

  function resetResults() {
    correctionResult.value = null;
    translationResult.value = null;
    polishResult.value = null;
    summaryResult.value = null;
    rawFallback.value = null;
    error.value = null;
  }

  async function runCorrection(text: string, sessionId: string | null = null) {
    resetResults();
    action.value = 'correct';
    sourceText.value = text;
    isOpen.value = true;
    isLoading.value = true;
    try {
      const raw = await analyzeText(text);
      const parsed = safeParseLLMJson<CorrectionResult>(
        raw,
        (rawText) => correctionFallback(rawText, text),
      );
      correctionResult.value = parsed.result;
      rawFallback.value = parsed.rawFallback;
      await persistCorrection(text, parsed.result, sessionId);
    } catch (errorCause) {
      error.value = classifyFeedbackError(errorCause);
    } finally {
      isLoading.value = false;
    }
  }

  async function runTranslation(text: string, targetLang: string = '中文') {
    resetResults();
    action.value = 'translate';
    sourceText.value = text;
    isOpen.value = true;
    isLoading.value = true;
    try {
      const raw = await translateText(text, targetLang);
      const parsed = safeParseLLMJson<TranslationResult>(raw, translationFallback);
      translationResult.value = parsed.result;
      rawFallback.value = parsed.rawFallback;
    } catch (errorCause) {
      error.value = classifyFeedbackError(errorCause);
    } finally {
      isLoading.value = false;
    }
  }

  async function runPolish(text: string) {
    resetResults();
    action.value = 'polish';
    sourceText.value = text;
    isOpen.value = true;
    isLoading.value = true;
    try {
      const raw = await polishText(text);
      const parsed = safeParseLLMJson<PolishResult>(raw, polishFallback);
      polishResult.value = parsed.result;
      rawFallback.value = parsed.rawFallback;
    } catch (errorCause) {
      error.value = classifyFeedbackError(errorCause);
    } finally {
      isLoading.value = false;
    }
  }

  async function runSummary(messages: [string, string][], sessionId: string | null = null) {
    resetResults();
    action.value = null; // summary has its own display
    isOpen.value = true;
    isLoading.value = true;
    try {
      const raw = await generateSummary(messages, sessionId);
      const parsed = safeParseLLMJson<SummaryResult>(raw, summaryFallback);
      summaryResult.value = parsed.result;
      rawFallback.value = parsed.rawFallback;
    } catch (errorCause) {
      error.value = classifyFeedbackError(errorCause);
    } finally {
      isLoading.value = false;
    }
  }

  async function persistCorrection(text: string, result: CorrectionResult, sessionId: string | null) {
    try {
      await saveCorrection({
        sessionId,
        originalText: text,
        correctedText: result.corrected || text,
        explanation: result.explanation || null,
        betterExpression: result.betterExpression || null,
        createdAt: new Date().toISOString(),
      });
    } catch (errorCause) {
      logError('Failed to save correction', errorCause);
    }
  }

  function close() {
    isOpen.value = false;
    // Keep results so user can reopen
  }

  return {
    isOpen,
    isLoading,
    action,
    sourceText,
    correctionResult,
    translationResult,
    polishResult,
    summaryResult,
    rawFallback,
    error,
    runCorrection,
    runTranslation,
    runPolish,
    runSummary,
    close,
  };
}
