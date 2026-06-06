<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { setLanguage } from '../../i18n';
import type { AppLanguage } from '../../i18n';
import {
  THEME_PRESETS,
  applyThemePreference,
  normalizeThemePreference,
} from '../../theme';
import type { ThemePreferenceId } from '../../theme';
import type {
  AiProvider,
  ApiConfig,
  ConfigInfo,
  ConfigProfileInfo,
  DiagnosticStatus,
  RuntimeDiagnostics,
} from '../../types/api';
import {
  clearActiveApiKey,
  createConfigProfile,
  deleteConfigProfile,
  getConfig,
  listConfigProfiles,
  renameConfigProfile,
  runRuntimeDiagnostics,
  saveConfig,
  switchConfigProfile,
} from '../../services/tauri/chat';
import { getTtsVoices } from '../../services/tauri/tts';
import type { VoiceInfo } from '../../services/tauri/tts';
import { useSpeechSynthesis } from '../../composables/useSpeechSynthesis';
import { logError, toErrorMessage } from '../../utils/errors';

const emit = defineEmits<{
  close: [];
}>();

interface SavedConfigSnapshot {
  profileId: string;
  profileName: string;
  provider: AiProvider;
  baseUrl: string;
  model: string;
}

const providerDefaults: Record<AiProvider, { baseUrl: string; model: string }> = {
  openai: {
    baseUrl: 'https://api.openai.com/v1',
    model: 'gpt-4o-mini',
  },
  ollama: {
    baseUrl: 'http://localhost:11434/v1',
    model: 'llama3.2',
  },
  custom: {
    baseUrl: '',
    model: '',
  },
};

const form = ref<ApiConfig>({
  provider: 'openai',
  baseUrl: providerDefaults.openai.baseUrl,
  apiKey: '',
  model: providerDefaults.openai.model,
});
const savedConfig = ref<SavedConfigSnapshot>({
  profileId: 'default',
  profileName: 'Default',
  provider: form.value.provider,
  baseUrl: form.value.baseUrl,
  model: form.value.model,
});
const profiles = ref<ConfigProfileInfo[]>([]);
const activeProfileId = ref('default');
const profilesLoading = ref(false);
const profileError = ref('');
const hasExistingKey = ref(false);
const clearingKey = ref(false);
const saving = ref(false);
const saveSuccess = ref(false);
const saveError = ref('');
const diagnostics = ref<RuntimeDiagnostics | null>(null);
const diagnosticsLoading = ref(false);
const diagnosticsError = ref('');
const diagnosticsCopied = ref(false);

const { t, locale } = useI18n();
const {
  speak,
  stop: stopVoicePreview,
  isSpeaking: isPreviewSpeaking,
  speechError,
  speechNotice,
  activeSpeechEngine,
} = useSpeechSynthesis();

const ttsVoices = ref<VoiceInfo[]>([]);
const selectedVoice = ref(localStorage.getItem('tts_voice') || 'en-US-AriaNeural');
const voicesLoading = ref(false);
const selectedTheme = ref<ThemePreferenceId>(normalizeThemePreference(localStorage.getItem('theme')));
const selectedLang = ref<AppLanguage>(locale.value as AppLanguage);
const autoFeedbackEnabled = ref(localStorage.getItem('auto_feedback_enabled') !== 'false');

const popularLocales = ['en-US', 'en-GB', 'en-AU', 'zh-CN', 'zh-TW', 'ja-JP', 'ko-KR', 'fr-FR', 'de-DE', 'es-ES'];

