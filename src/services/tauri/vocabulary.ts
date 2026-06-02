import { invoke } from '@tauri-apps/api/core';
import type { VocabularyItem } from '../../types/feedback';

export async function getVocabulary(): Promise<VocabularyItem[]> {
  return invoke<VocabularyItem[]>('get_vocabulary');
}

export async function saveVocabulary(item: VocabularyItem): Promise<void> {
  await invoke('save_vocabulary', { item });
}

export async function deleteVocabulary(id: string): Promise<void> {
  await invoke('delete_vocabulary', { id });
}
