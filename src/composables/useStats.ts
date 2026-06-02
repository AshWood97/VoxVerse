import { ref } from 'vue';
import type { LearningStats } from '../services/tauri/stats';
import { getLearningStats } from '../services/tauri/stats';
import { toErrorMessage } from '../utils/errors';

const statsState = ref<LearningStats | null>(null);
const isLoading = ref(false);
const error = ref<string | null>(null);

export function useStats() {
  async function loadStats() {
    isLoading.value = true;
    error.value = null;
    try {
      statsState.value = await getLearningStats();
    } catch (errorCause) {
      error.value = toErrorMessage(errorCause);
    } finally {
      isLoading.value = false;
    }
  }

  return {
    stats: statsState,
    isLoading,
    error,
    loadStats,
  };
}
