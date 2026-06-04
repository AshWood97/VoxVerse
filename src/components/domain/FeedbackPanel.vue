<script setup lang="ts">
import type {
  CorrectionResult,
  TranslationResult,
  PolishResult,
  SummaryResult,
  FeedbackAction,
} from '../../types/feedback';
import DiffView from '../base/DiffView.vue';

defineProps<{
  isOpen: boolean;
  isLoading: boolean;
  action: FeedbackAction | null;
  sourceText: string;
  correctionResult: CorrectionResult | null;
  translationResult: TranslationResult | null;
  polishResult: PolishResult | null;
  summaryResult: SummaryResult | null;
  rawFallback: string | null;
  error: string | null;
}>();

const emit = defineEmits<{
  close: [];
  saveWord: [word: string, meaning: string, context: string];
}>();

function getActionTitle(action: FeedbackAction | null): string {
  if (!action) return '📊 Learning Report';
  const titles: Record<FeedbackAction, string> = {
    correct: '✏️ Grammar Check',
    translate: '🌐 Translation',
    polish: '✨ Expression Polish',
  };
  return titles[action] || '';
}
</script>

<template>
  <transition name="slide-right">
    <aside v-if="isOpen" class="feedback-panel">
      <div class="fp-header">
        <h3 class="fp-title">{{ getActionTitle(action) }}</h3>
        <button class="fp-close" @click="emit('close')">✕</button>
      </div>

      <!-- Loading -->
      <div v-if="isLoading" class="fp-loading">
        <div class="fp-spinner"></div>
        <p>Analyzing...</p>
      </div>

      <!-- Error -->
      <div v-else-if="error" class="fp-error">
        <p>⚠️ {{ error }}</p>
      </div>

      <!-- Correction Result -->
      <div v-else-if="action === 'correct' && correctionResult" class="fp-body">
        <div class="fp-section">
          <div class="fp-label">Original</div>
          <p class="fp-source-text">{{ sourceText }}</p>
        </div>

        <div v-if="correctionResult.hasErrors" class="fp-section">
          <div class="fp-label">Correction</div>
          <DiffView :original="sourceText" :corrected="correctionResult.corrected" />
        </div>
        <div v-else class="fp-success">
          ✅ No errors found! Great job!
        </div>

        <div v-if="correctionResult.explanation" class="fp-section">
          <div class="fp-label">Explanation</div>
          <p class="fp-text">{{ correctionResult.explanation }}</p>
        </div>

        <div v-if="correctionResult.betterExpression" class="fp-section">
          <div class="fp-label">💡 More Natural Expression</div>
          <p class="fp-highlight">{{ correctionResult.betterExpression }}</p>
        </div>

        <div v-if="correctionResult.vocabulary?.length" class="fp-section">
          <div class="fp-label">📚 Vocabulary</div>
          <div
            v-for="(v, i) in correctionResult.vocabulary"
            :key="i"
            class="fp-vocab-item"
          >
            <span class="fp-vocab-word">{{ v.word }}</span>
            <span class="fp-vocab-meaning">{{ v.meaning }}</span>
            <button
              class="fp-vocab-save"
              @click="emit('saveWord', v.word, v.meaning, sourceText)"
              title="Save to vocabulary"
            >⭐</button>
          </div>
        </div>

        <div v-if="rawFallback" class="fp-section fp-raw-fallback">
          <div class="fp-label">Raw Model Response</div>
          <p class="fp-text">{{ rawFallback }}</p>
        </div>
      </div>

      <!-- Translation Result -->
      <div v-else-if="action === 'translate' && translationResult" class="fp-body">
        <div class="fp-section">
          <div class="fp-label">Original</div>
          <p class="fp-source-text">{{ sourceText }}</p>
        </div>
        <div class="fp-section">
          <div class="fp-label">Translation</div>
          <p class="fp-highlight">{{ translationResult.translation }}</p>
        </div>
        <div v-if="translationResult.notes" class="fp-section">
          <div class="fp-label">Notes</div>
          <p class="fp-text">{{ translationResult.notes }}</p>
        </div>
        <div v-if="rawFallback" class="fp-section fp-raw-fallback">
          <div class="fp-label">Raw Model Response</div>
          <p class="fp-text">{{ rawFallback }}</p>
        </div>
      </div>

      <!-- Polish Result -->
      <div v-else-if="action === 'polish' && polishResult" class="fp-body">
        <div class="fp-section">
          <div class="fp-label">Original</div>
          <p class="fp-source-text">{{ sourceText }}</p>
        </div>
        <div class="fp-section">
          <div class="fp-label">Polished</div>
          <DiffView :original="sourceText" :corrected="polishResult.polished" />
        </div>
        <div v-if="polishResult.changes?.length" class="fp-section">
          <div class="fp-label">Changes</div>
          <div v-for="(c, i) in polishResult.changes" :key="i" class="fp-change-item">
            <div class="fp-change-before">{{ c.original }}</div>
            <div class="fp-change-arrow">→</div>
            <div class="fp-change-after">{{ c.improved }}</div>
            <div class="fp-change-reason">{{ c.reason }}</div>
          </div>
        </div>
        <div v-if="rawFallback" class="fp-section fp-raw-fallback">
          <div class="fp-label">Raw Model Response</div>
          <p class="fp-text">{{ rawFallback }}</p>
        </div>
      </div>

      <!-- Summary Result -->
      <div v-else-if="summaryResult" class="fp-body">
        <div class="fp-section">
          <div class="fp-label">Overall Score</div>
          <div class="fp-score">
            <span class="fp-score-num">{{ summaryResult.overallScore }}</span>
            <span class="fp-score-max">/10</span>
          </div>
        </div>
        <div v-if="summaryResult.scoreBreakdown" class="fp-section">
          <div class="fp-label">Skill Breakdown</div>
          <div class="fp-score-grid">
            <div
              v-for="(score, key) in summaryResult.scoreBreakdown"
              :key="key"
              class="fp-score-chip"
            >
              <span class="fp-score-chip__label">{{ key }}</span>
              <span class="fp-score-chip__value">{{ score }}/10</span>
            </div>
          </div>
        </div>
        <div v-if="summaryResult.strengths?.length" class="fp-section">
          <div class="fp-label">💪 Strengths</div>
          <ul class="fp-list fp-list--good">
            <li v-for="(s, i) in summaryResult.strengths" :key="i">{{ s }}</li>
          </ul>
        </div>
        <div v-if="summaryResult.improvements?.length" class="fp-section">
          <div class="fp-label">📈 Areas to Improve</div>
          <ul class="fp-list fp-list--warn">
            <li v-for="(s, i) in summaryResult.improvements" :key="i">{{ s }}</li>
          </ul>
        </div>
        <div v-if="summaryResult.commonErrors?.length" class="fp-section">
          <div class="fp-label">Common Errors</div>
          <div v-for="(e, i) in summaryResult.commonErrors" :key="i" class="fp-change-item">
            <div class="fp-change-before">{{ e.error }}</div>
            <div class="fp-change-arrow">→</div>
            <div class="fp-change-after">{{ e.correction }}</div>
          </div>
        </div>
        <div v-if="summaryResult.nextDrills?.length" class="fp-section">
          <div class="fp-label">Next Drills</div>
          <ul class="fp-list fp-list--drill">
            <li v-for="(drill, i) in summaryResult.nextDrills" :key="i">{{ drill }}</li>
          </ul>
        </div>
        <div v-if="rawFallback" class="fp-section fp-raw-fallback">
          <div class="fp-label">Raw Model Response</div>
          <p class="fp-text">{{ rawFallback }}</p>
        </div>
      </div>
    </aside>
  </transition>