const requiresApiKey = computed(() => form.value.provider === 'openai');
const baseUrlPlaceholder = computed(() => providerDefaults[form.value.provider].baseUrl || 'https://your-endpoint.example/v1');
const modelPlaceholder = computed(() => providerDefaults[form.value.provider].model || 'your-model-name');
const selectedVoiceInfo = computed(() => ttsVoices.value.find((voice) => voice.name === selectedVoice.value));
const themeChoices = computed(() => [
  {
    id: 'system' as ThemePreferenceId,
    name: t('settings.themeSystem'),
    description: t('settings.themeSystemHint'),
  },
  ...THEME_PRESETS.map((preset) => ({
    id: preset.id,
    name: t(preset.nameKey),
    description: t(preset.descriptionKey),
  })),
]);
const selectedThemeDescription = computed(() => (
  themeChoices.value.find((theme) => theme.id === selectedTheme.value)?.description || ''
));
const previewLocale = computed(() => selectedVoiceInfo.value?.locale || (selectedLang.value === 'zh' ? 'zh-CN' : 'en-US'));
const previewText = computed(() => (
  previewLocale.value.startsWith('zh')
    ? t('settings.ttsPreviewTextZh')
    : t('settings.ttsPreviewTextEn')
));
const speechEngineLabel = computed(() => {
  if (activeSpeechEngine.value === 'edge') {
    return 'Edge TTS';
  }

  if (activeSpeechEngine.value === 'browser') {
    return 'Browser Speech';
  }

  return '';
});
const isConfigDirty = computed(() => (
  form.value.provider !== savedConfig.value.provider
  || form.value.baseUrl.trim() !== savedConfig.value.baseUrl
  || form.value.model.trim() !== savedConfig.value.model
  || Boolean(form.value.apiKey)
));
const selectedProfile = computed(() => profiles.value.find((profile) => profile.id === activeProfileId.value) || null);
const canRenameProfile = computed(() => Boolean(selectedProfile.value) && !profilesLoading.value && !saving.value);
const canDeleteProfile = computed(() => (
  Boolean(selectedProfile.value)
  && selectedProfile.value?.id !== 'default'
  && !profilesLoading.value
  && !saving.value
));
const canClearApiKey = computed(() => hasExistingKey.value && !saving.value && !profilesLoading.value && !clearingKey.value);
const canRunDiagnostics = computed(() => !saving.value && !diagnosticsLoading.value && !isConfigDirty.value);
const canCopyDiagnostics = computed(() => Boolean(diagnostics.value) && !diagnosticsLoading.value);
const canPreviewVoice = computed(() => !voicesLoading.value && Boolean(selectedVoice.value));
const diagnosticChecks = computed(() => {
  if (!diagnostics.value) {
    return [];
  }

  return [
    { key: 'chat', title: t('settings.diagnosticChat'), ...diagnostics.value.chat },
    { key: 'stt', title: t('settings.diagnosticStt'), ...diagnostics.value.stt },
    { key: 'tts', title: t('settings.diagnosticTts'), ...diagnostics.value.tts },
  ];
});
const providerHint = computed(() => {
  switch (form.value.provider) {
    case 'ollama':
      return t('settings.providerHintOllama');
    case 'custom':
      return t('settings.providerHintCustom');
    default:
      return t('settings.providerHintOpenAI');
  }
});
const apiKeyHint = computed(() => (
  requiresApiKey.value
    ? t('settings.apiKeyHint')
    : t('settings.apiKeyOptionalHint')
));
const apiKeyPlaceholder = computed(() => {
  if (!requiresApiKey.value) {
    return t('settings.apiKeyOptional');
  }

  return hasExistingKey.value ? t('settings.apiKeyKeep') : 'sk-...';
});

function providerLabel(provider: AiProvider) {
  switch (provider) {
    case 'ollama':
      return t('settings.providerOllama');
    case 'custom':
      return t('settings.providerCustom');
    default:
      return t('settings.providerOpenAI');
  }
}

function profileOptionLabel(profile: ConfigProfileInfo) {
  return `${profile.name} · ${providerLabel(profile.provider)} · ${profile.model}`;
}

function applyConfigInfo(config: ConfigInfo) {
  activeProfileId.value = config.active_profile_id;
  form.value.provider = config.provider;
  form.value.baseUrl = config.base_url;
  form.value.model = config.model;
  form.value.apiKey = '';
  hasExistingKey.value = config.has_api_key;
  savedConfig.value = {
    profileId: config.active_profile_id,
    profileName: config.active_profile_name,
    provider: config.provider,
    baseUrl: config.base_url,
    model: config.model,
  };
  diagnostics.value = null;
  diagnosticsError.value = '';
  diagnosticsCopied.value = false;
}

async function loadProfiles() {
  profilesLoading.value = true;
  profileError.value = '';

  try {
    profiles.value = await listConfigProfiles();
  } catch (errorCause) {
    profileError.value = toErrorMessage(errorCause, 'Failed to load config profiles.');
  } finally {
    profilesLoading.value = false;
  }
}

function isKnownDefault(value: string, field: 'baseUrl' | 'model') {
  return Object.values(providerDefaults).some((defaults) => defaults[field] === value);
}

