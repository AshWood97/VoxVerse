<script setup lang="ts">
import type { VoiceRuntimeTone } from '../../types/voice';

withDefaults(defineProps<{
  label: string;
  detail?: string;
  tone?: VoiceRuntimeTone;
  variant?: 'bar' | 'pill';
}>(), {
  detail: '',
  tone: 'active',
  variant: 'pill',
});
</script>

<template>
  <div
    class="runtime-badge"
    :class="[
      `runtime-badge--${variant}`,
      `runtime-badge--${tone}`,
    ]"
  >
    <span class="runtime-badge-dot"></span>
    <span class="runtime-badge-label">{{ label }}</span>
    <span v-if="detail" class="runtime-badge-detail">{{ detail }}</span>
  </div>
</template>

<style scoped>
.runtime-badge {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  font-size: var(--font-size-xs);
  color: var(--accent-secondary);
}

.runtime-badge--bar {
  padding: 4px 16px;
  background: var(--bg-secondary);
}

.runtime-badge--pill {
  min-height: 28px;
  padding: 0 var(--space-sm);
  border: 1px solid var(--accent-secondary-border);
  border-radius: var(--radius-full);
  background: var(--accent-secondary-soft);
}

.runtime-badge--fallback {
  color: var(--status-warning);
}

.runtime-badge--pill.runtime-badge--fallback {
  border-color: var(--status-warning-border);
  background: var(--status-warning-soft);
}

.runtime-badge--error {
  color: var(--accent-danger);
}

.runtime-badge--pill.runtime-badge--error {
  border-color: var(--status-danger-border);
  background: var(--status-danger-soft);
}

.runtime-badge-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
  animation: pulse 1s infinite;
  flex-shrink: 0;
}

.runtime-badge-label {
  font-weight: 700;
}

.runtime-badge--bar .runtime-badge-label {
  font-weight: 500;
}

.runtime-badge-detail {
  color: var(--text-tertiary);
}
</style>
