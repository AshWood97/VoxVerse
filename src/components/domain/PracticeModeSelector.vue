<script setup lang="ts">
import type { PracticeMode } from '../../types/practice';

defineProps<{
  modes: PracticeMode[];
  activeId: string;
}>();

const emit = defineEmits<{
  select: [id: string];
  close: [];
}>();
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="mode-modal">
      <div class="modal-header">
        <div>
          <p class="eyebrow">Practice goal</p>
          <h2>Choose a Practice Mode</h2>
        </div>
        <button class="close-btn" @click="emit('close')">x</button>
      </div>

      <div class="mode-list">
        <button
          v-for="mode in modes"
          :key="mode.id"
          class="mode-card"
          :class="{ 'mode-card--active': mode.id === activeId }"
          type="button"
          @click="emit('select', mode.id)"
        >
          <span class="mode-name">{{ mode.name }}</span>
          <span class="mode-desc">{{ mode.description }}</span>
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

.mode-modal {
  background: linear-gradient(145deg, var(--bg-secondary), var(--bg-primary));
  border: 1px solid var(--border-visible);
  border-radius: var(--radius-xl);
  width: 560px;
  max-width: 92vw;
  max-height: 82vh;
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow-lg);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-md);
  padding: var(--space-lg);
  border-bottom: 1px solid var(--border-subtle);
}

.eyebrow {
  margin: 0 0 4px;
  color: var(--accent-secondary);
  font-size: var(--font-size-xs);
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.modal-header h2 {
  margin: 0;
  color: var(--text-primary);
  font-size: var(--font-size-lg);
  font-weight: 700;
}

.close-btn {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm);
  color: var(--text-tertiary);
  transition: all var(--transition-fast);
}

.close-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.mode-list {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-md);
  padding: var(--space-lg);
  overflow-y: auto;
}

.mode-card {
  min-height: 132px;
  padding: var(--space-lg);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  text-align: left;
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  cursor: pointer;
  transition: transform var(--transition-fast), border-color var(--transition-fast), background var(--transition-fast);
}

.mode-card:hover {
  background: var(--bg-hover);
  border-color: var(--border-visible);
  transform: translateY(-2px);
}

.mode-card--active {
  border-color: var(--accent-secondary);
  background: var(--accent-secondary-soft);
}

.mode-name {
  color: var(--text-primary);
  font-size: var(--font-size-base);
  font-weight: 700;
}

.mode-desc {
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  line-height: 1.5;
}

@media (max-width: 620px) {
  .mode-list {
    grid-template-columns: 1fr;
  }
}

@keyframes fadeIn {
  from { opacity: 0; transform: scale(0.98); }
  to { opacity: 1; transform: scale(1); }
}
</style>
