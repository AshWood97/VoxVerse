<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import type { Character } from '../../types/chat';

const props = defineProps<{
  initialData?: Character | null;
}>();

const emit = defineEmits<{
  close: [];
  save: [Char: Character];
}>();

const { t } = useI18n();

const formData = ref<Partial<Character>>({
  name: '',
  language: 'English (US)',
  personality: '',
  style: '',
  avatar: '👤',
  systemPrompt: '',
  greeting: '',
  voiceConfig: { lang: 'en-US', namePattern: '' },
});

onMounted(() => {
  if (props.initialData) {
    // Deep clone to prevent mutating original before save
    formData.value = JSON.parse(JSON.stringify(props.initialData));
    if (!formData.value.voiceConfig) {
      formData.value.voiceConfig = { lang: 'en-US', namePattern: '' };
    }
  }
});

function handleFileUpload(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) return;
  
  // Guard against exceptionally massive files hanging the browser reader
  if (file.size > 5 * 1024 * 1024) {
    alert(t('characterModal.imageTooLarge'));
    return;
  }

  const reader = new FileReader();
  reader.onload = (e) => {
    const img = new Image();
    img.onload = () => {
      // Compress using canvas
      const canvas = document.createElement('canvas');
      const MAX_SIZE = 128; // Small size for avatars
      let width = img.width;
      let height = img.height;

      if (width > height) {
        if (width > MAX_SIZE) {
          height *= MAX_SIZE / width;
          width = MAX_SIZE;
        }
      } else {
        if (height > MAX_SIZE) {
          width *= MAX_SIZE / height;
          height = MAX_SIZE;
        }
      }

      canvas.width = width;
      canvas.height = height;
      const ctx = canvas.getContext('2d');
      if (ctx) {
        ctx.drawImage(img, 0, 0, width, height);
        // Convert to compressed WebP Base64
        formData.value.avatar = canvas.toDataURL('image/webp', 0.8);
      }
    };
    img.src = e.target?.result as string;
  };
  reader.readAsDataURL(file);
}

