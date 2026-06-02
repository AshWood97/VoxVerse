<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { useRelationship } from '../../composables/useRelationship';

const props = defineProps<{
  characterId: string;
  characterName: string;
}>();

const emit = defineEmits<{
  close: [];
}>();

const { relationshipState, isLoading, loadRelationshipState, modifyRelationshipState } = useRelationship();

const newPref = ref('');
const newBoundary = ref('');
const newCommitment = ref('');

function fetchRelationship() {
  if (props.characterId) {
    loadRelationshipState(props.characterId);
  }
}

onMounted(fetchRelationship);
watch(() => props.characterId, fetchRelationship);

async function updateStat(type: 'intimacy' | 'trust', delta: number) {
  if (!relationshipState.value) return;
  
  const currentIntimacy = relationshipState.value.intimacy_level;
  const currentTrust = relationshipState.value.trust_level;
  
  let updates = {};
  if (type === 'intimacy') {
    updates = { intimacy_level: Math.max(0, Math.min(100, currentIntimacy + delta)) };
  } else {
    updates = { trust_level: Math.max(0, Math.min(100, currentTrust + delta)) };
  }

  try {
    await modifyRelationshipState(props.characterId, updates);
  } catch (err) {
    console.error(err);
  }
}

async function updatePlotStage(event: Event) {
  const stage = (event.target as HTMLInputElement).value;
  try {
    await modifyRelationshipState(props.characterId, { plot_stage: stage });
  } catch (err) {
    console.error(err);
  }
}

async function addMetadataItem(key: 'preferences' | 'boundaries' | 'commitments', val: string) {
  if (!relationshipState.value || !val.trim()) return;
  
  const currentObj = (relationshipState.value as any)[key === 'preferences' ? 'user_preferences' : key] || {};
  const newKey = `item_${Date.now()}`;
  const updatedObj = { ...currentObj, [newKey]: val.trim() };
  
  let payload = {};
  if (key === 'preferences') {
    payload = { user_preferences: updatedObj };
  } else if (key === 'boundaries') {
    payload = { boundaries: updatedObj };
  } else {
    payload = { commitments: updatedObj };
  }

  try {
    await modifyRelationshipState(props.characterId, payload);
    if (key === 'preferences') newPref.value = '';
    else if (key === 'boundaries') newBoundary.value = '';
    else newCommitment.value = '';
  } catch (err) {
    console.error(err);
  }
}

async function deleteMetadataItem(key: 'preferences' | 'boundaries' | 'commitments', itemKey: string) {
  if (!relationshipState.value) return;
  
  const currentObj = (relationshipState.value as any)[key === 'preferences' ? 'user_preferences' : key] || {};
  const updatedObj = { ...currentObj };
  delete updatedObj[itemKey];

  let payload = {};
  if (key === 'preferences') {
    payload = { user_preferences: updatedObj };
  } else if (key === 'boundaries') {
    payload = { boundaries: updatedObj };
  } else {
    payload = { commitments: updatedObj };
  }

  try {
    await modifyRelationshipState(props.characterId, payload);
  } catch (err) {
    console.error(err);
  }
}
</script>

<template>
  <div class="relationship-card">
    <div class="card-header">
      <h3>🎭 Relationship Dynamics</h3>
      <button class="close-btn" @click="emit('close')">✕</button>
    </div>
    
    <div v-if="isLoading && !relationshipState" class="loader-container">
      <span class="loader"></span>
      <p>Synchronizing bond analytics...</p>
    </div>

    <div v-else-if="relationshipState" class="card-body">
      <!-- Intimacy & Trust Sliders -->
      <div class="stat-section">
        <!-- Intimacy -->
        <div class="stat-row">
          <div class="stat-info">
            <span class="stat-label">❤️ Intimacy</span>
            <span class="stat-value">{{ relationshipState.intimacy_level }}/100</span>
          </div>
          <div class="gauge-container">
            <div class="gauge-bar intimacy-bar" :style="{ width: `${relationshipState.intimacy_level}%` }"></div>
          </div>
          <div class="stat-controls">
            <button @click="updateStat('intimacy', -5)" class="stat-adjust-btn">-5</button>
            <button @click="updateStat('intimacy', 5)" class="stat-adjust-btn">+5</button>
          </div>
        </div>

        <!-- Trust -->
        <div class="stat-row">
          <div class="stat-info">
            <span class="stat-label">🤝 Trust</span>
            <span class="stat-value">{{ relationshipState.trust_level }}/100</span>
          </div>
          <div class="gauge-container">
            <div class="gauge-bar trust-bar" :style="{ width: `${relationshipState.trust_level}%` }"></div>
          </div>
          <div class="stat-controls">
            <button @click="updateStat('trust', -5)" class="stat-adjust-btn">-5</button>
            <button @click="updateStat('trust', 5)" class="stat-adjust-btn">+5</button>
          </div>
        </div>
      </div>

      <!-- Plot Stage -->
      <div class="plot-stage-section">
        <label class="section-label">🎬 Story Phase / Plot Stage</label>
        <input
          type="text"
          :value="relationshipState.plot_stage || ''"
          @change="updatePlotStage"
          placeholder="e.g. Acquaintances / Mid-Session Drift"
          class="stage-input"
        />
      </div>

      <!-- Metadata (Preferences, Commitments, Boundaries) -->
      <div class="meta-section">
        <!-- User Preferences -->
        <div class="meta-subsection">
          <span class="subsection-label">📌 User Preferences</span>
          <div class="meta-tags">
            <div
              v-for="(val, itemKey) in (relationshipState.user_preferences || {})"
              :key="itemKey"
              class="meta-tag preference-tag"
            >
              <span>{{ val }}</span>
              <button @click="deleteMetadataItem('preferences', itemKey as string)" class="tag-del-btn">×</button>
            </div>
          </div>
          <div class="input-row">
            <input
              type="text"
              v-model="newPref"
              placeholder="Add preference..."
              @keyup.enter="addMetadataItem('preferences', newPref)"
              class="inline-input"
            />
            <button @click="addMetadataItem('preferences', newPref)" class="inline-add-btn">+</button>
          </div>
        </div>

        <!-- Commitments -->
        <div class="meta-subsection">
          <span class="subsection-label">🤝 Commitments & Agreements</span>
          <div class="meta-tags">
            <div
              v-for="(val, itemKey) in (relationshipState.commitments || {})"
              :key="itemKey"
              class="meta-tag commitment-tag"
            >
              <span>{{ val }}</span>
              <button @click="deleteMetadataItem('commitments', itemKey as string)" class="tag-del-btn">×</button>
            </div>
          </div>
          <div class="input-row">
            <input
              type="text"
              v-model="newCommitment"
              placeholder="Add agreement..."
              @keyup.enter="addMetadataItem('commitments', newCommitment)"
              class="inline-input"
            />
            <button @click="addMetadataItem('commitments', newCommitment)" class="inline-add-btn">+</button>
          </div>
        </div>

        <!-- Boundaries -->
        <div class="meta-subsection">
          <span class="subsection-label">⚠️ Boundaries & Red Lines</span>
          <div class="meta-tags">
            <div
              v-for="(val, itemKey) in (relationshipState.boundaries || {})"
              :key="itemKey"
              class="meta-tag boundary-tag"
            >
              <span>{{ val }}</span>
              <button @click="deleteMetadataItem('boundaries', itemKey as string)" class="tag-del-btn">×</button>
            </div>
          </div>
          <div class="input-row">
            <input
              type="text"
              v-model="newBoundary"
              placeholder="Add boundary..."
              @keyup.enter="addMetadataItem('boundaries', newBoundary)"
              class="inline-input"
            />
            <button @click="addMetadataItem('boundaries', newBoundary)" class="inline-add-btn">+</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.relationship-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: var(--shadow-sm);
  transition: border-color var(--transition-fast);
}
.relationship-card:hover {
  border-color: var(--border-visible);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-md) var(--space-lg);
  border-bottom: 1px solid var(--border-subtle);
  background: rgba(255, 255, 255, 0.01);
}
.card-header h3 {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--text-primary);
}
.close-btn {
  color: var(--text-tertiary);
  font-size: 0.9rem;
}
.close-btn:hover {
  color: var(--text-primary);
}

