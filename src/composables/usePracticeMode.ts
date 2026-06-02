import { computed, ref } from 'vue';
import { getPracticeModes } from '../services/tauri/practice';
import { DEFAULT_PRACTICE_MODE_ID, type PracticeMode } from '../types/practice';
import { logError } from '../utils/errors';

const practiceModesState = ref<PracticeMode[]>([]);
const activePracticeModeId = ref<string>(DEFAULT_PRACTICE_MODE_ID);

export function usePracticeMode() {
  async function loadPracticeModes() {
    try {
      practiceModesState.value = await getPracticeModes();
      if (!practiceModesState.value.some((mode) => mode.id === activePracticeModeId.value)) {
        activePracticeModeId.value = DEFAULT_PRACTICE_MODE_ID;
      }
    } catch (error) {
      logError('Failed to load practice modes', error);
    }
  }

  function selectPracticeMode(id: string | null | undefined) {
    activePracticeModeId.value = id || DEFAULT_PRACTICE_MODE_ID;
  }

  function getPracticeModeById(id: string | null | undefined): PracticeMode | undefined {
    return practiceModesState.value.find((mode) => mode.id === (id || DEFAULT_PRACTICE_MODE_ID));
  }

  const activePracticeMode = computed(() => getPracticeModeById(activePracticeModeId.value) || null);

  function getPracticeModePromptSuffix(): string {
    const mode = activePracticeMode.value;
    if (!mode) return '';

    return `\n\n[Practice Mode: ${mode.name}]\n${mode.promptSuffix}`;
  }

  return {
    practiceModes: computed(() => practiceModesState.value),
    activePracticeModeId: computed(() => activePracticeModeId.value),
    activePracticeMode,
    loadPracticeModes,
    selectPracticeMode,
    getPracticeModeById,
    getPracticeModePromptSuffix,
  };
}