function applyProviderDefaults(nextProvider: AiProvider, previousProvider?: AiProvider) {
  const currentBaseUrl = form.value.baseUrl.trim();
  const currentModel = form.value.model.trim();
  const previousDefaults = previousProvider ? providerDefaults[previousProvider] : undefined;
  const nextDefaults = providerDefaults[nextProvider];

  if (!currentBaseUrl || currentBaseUrl === previousDefaults?.baseUrl || isKnownDefault(currentBaseUrl, 'baseUrl')) {
    form.value.baseUrl = nextDefaults.baseUrl;
  }

  if (!currentModel || currentModel === previousDefaults?.model || isKnownDefault(currentModel, 'model')) {
    form.value.model = nextDefaults.model;
  }
}

function validateProviderFields() {
  const normalizedBaseUrl = form.value.baseUrl.trim();
  const normalizedModel = form.value.model.trim();

  if (!normalizedBaseUrl) {
    return t('settings.apiBaseUrlRequired');
  }

  if (!/^https?:\/\//i.test(normalizedBaseUrl)) {
    return t('settings.apiBaseUrlInvalid');
  }

  if (!normalizedModel) {
    return t('settings.modelRequired');
  }

  return '';
}

function hasDuplicateProfileName(name: string, excludeProfileId?: string) {
  const normalizedName = name.trim().toLowerCase();
  return profiles.value.some((profile) => (
    profile.id !== excludeProfileId
    && profile.name.trim().toLowerCase() === normalizedName
  ));
}

function handleVoiceChange() {
  localStorage.setItem('tts_voice', selectedVoice.value);
}

function diagnosticBadgeClass(status: DiagnosticStatus) {
  return `diagnostic-badge--${status}`;
}

function diagnosticBadgeLabel(status: DiagnosticStatus) {
  return t(`settings.diagnosticStatus.${status}`);
}

function buildDiagnosticsReport() {
  const lines = [
    '# VoxVerse Runtime Diagnostics',
    '',
    `Generated: ${new Date().toISOString()}`,
    `Profile: ${savedConfig.value.profileName} (${savedConfig.value.profileId})`,
    `Provider: ${savedConfig.value.provider}`,
    `Base URL: ${savedConfig.value.baseUrl}`,
    `Model: ${savedConfig.value.model}`,
    `API key saved: ${hasExistingKey.value ? 'yes' : 'no'}`,
    `Automatic feedback: ${autoFeedbackEnabled.value ? 'enabled' : 'disabled'}`,
    '',
    '## Checks',
  ];

  for (const check of diagnosticChecks.value) {
    lines.push(
      '',
      `### ${check.title}`,
      `Status: ${check.status}`,
      `Summary: ${check.summary}`,
      `Details: ${check.details}`,
    );
  }

  return lines.join('\n');
}

async function handleClearApiKey() {
  if (!hasExistingKey.value) {
    return;
  }

  if (!window.confirm(t('settings.apiKeyClearConfirm'))) {
    return;
  }

  clearingKey.value = true;
  saveError.value = '';
  saveSuccess.value = false;

  try {
    const config = await clearActiveApiKey();
    applyConfigInfo(config);
    await loadProfiles();
    saveSuccess.value = true;

    setTimeout(() => {
      saveSuccess.value = false;
    }, 800);
  } catch (errorCause) {
    saveError.value = toErrorMessage(errorCause, 'Failed to clear saved API key.');
  } finally {
    clearingKey.value = false;
  }
}

async function copyTextToClipboard(text: string) {
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }

  const textarea = document.createElement('textarea');
  textarea.value = text;
  textarea.setAttribute('readonly', 'true');
  textarea.style.position = 'fixed';
  textarea.style.opacity = '0';
  document.body.appendChild(textarea);
  textarea.select();

  try {
    document.execCommand('copy');
  } finally {
    document.body.removeChild(textarea);
  }
}

