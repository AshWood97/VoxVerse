import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { SpeechEngine, SpeechStatus } from '../types/voice';
import { logError } from '../utils/errors';

interface WindowWithWebkitAudioContext extends Window {
  webkitAudioContext?: typeof AudioContext;
}

type UiLanguage = 'en' | 'zh';

const isSpeaking = ref(false);
const speechError = ref<string | null>(null);
const speechNotice = ref<string | null>(null);
const activeSpeechEngine = ref<SpeechEngine | null>(null);
const speechStatus = ref<SpeechStatus>('idle');
const TTS_MAX_TEXT_CHARS = 4_000;

let audioContext: AudioContext | null = null;
let currentSource: AudioBufferSourceNode | null = null;

function getUiLanguage(): UiLanguage {
  return localStorage.getItem('app_language') === 'zh' ? 'zh' : 'en';
}

function createFallbackNotice(edgeMessage: string) {
  if (getUiLanguage() === 'zh') {
    return `Edge TTS 不可用，已自动切换到浏览器语音。${edgeMessage}`;
  }

  return `Edge TTS unavailable, switched to browser speech. ${edgeMessage}`;
}

function createSpeechFailureMessage(edgeMessage: string, browserMessage: string) {
  if (getUiLanguage() === 'zh') {
    return `语音播放失败。Edge TTS: ${edgeMessage}。浏览器回退: ${browserMessage}`;
  }

  return `Speech playback failed. Edge TTS: ${edgeMessage}. Browser fallback: ${browserMessage}`;
}

function createTruncatedNotice() {
  if (getUiLanguage() === 'zh') {
    return `TTS 文本过长，本次只朗读前 ${TTS_MAX_TEXT_CHARS} 个字符。`;
  }

  return `TTS text is long, so only the first ${TTS_MAX_TEXT_CHARS} characters will be spoken.`;
}

function limitSpeechText(text: string) {
  const trimmed = text.trim();
  const chars = Array.from(trimmed);

  if (chars.length <= TTS_MAX_TEXT_CHARS) {
    return { text: trimmed, truncated: false };
  }

  return {
    text: chars.slice(0, TTS_MAX_TEXT_CHARS).join(''),
    truncated: true,
  };
}

function initAudioContext() {
  if (typeof window !== 'undefined' && !audioContext) {
    const audioContextConstructor =
      window.AudioContext || (window as WindowWithWebkitAudioContext).webkitAudioContext;

    if (audioContextConstructor) {
      audioContext = new audioContextConstructor();
    }
  }
}

function clearStatus() {
  speechError.value = null;
  speechNotice.value = null;
  if (speechStatus.value === 'error' || speechStatus.value === 'fallback') {
    speechStatus.value = 'idle';
  }
}

function stopBrowserSpeech() {
  if (typeof window === 'undefined' || !('speechSynthesis' in window)) {
    return;
  }

  window.speechSynthesis.cancel();
}

function resolveVoiceName(voiceConfig: { lang: string; namePattern?: string }) {
  const savedVoice = localStorage.getItem('tts_voice');

  let defaultVoice = 'en-US-AriaNeural';
  if (voiceConfig.lang.startsWith('zh')) {
    defaultVoice = 'zh-CN-XiaoxiaoNeural';
  } else if (voiceConfig.lang.startsWith('ja')) {
    defaultVoice = 'ja-JP-NanamiNeural';
  } else if (voiceConfig.lang.startsWith('en-GB')) {
    defaultVoice = 'en-GB-SoniaNeural';
  }

  return voiceConfig.namePattern || savedVoice || defaultVoice;
}

async function speakWithEdgeTts(text: string, voiceName: string) {
  initAudioContext();

  if (!audioContext) {
    throw new Error('Audio output is unavailable in this WebView.');
  }

  if (audioContext.state === 'suspended') {
    await audioContext.resume();
  }

  const bytes: Uint8Array = await invoke('synthesize_speech', { text, voiceName });
  if (!bytes || bytes.length === 0) {
    throw new Error('Received empty audio from Edge TTS.');
  }

  const audioBuffer = await audioContext.decodeAudioData(bytes.buffer.slice(0));
  currentSource = audioContext.createBufferSource();
  currentSource.buffer = audioBuffer;
  currentSource.connect(audioContext.destination);

  await new Promise<void>((resolve, reject) => {
    if (!currentSource) {
      reject(new Error('Audio playback source was not initialized.'));
      return;
    }

    currentSource.onended = () => {
      currentSource?.disconnect();
      currentSource = null;
      resolve();
    };

    currentSource.start(0);
  });
}