.card-body {
  padding: var(--space-lg);
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

.stat-section {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.stat-row {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.stat-info {
  display: flex;
  justify-content: space-between;
  font-size: var(--font-size-xs);
  font-weight: 600;
}
.stat-label {
  color: var(--text-secondary);
}
.stat-value {
  color: var(--text-primary);
}

.gauge-container {
  height: 8px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-full);
  overflow: hidden;
  position: relative;
}
.gauge-bar {
  height: 100%;
  border-radius: var(--radius-full);
  transition: width 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.intimacy-bar {
  background: linear-gradient(90deg, #fd79a8, #e84393);
}
.trust-bar {
  background: linear-gradient(90deg, #74b9ff, #0984e3);
}

.stat-controls {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-xs);
  margin-top: 2px;
}
.stat-adjust-btn {
  padding: 2px 8px;
  font-size: 10px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  cursor: pointer;
  color: var(--text-secondary);
  transition: all var(--transition-fast);
}
.stat-adjust-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.plot-stage-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.section-label {
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--text-secondary);
}
.stage-input {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: var(--font-size-sm);
  padding: var(--space-sm);
  width: 100%;
  transition: all var(--transition-fast);
}
.stage-input:focus {
  outline: none;
  border-color: var(--accent-primary);
}

.meta-section {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}
.meta-subsection {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.subsection-label {
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--text-tertiary);
}

.meta-tags {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-xs);
  margin-bottom: 4px;
}

.meta-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  font-size: var(--font-size-xs);
  color: var(--text-primary);
  border: 1px solid var(--border-subtle);
  animation: tagAppear 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}
.preference-tag {
  background: rgba(253, 121, 168, 0.08);
  border-color: rgba(253, 121, 168, 0.2);
}
.commitment-tag {
  background: rgba(116, 185, 255, 0.08);
  border-color: rgba(116, 185, 255, 0.2);
}
.boundary-tag {
  background: rgba(231, 76, 60, 0.08);
  border-color: rgba(231, 76, 60, 0.2);
}

.tag-del-btn {
  font-size: 14px;
  color: var(--text-tertiary);
  cursor: pointer;
  line-height: 1;
}
.tag-del-btn:hover {
  color: #e74c3c;
}

.input-row {
  display: flex;
  gap: var(--space-xs);
}
.inline-input {
  flex: 1;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: var(--font-size-xs);
  padding: var(--space-sm);
}
.inline-input:focus {
  outline: none;
  border-color: var(--accent-primary);
}
.inline-add-btn {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: bold;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.inline-add-btn:hover {
  background: var(--accent-primary);
  color: #fff;
  border-color: var(--accent-primary);
}

.loader-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-xl);
  color: var(--text-secondary);
  font-size: var(--font-size-xs);
  gap: var(--space-sm);
}
.loader {
  width: 20px;
  height: 20px;
  border: 2px solid var(--border-subtle);
  border-bottom-color: var(--accent-primary);
  border-radius: 50%;
  animation: rotation 1s linear infinite;
}

@keyframes rotation {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
@keyframes tagAppear {
  from { transform: scale(0.9); opacity: 0; }
  to { transform: scale(1); opacity: 1; }
}
</style>