</template>

<style scoped>
.feedback-panel {
  width: 340px;
  height: 100%;
  background: var(--bg-secondary);
  border-left: 1px solid var(--border-subtle);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  overflow: hidden;
}

.fp-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-md) var(--space-lg);
  border-bottom: 1px solid var(--border-subtle);
}

.fp-title {
  font-size: var(--font-size-base);
  font-weight: 600;
  color: var(--text-primary);
}

.fp-close {
  width: 28px;
  height: 28px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  transition: all var(--transition-fast);
}
.fp-close:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.fp-body {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-md) var(--space-lg);
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

.fp-section {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.fp-label {
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.fp-source-text {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  padding: var(--space-sm) var(--space-md);
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  border-left: 3px solid var(--border-visible);
}

.fp-text {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  line-height: 1.6;
}

.fp-raw-fallback {
  border-top: 1px solid var(--border-subtle);
  padding-top: var(--space-md);
}

.fp-highlight {
  font-size: var(--font-size-sm);
  color: var(--accent-secondary);
  padding: var(--space-sm) var(--space-md);
  background: rgba(0, 206, 201, 0.08);
  border-radius: var(--radius-md);
  border-left: 3px solid var(--accent-secondary);
  line-height: 1.6;
}

.fp-success {
  padding: var(--space-md);
  background: rgba(46, 204, 113, 0.1);
  border-radius: var(--radius-md);
  color: #2ecc71;
  font-size: var(--font-size-sm);
  font-weight: 500;
  text-align: center;
}

/* Vocabulary items */
.fp-vocab-item {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-xs) var(--space-sm);
  border-radius: var(--radius-sm);
  background: var(--bg-tertiary);
}
.fp-vocab-word {
  font-weight: 600;
  color: var(--accent-primary);
  font-size: var(--font-size-sm);
}
.fp-vocab-meaning {
  flex: 1;
  font-size: var(--font-size-xs);
  color: var(--text-secondary);
}
.fp-vocab-save {
  opacity: 0.4;
  font-size: 0.9rem;
  transition: opacity var(--transition-fast), transform var(--transition-fast);
}
.fp-vocab-save:hover {
  opacity: 1;
  transform: scale(1.2);
}

/* Change items (polish / summary errors) */
.fp-change-item {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  gap: var(--space-xs);
  padding: var(--space-sm);
  background: var(--bg-tertiary);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-sm);
  align-items: center;
}
.fp-change-before {
  color: #e74c3c;
  text-decoration: line-through;
  opacity: 0.7;
}
.fp-change-arrow {
  color: var(--text-tertiary);
  text-align: center;
}
.fp-change-after {
  color: #2ecc71;
  font-weight: 500;
}
.fp-change-reason {
  grid-column: 1 / -1;
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
  margin-top: var(--space-xs);
}

