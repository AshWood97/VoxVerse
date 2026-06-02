<script setup lang="ts">
import { computed } from 'vue';
import type { ChatSession } from '../../services/tauri/session';
import type { Scenario } from '../../types/feedback';
import type { PracticeMode } from '../../types/practice';

const props = defineProps<{
  sessions: ChatSession[];
  activeSessionId: string | null;
  practiceModes?: PracticeMode[];
  scenarios?: Scenario[];
}>();

const emit = defineEmits<{
  select: [id: string];
  delete: [id: string];
  close: [];
}>();

// Sort by recently updated first
const sortedSessions = computed(() => {
  return [...props.sessions].sort((a, b) => {
    return new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime();
  });
});

function formatDate(iso: string): string {
  const date = new Date(iso);
  return date.toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function modeName(id: string | null | undefined): string {
  return props.practiceModes?.find((mode) => mode.id === (id || 'free_talk'))?.name || 'Free Talk';
}

function scenarioName(id: string | null | undefined): string {
  if (!id) return 'No scenario';
  return props.scenarios?.find((scenario) => scenario.id === id)?.name || id;
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="history-modal">
      <div class="modal-header">
        <h2>🕰️ Chat History</h2>
        <button class="close-btn" @click="emit('close')">✕</button>
      </div>

      <div class="history-list" v-if="sortedSessions.length > 0">
        <div 
          v-for="session in sortedSessions" 
          :key="session.id"
          class="history-item"
          :class="{ 'history-item--active': session.id === activeSessionId }"
          @click="emit('select', session.id)"
        >
          <div class="history-main">
            <span class="history-title">{{ session.title || 'Conversation' }}</span>
            <span class="history-context">
              {{ modeName(session.modeId) }} / {{ scenarioName(session.scenarioId) }}
            </span>
            <span class="history-date">{{ formatDate(session.updatedAt) }}</span>
          </div>
          <button 
            type="button" 
            class="history-delete" 
            @click.stop="emit('delete', session.id)"
            title="Delete Session"
          >
            🗑️
          </button>
        </div>
      </div>
      
      <div v-else class="history-empty">
        <p>No chat history available for this character.</p>
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

.history-modal {
  background: var(--bg-secondary);
  border: 1px solid var(--border-visible);
  border-radius: var(--radius-xl);
  width: 480px;
  max-width: 90vw;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow-lg);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-lg);
  border-bottom: 1px solid var(--border-subtle);
}
.modal-header h2 {
  font-size: var(--font-size-lg);
  font-weight: 600;
  color: var(--text-primary);
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

.history-list {
  padding: var(--space-md);
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
  overflow-y: auto;
}

.history-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-md);
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  border: 1px solid transparent;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.history-item:hover {
  background: var(--bg-hover);
  border-color: var(--border-subtle);
}
.history-item--active {
  border-color: var(--accent-primary);
  background: rgba(108, 92, 231, 0.05);
}

.history-main {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}
.history-title {
  font-size: var(--font-size-sm);
  font-weight: 500;
  color: var(--text-primary);
}
.history-date {
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
}

.history-context {
  color: var(--accent-secondary);
  font-size: var(--font-size-xs);
  font-weight: 600;
}

.history-delete {
  background: none;
  border: none;
  font-size: 1.1rem;
  opacity: 0.3;
  color: #e74c3c;
  cursor: pointer;
  transition: opacity var(--transition-fast);
  padding: var(--space-xs);
}
.history-item:hover .history-delete {
  opacity: 0.7;
}
.history-delete:hover {
  opacity: 1 !important;
}

.history-empty {
  padding: var(--space-2xl);
  text-align: center;
  color: var(--text-tertiary);
  font-size: var(--font-size-sm);
}

@keyframes fadeIn {
  from { opacity: 0; transform: scale(0.98); }
  to { opacity: 1; transform: scale(1); }
}
</style>
