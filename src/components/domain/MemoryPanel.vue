<script setup lang="ts">
import { ref, watch, onMounted, computed } from 'vue';
import { useMemory } from '../../composables/useMemory';
import type { FactType } from '../../types/memory';

const props = defineProps<{
  characterId: string;
  characterName: string;
}>();

const emit = defineEmits<{
  close: [];
}>();

const { facts, isLoading, loadMemoryFacts, addFact, removeFact, toggleVisibility } = useMemory();

const selectedTypeFilter = ref<string>('all');
const newFactContent = ref('');
const newFactType = ref<FactType>('custom');
const isAddingFact = ref(false);

const factTypes: { value: string; label: string; icon: string }[] = [
  { value: 'all', label: 'All Memories', icon: '🧠' },
  { value: 'event', label: 'Events', icon: '📅' },
  { value: 'preference', label: 'Preferences', icon: '❤️' },
  { value: 'commitment', label: 'Commitments', icon: '🤝' },
  { value: 'trait', label: 'Traits', icon: '🎭' },
  { value: 'custom', label: 'Custom', icon: '📝' },
];

const newFactTypes: { value: FactType; label: string }[] = [
  { value: 'event', label: 'Event' },
  { value: 'preference', label: 'Preference' },
  { value: 'commitment', label: 'Commitment' },
  { value: 'trait', label: 'Trait' },
  { value: 'custom', label: 'Custom' },
];

function fetchFacts() {
  if (props.characterId) {
    loadMemoryFacts(props.characterId);
  }
}

onMounted(fetchFacts);
watch(() => props.characterId, fetchFacts);

async function handleAddFact() {
  const content = newFactContent.value.trim();
  if (!content) return;
  try {
    await addFact(props.characterId, newFactType.value, content);
    newFactContent.value = '';
    isAddingFact.value = false;
  } catch (err) {
    console.error(err);
  }
}

async function handleRemoveFact(id: string) {
  await removeFact(id);
}

async function handleToggleVisibility(id: string, currentVisible: boolean) {
  await toggleVisibility(id, !currentVisible);
}

const filteredFacts = computed(() => {
  if (selectedTypeFilter.value === 'all') {
    return facts.value;
  }
  return facts.value.filter(f => f.fact_type === selectedTypeFilter.value);
});

function getFactIcon(type: string) {
  switch (type) {
    case 'event': return '📅';
    case 'preference': return '❤️';
    case 'commitment': return '🤝';
    case 'trait': return '🎭';
    default: return '📝';
  }
}

