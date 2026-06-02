import { createI18n } from 'vue-i18n';
import en from './locales/en.json';
import zh from './locales/zh.json';

export type AppLanguage = 'en' | 'zh';

function isAppLanguage(value: string | null): value is AppLanguage {
  return value === 'en' || value === 'zh';
}

// Read language from localStorage or default to system language
const savedLang = localStorage.getItem('app_language');
const systemLang: AppLanguage = navigator.language.startsWith('zh') ? 'zh' : 'en';
const locale: AppLanguage = isAppLanguage(savedLang) ? savedLang : systemLang;

export const i18n = createI18n({
  legacy: false, // For Vue 3 Composition API
  locale,
  fallbackLocale: 'en',
  messages: {
    en,
    zh
  }
});

// Helper to switch language
export function setLanguage(lang: AppLanguage) {
  i18n.global.locale.value = lang;
  localStorage.setItem('app_language', lang);
}
