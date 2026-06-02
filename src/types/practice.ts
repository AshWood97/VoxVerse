export interface PracticeMode {
  id: string;
  name: string;
  description: string;
  promptSuffix: string;
  isPreset: boolean;
  createdAt: string;
}

export const DEFAULT_PRACTICE_MODE_ID = 'free_talk';

