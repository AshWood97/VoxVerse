<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import type { Character } from '../../types/chat';

const props = defineProps<{
  characters: Character[];
  activeId: string;
}>();

const emit = defineEmits<{
  select: [id: string];
  openSettings: [];
  create: [];
  import: [];
  discover: [];
  edit: [id: string];
  delete: [id: string];
  export: [id: string];
}>();

function isPreset(id: string) {
  return ['emily', 'kenji', 'sophie'].includes(id);
}

const relationshipStats = ref<Record<string, { intimacy: number; trust: number }>>({});

async function loadRelationshipStats() {
  for (const char of props.characters) {
    try {
      const state: any = await invoke('get_relationship_state', { characterId: char.id });
      relationshipStats.value[char.id] = {
        intimacy: state.intimacy_level,
        trust: state.trust_level,
      };
    } catch (e) {
      relationshipStats.value[char.id] = { intimacy: 0, trust: 0 };
    }
  }
}

onMounted(loadRelationshipStats);
watch(() => props.characters, loadRelationshipStats, { deep: true });
watch(() => props.activeId, loadRelationshipStats);

const { t } = useI18n();
</script>

<template>
  <aside class="sidebar">
    <!-- Logo -->
    <div class="sidebar-logo">
      <span class="sidebar-logo-icon">🎙️</span>
      <h1 class="sidebar-logo-text">{{ t('sidebar.title') }}</h1>
    </div>

    <!-- Character list -->
    <div class="sidebar-section">
      <div class="sidebar-section-title">
        {{ t('sidebar.characters') }}
        <div class="section-actions">
          <button class="add-char-btn discover-btn" @click="emit('discover')" :title="t('sidebar.discover')">🌟</button>
          <button class="add-char-btn" @click="emit('import')" :title="t('sidebar.import')">📥</button>
          <button class="add-char-btn" @click="emit('create')" :title="t('sidebar.create')">➕</button>
        </div>
      </div>
      <div class="character-list">
        <div
          v-for="char in characters"
          :key="char.id"
          class="character-item-wrap"
        >
          <button
            class="character-item"
            :class="{ 'character-item--active': char.id === activeId }"
            @click="emit('select', char.id)"
          >
            <span class="character-item-avatar">
              <img v-if="char.avatar.startsWith('data:')" :src="char.avatar" class="avatar-img" />
              <template v-else>{{ char.avatar }}</template>
            </span>
            <div class="character-item-info">
              <div class="character-item-name">{{ char.name }}</div>
              <div class="character-item-meta">
                <span class="character-item-lang">{{ char.language }}</span>
                <span v-if="relationshipStats[char.id]" class="character-item-bond" title="Intimacy & Trust">
                  ❤️{{ relationshipStats[char.id].intimacy }} 🤝{{ relationshipStats[char.id].trust }}
                </span>
              </div>
            </div>
          </button>
          
          <div class="character-actions" v-if="!isPreset(char.id)">
            <button @click.stop="emit('export', char.id)" :title="t('sidebar.export')">📤</button>
            <button @click.stop="emit('edit', char.id)" :title="t('sidebar.edit')">✏️</button>
            <button @click.stop="emit('delete', char.id)" :title="t('sidebar.delete')" class="delete-btn">🗑️</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Spacer -->
    <div class="sidebar-spacer"></div>

    <!-- Settings button -->
    <div class="sidebar-footer">
      <button class="settings-btn" @click="emit('openSettings')">
        ⚙️ {{ t('sidebar.settings') }}
      </button>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: var(--sidebar-width);
  height: 100%;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-subtle);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.sidebar-logo {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-lg) var(--space-md);
  border-bottom: 1px solid var(--border-subtle);
}
.sidebar-logo-icon {
  font-size: 1.5rem;
}
.sidebar-logo-text {
  font-size: var(--font-size-xl);
  font-weight: 700;
  background: var(--accent-gradient);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.sidebar-section {
  padding: var(--space-md);
}
.sidebar-section-title {
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: var(--space-sm);
}

.character-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.character-item {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
  transition: background var(--transition-fast);
  width: 100%;
  text-align: left;
}
.character-item:hover {
  background: var(--bg-hover);
}
.character-item--active {
  background: var(--bg-elevated);
  border: 1px solid var(--border-visible);
}

.character-item-avatar {
  font-size: 1.5rem;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-tertiary);
  border-radius: var(--radius-full);
  flex-shrink: 0;
}
.character-item--active .character-item-avatar {
  background: var(--accent-primary);
}

.character-item-info {
  min-width: 0;
}
.character-item-name {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--text-primary);
}
.character-item-lang {
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
}

.character-item-meta {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  margin-top: 2px;
}

.character-item-bond {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  background: var(--bg-tertiary);
  padding: 1px 4px;
  border-radius: var(--radius-sm);
  font-size: 10px;
  color: var(--text-secondary);
}

.character-item-wrap {
  position: relative;
  display: flex;
  align-items: center;
}

.character-actions {
  position: absolute;
  right: var(--space-sm);
  display: flex;
  gap: var(--space-xs);
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.character-item-wrap:hover .character-actions {
  opacity: 1;
}

.character-actions button {
  background: var(--bg-primary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  padding: 4px;
  font-size: 0.8rem;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.character-actions button:hover {
  background: var(--bg-hover);
  transform: scale(1.1);
}

.character-actions .delete-btn:hover {
  background: var(--status-danger-soft);
  border-color: var(--status-danger-border);
  color: var(--status-danger);
}

.add-char-btn {
  background: none;
  border: none;
  font-size: 1.2rem;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 0 4px;
  border-radius: var(--radius-sm);
}
.add-char-btn:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
}

.discover-btn {
  color: var(--accent-secondary) !important;
}
.discover-btn:hover {
  background: var(--accent-secondary-soft) !important;
}

.avatar-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 50%;
}

.sidebar-section-title {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: var(--space-sm);
}

.section-actions {
  display: flex;
  gap: 2px;
}

.sidebar-spacer {
  flex: 1;
}

.sidebar-footer {
  padding: var(--space-md);
  border-top: 1px solid var(--border-subtle);
}
.settings-btn {
  width: 100%;
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  transition: background var(--transition-fast), color var(--transition-fast);
  text-align: left;
}
.settings-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
</style>
