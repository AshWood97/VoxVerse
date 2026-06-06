import { invoke } from '@tauri-apps/api/core';
import type { RelationshipState } from '../../types/memory';

export interface RelationshipUpdatePayload {
  intimacy_level?: number;
  trust_level?: number;
  plot_stage?: string;
  learningGoal?: string;
  user_preferences?: Record<string, unknown>;
  boundaries?: Record<string, unknown>;
  commitments?: Record<string, unknown>;
}

export async function getRelationshipState(
  characterId: string
): Promise<RelationshipState> {
  return invoke<RelationshipState>('get_relationship_state', { characterId });
}

export async function updateRelationshipState(
  characterId: string,
  updates: RelationshipUpdatePayload
): Promise<RelationshipState> {
  return invoke<RelationshipState>('update_relationship_state', {
    characterId,
    updates,
  });
}
