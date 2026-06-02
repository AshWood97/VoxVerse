import { invoke } from '@tauri-apps/api/core';
import type { Character } from '../../types/chat';

export async function getCharacters(): Promise<Character[]> {
  return invoke<Character[]>('get_characters');
}

export async function saveCharacter(character: Character): Promise<void> {
  await invoke('save_character', { character });
}

export async function deleteCharacter(id: string): Promise<void> {
  await invoke('delete_character', { id });
}
