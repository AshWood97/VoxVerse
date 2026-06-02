import { ref, computed } from 'vue';
import type { Message } from '../types/chat';
import {
  type ChatSession,
  getSessions,
  getSessionMessages,
  createSession,
  updateSessionContext,
  updateSessionTitle,
  deleteSession,
  saveMessage,
} from '../services/tauri/session';
import { logError } from '../utils/errors';

const SESSION_TITLE_MAX_LENGTH = 48;

function buildSessionTitle(content: string): string {
  const normalized = content.replace(/\s+/g, ' ').trim();
  if (!normalized) {
    return 'Conversation';
  }

  if (normalized.length <= SESSION_TITLE_MAX_LENGTH) {
    return normalized;
  }

  return `${normalized.slice(0, SESSION_TITLE_MAX_LENGTH - 1).trimEnd()}…`;
}

export function useSession() {
  const sessions = ref<ChatSession[]>([]);
  const currentSessionId = ref<string | null>(null);
  const currentMessages = ref<Message[]>([]);

  /**
   * Load metadata for all sessions of a specific character.
   */
  async function loadSessions(characterId: string) {
    try {
      sessions.value = await getSessions(characterId);
    } catch (error) {
      logError('Failed to load sessions', error);
    }
  }

  function clearCurrentSession() {
    currentSessionId.value = null;
    currentMessages.value = [];
  }

  /**
   * Load complete conversation for a specific session.
   */
  async function loadSessionData(sessionId: string) {
    try {
      const msgs = await getSessionMessages(sessionId);
      currentMessages.value = msgs;
      currentSessionId.value = sessionId;
    } catch (error) {
      logError(`Failed to load messages for session ${sessionId}`, error);
    }
  }

  /**
   * Starts a new blank session.
   */
  async function startNewSession(
    characterId: string,
    modeId: string | null = null,
    scenarioId: string | null = null,
  ) {
    const newId = `sess-${Date.now()}`;
    const now = new Date().toISOString();
    const sess: ChatSession = {
      id: newId,
      characterId,
      title: null,
      modeId,
      scenarioId,
      createdAt: now,
      updatedAt: now,
      messages: [],
    };
    
    await createSession(sess);
    currentSessionId.value = newId;
    currentMessages.value = [];
    
    // Refresh list so it shows at the top
    await loadSessions(characterId);
    return newId;
  }

  async function updateCurrentSessionContext(modeId: string | null, scenarioId: string | null) {
    if (!currentSessionId.value) {
      return null;
    }

    const context = await updateSessionContext(currentSessionId.value, modeId, scenarioId);
    const session = sessions.value.find((item) => item.id === currentSessionId.value);
    if (session) {
      session.modeId = context.modeId;
      session.scenarioId = context.scenarioId;
      session.updatedAt = new Date().toISOString();
    }
    return context;
  }

  /**
   * Save a single message to the current session.
   */
  async function addMessage(msg: Message) {
    if (!currentSessionId.value) {
      throw new Error('No active session is available for this message.');
    }
    
    // Ensure message has sessionId
    const msgToSave = { ...msg, sessionId: currentSessionId.value };
    
    // Persist first so the UI does not show messages that failed to save.
    await saveMessage(msgToSave);
    currentMessages.value.push(msgToSave);
    
    // Update local session list timestamp
    const sessToUpdate = sessions.value.find(s => s.id === currentSessionId.value);
    if (sessToUpdate) {
      sessToUpdate.updatedAt = new Date().toISOString();

      if (msg.role === 'user' && !sessToUpdate.title) {
        const title = buildSessionTitle(msg.content);
        sessToUpdate.title = title;
        try {
          await updateSessionTitle(currentSessionId.value, title);
        } catch (error) {
          logError(`Failed to update title for session ${currentSessionId.value}`, error);
        }
      }
    }
  }

  /**
   * Delete a session and clear current state if it was active.
   */
  async function removeSession(id: string, currentCharacterId: string) {
    await deleteSession(id);
    if (currentSessionId.value === id) {
      currentSessionId.value = null;
      currentMessages.value = [];
    }
    await loadSessions(currentCharacterId);
  }

  return {
    sessions: computed(() => sessions.value),
    currentSessionId: computed(() => currentSessionId.value),
    currentMessages: computed(() => currentMessages.value),
    loadSessions,
    loadSessionData,
    clearCurrentSession,
    startNewSession,
    updateCurrentSessionContext,
    addMessage,
    removeSession,
    updateTitle: updateSessionTitle,
  };
}
