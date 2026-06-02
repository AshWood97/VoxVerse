import { ref } from 'vue';
import type { RelationshipState } from '../types/memory';
import { getRelationshipState, updateRelationshipState, type RelationshipUpdatePayload } from '../services/tauri/relationship';
import { logError } from '../utils/errors';

export function useRelationship() {
  const state = ref<RelationshipState | null>(null);
  const isLoading = ref(false);

  async function loadRelationshipState(characterId: string) {
    if (!characterId) return;
    isLoading.value = true;
    try {
      state.value = await getRelationshipState(characterId);
    } catch (error) {
      logError('Failed to load relationship state', error);
    } finally {
      isLoading.value = false;
    }
  }

  async function modifyRelationshipState(characterId: string, updates: RelationshipUpdatePayload) {
    isLoading.value = true;
    try {
      state.value = await updateRelationshipState(characterId, updates);
    } catch (error) {
      logError('Failed to update relationship state', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  }

  return {
    relationshipState: state,
    isLoading,
    loadRelationshipState,
    modifyRelationshipState,
  };
}