async function handleSave() {
  if (requiresApiKey.value && !form.value.apiKey && !hasExistingKey.value) {
    saveError.value = t('settings.apiKeyRequired');
    return;
  }

  const validationError = validateProviderFields();
  if (validationError) {
    saveError.value = validationError;
    return;
  }

  saving.value = true;
  saveError.value = '';
  saveSuccess.value = false;

  const willHaveStoredKey = hasExistingKey.value || Boolean(form.value.apiKey);
  const normalizedBaseUrl = form.value.baseUrl.trim();
  const normalizedModel = form.value.model.trim();

  try {
    await saveConfig({
      provider: form.value.provider,
      baseUrl: normalizedBaseUrl,
      apiKey: form.value.apiKey,
      model: normalizedModel,
    });

    form.value.baseUrl = normalizedBaseUrl;
    form.value.model = normalizedModel;
    form.value.apiKey = '';
    hasExistingKey.value = willHaveStoredKey;
    savedConfig.value = {
      profileId: activeProfileId.value,
      profileName: savedConfig.value.profileName,
      provider: form.value.provider,
      baseUrl: normalizedBaseUrl,
      model: normalizedModel,
    };
    await loadProfiles();
    diagnostics.value = null;
    diagnosticsError.value = '';
    diagnosticsCopied.value = false;
    saveSuccess.value = true;

    setTimeout(() => {
      saveSuccess.value = false;
    }, 800);
  } catch (errorCause) {
    saveError.value = toErrorMessage(errorCause);
  } finally {
    saving.value = false;
  }
}

async function handleProfileSelect() {
  const nextProfileId = activeProfileId.value;

  if (!nextProfileId || nextProfileId === savedConfig.value.profileId) {
    return;
  }

  if (isConfigDirty.value && !window.confirm(t('settings.profileSwitchUnsaved'))) {
    activeProfileId.value = savedConfig.value.profileId;
    return;
  }

  profilesLoading.value = true;
  profileError.value = '';

  try {
    const config = await switchConfigProfile(nextProfileId);
    applyConfigInfo(config);
    await loadProfiles();
  } catch (errorCause) {
    profileError.value = toErrorMessage(errorCause, 'Failed to switch config profile.');
    activeProfileId.value = savedConfig.value.profileId;
  } finally {
    profilesLoading.value = false;
  }
}

async function handleCreateProfile() {
  const suggestedName = form.value.model.trim() || providerLabel(form.value.provider);
  const profileName = window.prompt(t('settings.profileNamePrompt'), suggestedName);

  if (profileName === null) {
    return;
  }

  const normalizedName = profileName.trim();
  if (!normalizedName) {
    profileError.value = t('settings.profileNameRequired');
    return;
  }

  if (hasDuplicateProfileName(normalizedName)) {
    profileError.value = t('settings.profileNameDuplicate');
    return;
  }

  const validationError = validateProviderFields();
  if (validationError) {
    profileError.value = validationError;
    return;
  }

  if (requiresApiKey.value && !form.value.apiKey) {
    profileError.value = t('settings.profileCreateKeyRequired');
    return;
  }

  profilesLoading.value = true;
  profileError.value = '';
  saveError.value = '';
  saveSuccess.value = false;

  try {
    const config = await createConfigProfile(normalizedName, {
      provider: form.value.provider,
      baseUrl: form.value.baseUrl.trim(),
      apiKey: form.value.apiKey,
      model: form.value.model.trim(),
    });
    applyConfigInfo(config);
    await loadProfiles();
    saveSuccess.value = true;

    setTimeout(() => {
      saveSuccess.value = false;
    }, 800);
  } catch (errorCause) {
    profileError.value = toErrorMessage(errorCause, 'Failed to create config profile.');
  } finally {
    profilesLoading.value = false;
  }
}

async function handleRenameProfile() {
  const profile = selectedProfile.value;
  if (!profile) {
    return;
  }

  const profileName = window.prompt(t('settings.profileRenamePrompt'), profile.name);
  if (profileName === null) {
    return;
  }

  const normalizedName = profileName.trim();
  if (!normalizedName) {
    profileError.value = t('settings.profileNameRequired');
    return;
  }

  if (hasDuplicateProfileName(normalizedName, profile.id)) {
    profileError.value = t('settings.profileNameDuplicate');
    return;
  }

  profilesLoading.value = true;
  profileError.value = '';
  saveError.value = '';
  saveSuccess.value = false;

  try {
    const config = await renameConfigProfile(profile.id, normalizedName);
    applyConfigInfo(config);
    await loadProfiles();
    saveSuccess.value = true;

    setTimeout(() => {
      saveSuccess.value = false;
    }, 800);
  } catch (errorCause) {
    profileError.value = toErrorMessage(errorCause, 'Failed to rename config profile.');
  } finally {
    profilesLoading.value = false;
  }
}