function formatDate(isoString: string) {
  try {
    const d = new Date(isoString);
    return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
  } catch {
    return isoString;
  }
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="memory-modal">
      <!-- Header -->
      <div class="modal-header">
        <div class="header-title">
          <h2>🧠 Memory Management</h2>
          <span class="subtitle">Cognitive background facts for {{ characterName }}</span>
        </div>
        <button class="close-btn" @click="emit('close')">✕</button>
      </div>

      <!-- Main Layout -->
      <div class="modal-body">
        <!-- Sidebar filters and quick action -->
        <div class="sidebar-panel">
          <div class="panel-section">
            <h3 class="section-title">Filters</h3>
            <div class="filter-list">
              <button
                v-for="filter in factTypes"
                :key="filter.value"
                class="filter-item"
                :class="{ 'filter-item--active': selectedTypeFilter === filter.value }"
                @click="selectedTypeFilter = filter.value"
              >
                <span class="filter-icon">{{ filter.icon }}</span>
                <span class="filter-label">{{ filter.label }}</span>
                <span class="filter-count" v-if="filter.value === 'all'">{{ facts.length }}</span>
                <span class="filter-count" v-else>{{ facts.filter(f => f.fact_type === filter.value).length }}</span>
              </button>
            </div>
          </div>

          <div class="panel-section add-fact-section">
            <h3 class="section-title">Add Fact</h3>
            <form @submit.prevent="handleAddFact" class="add-fact-form">
              <div class="form-group">
                <label>Type</label>
                <select v-model="newFactType" class="form-select">
                  <option v-for="t in newFactTypes" :key="t.value" :value="t.value">
                    {{ getFactIcon(t.value) }} {{ t.label }}
                  </option>
                </select>
              </div>
              <div class="form-group">
                <label>Content</label>
                <textarea
                  v-model="newFactContent"
                  placeholder="e.g. User mentioned they prefer dark roast coffee."
                  rows="3"
                  class="form-textarea"
                  required
                ></textarea>
              </div>
              <button type="submit" class="submit-btn" :disabled="!newFactContent.trim()">
                ✨ Save Memory
              </button>
            </form>
          </div>
        </div>

        <!-- Facts List -->
        <div class="facts-content">
          <div v-if="isLoading" class="state-container">
            <span class="loader"></span>
            <p>Scanning neural patterns...</p>
          </div>
          
          <div v-else-if="filteredFacts.length === 0" class="state-container empty-state">
            <span class="empty-icon">🏜️</span>
            <p>No memories found in this category.</p>
            <span class="empty-hint">Start a conversation or add a custom fact to seed this character's cognitive base.</span>
          </div>

          <div v-else class="facts-list">
            <div
              v-for="fact in filteredFacts"
              :key="fact.id"
              class="fact-card"
              :class="{ 'fact-card--disabled': !fact.is_visible }"
            >
              <div class="fact-type-badge" :title="fact.fact_type">
                {{ getFactIcon(fact.fact_type) }}
              </div>
              <div class="fact-main">
                <div class="fact-text">{{ fact.content }}</div>
                <div class="fact-meta">
                  <span class="meta-item">Confidence: {{ Math.round(fact.confidence * 100) }}%</span>
                  <span class="meta-divider">•</span>
                  <span class="meta-item">{{ formatDate(fact.created_at) }}</span>
                </div>
              </div>
              <div class="fact-actions">
                <button
                  class="action-btn toggle-visibility-btn"
                  :class="{ 'visible': fact.is_visible }"
                  @click="handleToggleVisibility(fact.id, fact.is_visible)"
                  :title="fact.is_visible ? 'Disable Prompt Injection' : 'Enable Prompt Injection'"
                >
                  {{ fact.is_visible ? '👁️ Active' : '👁️‍🗨️ Muted' }}
                </button>
                <button
                  class="action-btn delete-btn"
                  @click="handleRemoveFact(fact.id)"
                  title="Forget Memory"
                >
                  🗑️
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(10, 10, 15, 0.75);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  animation: fadeIn 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}

.memory-modal {
  background: var(--bg-secondary);
  border: 1px solid var(--border-visible);
  border-radius: var(--radius-xl);
  width: 860px;
  max-width: 95vw;
  height: 620px;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
  overflow: hidden;
  animation: scaleIn 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-lg) var(--space-xl);
  border-bottom: 1px solid var(--border-subtle);
  background: rgba(255, 255, 255, 0.02);
}
.header-title h2 {
  font-size: var(--font-size-lg);
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: 2px;
}
.header-title .subtitle {
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
}

.close-btn {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  transition: all var(--transition-fast);
}
.close-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.modal-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.sidebar-panel {
  width: 280px;
  border-right: 1px solid var(--border-subtle);
  padding: var(--space-lg);
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
  background: rgba(0, 0, 0, 0.1);
  overflow-y: auto;
}

.panel-section {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}
.section-title {
  font-size: var(--font-size-xs);
  font-weight: 700;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: var(--space-xs);
}

