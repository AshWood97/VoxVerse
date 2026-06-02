<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { fetchMarketCharacters } from '../../services/market';
import type { MarketCharacter } from '../../services/market';
import type { Character } from '../../types/chat';

const emit = defineEmits<{
  close: [];
  import: [char: Character];
}>();

const { t } = useI18n();

const loading = ref(true);
const error = ref('');
const characters = ref<MarketCharacter[]>([]);

// Tracking which characters are currently being downloaded/imported
const downloadingIds = ref<Set<string>>(new Set());
// Tracking successfully added ones
const downloadedIds = ref<Set<string>>(new Set());

onMounted(async () => {
  try {
    characters.value = await fetchMarketCharacters();
  } catch (err) {
    error.value = t('market.error');
  } finally {
    loading.value = false;
  }
});

async function handleDownload(char: MarketCharacter) {
  if (downloadedIds.value.has(char.id) || downloadingIds.value.has(char.id)) {
    return;
  }
  
  downloadingIds.value.add(char.id);
  
  // Transform MarketCharacter safely into a local Character schema (strip Market-only properties)
  const importedChar: Character = {
    id: `char-${Date.now()}-${Math.floor(Math.random()*1000)}`, // New localized ID
    name: char.name,
    language: char.language,
    personality: char.personality,
    style: char.style,
    avatar: char.avatar,
    systemPrompt: char.systemPrompt,
    greeting: char.greeting,
    voiceConfig: char.voiceConfig,
  };
  
  // Fake a tiny delay so it feels like downloading
  await new Promise(r => setTimeout(r, 600));
  
  emit('import', importedChar);
  
  downloadingIds.value.delete(char.id);
  downloadedIds.value.add(char.id);
}

function handleOverlayClick(e: MouseEvent) {
  if ((e.target as HTMLElement).classList.contains('modal-overlay')) {
    emit('close');
  }
}
</script>

<template>
  <div class="modal-overlay" @click="handleOverlayClick">
    <div class="modal-content">
      
      <!-- Header -->
      <div class="modal-header">
        <div class="header-title">
          <span class="icon">🌟</span>
          <h2>{{ t('market.title') }}</h2>
        </div>
        <button class="close-btn" @click="emit('close')">✕</button>
      </div>

      <!-- Body -->
      <div class="modal-body">
        
        <div v-if="loading" class="status-view">
          <div class="spinner"></div>
          <p>{{ t('market.loading') }}</p>
        </div>
        
        <div v-else-if="error" class="status-view error">
          ⚠️ {{ error }}
        </div>
        
        <div v-else class="market-grid">
          <div 
            v-for="char in characters" 
            :key="char.id" 
            class="market-card"
          >
            <div class="card-header">
              <div class="card-avatar">{{ char.avatar }}</div>
              <div class="card-title-area">
                <h3 class="char-name">{{ char.name }}</h3>
                <span class="char-lang">{{ char.language }}</span>
              </div>
            </div>
            
            <p class="char-desc">{{ char.personality }}</p>
            
            <div class="tags-row">
              <span class="tag" v-for="tag in char.tags" :key="tag">{{ tag }}</span>
            </div>
            
            <div class="card-footer">
              <span class="downloads">⬇️ {{ char.downloads?.toLocaleString() }}</span>
              
              <button 
                class="download-btn"
                :class="{ 'downloaded': downloadedIds.has(char.id), 'downloading': downloadingIds.has(char.id) }"
                :disabled="downloadedIds.has(char.id) || downloadingIds.has(char.id)"
                @click="handleDownload(char)"
              >
                <span v-if="downloadedIds.has(char.id)">✅ {{ t('market.downloaded') }}</span>
                <span v-else-if="downloadingIds.has(char.id)">⏳ {{ t('market.downloading') }}</span>
                <span v-else>➕ {{ t('market.download') }}</span>
              </button>
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
  background: rgba(0, 0, 0, 0.65);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
  animation: fadeIn 0.2s ease-out;
}

.modal-content {
  background: var(--bg-secondary);
  border: 1px solid var(--border-visible);
  border-radius: var(--radius-xl);
  width: 900px;
  max-width: 95vw;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow-lg);
  overflow: hidden;
  animation: slideUp 0.25s ease-out;
}

@keyframes slideUp {
  from { transform: translateY(20px); opacity: 0; }
  to   { transform: translateY(0);    opacity: 1; }
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-lg);
  border-bottom: 1px solid var(--border-subtle);
  background: var(--bg-secondary);
  flex-shrink: 0;
}

.header-title {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.header-title h2 {
  font-size: var(--font-size-lg);
  font-weight: 700;
  color: var(--text-primary);
}

.close-btn {
  width: 32px; height: 32px;
  border-radius: var(--radius-sm);
  color: var(--text-tertiary);
  display: flex; align-items: center; justify-content: center;
  transition: background var(--transition-fast);
}
.close-btn:hover { background: var(--bg-hover); color: var(--text-primary); }

.modal-body {
  padding: var(--space-lg);
  overflow-y: auto;
  flex: 1;
}

.status-view {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 200px;
  gap: var(--space-md);
  color: var(--text-secondary);
}

.spinner {
  width: 36px; height: 36px;
  border: 4px solid var(--border-subtle);
  border-top-color: var(--accent-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }

/* Grid */
.market-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: var(--space-lg);
}

.market-card {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  padding: var(--space-md);
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  transition: all var(--transition-fast);
}

.market-card:hover {
  border-color: var(--border-visible);
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.card-header {
  display: flex;
  align-items: center;
  gap: var(--space-md);
}

.card-avatar {
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
  width: 64px;
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 2rem;
  border: 1px solid var(--border-subtle);
}

.card-title-area {
  display: flex;
  flex-direction: column;
}

.char-name {
  font-size: var(--font-size-base);
  font-weight: 600;
  color: var(--text-primary);
}

.char-lang {
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
}

.char-desc {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
  flex: 1; /* Pushes footer down if short */
}

.tags-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: auto;
}

.tag {
  background: var(--bg-primary);
  color: var(--text-tertiary);
  padding: 2px 8px;
  border-radius: var(--radius-full);
  font-size: 0.7rem;
  border: 1px solid var(--border-subtle);
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: var(--space-sm);
  padding-top: var(--space-sm);
  border-top: 1px solid var(--border-subtle);
}

.downloads {
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
}

.download-btn {
  background: var(--bg-primary);
  color: var(--accent-primary);
  border: 1px solid rgba(108, 92, 231, 0.3);
  padding: 6px 14px;
  border-radius: var(--radius-full);
  font-size: var(--font-size-sm);
  font-weight: 600;
  transition: all var(--transition-fast);
}

.download-btn:not(:disabled):hover {
  background: rgba(108, 92, 231, 0.1);
  border-color: var(--accent-primary);
}

.download-btn.downloading {
  opacity: 0.7;
  cursor: wait;
}

.download-btn.downloaded {
  background: var(--bg-primary);
  color: var(--text-tertiary);
  border-color: var(--border-subtle);
  cursor: default;
}
</style>