async function handleDeleteProfile() {
  const profile = selectedProfile.value;
  if (!profile || profile.id === 'default') {
    return;
  }

  if (!window.confirm(t('settings.profileDeleteConfirm', { name: profile.name }))) {
    return;
  }

  profilesLoading.value = true;
  profileError.value = '';
  saveError.value = '';
  saveSuccess.value = false;

  try {
    const config = await deleteConfigProfile(profile.id);
    applyConfigInfo(config);
    await loadProfiles();
    saveSuccess.value = true;

    setTimeout(() => {
      saveSuccess.value = false;
    }, 800);
  } catch (errorCause) {
    profileError.value = toErrorMessage(errorCause, 'Failed to delete config profile.');
  } finally {
    profilesLoading.value = false;
  }
}

async function handleRunDiagnostics() {
  if (isConfigDirty.value) {
    diagnosticsError.value = t('settings.saveBeforeDiagnostics');
    return;
  }

  diagnosticsLoading.value = true;
  diagnosticsError.value = '';
  diagnosticsCopied.value = false;

  try {
    diagnostics.value = await runRuntimeDiagnostics();
  } catch (errorCause) {
    diagnosticsError.value = toErrorMessage(errorCause);
  } finally {
    diagnosticsLoading.value = false;
  }
}

async function handleCopyDiagnostics() {
  if (!diagnostics.value) {
    return;
  }

  diagnosticsError.value = '';

  try {
    await copyTextToClipboard(buildDiagnosticsReport());
    diagnosticsCopied.value = true;
    setTimeout(() => {
      diagnosticsCopied.value = false;
    }, 1200);
  } catch (errorCause) {
    diagnosticsError.value = toErrorMessage(errorCause, 'Failed to copy diagnostics report.');
  }
}

async function handleVoicePreview() {
  if (isPreviewSpeaking.value) {
    stopVoicePreview();
    return;
  }

  await speak(previewText.value, {
    lang: previewLocale.value,
    namePattern: selectedVoice.value,
  });
}

function handleAutoFeedbackChange() {
  localStorage.setItem('auto_feedback_enabled', autoFeedbackEnabled.value ? 'true' : 'false');
  window.dispatchEvent(new CustomEvent('voxverse:auto-feedback-changed', {
    detail: { enabled: autoFeedbackEnabled.value },
  }));
}

function handleOverlayClick(e: MouseEvent) {
  if ((e.target as HTMLElement).classList.contains('modal-overlay')) {
    emit('close');
  }
}

onMounted(async () => {
  try {
    const config: ConfigInfo = await getConfig();
    applyConfigInfo(config);
    await loadProfiles();
  } catch {
    // Keep defaults if config cannot be loaded.
  }

  voicesLoading.value = true;
  try {
    ttsVoices.value = await getTtsVoices();
  } catch (errorCause) {
    logError('Could not load TTS voices', errorCause);
  } finally {
    voicesLoading.value = false;
  }
});

watch(selectedTheme, (newTheme) => {
  const normalizedTheme = applyThemePreference(newTheme);
  localStorage.setItem('theme', normalizedTheme);
});

watch(selectedLang, (newLang) => {
  setLanguage(newLang);
});

watch(() => form.value.provider, (nextProvider, previousProvider) => {
  applyProviderDefaults(nextProvider, previousProvider);
  saveError.value = '';
});

watch(
  () => [form.value.provider, form.value.baseUrl, form.value.model, form.value.apiKey],
  () => {
    diagnostics.value = null;
    diagnosticsError.value = '';
    diagnosticsCopied.value = false;
  },
);
</script>

