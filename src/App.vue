<script setup lang="ts">
import { computed, ref, watch, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import SideBar from './components/domain/SideBar.vue';
import ChatPanel from './components/domain/ChatPanel.vue';
import SettingsModal from './components/domain/SettingsModal.vue';
import CharacterModal from './components/domain/CharacterModal.vue';
import FeedbackPanel from './components/domain/FeedbackPanel.vue';
import PracticeModeSelector from './components/domain/PracticeModeSelector.vue';
import ScenarioSelector from './components/domain/ScenarioSelector.vue';
import VocabularyModal from './components/domain/VocabularyModal.vue';

import SessionHistory from './components/domain/SessionHistory.vue';
import StatsPanel from './components/domain/StatsPanel.vue';
import CharacterMarketModal from './components/domain/CharacterMarketModal.vue';
import MemoryPanel from './components/domain/MemoryPanel.vue';
import RelationshipCard from './components/domain/RelationshipCard.vue';
import AchievementToast from './components/base/AchievementToast.vue';

import { useAchievements } from './composables/useAchievements';

import { useCharacter } from './composables/useCharacter';
import { useChat } from './composables/useChat';
import { useSession } from './composables/useSession';
import { usePracticeMode } from './composables/usePracticeMode';
import { useScenario } from './composables/useScenario';
import { useVocabulary } from './composables/useVocabulary';
import { useFeedback } from './composables/useFeedback';
import { useStats } from './composables/useStats';
import type { Character } from './types/chat';
import { logError, toErrorMessage } from './utils/errors';

const { characters, loadCharacters, getCharacterById, addOrUpdateCharacter, removeCharacter, exportCharacter, importCharacter } = useCharacter();

// Lv.3 Composables
const {
  sessions,
  currentSessionId,
  currentMessages,
  startNewSession,
  loadSessions,
  loadSessionData,
  clearCurrentSession,
  removeSession,
  addMessage,
  updateCurrentSessionContext,
} = useSession();

const {
  practiceModes,
  activePracticeModeId,
  activePracticeMode,
  loadPracticeModes,
  selectPracticeMode,
  getPracticeModePromptSuffix,
} = usePracticeMode();

const {
  scenarios,
  activeScenarioId,
  loadScenarios,
  selectScenario,
  getScenarioPromptSuffix,
} = useScenario();

const {
  vocabulary,
  loadVocabulary,
  quickAdd,
  removeWord,
} = useVocabulary();

const feedback = useFeedback();
const statsComposable = useStats();
const { t } = useI18n();
const { recentUnlock, monitor } = useAchievements();

monitor(statsComposable.stats);

const { 
  isLoading, 
  error, 
  speechError,
  speechNotice,
  send, 
  initWithGreeting, 
  retryLastResponse,
  autoPlayTTS, 
  isSpeaking, 
  activeSpeechEngine,
  speechStatus,
  stopSpeaking,
  playText
} = useChat();

const activeCharacterId = ref('');
const activeCharacter = ref<Character | null>(null);
const canRetryLastMessage = computed(() => (
  Boolean(error.value)
  && !isLoading.value
  && currentMessages.value[currentMessages.value.length - 1]?.role === 'user'
));

const showSettings = ref(false);
const showCharacterModal = ref(false);
const showPracticeModeModal = ref(false);
const showScenarioModal = ref(false);
const showVocabModal = ref(false);
const showHistoryModal = ref(false);
const showStatsModal = ref(false);
const showMarketModal = ref(false);
const showMemoryModal = ref(false);
const showRelationshipModal = ref(false);

const editingCharacter = ref<Character | null>(null);
const isReady = ref(false);
const fileInputRef = ref<HTMLInputElement | null>(null);

onMounted(async () => {
  await Promise.all([
    loadCharacters(),
    loadPracticeModes(),
    loadScenarios(),
    loadVocabulary()
  ]);

  if (characters.value.length > 0) {
    await startConversationForCharacter(characters.value[0], true);
  }
  
  // Initial stats load to check for existing achievements silently
  await statsComposable.loadStats();
  
  isReady.value = true;
});

// Handle character switching
watch(activeCharacterId, async (newId) => {
  if (!isReady.value) return;
  const char = getCharacterById(newId);
  if (char) {
    await startConversationForCharacter(char);
  }
});

async function startConversationForCharacter(char: Character, syncActiveId: boolean = false) {
  if (syncActiveId) {
    activeCharacterId.value = char.id;
  }

  activeCharacter.value = char;
  await loadSessions(char.id);

  if (sessions.value.length > 0) {
    await loadSessionAndContext(sessions.value[0].id);
  } else {
    clearCurrentSession();
    selectPracticeMode('free_talk');
    selectScenario(null);
  }
}

async function loadSessionAndContext(sessionId: string) {
  await loadSessionData(sessionId);
  const session = sessions.value.find((item) => item.id === sessionId);
  selectPracticeMode(session?.modeId || 'free_talk');
  selectScenario(session?.scenarioId || null);
}

async function persistCurrentSessionContext() {
  if (!currentSessionId.value) return;

  try {
    await updateCurrentSessionContext(activePracticeModeId.value, activeScenarioId.value);
  } catch (error) {
    logError('Failed to persist session context', error);
  }
}

async function handleNewSession() {
  if (!activeCharacter.value) return;
  const sessionId = await startNewSession(
    activeCharacter.value.id,
    activePracticeModeId.value,
    activeScenarioId.value,
  );
  await initWithGreeting(activeCharacter.value, sessionId, addMessage);
}

async function handleLoadSession(sessionId: string) {
  showHistoryModal.value = false;
  await loadSessionAndContext(sessionId);
}

async function handleDeleteSession(sessionId: string) {
  if (!activeCharacter.value) return;
  if (confirm(t('app.deleteSessionConfirm'))) {
    await removeSession(sessionId, activeCharacter.value.id);
    // If we deleted the active one, pick the next most recent or create a new one
    if (!currentSessionId.value) {
      if (sessions.value.length > 0) {
        await loadSessionAndContext(sessions.value[0].id);
      } else {
        clearCurrentSession();
      }
    }
  }
}

function handleCreateCharacter() {
  editingCharacter.value = null;
  showCharacterModal.value = true;
}

function handleEditCharacter(id: string) {
  const char = getCharacterById(id);
  if (char) {
    editingCharacter.value = char;
    showCharacterModal.value = true;
  }
}

async function handleDeleteCharacter(id: string) {
  if (confirm(t('sidebar.confirmDelete'))) {
    await removeCharacter(id);
    if (activeCharacterId.value === id && characters.value.length > 0) {
      await startConversationForCharacter(characters.value[0], true);
    }
  }
}

async function handleSaveCharacter(char: Character) {
  await addOrUpdateCharacter(char);
  showCharacterModal.value = false;
  
  if (activeCharacterId.value === char.id || characters.value.length === 1) {
    activeCharacterId.value = char.id;
    activeCharacter.value = char;
  }
}

async function handleSend(content: string) {
  if (activeCharacter.value) {
    const sessionId = currentSessionId.value || await startNewSession(
      activeCharacter.value.id,
      activePracticeModeId.value,
      activeScenarioId.value,
    );
    const promptSuffix = getPracticeModePromptSuffix() + getScenarioPromptSuffix();
    const sent = await send(
      content, 
      activeCharacter.value, 
      sessionId, 
      promptSuffix, 
      currentMessages.value, 
      addMessage
    );
    // Refresh stats after message sent
    if (sent) {
      statsComposable.loadStats();
      void feedback.runCorrection(content, sessionId).then(() => statsComposable.loadStats());
    }
  }
}

async function handleRetryLastMessage() {
  if (!activeCharacter.value || !currentSessionId.value) return;
  const promptSuffix = getPracticeModePromptSuffix() + getScenarioPromptSuffix();
  const sent = await retryLastResponse(
    activeCharacter.value,
    currentSessionId.value,
    promptSuffix,
    currentMessages.value,
    addMessage,
  );

  if (sent) {
    statsComposable.loadStats();
  }
}

// Top Bar Actions
async function handlePracticeModeSelect(id: string) {
  selectPracticeMode(id);
  showPracticeModeModal.value = false;
  await persistCurrentSessionContext();
}

async function handleScenarioSelect(id: string | null) {
  selectScenario(id);
  showScenarioModal.value = false;
  await persistCurrentSessionContext();
}

async function handleSaveVocab(word: string, meaning: string, context: string) {
  await quickAdd(word, meaning, context, activeCharacterId.value);
  statsComposable.loadStats(); // Trigger achievements check
  alert(t('app.vocabularySaved', { word }));
}

async function handleSummary() {
  if (!currentMessages.value.length) return;
  const tuples: [string, string][] = currentMessages.value
    .filter(m => m.role !== 'system')
    .map(m => [m.role, m.content]);
  await feedback.runSummary(tuples, currentSessionId.value);
}

function triggerImport() {
  fileInputRef.value?.click();
}

async function handleFileImport(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) return;
  try {
    const char = await importCharacter(file);
    await startConversationForCharacter(char, true);
    alert(t('app.characterImported'));
  } catch (error) {
    alert(toErrorMessage(error, t('app.characterImportFailed')));
  } finally {
    if (fileInputRef.value) fileInputRef.value.value = '';
  }
}

