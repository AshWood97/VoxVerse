<script setup lang="ts">
import { ref } from 'vue';
import type { Message } from '../../types/chat';

const props = defineProps<{
  message: Message;
  characterAvatar?: string;
  characterName?: string;
}>();

const emit = defineEmits<{
  play: [];
  correct: [text: string];
  translate: [text: string];
  polish: [text: string];
}>();

const isUser = props.message.role === 'user';
const timeStr = new Date(props.message.timestamp).toLocaleTimeString([], {
  hour: '2-digit',
  minute: '2-digit',
});

const showMenu = ref(false);

function toggleMenu() {
  showMenu.value = !showMenu.value;
}

function handleAction(action: string) {
  showMenu.value = false;
  if (action === 'correct') emit('correct', props.message.content);
  if (action === 'translate') emit('translate', props.message.content);
  if (action === 'polish') emit('polish', props.message.content);
}
</script>

<template>
  <div class="bubble-row" :class="{ 'bubble-row--user': isUser }">
    <div v-if="!isUser" class="bubble-avatar">{{ characterAvatar || '🤖' }}</div>
    <div class="bubble-content">
      <div v-if="!isUser" class="bubble-name">{{ characterName || 'AI' }}</div>
      <div class="bubble" :class="isUser ? 'bubble--user' : 'bubble--ai'">
        <p class="bubble-text">{{ message.content }}</p>
      </div>
      <div class="bubble-actions">
        <div class="bubble-time">{{ timeStr }}</div>
        <button
          v-if="!isUser && message.content"
          class="bubble-action-btn"
          @click="emit('play')"
          title="Play message"
        >▶️</button>
        <!-- Quick actions menu trigger -->
        <div v-if="message.content" class="quick-menu-wrap">
          <button
            class="bubble-action-btn bubble-menu-btn"
            @click="toggleMenu"
            title="Actions"
          >···</button>
          <transition name="pop">
            <div v-if="showMenu" class="quick-menu" @mouseleave="showMenu = false">
              <button v-if="isUser" @click="handleAction('correct')">✏️ Check Grammar</button>
              <button v-if="isUser" @click="handleAction('polish')">✨ Polish</button>
              <button @click="handleAction('translate')">🌐 Translate</button>
            </div>
          </transition>
        </div>
      </div>
    </div>
    <div v-if="isUser" class="bubble-avatar bubble-avatar--user">🎤</div>
  </div>
</template>

<style scoped>
.bubble-row {
  display: flex;
  align-items: flex-start;
  gap: var(--space-sm);
  padding: var(--space-xs) var(--space-md);
  animation: fadeIn var(--transition-normal) ease-out;
}
.bubble-row--user {
  flex-direction: row-reverse;
}

.bubble-avatar {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-full);
  background: var(--bg-elevated);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.2rem;
  flex-shrink: 0;
  border: 1px solid var(--border-subtle);
}
.bubble-avatar--user {
  background: var(--accent-primary);
}

.bubble-content {
  max-width: 70%;
  display: flex;
  flex-direction: column;
}
.bubble-row--user .bubble-content {
  align-items: flex-end;
}

.bubble-name {
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
  margin-bottom: 2px;
  padding-left: var(--space-xs);
}

.bubble {
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-lg);
  line-height: 1.6;
  word-break: break-word;
}
.bubble--ai {
  background: var(--bubble-ai);
  color: var(--bubble-ai-text);
  border-bottom-left-radius: var(--radius-sm);
}
.bubble--user {
  background: var(--bubble-user);
  color: var(--bubble-user-text);
  border-bottom-right-radius: var(--radius-sm);
}

.bubble-text {
  font-size: var(--font-size-sm);
  white-space: pre-wrap;
}

.bubble-actions {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  margin-top: 2px;
  padding: 0 var(--space-xs);
}

.bubble-time {
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
}

.bubble-action-btn {
  background: none;
  border: none;
  font-size: 0.9rem;
  opacity: 0.4;
  cursor: pointer;
  transition: opacity var(--transition-fast), transform var(--transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2px;
}
.bubble-action-btn:hover {
  opacity: 1;
  transform: scale(1.1);
}

.bubble-menu-btn {
  font-weight: 700;
  letter-spacing: 1px;
  font-size: 0.8rem;
  color: var(--text-tertiary);
}

/* Quick menu */
.quick-menu-wrap {
  position: relative;
}
.quick-menu {
  position: absolute;
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  background: var(--bg-elevated);
  border: 1px solid var(--border-visible);
  border-radius: var(--radius-md);
  padding: var(--space-xs);
  display: flex;
  flex-direction: column;
  gap: 2px;
  box-shadow: var(--shadow-md);
  z-index: 10;
  min-width: 140px;
}
.quick-menu button {
  padding: var(--space-xs) var(--space-sm);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-xs);
  color: var(--text-secondary);
  text-align: left;
  white-space: nowrap;
  transition: background var(--transition-fast), color var(--transition-fast);
}
.quick-menu button:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

/* Pop animation for menu */
.pop-enter-active {
  animation: popIn 0.15s ease-out;
}
.pop-leave-active {
  animation: popIn 0.1s ease-in reverse;
}
@keyframes popIn {
  from { opacity: 0; transform: translateX(-50%) scale(0.9); }
  to { opacity: 1; transform: translateX(-50%) scale(1); }
}
</style>

