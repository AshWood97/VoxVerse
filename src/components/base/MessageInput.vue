<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useSpeechRecognition } from '../../composables/useSpeechRecognition';
import type { VoiceRuntimeTone } from '../../types/voice';
import RuntimeStatusBadge from './RuntimeStatusBadge.vue';

const emit = defineEmits<{
  send: [content: string];
}>();

const props = defineProps<{
  disabled?: boolean;
  lang?: string;
}>();

const inputText = ref('');
const textareaRef = ref<HTMLTextAreaElement | null>(null);
const { t } = useI18n();

const {
  isSupported,
  isListening,
  transcript,
  error,
  recognitionNotice,
  activeRecognitionEngine,
  recognitionStatus,
  toggle,
} = useSpeechRecognition();

const recognitionEngineLabel = computed(() => {
  if (activeRecognitionEngine.value === 'whisper') {
    return t('voiceRuntime.engine.whisper');
  }

  if (activeRecognitionEngine.value === 'browser') {
    return t('voiceRuntime.engine.browserStt');
  }

  return '';
});

const micTitle = computed(() => {
  if (recognitionStatus.value === 'transcribing') {
    return t('voiceRuntime.input.transcribingTitle');
  }

  if (isListening.value && recognitionEngineLabel.value) {
    return t('voiceRuntime.input.stop', { engine: recognitionEngineLabel.value });
  }

  return t('voiceRuntime.input.clickToSpeak');
});

const isTranscribing = computed(() => recognitionStatus.value === 'transcribing');

const runtimeLabel = computed(() => {
  if (recognitionStatus.value === 'recording' && recognitionEngineLabel.value) {
    return t('voiceRuntime.input.recording', { engine: recognitionEngineLabel.value });
  }

  if (recognitionStatus.value === 'transcribing') {
    return t('voiceRuntime.input.transcribing');
  }

  if (recognitionStatus.value === 'fallback') {
    return t('voiceRuntime.input.fallbackReady');
  }

  if (recognitionStatus.value === 'error') {
    return t('voiceRuntime.input.needsAttention');
  }

  return '';
});

const runtimeTone = computed<VoiceRuntimeTone>(() => {
  if (recognitionStatus.value === 'fallback') {
    return 'fallback';
  }

  if (recognitionStatus.value === 'error') {
    return 'error';
  }

  return 'active';
});

const inputPlaceholder = computed(() => {
  if (isTranscribing.value) {
    return t('voiceRuntime.input.transcribingPlaceholder');
  }

  if (isListening.value) {
    return t('voiceRuntime.input.listeningPlaceholder');
  }

  return t('voiceRuntime.input.textPlaceholder');
});

function resizeTextarea() {
  if (!textareaRef.value) {
    return;
  }

  textareaRef.value.style.height = 'auto';
  textareaRef.value.style.height = `${Math.min(textareaRef.value.scrollHeight, 120)}px`;
}

watch(transcript, (newText) => {
  if (!newText) {
    return;
  }

  const current = inputText.value.trim();
  inputText.value = current ? `${current} ${newText}` : newText;
  transcript.value = '';
  resizeTextarea();
});

function handleMicClick() {
  if (isTranscribing.value) {
    return;
  }

  toggle(props.lang || 'en-US');
}

function handleSend() {
  if (isListening.value) {
    toggle(props.lang || 'en-US');
  }

  const trimmed = inputText.value.trim();
  if (!trimmed || props.disabled) return;

  emit('send', trimmed);
  inputText.value = '';
  resizeTextarea();
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault();
    handleSend();
  }
}

function autoResize(event: Event) {
  const target = event.target as HTMLTextAreaElement;
  target.style.height = 'auto';
  target.style.height = `${Math.min(target.scrollHeight, 120)}px`;
}
</script>

