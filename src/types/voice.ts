export type RecognitionEngine = 'whisper' | 'browser';

export type RecognitionStatus =
  | 'idle'
  | 'recording'
  | 'transcribing'
  | 'fallback'
  | 'error';

export type SpeechEngine = 'edge' | 'browser';

export type SpeechStatus =
  | 'idle'
  | 'speaking'
  | 'fallback'
  | 'error';

export type VoiceRuntimeTone =
  | 'active'
  | 'thinking'
  | 'speaking'
  | 'fallback'
  | 'error';