function handleSave() {
  if (!formData.value.name || !formData.value.systemPrompt) {
    alert(t('characterModal.requiredFields'));
    return;
  }
  
  const payload: Character = {
    id: props.initialData?.id || `char-${Date.now()}`,
    name: formData.value.name!,
    language: formData.value.language || '',
    personality: formData.value.personality || '',
    style: formData.value.style || '',
    avatar: formData.value.avatar || '👤',
    systemPrompt: formData.value.systemPrompt!,
    greeting: formData.value.greeting || '',
    voiceConfig: formData.value.voiceConfig?.lang ? {
      lang: formData.value.voiceConfig.lang,
      namePattern: formData.value.voiceConfig.namePattern || undefined,
    } : undefined,
  };
  
  emit('save', payload);
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal">
      <div class="modal-header">
        <h2>{{ initialData ? t('characterModal.editTitle') : t('characterModal.createTitle') }}</h2>
        <button class="close-btn" @click="emit('close')">✕</button>
      </div>

      <div class="modal-body">
        <div class="form-group avatar-group">
          <label>{{ t('characterModal.avatar') }}</label>
          <div class="avatar-preview">
            <img v-if="formData.avatar?.startsWith('data:')" :src="formData.avatar" class="avatar-img" />
            <span v-else class="avatar-icon">{{ formData.avatar }}</span>
            <input type="file" accept="image/png, image/jpeg, image/webp" @change="handleFileUpload" class="file-input" />
          </div>
          <div class="avatar-tips">{{ t('characterModal.avatarTips') }}</div>
          <input v-if="!formData.avatar?.startsWith('data:')" v-model="formData.avatar" type="text" class="input-emoji" />
        </div>

        <div class="form-row">
          <div class="form-group">
            <label>{{ t('characterModal.name') }} *</label>
            <input v-model="formData.name" type="text" required />
          </div>
          <div class="form-group">
            <label>{{ t('characterModal.language') }}</label>
            <input v-model="formData.language" type="text" :placeholder="t('characterModal.languagePlaceholder')" />
          </div>
        </div>

        <div class="form-group">
          <label>{{ t('characterModal.personality') }}</label>
          <input v-model="formData.personality" type="text" :placeholder="t('characterModal.personalityPlaceholder')" />
        </div>

        <div class="form-group">
          <label>{{ t('characterModal.systemPrompt') }} *</label>
          <textarea v-model="formData.systemPrompt" rows="6" required :placeholder="t('characterModal.systemPromptPlaceholder')"></textarea>
        </div>

        <div class="form-group">
          <label>{{ t('characterModal.greeting') }}</label>
          <input v-model="formData.greeting" type="text" />
        </div>

        <div class="form-row">
          <div class="form-group">
            <label>{{ t('characterModal.ttsLanguageCode') }}</label>
            <input v-model="formData.voiceConfig!.lang" type="text" :placeholder="t('characterModal.ttsLanguagePlaceholder')" />
          </div>
          <div class="form-group">
            <label>{{ t('characterModal.voicePattern') }}</label>
            <input v-model="formData.voiceConfig!.namePattern" type="text" :placeholder="t('characterModal.voicePatternPlaceholder')" />
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn--secondary" @click="emit('close')">{{ t('settings.cancel') }}</button>
        <button class="btn btn--primary" @click="handleSave">{{ t('characterModal.save') }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Modal base styles similar to SettingsModal.vue */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  animation: fadeIn var(--transition-normal) ease-out;
}

.modal {
  background: var(--bg-primary);
  width: 90%;
  max-width: 600px;
  max-height: 90vh;
  border-radius: var(--radius-xl);
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
  border: 1px solid var(--border-subtle);
  animation: slideUp var(--transition-normal) ease-out;
}

.modal-header {
  padding: var(--space-lg);
  border-bottom: 1px solid var(--border-subtle);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.modal-header h2 {
  font-size: var(--font-size-xl);
  font-weight: 600;
  color: var(--text-primary);
}

.close-btn {
  background: none;
  border: none;
  font-size: 1.2rem;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: var(--space-xs);
  transition: color var(--transition-fast);
}

.close-btn:hover {
  color: var(--text-primary);
}

.modal-body {
  padding: var(--space-lg);
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.form-row {
  display: flex;
  gap: var(--space-md);
}
.form-row > .form-group {
  flex: 1;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.form-group label {
  font-size: var(--font-size-xs);
  font-weight: 500;
  color: var(--text-secondary);
}

input[type="text"],
textarea {
  padding: var(--space-sm) var(--space-md);
  border: 1px solid var(--border-visible);
  border-radius: var(--radius-md);
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: var(--font-size-sm);
  transition: all var(--transition-fast);
}

input[type="text"]:focus,
textarea:focus {
  outline: none;
  border-color: var(--accent-primary);
  background: var(--bg-primary);
  box-shadow: 0 0 0 2px rgba(52, 152, 219, 0.2);
}

.avatar-preview {
  position: relative;
  width: 64px;
  height: 64px;
  border-radius: 50%;
  background: var(--bg-elevated);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  border: 2px dashed var(--border-visible);
  cursor: pointer;
}
.avatar-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.avatar-icon {
  font-size: 2rem;
}
.file-input {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  opacity: 0;
  cursor: pointer;
}
.avatar-tips {
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
}
.input-emoji {
  width: 64px !important;
  text-align: center;
}

.modal-footer {
  padding: var(--space-lg);
  border-top: 1px solid var(--border-subtle);
  display: flex;
  justify-content: flex-end;
  gap: var(--space-md);
}

.btn {
  padding: var(--space-sm) var(--space-lg);
  border-radius: var(--radius-md);
  font-size: var(--font-size-sm);
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn--secondary {
  background: var(--bg-secondary);
  color: var(--text-primary);
  border: 1px solid var(--border-visible);
}

.btn--secondary:hover {
  background: var(--bg-hover);
}

.btn--primary {
  background: var(--accent-primary);
  color: white;
  border: none;
}

.btn--primary:hover {
  background: #2980b9;
}
</style>