.filter-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.filter-item {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
  text-align: left;
  transition: all var(--transition-fast);
  color: var(--text-secondary);
}
.filter-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.filter-item--active {
  background: var(--bg-elevated);
  border: 1px solid var(--border-visible);
  color: var(--text-primary);
}
.filter-icon {
  font-size: 1.1rem;
}
.filter-label {
  flex: 1;
  font-size: var(--font-size-sm);
  font-weight: 500;
}
.filter-count {
  font-size: var(--font-size-xs);
  background: var(--bg-tertiary);
  padding: 2px 6px;
  border-radius: var(--radius-full);
  color: var(--text-tertiary);
}
.filter-item--active .filter-count {
  background: var(--accent-primary);
  color: #fff;
}

.add-fact-section {
  margin-top: auto;
  border-top: 1px solid var(--border-subtle);
  padding-top: var(--space-lg);
}
.add-fact-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}
.form-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.form-group label {
  font-size: var(--font-size-xs);
  color: var(--text-secondary);
  font-weight: 600;
}
.form-select, .form-textarea {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: var(--font-size-sm);
  padding: var(--space-sm);
  transition: border var(--transition-fast);
}
.form-select:focus, .form-textarea:focus {
  outline: none;
  border-color: var(--accent-primary);
}
.form-select {
  height: 36px;
  cursor: pointer;
}
.form-textarea {
  resize: none;
}
.submit-btn {
  background: var(--accent-primary);
  color: #fff;
  border: none;
  border-radius: var(--radius-md);
  padding: var(--space-sm) var(--space-md);
  font-size: var(--font-size-sm);
  font-weight: 600;
  cursor: pointer;
  transition: opacity var(--transition-fast);
}
.submit-btn:hover:not(:disabled) {
  opacity: 0.9;
}
.submit-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.facts-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-lg) var(--space-xl);
  display: flex;
  flex-direction: column;
}

.facts-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.fact-card {
  display: flex;
  gap: var(--space-md);
  padding: var(--space-md) var(--space-lg);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  align-items: flex-start;
  transition: all var(--transition-fast);
}
.fact-card:hover {
  border-color: var(--border-visible);
  background: var(--bg-hover);
}
.fact-card--disabled {
  opacity: 0.6;
  border-style: dashed;
}

.fact-type-badge {
  font-size: 1.4rem;
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-primary);
  border-radius: var(--radius-md);
  flex-shrink: 0;
}

.fact-main {
  flex: 1;
  min-width: 0;
}
.fact-text {
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  line-height: 1.5;
  word-break: break-word;
}
.fact-meta {
  margin-top: 6px;
  display: flex;
  align-items: center;
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
}
.meta-divider {
  margin: 0 var(--space-xs);
}

.fact-actions {
  display: flex;
  gap: var(--space-xs);
  align-self: center;
}
.action-btn {
  padding: 6px 12px;
  border-radius: var(--radius-md);
  font-size: var(--font-size-xs);
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast);
  border: 1px solid var(--border-subtle);
  background: var(--bg-primary);
  color: var(--text-secondary);
}
.action-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.toggle-visibility-btn.visible {
  background: rgba(46, 204, 113, 0.1);
  border-color: rgba(46, 204, 113, 0.3);
  color: #2ecc71;
}
.toggle-visibility-btn:not(.visible) {
  background: rgba(243, 156, 18, 0.1);
  border-color: rgba(243, 156, 18, 0.3);
  color: #f39c12;
}
.fact-actions .delete-btn {
  padding: 6px;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.fact-actions .delete-btn:hover {
  background: rgba(231, 76, 60, 0.1);
  border-color: rgba(231, 76, 60, 0.3);
  color: #e74c3c;
}

.state-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  text-align: center;
  color: var(--text-secondary);
  gap: var(--space-md);
}
.empty-icon {
  font-size: 3rem;
}
.empty-hint {
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
  max-width: 320px;
}

.loader {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border-subtle);
  border-bottom-color: var(--accent-primary);
  border-radius: 50%;
  display: inline-block;
  box-sizing: border-box;
  animation: rotation 1s linear infinite;
}

@keyframes rotation {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}
@keyframes scaleIn {
  from { transform: scale(0.95); opacity: 0; }
  to { transform: scale(1); opacity: 1; }
}
</style>
