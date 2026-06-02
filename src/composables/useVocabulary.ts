import { ref, computed } from 'vue';
import type { VocabularyItem } from '../types/feedback';
import { getVocabulary, saveVocabulary, deleteVocabulary } from '../services/tauri/vocabulary';
import { logError } from '../utils/errors';

const vocabularyState = ref<VocabularyItem[]>([]);

export function useVocabulary() {
  async function loadVocabulary() {
    try {
      vocabularyState.value = await getVocabulary();
    } catch (error) {
      logError('Failed to load vocabulary', error);
    }
  }

  async function addWord(item: VocabularyItem) {
    await saveVocabulary(item);
    await loadVocabulary();
  }

  /**
   * Quick-add a word/phrase with minimal info.
   * Generates ID and timestamp automatically.
   */
  async function quickAdd(wordOrPhrase: string, translation?: string, context?: string, characterId?: string) {
    const item: VocabularyItem = {
      id: `vocab-${Date.now()}`,
      wordOrPhrase,
      translation,
      context,
      characterId,
      createdAt: new Date().toISOString(),
    };
    await addWord(item);
    return item;
  }

  async function removeWord(id: string) {
    await deleteVocabulary(id);
    await loadVocabulary();
  }

  async function updateWord(item: VocabularyItem) {
    await saveVocabulary(item);
    await loadVocabulary();
  }

  return {
    vocabulary: computed(() => vocabularyState.value),
    loadVocabulary,
    addWord,
    quickAdd,
    removeWord,
    updateWord,
  };
}