<template>
  <div class="modal-overlay" @click="handleOverlayClick">
    <div class="modal">
      <div class="modal-header">
        <h2 class="modal-title">{{ t('settings.title') }}</h2>
        <button class="modal-close" type="button" @click="emit('close')">x</button>
      </div>

      <div class="modal-body">
        <div class="field">
          <label class="field-label">{{ t('settings.configProfile') }}</label>
          <div class="profile-row">
            <select
              v-model="activeProfileId"
              class="field-input"
              :disabled="profilesLoading || saving"
              @change="handleProfileSelect"
            >
              <option
                v-for="profile in profiles"
                :key="profile.id"
                :value="profile.id"
              >
                {{ profileOptionLabel(profile) }}
              </option>
            </select>
            <button
              class="btn btn--ghost btn--small"
              type="button"
              :disabled="profilesLoading || saving"
              @click="handleCreateProfile"
            >
              {{ t('settings.profileSaveAsNew') }}
            </button>
            <button
              class="btn btn--ghost btn--small"
              type="button"
              :disabled="!canRenameProfile"
              @click="handleRenameProfile"
            >
              {{ t('settings.profileRename') }}
            </button>
            <button
              class="btn btn--danger btn--small"
              type="button"
              :disabled="!canDeleteProfile"
              @click="handleDeleteProfile"
            >
              {{ t('settings.profileDelete') }}
            </button>
          </div>
          <p class="field-hint">{{ t('settings.profileHint') }}</p>
          <p v-if="profileError" class="field-error">{{ profileError }}</p>
        </div>

        <div class="field">
          <label class="field-label">{{ t('settings.provider') }}</label>
          <select v-model="form.provider" class="field-input">
            <option value="openai">{{ t('settings.providerOpenAI') }}</option>
            <option value="ollama">{{ t('settings.providerOllama') }}</option>
            <option value="custom">{{ t('settings.providerCustom') }}</option>
          </select>
          <p class="field-hint">{{ providerHint }}</p>
          <p v-if="form.provider === 'ollama'" class="field-warning">{{ t('settings.sttUnavailableOllama') }}</p>
        </div>

        <div class="field">
          <label class="field-label">{{ t('settings.apiBaseUrl') }}</label>
          <input
            v-model="form.baseUrl"
            class="field-input"
            type="text"
            :placeholder="baseUrlPlaceholder"
          />
          <p class="field-hint">{{ t('settings.apiBaseUrlHint') }}</p>
        </div>

        <div class="field">
          <label class="field-label">
            {{ t('settings.apiKey') }}
            <span v-if="hasExistingKey" class="field-badge">{{ t('settings.apiKeySaved') }}</span>
          </label>
          <input
            v-model="form.apiKey"
            class="field-input"
            type="password"
            :placeholder="apiKeyPlaceholder"
          />
          <div class="field-actions">
            <button
              class="btn btn--ghost btn--small"
              type="button"
              :disabled="!canClearApiKey"
              @click="handleClearApiKey"
            >
              {{ clearingKey ? t('settings.apiKeyClearing') : t('settings.apiKeyClear') }}
            </button>
          </div>
          <p class="field-hint">{{ apiKeyHint }}</p>
        </div>

        <div class="field">
          <label class="field-label">{{ t('settings.model') }}</label>
          <input
            v-model="form.model"
            class="field-input"
            type="text"
            :placeholder="modelPlaceholder"
          />
        </div>

        <div class="field">
          <label class="field-label">{{ t('settings.ttsVoice') }}</label>
          <select
            v-model="selectedVoice"
            class="field-input"
            :disabled="voicesLoading"
            @change="handleVoiceChange"
          >
            <option v-if="voicesLoading" value="">{{ t('settings.loadingVoices') }}</option>
            <optgroup v-for="voiceLocale in popularLocales" :key="voiceLocale" :label="voiceLocale">
              <option
                v-for="voice in ttsVoices.filter((item) => item.locale === voiceLocale)"
                :key="voice.name"
                :value="voice.name"
              >
                {{ voice.friendly_name || voice.name }} ({{ voice.gender }})
              </option>
            </optgroup>
            <optgroup :label="t('settings.otherVoices')">
              <option
                v-for="voice in ttsVoices.filter((item) => !popularLocales.includes(item.locale))"
                :key="voice.name"
                :value="voice.name"
              >
                {{ voice.locale }} - {{ voice.friendly_name || voice.name }}
              </option>
            </optgroup>
          </select>
          <p class="field-hint">{{ t('settings.ttsVoiceHint') }}</p>
          <div class="field-actions">
            <button
              class="btn btn--ghost btn--small"
              type="button"
              :disabled="!canPreviewVoice"
              @click="handleVoicePreview"
            >
              {{ isPreviewSpeaking ? t('settings.stopPreview') : t('settings.testVoice') }}
            </button>
          </div>
          <p v-if="speechNotice" class="field-warning">{{ speechNotice }}</p>
          <p v-if="activeSpeechEngine" class="field-hint">
            {{ t('settings.ttsEngineActive', { engine: speechEngineLabel }) }}
          </p>
          <p v-if="speechError" class="field-error">{{ speechError }}</p>
        </div>

        <div class="field diagnostics-field">
          <div class="diagnostics-header">
            <div>
              <label class="field-label">{{ t('settings.diagnosticsTitle') }}</label>
              <p class="field-hint">{{ t('settings.diagnosticsHint') }}</p>
              <p class="field-hint">{{ t('settings.diagnosticsPrivacyHint') }}</p>
            </div>
            <div class="diagnostics-actions">
              <button
                class="btn btn--ghost btn--small"
                type="button"
                :disabled="!canRunDiagnostics"
                @click="handleRunDiagnostics"
              >
                {{ diagnosticsLoading ? t('settings.diagnosticsRunning') : t('settings.runDiagnostics') }}
              </button>
              <button
                class="btn btn--ghost btn--small"
                type="button"
                :disabled="!canCopyDiagnostics"
                @click="handleCopyDiagnostics"
              >
                {{ diagnosticsCopied ? t('settings.diagnosticsCopied') : t('settings.copyDiagnostics') }}
              </button>
            </div>
          </div>

          <p v-if="isConfigDirty" class="field-warning">{{ t('settings.saveBeforeDiagnostics') }}</p>
          <div v-if="diagnostics" class="diagnostics-list">
            <div
              v-for="check in diagnosticChecks"
              :key="check.key"
              class="diagnostic-card"
            >
              <div class="diagnostic-top">
                <span class="diagnostic-title">{{ check.title }}</span>
                <span class="diagnostic-badge" :class="diagnosticBadgeClass(check.status)">
                  {{ diagnosticBadgeLabel(check.status) }}
                </span>
              </div>
              <p class="diagnostic-summary">{{ check.summary }}</p>
              <p class="diagnostic-details">{{ check.details }}</p>
            </div>
          </div>
          <p v-else class="diagnostic-empty">{{ t('settings.diagnosticsEmpty') }}</p>
        </div>

        <div class="field privacy-field">
          <label class="field-label">{{ t('settings.privacyTitle') }}</label>
          <label class="toggle-row">
            <input
              v-model="autoFeedbackEnabled"
              type="checkbox"
              @change="handleAutoFeedbackChange"
            />
            <span>{{ t('settings.autoFeedback') }}</span>
          </label>
          <p class="field-hint">{{ t('settings.autoFeedbackHint') }}</p>
          <div class="privacy-notes">
            <p>{{ t('settings.localDataHint') }}</p>
            <p>{{ t('settings.keyringDataHint') }}</p>
            <p>{{ t('settings.exportDataHint') }}</p>
          </div>
        </div>

        <div v-if="saveError" class="field-error">{{ saveError }}</div>
        <div v-if="diagnosticsError" class="field-error">{{ diagnosticsError }}</div>
        <div v-if="saveSuccess" class="field-success">{{ t('settings.savedSuccess') }}</div>
      </div>

      <div class="field section-field">
        <label class="field-label">{{ t('settings.language') }}</label>
        <select v-model="selectedLang" class="field-input">
          <option value="en">English</option>
          <option value="zh">简体中文</option>
        </select>
      </div>

      <div class="field section-field section-field--spaced">
        <label class="field-label">{{ t('settings.theme') }}</label>
        <select v-model="selectedTheme" class="field-input">
          <option
            v-for="theme in themeChoices"
            :key="theme.id"
            :value="theme.id"
          >
            {{ theme.name }}
          </option>
        </select>
        <p class="field-hint">{{ selectedThemeDescription }}</p>
      </div>

      <div class="modal-footer">
        <button class="btn btn--ghost" type="button" @click="emit('close')">{{ t('settings.cancel') }}</button>
        <button class="btn btn--primary" type="button" :disabled="saving" @click="handleSave">
          {{ saving ? t('settings.saving') : t('settings.save') }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: var(--overlay-bg);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  animation: fadeIn 0.2s ease-out;
}

