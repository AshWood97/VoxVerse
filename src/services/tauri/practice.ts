import { invoke } from '@tauri-apps/api/core';
import type { PracticeMode } from '../../types/practice';

export async function getPracticeModes(): Promise<PracticeMode[]> {
  return invoke<PracticeMode[]>('get_practice_modes');
}