async function loadBrowserVoices() {
  if (typeof window === 'undefined' || !('speechSynthesis' in window)) {
    return [] as SpeechSynthesisVoice[];
  }

  const synth = window.speechSynthesis;
  const existingVoices = synth.getVoices();
  if (existingVoices.length > 0) {
    return existingVoices;
  }

  return new Promise<SpeechSynthesisVoice[]>((resolve) => {
    const finish = () => {
      cleanup();
      resolve(synth.getVoices());
    };

    const cleanup = () => {
      window.clearTimeout(timeoutId);
      synth.removeEventListener?.('voiceschanged', finish);
    };

    const timeoutId = window.setTimeout(finish, 600);
    synth.addEventListener?.('voiceschanged', finish);
  });
}

function pickBrowserVoice(
  voices: SpeechSynthesisVoice[],
  lang: string,
  voiceName: string,
) {
  const lowerVoiceName = voiceName.toLowerCase();
  const normalizedLang = lang.toLowerCase();
  const langPrefix = normalizedLang.split('-')[0];

  return (
    voices.find((voice) => voice.name.toLowerCase() === lowerVoiceName)
    || voices.find((voice) => voice.lang.toLowerCase() === normalizedLang)
    || voices.find((voice) => voice.lang.toLowerCase().startsWith(`${langPrefix}-`))
    || voices.find((voice) => voice.default)
    || voices[0]
  );
}

async function speakWithBrowserFallback(
  text: string,
  voiceConfig: { lang: string; namePattern?: string },
  voiceName: string,
) {
  if (typeof window === 'undefined' || !('speechSynthesis' in window)) {
    throw new Error('Browser speech synthesis is unavailable.');
  }

  const synth = window.speechSynthesis;
  const voices = await loadBrowserVoices();
  const matchedVoice = pickBrowserVoice(voices, voiceConfig.lang, voiceName);
  const utterance = new SpeechSynthesisUtterance(text);

  utterance.lang = matchedVoice?.lang || voiceConfig.lang;
  if (matchedVoice) {
    utterance.voice = matchedVoice;
  }

  synth.cancel();

  await new Promise<void>((resolve, reject) => {
    utterance.onend = () => {
      resolve();
    };
    utterance.onerror = (event) => {
      reject(new Error(event.error || 'Browser speech synthesis failed.'));
    };

    synth.speak(utterance);
  });
}

export function useSpeechSynthesis() {
  function stop() {
    currentSource?.stop();
    currentSource?.disconnect();
    currentSource = null;
    stopBrowserSpeech();
    activeSpeechEngine.value = null;
    speechStatus.value = 'idle';
    isSpeaking.value = false;
  }

  async function speak(text: string, voiceConfig: { lang: string; namePattern?: string }) {
    stop();
    clearStatus();

    const limitedSpeech = limitSpeechText(text);

    if (!limitedSpeech.text) {
      activeSpeechEngine.value = null;
      speechStatus.value = 'idle';
      return null;
    }

    const truncatedNotice = limitedSpeech.truncated ? createTruncatedNotice() : null;
    const voiceName = resolveVoiceName(voiceConfig);

    try {
      isSpeaking.value = true;
      activeSpeechEngine.value = 'edge';
      speechStatus.value = 'speaking';
      await speakWithEdgeTts(limitedSpeech.text, voiceName);
      speechNotice.value = truncatedNotice;
      speechStatus.value = 'idle';
      return 'edge' as const;
    } catch (edgeErrorCause) {
      const edgeMessage = logError('Edge TTS synthesis error', edgeErrorCause);

      try {
        activeSpeechEngine.value = 'browser';
        speechStatus.value = 'fallback';
        await speakWithBrowserFallback(limitedSpeech.text, voiceConfig, voiceName);
        speechNotice.value = [truncatedNotice, createFallbackNotice(edgeMessage)]
          .filter(Boolean)
          .join(' ');
        speechStatus.value = 'idle';
        return 'browser' as const;
      } catch (browserErrorCause) {
        const browserMessage = logError('Browser speech synthesis fallback error', browserErrorCause);
        activeSpeechEngine.value = null;
        speechStatus.value = 'error';
        speechError.value = createSpeechFailureMessage(edgeMessage, browserMessage);
        return null;
      }
    } finally {
      isSpeaking.value = false;
    }
  }

  return {
    isSpeaking,
    speechError,
    speechNotice,
    activeSpeechEngine,
    speechStatus,
    speak,
    stop,
    clearStatus,
  };
}
