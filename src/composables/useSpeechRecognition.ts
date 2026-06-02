import { onUnmounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getConfig } from '../services/tauri/chat';
import type { RecognitionEngine, RecognitionStatus } from '../types/voice';
import { logError, toErrorMessage } from '../utils/errors';

type UiLanguage = 'en' | 'zh';

const WHISPER_MAX_RECORDING_MS = 90_000;
const WHISPER_MAX_AUDIO_BYTES = 25 * 1024 * 1024;

interface BrowserSpeechRecognitionAlternative {
  transcript: string;
}

interface BrowserSpeechRecognitionResult {
  readonly isFinal: boolean;
  readonly length: number;
  [index: number]: BrowserSpeechRecognitionAlternative;
}

interface BrowserSpeechRecognitionResultList {
  readonly length: number;
  [index: number]: BrowserSpeechRecognitionResult;
}

interface BrowserSpeechRecognitionEvent {
  readonly resultIndex: number;
  readonly results: BrowserSpeechRecognitionResultList;
}

interface BrowserSpeechRecognitionErrorEvent {
  readonly error?: string;
  readonly message?: string;
}

interface BrowserSpeechRecognition {
  continuous: boolean;
  interimResults: boolean;
  lang: string;
  onresult: ((event: BrowserSpeechRecognitionEvent) => void) | null;
  onerror: ((event: BrowserSpeechRecognitionErrorEvent) => void) | null;
  onend: (() => void) | null;
  start: () => void;
  stop: () => void;
  abort: () => void;
}

interface BrowserSpeechRecognitionConstructor {
  new (): BrowserSpeechRecognition;
}

interface WindowWithSpeechRecognition extends Window {
  SpeechRecognition?: BrowserSpeechRecognitionConstructor;
  webkitSpeechRecognition?: BrowserSpeechRecognitionConstructor;
}

function getUiLanguage(): UiLanguage {
  return localStorage.getItem('app_language') === 'zh' ? 'zh' : 'en';
}

function browserUnavailableMessage() {
  return getUiLanguage() === 'zh'
    ? '当前 WebView 不支持浏览器语音识别。'
    : 'Browser speech recognition is unavailable in this WebView.';
}

function browserFallbackNotice(reason: string) {
  return getUiLanguage() === 'zh'
    ? `${reason} 已切换到浏览器语音识别。`
    : `${reason} Switched to browser speech recognition.`;
}

function whisperRetryNotice(reason: string) {
  return getUiLanguage() === 'zh'
    ? `${reason} 浏览器语音识别已准备好，请再次点击麦克风重试。`
    : `${reason} Browser speech recognition is ready; click the microphone again to retry.`;
}

function getBrowserRecognitionConstructor() {
  if (typeof window === 'undefined') {
    return undefined;
  }

  const speechWindow = window as WindowWithSpeechRecognition;
  return speechWindow.SpeechRecognition || speechWindow.webkitSpeechRecognition;
}

function canUseBrowserRecognition() {
  return Boolean(getBrowserRecognitionConstructor());
}

function canUseWhisperRecording() {
  return typeof navigator !== 'undefined'
    && Boolean(navigator.mediaDevices?.getUserMedia)
    && typeof MediaRecorder !== 'undefined';
}

