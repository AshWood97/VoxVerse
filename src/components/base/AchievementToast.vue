<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import type { Achievement } from '../../composables/useAchievements';

defineProps<{
  achievement: Achievement | null;
}>();

const { t } = useI18n();
</script>

<template>
  <Transition name="toast">
    <div v-if="achievement" class="achievement-toast">
      <div class="toast-icon">
        <span class="icon">{{ achievement.icon }}</span>
      </div>
      <div class="toast-content">
        <span class="toast-title">{{ t('achievements.unlocked') }}</span>
        <h4 class="achievement-name">{{ achievement.title }}</h4>
        <p class="achievement-desc">{{ achievement.description }}</p>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.achievement-toast {
  position: fixed;
  top: var(--space-xl);
  right: var(--space-xl);
  width: 340px;
  background: var(--bg-secondary);
  border: 2px solid var(--accent-secondary);
  border-radius: var(--radius-lg);
  box-shadow: 0 0 20px var(--accent-secondary-soft), var(--shadow-lg);
  padding: var(--space-sm) var(--space-md);
  display: flex;
  align-items: center;
  gap: var(--space-md);
  z-index: 9999;
  overflow: hidden;
}

.achievement-toast::before {
  content: '';
  position: absolute;
  top: 0; left: 0; right: 0; bottom: 0;
  background: linear-gradient(135deg, var(--accent-secondary-soft) 0%, transparent 100%);
  z-index: 0;
  pointer-events: none;
}

.toast-icon {
  flex-shrink: 0;
  width: 54px;
  height: 54px;
  background: var(--bg-tertiary);
  border-radius: 50%;
  border: 1px solid var(--border-visible);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.8rem;
  z-index: 1;
  animation: bounceIn 0.6s cubic-bezier(0.175, 0.885, 0.32, 1.275) both;
  animation-delay: 0.1s;
}

@keyframes bounceIn {
  0% { transform: scale(0); opacity: 0; }
  50% { transform: scale(1.1); opacity: 1; }
  100% { transform: scale(1); opacity: 1; }
}

.toast-content {
  display: flex;
  flex-direction: column;
  z-index: 1;
}

.toast-title {
  font-size: 0.7rem;
  text-transform: uppercase;
  font-weight: 700;
  color: var(--accent-secondary);
  letter-spacing: 0.5px;
  margin-bottom: 2px;
}

.achievement-name {
  font-size: var(--font-size-base);
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
  line-height: 1.2;
}

.achievement-desc {
  font-size: var(--font-size-xs);
  color: var(--text-secondary);
  margin-top: 4px;
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(100px) scale(0.8);
}

.toast-leave-to {
  opacity: 0;
  transform: translateY(-50px) scale(0.9);
}
</style>
