<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import type { Character, Message } from '../../types/chat';
import type { SpeechEngine, SpeechStatus, VoiceRuntimeTone } from '../../types/voice';
import ChatBubble from '../base/ChatBubble.vue';
import MessageInput from '../base/MessageInput.vue';
import RuntimeStatusBadge from '../base/RuntimeStatusBadge.vue';

const props = defineProps<{
  messages: Message[];
  character: Character;
  isLoading: boolean;
  error: string | null;
  speechError: string | null;
  speechNotice: string | null;
  canRetryLast: boolean;
  autoPlay: boolean;
  isSpeaking: boolean;
  activeSpeechEngine: SpeechEngine | null;
  speechStatus: SpeechStatus;
}>();

const emit = defineEmits<{
  send: [content: string];
  retryLast: [];
  'update:autoPlay': [value: boolean];
  stopSpeaking: [];
  playMessage: [content: string];
  correct: [content: string];
  translate: [content: string];
  polish: [content: string];
}>();

const scrollContainer = ref<HTMLDivElement | null>(null);
const { t } = useI18n();

const speechEngineLabel = computed(() => {
  if (props.activeSpeechEngine === 'edge') {
    return t('voiceRuntime.engine.edgeTts');
  }

  if (props.activeSpeechEngine === 'browser') {
    return t('voiceRuntime.engine.browserTts');
  }

  return t('voiceRuntime.engine.speech');
});

const runtimeStatus = computed(() => {
  if (props.isLoading) {
    return {
      tone: 'thinking' as VoiceRuntimeTone,
      label: t('voiceRuntime.chat.thinking'),
      detail: t('voiceRuntime.chat.streaming'),
    };
  }

  if (props.speechStatus === 'fallback') {
    return {
      tone: 'fallback' as VoiceRuntimeTone,
      label: t('voiceRuntime.chat.fallback'),
      detail: t('voiceRuntime.chat.browserTtsFallback'),
    };
  }

  if (props.isSpeaking || props.speechStatus === 'speaking') {
    return {
      tone: 'speaking' as VoiceRuntimeTone,
      label: t('voiceRuntime.chat.speaking'),
      detail: t('voiceRuntime.chat.engineActive', { engine: speechEngineLabel.value }),
    };
  }

  if (props.speechStatus === 'error') {
    return {
      tone: 'error' as VoiceRuntimeTone,
      label: t('voiceRuntime.chat.speechIssue'),
      detail: t('voiceRuntime.chat.checkMessage'),
    };
  }

  return null;
});

function scrollToBottomImmediate() {
  if (scrollContainer.value) {
    scrollContainer.value.scrollTop = scrollContainer.value.scrollHeight;
  }
}

let scrollThrottle: number | undefined;
function scrollToBottomThrottled() {
  if (scrollThrottle) return;

  scrollThrottle = window.setTimeout(async () => {
    await nextTick();
    scrollToBottomImmediate();
    scrollThrottle = undefined;
  }, 100);
}

watch(
  () => props.messages.length,
  async () => {
    await nextTick();
    scrollToBottomImmediate();
  },
);

watch(
  () => props.messages[props.messages.length - 1]?.content,
  () => {
    scrollToBottomThrottled();
  },
);
</script>