export function useSpeechRecognition() {
  const isListening = ref(false);
  const isSupported = ref(canUseWhisperRecording() || canUseBrowserRecognition());
  const transcript = ref('');
  const error = ref<string | null>(null);
  const recognitionNotice = ref<string | null>(null);
  const activeRecognitionEngine = ref<RecognitionEngine | null>(null);
  const recognitionStatus = ref<RecognitionStatus>('idle');

  let mediaRecorder: MediaRecorder | null = null;
  let audioChunks: Blob[] = [];
  let currentStream: MediaStream | null = null;
  let browserRecognition: BrowserSpeechRecognition | null = null;
  let preferBrowserRecognition = false;
  let whisperStopTimer: number | null = null;

  function cleanupStream() {
    currentStream?.getTracks().forEach((track) => track.stop());
    currentStream = null;
  }

  function clearWhisperStopTimer() {
    if (whisperStopTimer !== null) {
      window.clearTimeout(whisperStopTimer);
      whisperStopTimer = null;
    }
  }

  function clearStatus() {
    error.value = null;
    recognitionNotice.value = null;
    if (recognitionStatus.value === 'error' || recognitionStatus.value === 'fallback') {
      recognitionStatus.value = 'idle';
    }
  }

  function stopBrowserRecognition() {
    if (!browserRecognition) {
      return;
    }

    browserRecognition.stop();
  }

  function startBrowserRecognition(lang: string, notice?: string) {
    const RecognitionConstructor = getBrowserRecognitionConstructor();
    if (!RecognitionConstructor) {
      error.value = browserUnavailableMessage();
      activeRecognitionEngine.value = null;
      recognitionStatus.value = 'error';
      isListening.value = false;
      return;
    }

    const recognition = new RecognitionConstructor();
    let finalTranscript = '';
    let ended = false;

    browserRecognition = recognition;
    activeRecognitionEngine.value = 'browser';
    recognitionStatus.value = 'recording';
    recognitionNotice.value = notice || null;
    error.value = null;

    recognition.continuous = false;
    recognition.interimResults = true;
    recognition.lang = lang;

    recognition.onresult = (event) => {
      for (let i = event.resultIndex; i < event.results.length; i += 1) {
        const result = event.results[i];
        const alternative = result[0];
        if (result.isFinal && alternative?.transcript) {
          finalTranscript = `${finalTranscript} ${alternative.transcript}`.trim();
        }
      }
    };

    recognition.onerror = (event) => {
      const message = event.message || event.error || 'Browser speech recognition failed.';
      error.value = getUiLanguage() === 'zh'
        ? `浏览器语音识别失败: ${message}`
        : `Browser STT Error: ${message}`;
      recognitionStatus.value = 'error';
    };

    recognition.onend = () => {
      if (ended) {
        return;
      }

      ended = true;
      if (finalTranscript) {
        transcript.value = finalTranscript;
      }

      browserRecognition = null;
      activeRecognitionEngine.value = null;
      if (recognitionStatus.value !== 'error') {
        recognitionStatus.value = 'idle';
      }
      isListening.value = false;
    };

    try {
      recognition.start();
      isListening.value = true;
    } catch (errorCause) {
      browserRecognition = null;
      activeRecognitionEngine.value = null;
      recognitionStatus.value = 'error';
      isListening.value = false;
      error.value = `Browser STT Error: ${toErrorMessage(errorCause)}`;
      logError('Failed to start browser speech recognition', errorCause);
    }
  }

  async function startWhisperRecording() {
    currentStream = await navigator.mediaDevices.getUserMedia({ audio: true });
    mediaRecorder = new MediaRecorder(currentStream);
    activeRecognitionEngine.value = 'whisper';
    recognitionStatus.value = 'recording';

    mediaRecorder.ondataavailable = (event) => {
      if (event.data.size > 0) {
        audioChunks.push(event.data);
      }
    };

    mediaRecorder.onstop = async () => {
      clearWhisperStopTimer();
      isListening.value = false;
      cleanupStream();

      if (audioChunks.length === 0) {
        mediaRecorder = null;
        activeRecognitionEngine.value = null;
        recognitionStatus.value = 'idle';
        return;
      }

      const audioBlob = new Blob(audioChunks, { type: mediaRecorder?.mimeType || 'audio/webm' });

      try {
        if (audioBlob.size > WHISPER_MAX_AUDIO_BYTES) {
          throw new Error(
            `Recording is too large (${audioBlob.size} bytes). Please keep it under 25 MB.`,
          );
        }

        const arrayBuffer = await audioBlob.arrayBuffer();
        const audioData = Array.from(new Uint8Array(arrayBuffer));

        recognitionStatus.value = 'transcribing';
        transcript.value = await invoke<string>('transcribe', {
          audioData,
          mimeType: audioBlob.type,
        });
      } catch (errorCause) {
        const message = logError('Whisper STT error', errorCause);
        if (canUseBrowserRecognition()) {
          preferBrowserRecognition = true;
          recognitionStatus.value = 'fallback';
          recognitionNotice.value = whisperRetryNotice(`Whisper STT failed: ${message}.`);
          error.value = null;
        } else {
          recognitionStatus.value = 'error';
          error.value = `Whisper STT Error: ${message}`;
        }
      } finally {
        isListening.value = false;
        audioChunks = [];
        mediaRecorder = null;
        activeRecognitionEngine.value = null;
        if (recognitionStatus.value === 'transcribing') {
          recognitionStatus.value = 'idle';
        }
      }
    };

    mediaRecorder.start(1000);
    whisperStopTimer = window.setTimeout(() => {
      if (mediaRecorder && mediaRecorder.state !== 'inactive') {
        mediaRecorder.stop();
      }
    }, WHISPER_MAX_RECORDING_MS);
    isListening.value = true;
    recognitionStatus.value = 'recording';
  }

  onUnmounted(() => {
    stop();
  });

  async function start(lang: string = 'en-US') {
    if (isListening.value) return;

    transcript.value = '';
    recognitionStatus.value = 'idle';
    clearStatus();
    audioChunks = [];

    if (preferBrowserRecognition) {
      startBrowserRecognition(lang, recognitionNotice.value || undefined);
      return;
    }

    try {
      const config = await getConfig();
      if (!config.supports_stt) {
        startBrowserRecognition(
          lang,
          browserFallbackNotice('Whisper STT is unavailable for the current provider.'),
        );
        return;
      }

      if (config.requires_api_key && !config.has_api_key) {
        startBrowserRecognition(
          lang,
          browserFallbackNotice('Whisper STT needs an API key.'),
        );
        return;
      }

      if (!canUseWhisperRecording()) {
        startBrowserRecognition(
          lang,
          browserFallbackNotice('Whisper recording is unavailable in this WebView.'),
        );
        return;
      }

      await startWhisperRecording();
    } catch (errorCause) {
      cleanupStream();
      logError('Failed to start Whisper speech recognition', errorCause);

      if (canUseBrowserRecognition()) {
        startBrowserRecognition(
          lang,
          browserFallbackNotice(`Whisper setup failed: ${toErrorMessage(errorCause)}.`),
        );
        return;
      }

      error.value = `Failed to access microphone: ${toErrorMessage(errorCause)}`;
      recognitionStatus.value = 'error';
    }
  }

  function stop() {
    if (browserRecognition) {
      stopBrowserRecognition();
      return;
    }

    if (mediaRecorder && mediaRecorder.state !== 'inactive') {
      mediaRecorder.stop();
      return;
    }

    clearWhisperStopTimer();
    cleanupStream();
    mediaRecorder = null;
    activeRecognitionEngine.value = null;
    recognitionStatus.value = 'idle';
    isListening.value = false;
  }

  function toggle(lang: string = 'en-US') {
    if (isListening.value) {
      stop();
    } else {
      start(lang);
    }
  }

  return {
    isSupported,
    isListening,
    transcript,
    error,
    recognitionNotice,
    activeRecognitionEngine,
    recognitionStatus,
    start,
    stop,
    toggle,
  };
}