.modal {
  background: var(--bg-secondary);
  border: 1px solid var(--border-visible);
  border-radius: var(--radius-xl);
  width: 560px;
  max-width: 90vw;
  box-shadow: var(--shadow-lg);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-lg);
  border-bottom: 1px solid var(--border-subtle);
}

.modal-title {
  font-size: var(--font-size-lg);
  font-weight: 600;
}

.modal-close {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  transition: background var(--transition-fast);
  text-transform: uppercase;
}

.modal-close:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.modal-body {
  padding: var(--space-lg);
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

.field-label {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  font-size: var(--font-size-sm);
  font-weight: 500;
  color: var(--text-secondary);
  margin-bottom: var(--space-xs);
}

.field-badge {
  font-size: var(--font-size-xs);
  color: var(--accent-secondary);
  font-weight: 400;
}

.field-input {
  width: 100%;
  padding: var(--space-sm) var(--space-md);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: var(--font-size-sm);
  transition: border-color var(--transition-fast);
}

.field-input:focus {
  outline: none;
  border-color: var(--accent-primary);
}

.field-input::placeholder {
  color: var(--text-tertiary);
}

.profile-row {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.profile-row .field-input {
  flex: 1;
}

.field-hint {
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
  margin-top: var(--space-xs);
}

.field-warning {
  color: var(--status-warning);
  font-size: var(--font-size-xs);
  margin-top: var(--space-xs);
}

.field-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: var(--space-sm);
}

.diagnostics-field {
  padding-top: var(--space-xs);
}

.privacy-field {
  padding-top: var(--space-xs);
}

.toggle-row {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  color: var(--text-primary);
  font-size: var(--font-size-sm);
}

.toggle-row input {
  width: 16px;
  height: 16px;
  accent-color: var(--accent-primary);
}

.privacy-notes {
  margin-top: var(--space-sm);
  padding: var(--space-sm) var(--space-md);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-tertiary);
  color: var(--text-tertiary);
  font-size: var(--font-size-xs);
  line-height: 1.5;
}