async function handleMarketImport(char: Character) {
  try {
    await addOrUpdateCharacter(char);
    await startConversationForCharacter(char, true);
  } catch (error) {
    logError('Failed to import character from market', error);
  }
}
</script>

<template>
  <div v-if="isReady && activeCharacter" class="app-layout">
    <input type="file" ref="fileInputRef" accept=".json,.png" style="display: none" @change="handleFileImport" />

    <SideBar
      :characters="characters"
      :active-id="activeCharacterId"
      @select="activeCharacterId = $event"
      @open-settings="showSettings = true"
      @create="handleCreateCharacter"
      @import="triggerImport"
      @discover="showMarketModal = true"
      @edit="handleEditCharacter"
      @delete="handleDeleteCharacter"
      @export="exportCharacter"
    />

    <div class="main-content">
      <!-- Top ActionBar -->
      <div class="top-action-bar">
        <button class="top-btn" @click="showHistoryModal = true">
          🕰️ {{ t('app.history') }} ({{ sessions.length }})
        </button>
        <button class="top-btn top-btn--mode" @click="showPracticeModeModal = true">
          Mode: {{ activePracticeMode?.name || 'Free Talk' }}
        </button>
        <button class="top-btn" @click="showScenarioModal = true">
          🎬 {{ activeScenarioId ? t('app.changeScenario') : t('app.scenario') }}
        </button>
        <button class="top-btn" @click="showVocabModal = true">
          📚 {{ t('app.vocabulary') }} ({{ vocabulary.length }})
        </button>
        <button class="top-btn accent" @click="handleSummary">
          📊 {{ t('app.report') }}
        </button>
        <!-- Stats / Progress -->
        <button class="top-btn" @click="() => { showStatsModal = true; statsComposable.loadStats(); }" :title="t('app.progress')">
          🏆 {{ t('app.progress') }}
        </button>
        <!-- Memory & Relationship -->
        <button class="top-btn" @click="showMemoryModal = true">
          🧠 Memory
        </button>
        <button class="top-btn" @click="showRelationshipModal = true">
          🎭 Relationship
        </button>
        <!-- Add a "New Chat" quick button -->
        <button class="top-btn" @click="handleNewSession" style="margin-left: auto;">
          ➕ {{ t('app.newChat') }}
        </button>
      </div>

      <ChatPanel
        v-model:auto-play="autoPlayTTS"
        :messages="currentMessages"
        :character="activeCharacter"
        :is-loading="isLoading"
        :error="error"
        :speech-error="speechError"
        :speech-notice="speechNotice"
        :can-retry-last="canRetryLastMessage"
        :is-speaking="isSpeaking"
        :active-speech-engine="activeSpeechEngine"
        :speech-status="speechStatus"
        @send="handleSend"
        @retry-last="handleRetryLastMessage"
        @stop-speaking="stopSpeaking"
        @play-message="playText($event, activeCharacter)"
        @correct="feedback.runCorrection($event, currentSessionId).then(() => statsComposable.loadStats())"
        @translate="feedback.runTranslation($event, t('feedback.targetLanguage')).then(() => statsComposable.loadStats())"
        @polish="feedback.runPolish($event).then(() => statsComposable.loadStats())"
      />
    </div>

    <!-- Right Side Feedback Panel -->
    <FeedbackPanel
      :is-open="feedback.isOpen.value"
      :is-loading="feedback.isLoading.value"
      :action="feedback.action.value"
      :source-text="feedback.sourceText.value"
      :correction-result="feedback.correctionResult.value"
      :translation-result="feedback.translationResult.value"
      :polish-result="feedback.polishResult.value"
      :summary-result="feedback.summaryResult.value"
      :raw-fallback="feedback.rawFallback.value"
      :error="feedback.error.value"
      @close="feedback.close()"
      @save-word="handleSaveVocab"
    />

    <!-- Modals -->
    <SettingsModal v-if="showSettings" @close="showSettings = false" />
    
    <CharacterModal 
      v-if="showCharacterModal" 
      :initial-data="editingCharacter" 
      @close="showCharacterModal = false"
      @save="handleSaveCharacter"
    />

    <PracticeModeSelector
      v-if="showPracticeModeModal"
      :modes="practiceModes"
      :active-id="activePracticeModeId"
      @select="handlePracticeModeSelect"
      @close="showPracticeModeModal = false"
    />

    <ScenarioSelector
      v-if="showScenarioModal"
      :scenarios="scenarios"
      :active-id="activeScenarioId"
      @select="handleScenarioSelect"
      @close="showScenarioModal = false"
    />

    <VocabularyModal
      v-if="showVocabModal"
      :vocabulary="vocabulary"
      @delete="removeWord"
      @close="showVocabModal = false"
    />

    <SessionHistory
      v-if="showHistoryModal"
      :sessions="sessions"
      :active-session-id="currentSessionId"
      :practice-modes="practiceModes"
      :scenarios="scenarios"
      @select="handleLoadSession"
      @delete="handleDeleteSession"
      @close="showHistoryModal = false"
    />

    <StatsPanel
      v-if="showStatsModal"
      :stats="statsComposable.stats.value"
      :is-loading="statsComposable.isLoading.value"
      :error="statsComposable.error.value"
      @close="showStatsModal = false"
    />

    <CharacterMarketModal
      v-if="showMarketModal"
      @close="showMarketModal = false"
      @import="handleMarketImport"
    />

    <MemoryPanel
      v-if="showMemoryModal"
      :character-id="activeCharacterId"
      :character-name="activeCharacter?.name || ''"
      @close="showMemoryModal = false"
    />

    <div v-if="showRelationshipModal" class="modal-overlay-wrapper" @click.self="showRelationshipModal = false">
      <div class="relationship-modal-container">
        <RelationshipCard
          :character-id="activeCharacterId"
          :character-name="activeCharacter?.name || ''"
          @close="showRelationshipModal = false"
        />
      </div>
    </div>

    <AchievementToast :achievement="recentUnlock" />
  </div>
  <div v-else class="loading-screen">
    {{ t('app.loading') }}
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  background: var(--bg-primary);
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.top-action-bar {
  display: flex;
  align-items: center;
  gap: var(--space-md);
  padding: var(--space-sm) var(--space-lg);
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-subtle);
}

