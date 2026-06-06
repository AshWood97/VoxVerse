import { invoke } from '@tauri-apps/api/core';
import type { MemoryFact } from '../../types/memory';

export interface MemoryFactFilters {
  fact_type?: string;
  include_deleted?: boolean;
  limit?: number;
}

export async function getMemoryFacts(
  characterId: string,
  filters?: MemoryFactFilters
): Promise<MemoryFact[]> {
  return invoke<MemoryFact[]>('get_memory_facts', {
    characterId,
    filters: filters || null,
  });
}

export async function createMemoryFact(
  characterId: string,
  factType: string,
  content: string
): Promise<MemoryFact> {
  return invoke<MemoryFact>('create_memory_fact', {
    characterId,
    factType,
    content,
  });
}

export async function deleteMemoryFact(factId: string): Promise<boolean> {
  return invoke<boolean>('delete_memory_fact', { factId });
}

export async function clearCharacterMemory(characterId: string): Promise<number> {
  return invoke<number>('clear_character_memory', { characterId });
}

export async function toggleMemoryVisibility(
  factId: string,
  visible: boolean
): Promise<boolean> {
  return invoke<boolean>('toggle_memory_visibility', { factId, visible });
}