<template>
  <div class="chat-panel">
    <div class="chat-header">
      <span class="chat-header-avatar">{{ character.avatar }}</span>
      <div class="chat-header-info">
        <h2 class="chat-header-name">{{ character.name }}</h2>
        <p class="chat-header-desc">{{ character.language }} - {{ character.personality.split('.')[0] }}</p>
      </div>
      <div class="chat-header-actions">
        <button
          v-if="isSpeaking"
          class="action-btn stop-btn"
          type="button"
          :title="t('voiceRuntime.chat.stopSpeaking')"
          @click="emit('stopSpeaking')"
        >
          {{ t('voiceRuntime.chat.stop') }}
        </button>
        <button
          class="action-btn"
          :class="{ 'action-btn--muted': !autoPlay }"
          type="button"
          :title="autoPlay ? t('voiceRuntime.chat.autoPlayOn') : t('voiceRuntime.chat.autoPlayOff')"
          @click="emit('update:autoPlay', !autoPlay)"
        >
          {{ autoPlay ? t('voiceRuntime.chat.tts') : t('voiceRuntime.chat.muted') }}
        </button>
        <RuntimeStatusBadge
          v-if="runtimeStatus"
          :label="runtimeStatus.label"
          :detail="runtimeStatus.detail"
          :tone="runtimeStatus.tone"
          variant="pill"
        />
      </div>
    </div>

    <div ref="scrollContainer" class="chat-messages">
      <div v-if="messages.length === 0" class="chat-empty">
        <div class="chat-empty-emoji">{{ character.avatar }}</div>
        <p class="chat-empty-text">{{ t('voiceRuntime.chat.empty', { name: character.name }) }}</p>
      </div>

      <ChatBubble
        v-for="msg in messages"
        :key="msg.id"
        :message="msg"
        :character-avatar="character.avatar"
        :character-name="character.name"
        @play="emit('playMessage', msg.content)"
        @correct="emit('correct', $event)"
        @translate="emit('translate', $event)"
        @polish="emit('polish', $event)"
      />
    </div>

    <div v-if="error" class="chat-error">
      <span>{{ error }}</span>
      <button
        v-if="canRetryLast"
        class="chat-error-retry"
        type="button"
        @click="emit('retryLast')"
      >
        {{ t('voiceRuntime.chat.retryLast') }}
      </button>
    </div>

    <div v-if="speechNotice" class="chat-notice">
      {{ speechNotice }}
    </div>

    <div v-if="speechError" class="chat-error chat-error--speech">
      {{ speechError }}
    </div>

    <MessageInput
      :disabled="isLoading"
      :lang="character.voiceConfig?.lang.substring(0, 5) || 'en-US'"
      @send="emit('send', $event)"
    />
  </div>
</template>

<style scoped>
.chat-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
}

.chat-header {
  display: flex;
  align-items: center;
  gap: var(--space-md);
  padding: var(--space-md) var(--space-lg);
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-subtle);
}

.chat-header-avatar {
  font-size: 2rem;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-elevated);
  border-radius: var(--radius-full);
}

.chat-header-info {
  flex: 1;
}

.chat-header-name {
  font-size: var(--font-size-lg);
  font-weight: 600;
  color: var(--text-primary);
}

.chat-header-desc {
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
}

.chat-header-actions {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.action-btn {
  min-width: 36px;
  height: 36px;
  padding: 0 var(--space-sm);
  border-radius: var(--radius-full);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: var(--font-size-xs);
  background: var(--bg-primary);
  border: 1px solid var(--border-subtle);
  transition: all var(--transition-fast);
}

.action-btn:hover {
  background: var(--bg-hover);
}

.action-btn--muted {
  opacity: 0.6;
}

.stop-btn {
  background: rgba(231, 76, 60, 0.1);
  border-color: rgba(231, 76, 60, 0.3);
}

.stop-btn:hover {
  background: rgba(231, 76, 60, 0.2);
}

.chat-messages {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-md) 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.chat-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-md);
}

.chat-empty-emoji {
  font-size: 4rem;
  animation: fadeIn 0.6s ease-out;
}

.chat-empty-text {
  color: var(--text-tertiary);
  font-size: var(--font-size-sm);
}

.chat-error {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-sm);
  padding: var(--space-sm) var(--space-md);
  background: rgba(231, 76, 60, 0.15);
  color: #e74c3c;
  font-size: var(--font-size-sm);
  border-top: 1px solid rgba(231, 76, 60, 0.3);
}

.chat-error-retry {
  padding: 4px 10px;
  border-radius: var(--radius-full);
  background: rgba(231, 76, 60, 0.16);
  border: 1px solid rgba(231, 76, 60, 0.32);
  color: #fecaca;
  font-size: var(--font-size-xs);
  font-weight: 700;
  white-space: nowrap;
}

.chat-error-retry:hover {
  background: rgba(231, 76, 60, 0.26);
}

.chat-error--speech {
  border-top-style: dashed;
}

.chat-notice {
  padding: var(--space-sm) var(--space-md);
  background: rgba(245, 158, 11, 0.14);
  color: #f59e0b;
  font-size: var(--font-size-sm);
  border-top: 1px solid rgba(245, 158, 11, 0.24);
}
</style>