.top-btn {
  padding: var(--space-xs) var(--space-md);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  font-size: var(--font-size-sm);
  font-weight: 500;
  color: var(--text-primary);
  cursor: pointer;
  transition: all var(--transition-fast);
}
.top-btn:hover {
  background: var(--bg-hover);
  border-color: var(--border-visible);
}
.top-btn.accent {
  color: var(--accent-primary);
  border-color: rgba(108, 92, 231, 0.3);
  background: rgba(108, 92, 231, 0.05);
}
.top-btn.accent:hover {
  background: rgba(108, 92, 231, 0.1);
  border-color: var(--accent-primary);
}

.top-btn--mode {
  color: var(--accent-secondary);
  border-color: rgba(0, 206, 201, 0.28);
  background: rgba(0, 206, 201, 0.06);
}

.top-btn--mode:hover {
  background: rgba(0, 206, 201, 0.12);
  border-color: var(--accent-secondary);
}

.loading-screen {
  display: flex;
  height: 100vh;
  width: 100vw;
  align-items: center;
  justify-content: center;
  background: var(--bg-primary);
  color: var(--text-tertiary);
  font-size: var(--font-size-lg);
}

.modal-overlay-wrapper {
  position: fixed;
  inset: 0;
  background: rgba(10, 10, 15, 0.75);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  animation: fadeIn 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}

.relationship-modal-container {
  width: 480px;
  max-width: 90vw;
  max-height: 85vh;
  overflow-y: auto;
  border-radius: var(--radius-lg);
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}
</style>
