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
  border: 1px solid rgba(0, 206, 201, 0.2);
  border-radius: var(--radius-full);
  background: rgba(0, 206, 201, 0.08);
}

.runtime-badge--fallback {
  color: #f59e0b;
}

.runtime-badge--pill.runtime-badge--fallback {
  border-color: rgba(245, 158, 11, 0.25);
  background: rgba(245, 158, 11, 0.1);
}

.runtime-badge--error {
  color: var(--accent-danger);
}

.runtime-badge--pill.runtime-badge--error {
  border-color: rgba(231, 76, 60, 0.25);
  background: rgba(231, 76, 60, 0.1);
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
