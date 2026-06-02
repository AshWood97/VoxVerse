import { ref, computed } from 'vue';
import type { Scenario } from '../types/feedback';
import { getScenarios, saveScenario, deleteScenario } from '../services/tauri/scenario';
import { logError } from '../utils/errors';

const scenariosState = ref<Scenario[]>([]);
const activeScenarioId = ref<string | null>(null);

export function useScenario() {
  async function loadScenarios() {
    try {
      scenariosState.value = await getScenarios();
    } catch (error) {
      logError('Failed to load scenarios', error);
    }
  }

  async function addOrUpdateScenario(scenario: Scenario) {
    await saveScenario(scenario);
    await loadScenarios();
  }

  async function removeScenario(id: string) {
    await deleteScenario(id);
    if (activeScenarioId.value === id) {
      activeScenarioId.value = null;
    }
    await loadScenarios();
  }

  function selectScenario(id: string | null) {
    activeScenarioId.value = id;
  }

  function getScenarioById(id: string): Scenario | undefined {
    return scenariosState.value.find((s) => s.id === id);
  }

  const activeScenario = computed(() => {
    if (!activeScenarioId.value) return null;
    return scenariosState.value.find((s) => s.id === activeScenarioId.value) || null;
  });

  /**
   * Build the scenario context text to inject into the system prompt.
   * Returns empty string if no scenario is active.
   */
  function getScenarioPromptSuffix(): string {
    const scenario = activeScenario.value;
    if (!scenario) return '';
    return `\n\n[Current Scenario: ${scenario.name}]\n${scenario.description}\nPlease stay in character for this scenario throughout the conversation.`;
  }

  return {
    scenarios: computed(() => scenariosState.value),
    activeScenarioId: computed(() => activeScenarioId.value),
    activeScenario,
    loadScenarios,
    addOrUpdateScenario,
    removeScenario,
    selectScenario,
    getScenarioById,
    getScenarioPromptSuffix,
  };
}
