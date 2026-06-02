<script setup lang="ts">
import { computed } from 'vue';
import type { VocabularyItem } from '../../types/feedback';

const props = defineProps<{
  vocabulary: VocabularyItem[];
}>();

const emit = defineEmits<{
  close: [];
  delete: [id: string];
}>();

const sortedVocab = computed(() => {
  return [...props.vocabulary].sort(
    (a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
  );
});

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="vocab-modal">
      <div class="modal-header">
        <h2>📚 Vocabulary Book</h2>
        <span class="vocab-count">{{ vocabulary.length }} words</span>
        <button class="close-btn" @click="emit('close')">✕</button>
      </div>

      <div v-if="sortedVocab.length === 0" class="vocab-empty">
        <span class="vocab-empty-icon">📖</span>
        <p>No words saved yet.</p>
        <p class="vocab-empty-hint">Use the ⭐ button in feedback results to save words!</p>
      </div>

      <div v-else class="vocab-list">
        <div v-for="item in sortedVocab" :key="item.id" class="vocab-item">
          <div class="vocab-main">
            <span class="vocab-word">{{ item.wordOrPhrase }}</span>
            <span v-if="item.translation" class="vocab-translation">{{ item.translation }}</span>
          </div>
          <div v-if="item.context" class="vocab-context">
            "...{{ item.context.substring(0, 80) }}..."
          </div>
          <div class="vocab-meta">
            <span class="vocab-date">{{ formatDate(item.createdAt) }}</span>
            <button class="vocab-delete" @click="emit('delete', item.id)" title="Delete">🗑️</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  animation: fadeIn 0.2s ease-out;
}

.vocab-modal {
  background: var(--bg-secondary);
  border: 1px solid var(--border-visible);
  border-radius: var(--radius-xl);
  width: 520px;
  max-width: 90vw;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow-lg);
}

.modal-header {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-lg);
  border-bottom: 1px solid var(--border-subtle);
}
.modal-header h2 {
  font-size: var(--font-size-lg);
  font-weight: 600;
  color: var(--text-primary);
  flex: 1;
}
.vocab-count {
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
  background: var(--bg-tertiary);
  padding: 2px 8px;
  border-radius: var(--radius-full);
}
.close-btn {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  transition: all var(--transition-fast);
}
.close-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.vocab-empty {
  padding: var(--space-2xl);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-sm);
  color: var(--text-tertiary);
}
.vocab-empty-icon {
  font-size: 3rem;
}
.vocab-empty-hint {
  font-size: var(--font-size-xs);
}

.vocab-list {
  overflow-y: auto;
  padding: var(--space-md);
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.vocab-item {
  padding: var(--space-sm) var(--space-md);
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-subtle);
  transition: border-color var(--transition-fast);
}
.vocab-item:hover {
  border-color: var(--border-visible);
}

.vocab-main {
  display: flex;
  align-items: baseline;
  gap: var(--space-sm);
}
.vocab-word {
  font-weight: 600;
  color: var(--accent-primary);
  font-size: var(--font-size-base);
}
.vocab-translation {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
}

.vocab-context {
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
  font-style: italic;
  margin-top: var(--space-xs);
}

.vocab-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: var(--space-xs);
}
.vocab-date {
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
}
.vocab-delete {
  opacity: 0.3;
  font-size: 0.8rem;
  transition: opacity var(--transition-fast);
}
.vocab-delete:hover {
  opacity: 1;
}
</style>