/* Summary score */
.fp-score {
  display: flex;
  align-items: baseline;
  gap: 2px;
}
.fp-score-num {
  font-size: var(--font-size-3xl);
  font-weight: 700;
  background: var(--accent-gradient);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}
.fp-score-max {
  font-size: var(--font-size-lg);
  color: var(--text-tertiary);
}

.fp-score-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-xs);
}

.fp-score-chip {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-xs);
  padding: var(--space-xs) var(--space-sm);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  background: var(--bg-tertiary);
}

.fp-score-chip__label {
  color: var(--text-secondary);
  font-size: var(--font-size-xs);
  text-transform: capitalize;
}

.fp-score-chip__value {
  color: var(--accent-secondary);
  font-size: var(--font-size-xs);
  font-weight: 700;
}

/* Lists */
.fp-list {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}
.fp-list li {
  font-size: var(--font-size-sm);
  padding-left: var(--space-md);
  position: relative;
  color: var(--text-secondary);
}
.fp-list li::before {
  position: absolute;
  left: 0;
}
.fp-list--good li::before { content: '✅'; }
.fp-list--warn li::before { content: '💡'; }
.fp-list--drill li::before { content: '→'; }

/* Loading */
.fp-loading {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-md);
  color: var(--text-tertiary);
}
.fp-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border-subtle);
  border-top-color: var(--accent-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.fp-error {
  padding: var(--space-lg);
  color: #e74c3c;
  font-size: var(--font-size-sm);
}

/* Animation */
.slide-right-enter-active,
.slide-right-leave-active {
  transition: transform var(--transition-normal), opacity var(--transition-normal);
}
.slide-right-enter-from,
.slide-right-leave-to {
  transform: translateX(100%);
  opacity: 0;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
