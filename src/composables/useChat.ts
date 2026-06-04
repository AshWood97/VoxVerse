import { ref, type Ref } from 'vue';
import type { Message, Character } from '../types/chat';
import type { SpeechEngine, SpeechStatus } from '../types/voice';
import { sendSessionMessage } from '../services/tauri/chat';
import { useSpeechSynthesis } from './useSpeechSynthesis';

export interface UseChatReturn {
  isLoading: Ref<boolean>;
  error: Ref<string | null>;
  speechError: Ref<string | null>;
  speechNotice: Ref<string | null>;
  streamingContent: Ref<string>;
  autoPlayTTS: Ref<boolean>;
  isSpeaking: Ref<boolean>;
  activeSpeechEngine: Ref<SpeechEngine | null>;
  speechStatus: Ref<SpeechStatus>;
  send: (
    content: string,
    character: Character,
    sessionId: string,
    scenarioPrompt: string,
    currentMessages: Message[],
    addMessage: (msg: Message) => Promise<void>
  ) => Promise<boolean>;
  retryLastResponse: (
    character: Character,
    sessionId: string,
    scenarioPrompt: string,
    currentMessages: Message[],
    addMessage: (msg: Message) => Promise<void>
  ) => Promise<boolean>;
  initWithGreeting: (
    character: Character,
    sessionId: string,
    addMessage: (msg: Message) => Promise<void>
  ) => Promise<void>;
  stopSpeaking: () => void;
  playText: (text: string, character: Character) => void;
}

let messageCounter = 0;
function createId(): string {
  messageCounter += 1;
  return `msg-${Date.now()}-${messageCounter}`;
}

export function useChat(): UseChatReturn {
  const isLoading = ref(false);
  const error = ref<string | null>(null);
  const streamingContent = ref('');
  const autoPlayTTS = ref(true);

  const {
    speak,
    stop: stopSpeaking,
    isSpeaking,
    activeSpeechEngine,
    speechStatus,
    speechError,
    speechNotice,
  } = useSpeechSynthesis();

  async function initWithGreeting(
    character: Character,
    sessionId: string,
    addMessage: (msg: Message) => Promise<void>
  ) {
    if (character.greeting) {
      const greetingMsg: Message = {
        id: createId(),
        role: 'assistant',
        content: character.greeting,
        timestamp: Date.now(),
        sessionId,
      };
      await addMessage(greetingMsg);
      
      if (autoPlayTTS.value && character.voiceConfig) {
        speak(character.greeting, character.voiceConfig);
      }
    }
  }

  async function requestAssistantResponse(
    character: Character,
    sessionId: string,
    scenarioPrompt: string,
    currentMessages: Message[],
    addMessage: (msg: Message) => Promise<void>,
  ): Promise<boolean> {
    isLoading.value = true;
    streamingContent.value = '';

    const history = currentMessages
      .filter((m) => m.role !== 'system')
      .map((m) => ({
        role: m.role,
        content: m.content,
      }));

    const finalSystemPrompt = character.systemPrompt + scenarioPrompt;

    try {
      const fullResponse = await sendSessionMessage(
        finalSystemPrompt,
        history,
        (token) => {
          streamingContent.value += token;
        },
        {
          characterId: character.id,
          sessionId,
        },
      );

      const assistantMsg: Message = {
        id: createId(),
        role: 'assistant',
        content: fullResponse || streamingContent.value,
        timestamp: Date.now(),
        sessionId,
      };
      await addMessage(assistantMsg);

      if (autoPlayTTS.value && character.voiceConfig && fullResponse) {
        speak(fullResponse, character.voiceConfig);
      }

      error.value = null;
      return true;
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
      return false;
    } finally {
      isLoading.value = false;
      streamingContent.value = '';
    }
  }

  async function send(
    content: string,
    character: Character,
    sessionId: string,
    scenarioPrompt: string,
    currentMessages: Message[],
    addMessage: (msg: Message) => Promise<void>
  ): Promise<boolean> {
    if (!content.trim() || isLoading.value) return false;

    error.value = null;

    const userMsg: Message = {
      id: createId(),
      role: 'user',
      content: content.trim(),
      timestamp: Date.now(),
      sessionId,
    };

    try {
      await addMessage(userMsg);
      return await requestAssistantResponse(character, sessionId, scenarioPrompt, currentMessages, addMessage);
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
      return false;
    }
  }

  async function retryLastResponse(
    character: Character,
    sessionId: string,
    scenarioPrompt: string,
    currentMessages: Message[],
    addMessage: (msg: Message) => Promise<void>,
  ): Promise<boolean> {
    if (isLoading.value) return false;
    const lastMessage = currentMessages[currentMessages.length - 1];
    if (!lastMessage || lastMessage.role !== 'user') {
      error.value = 'No saved user message is available to retry.';
      return false;
    }

    error.value = null;
    return requestAssistantResponse(character, sessionId, scenarioPrompt, currentMessages, addMessage);
  }

  function playText(text: string, character: Character) {
    if (character.voiceConfig) {
      speak(text, character.voiceConfig);
    }
  }

  return {
    isLoading,
    error,
    speechError,
    speechNotice,
    streamingContent,
    autoPlayTTS,
    isSpeaking,
    activeSpeechEngine,
    speechStatus,
    send,
    retryLastResponse,
    initWithGreeting,
    stopSpeaking,
    playText,
  };
}