.privacy-notes p + p {
  margin-top: 4px;
}

.diagnostics-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-md);
}

.diagnostics-actions {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  flex-shrink: 0;
}

.diagnostics-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  margin-top: var(--space-sm);
}

.diagnostic-card {
  padding: var(--space-md);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-tertiary);
}

.diagnostic-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-sm);
}

.diagnostic-title {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--text-primary);
}

.diagnostic-badge {
  padding: 4px 8px;
  border-radius: 999px;
  font-size: var(--font-size-xs);
  font-weight: 600;
}

.diagnostic-badge--success {
  background: var(--status-success-soft);
  color: var(--status-success);
}

.diagnostic-badge--warning {
  background: var(--status-warning-soft);
  color: var(--status-warning);
}

.diagnostic-badge--error {
  background: var(--status-danger-soft);
  color: var(--status-danger);
}

.diagnostic-summary {
  margin-top: var(--space-sm);
  color: var(--text-primary);
  font-size: var(--font-size-sm);
}

.diagnostic-details,
.diagnostic-empty {
  margin-top: var(--space-xs);
  color: var(--text-tertiary);
  font-size: var(--font-size-xs);
  line-height: 1.5;
}

.field-error {
  color: var(--status-danger);
  font-size: var(--font-size-sm);
}

.field-success {
  color: var(--accent-secondary);
  font-size: var(--font-size-sm);
}

.section-field {
  padding: 0 var(--space-lg);
}

.section-field--spaced {
  padding-top: var(--space-md);
  padding-bottom: var(--space-md);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-sm);
  padding: var(--space-md) var(--space-lg);
  border-top: 1px solid var(--border-subtle);
}

.btn {
  padding: var(--space-sm) var(--space-lg);
  border-radius: var(--radius-md);
  font-size: var(--font-size-sm);
  font-weight: 500;
  transition: all var(--transition-fast);
}

.btn--ghost {
  color: var(--text-secondary);
}

.btn--ghost:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.btn--primary {
  background: var(--accent-gradient);
  color: var(--text-on-accent);
}

.btn--danger {
  color: var(--status-danger);
}

.btn--danger:hover:not(:disabled) {
  background: var(--status-danger-soft);
  color: var(--status-danger);
}

.btn--primary:hover:not(:disabled) {
  opacity: 0.9;
}

.btn--primary:disabled,
.btn--ghost:disabled,
.btn--danger:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn--small {
  padding: var(--space-xs) var(--space-md);
  white-space: nowrap;
}
</style>
