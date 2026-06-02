import { invoke } from '@tauri-apps/api/core';
import type { Scenario } from '../../types/feedback';

export async function getScenarios(): Promise<Scenario[]> {
  return invoke<Scenario[]>('get_scenarios');
}

export async function saveScenario(scenario: Scenario): Promise<void> {
  await invoke('save_scenario', { scenario });
}

export async function deleteScenario(id: string): Promise<void> {
  await invoke('delete_scenario', { id });
}
