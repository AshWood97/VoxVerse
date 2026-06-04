<script setup lang="ts">
import type { Scenario } from '../../types/feedback';

defineProps<{
  scenarios: Scenario[];
  activeId: string | null;
}>();

const emit = defineEmits<{
  select: [id: string | null];
  close: [];
}>();
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="scenario-modal">
      <div class="modal-header">
        <h2>🎬 Choose a Scenario</h2>
        <button class="close-btn" @click="emit('close')">✕</button>
      </div>
      <div class="scenario-grid">
        <!-- Clear selection option -->
        <button
          class="scenario-card"
          :class="{ 'scenario-card--active': activeId === null }"
          @click="emit('select', null)"
        >
          <span class="scenario-icon">💬</span>
          <span class="scenario-name">Free Chat</span>
          <span class="scenario-desc">No specific scenario</span>
        </button>

        <button
          v-for="s in scenarios"
          :key="s.id"
          class="scenario-card"
          :class="{ 'scenario-card--active': s.id === activeId }"
          @click="emit('select', s.id)"
        >
          <span class="scenario-icon">{{ s.icon }}</span>
          <span class="scenario-name">{{ s.name }}</span>
          <span class="scenario-desc">{{ s.description.substring(0, 60) }}...</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: var(--overlay-bg);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  animation: fadeIn 0.2s ease-out;
}

.scenario-modal {
  background: var(--bg-secondary);
  border: 1px solid var(--border-visible);
  border-radius: var(--radius-xl);
  width: 640px;
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

.scenario-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: var(--space-md);
  padding: var(--space-lg);
  overflow-y: auto;
}

.scenario-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-xs);
  padding: var(--space-lg) var(--space-md);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  text-align: center;
  transition: all var(--transition-fast);
  cursor: pointer;
}
.scenario-card:hover {
  background: var(--bg-hover);
  border-color: var(--border-visible);
  transform: translateY(-2px);
}
.scenario-card--active {
  border-color: var(--accent-primary);
  background: var(--accent-soft);
}

.scenario-icon {
  font-size: 2rem;
}
.scenario-name {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--text-primary);
}
.scenario-desc {
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
  line-height: 1.4;
}
</style>
