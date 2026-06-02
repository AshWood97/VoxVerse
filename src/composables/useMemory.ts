import { ref } from 'vue';
import type { MemoryFact } from '../types/memory';
import { getMemoryFacts, createMemoryFact, deleteMemoryFact, toggleMemoryVisibility } from '../services/tauri/memory';
import { logError } from '../utils/errors';

export function useMemory() {
  const facts = ref<MemoryFact[]>([]);
  const isLoading = ref(false);

  async function loadMemoryFacts(characterId: string) {
    if (!characterId) return;
    isLoading.value = true;
    try {
      facts.value = await getMemoryFacts(characterId);
    } catch (error) {
      logError('Failed to load memory facts', error);
    } finally {
      isLoading.value = false;
    }
  }

  async function addFact(characterId: string, factType: string, content: string) {
    try {
      const newFact = await createMemoryFact(characterId, factType, content);
      facts.value = [newFact, ...facts.value];
      return newFact;
    } catch (error) {
      logError('Failed to add memory fact', error);
      throw error;
    }
  }

  async function removeFact(factId: string) {
    try {
      const success = await deleteMemoryFact(factId);
      if (success) {
        facts.value = facts.value.filter(f => f.id !== factId);
      }
      return success;
    } catch (error) {
      logError('Failed to delete memory fact', error);
      throw error;
    }
  }

  async function toggleVisibility(factId: string, visible: boolean) {
    try {
      const success = await toggleMemoryVisibility(factId, visible);
      if (success) {
        facts.value = facts.value.map(f => {
          if (f.id === factId) {
            return { ...f, is_visible: visible };
          }
          return f;
        });
      }
      return success;
    } catch (error) {
      logError('Failed to toggle memory visibility', error);
      throw error;
    }
  }

  return {
    facts,
    isLoading,
    loadMemoryFacts,
    addFact,
    removeFact,
    toggleVisibility,
  };
}
