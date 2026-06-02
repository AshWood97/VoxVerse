import { invoke } from '@tauri-apps/api/core';

export interface VoiceInfo {
  name: string;
  friendly_name: string;
  locale: string;
  gender: string;
}

export async function synthesizeSpeech(text: string, voiceName: string): Promise<Uint8Array> {
  return invoke<Uint8Array>('synthesize_speech', { text, voiceName });
}

export async function getTtsVoices(): Promise<VoiceInfo[]> {
  return invoke<VoiceInfo[]>('get_tts_voices');
}