<template>
  <div class="input-wrapper">
    <div v-if="recognitionNotice" class="mic-notice">{{ recognitionNotice }}</div>
    <div v-if="error" class="mic-error">{{ error }}</div>
    <RuntimeStatusBadge
      v-if="runtimeLabel"
      :label="runtimeLabel"
      :tone="runtimeTone"
      variant="bar"
    />

    <div class="input-container" :class="{ 'is-listening': isListening }">
      <button
        v-if="isSupported"
        class="mic-btn"
        :class="{ 'mic-btn--active': isListening }"
        type="button"
        :disabled="isTranscribing"
        :title="micTitle"
        @click="handleMicClick"
      >
        <div v-if="isListening" class="pulse-ring"></div>
        {{ t('voiceRuntime.input.mic') }}
      </button>

      <textarea
        ref="textareaRef"
        v-model="inputText"
        class="input-field"
        :disabled="disabled"
        :placeholder="inputPlaceholder"
        rows="1"
        @keydown="handleKeydown"
        @input="autoResize"
      />

      <button
        class="send-btn"
        type="button"
        :disabled="disabled || !inputText.trim()"
        @click="handleSend"
      >
        <span v-if="disabled" class="loading-dots">
          <span>.</span><span>.</span><span>.</span>
        </span>
        <span v-else>{{ t('voiceRuntime.input.send') }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.input-wrapper {
  display: flex;
  flex-direction: column;
}

.mic-error,
.mic-notice {
  padding: 4px 16px;
  font-size: var(--font-size-xs);
  background: var(--bg-secondary);
}

.mic-error {
  color: var(--accent-danger);
}

.mic-notice {
  color: #f59e0b;
}

.input-container {
  display: flex;
  align-items: flex-end;
  gap: var(--space-sm);
  padding: var(--space-md);
  background: var(--bg-secondary);
  border-top: 1px solid var(--border-subtle);
  transition: all var(--transition-fast);
}

.input-container.is-listening {
  background: rgba(231, 76, 60, 0.05);
  border-top-color: rgba(231, 76, 60, 0.2);
}

.mic-btn {
  position: relative;
  width: 42px;
  height: 42px;
  border-radius: var(--radius-full);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: var(--font-size-xs);
  font-weight: 700;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  transition: all var(--transition-fast);
  flex-shrink: 0;
}

.mic-btn:hover {
  background: var(--bg-hover);
  border-color: var(--border-visible);
}

.mic-btn:disabled {
  opacity: 0.5;
  cursor: wait;
}

.mic-btn--active {
  background: var(--accent-danger);
  border-color: var(--accent-danger);
  color: white;
}

.pulse-ring {
  position: absolute;
  inset: -4px;
  border-radius: 50%;
  border: 2px solid var(--accent-danger);
  animation: pulse-ring 1.5s cubic-bezier(0.215, 0.61, 0.355, 1) infinite;
}

.input-field {
  flex: 1;
  padding: var(--space-sm) var(--space-md);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: var(--font-size-sm);
  resize: none;
  min-height: 42px;
  max-height: 120px;
  transition: border-color var(--transition-fast);
  line-height: 1.5;
}

.input-field:focus {
  outline: none;
  border-color: var(--accent-primary);
}

.input-field::placeholder {
  color: var(--text-tertiary);
}

.input-field:disabled {
  opacity: 0.5;
}

.send-btn {
  min-width: 54px;
  height: 42px;
  padding: 0 var(--space-sm);
  border-radius: var(--radius-md);
  background: var(--accent-gradient);
  color: var(--text-on-accent);
  font-size: var(--font-size-sm);
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: opacity var(--transition-fast), transform var(--transition-fast);
  flex-shrink: 0;
}

.send-btn:hover:not(:disabled) {
  opacity: 0.9;
  transform: scale(1.03);
}

.send-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.loading-dots span {
  animation: pulse 1.4s infinite;
  font-weight: 700;
}

.loading-dots span:nth-child(2) {
  animation-delay: 0.2s;
}

.loading-dots span:nth-child(3) {
  animation-delay: 0.4s;
}
</style>
