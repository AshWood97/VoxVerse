export const THEME_PRESETS = [
  {
    id: 'graphite-minimal',
    nameKey: 'settings.themes.graphiteMinimal.name',
    descriptionKey: 'settings.themes.graphiteMinimal.description',
  },
  {
    id: 'warm-paper',
    nameKey: 'settings.themes.warmPaper.name',
    descriptionKey: 'settings.themes.warmPaper.description',
  },
  {
    id: 'aurora-glass',
    nameKey: 'settings.themes.auroraGlass.name',
    descriptionKey: 'settings.themes.auroraGlass.description',
  },
  {
    id: 'code-console',
    nameKey: 'settings.themes.codeConsole.name',
    descriptionKey: 'settings.themes.codeConsole.description',
  },
  {
    id: 'dark-signal',
    nameKey: 'settings.themes.darkSignal.name',
    descriptionKey: 'settings.themes.darkSignal.description',
  },
  {
    id: 'liquid-frost',
    nameKey: 'settings.themes.liquidFrost.name',
    descriptionKey: 'settings.themes.liquidFrost.description',
  },
  {
    id: 'enterprise-carbon',
    nameKey: 'settings.themes.enterpriseCarbon.name',
    descriptionKey: 'settings.themes.enterpriseCarbon.description',
  },
  {
    id: 'primer-dev',
    nameKey: 'settings.themes.primerDev.name',
    descriptionKey: 'settings.themes.primerDev.description',
  },
  {
    id: 'creative-spectrum',
    nameKey: 'settings.themes.creativeSpectrum.name',
    descriptionKey: 'settings.themes.creativeSpectrum.description',
  },
  {
    id: 'luxury-console',
    nameKey: 'settings.themes.luxuryConsole.name',
    descriptionKey: 'settings.themes.luxuryConsole.description',
  },
] as const;

export type ThemePresetId = typeof THEME_PRESETS[number]['id'];
export type ThemePreferenceId = ThemePresetId | 'system';

const DEFAULT_THEME: ThemePresetId = 'graphite-minimal';
const LIGHT_SYSTEM_THEME: ThemePresetId = 'warm-paper';
const DARK_SYSTEM_THEME: ThemePresetId = 'graphite-minimal';
const presetIds = new Set<string>(THEME_PRESETS.map((preset) => preset.id));
let removeSystemThemeListener: (() => void) | null = null;

export function isThemePresetId(value: string): value is ThemePresetId {
  return presetIds.has(value);
}

export function normalizeThemePreference(value: string | null | undefined): ThemePreferenceId {
  if (value === 'system') {
    return 'system';
  }

  if (value === 'dark') {
    return 'graphite-minimal';
  }

  if (value === 'light') {
    return 'warm-paper';
  }

  if (value && isThemePresetId(value)) {
    return value;
  }

  return DEFAULT_THEME;
}

export function resolveThemePreference(preference: ThemePreferenceId): ThemePresetId {
  if (preference !== 'system') {
    return preference;
  }

  if (typeof window === 'undefined' || !window.matchMedia) {
    return DARK_SYSTEM_THEME;
  }

  return window.matchMedia('(prefers-color-scheme: light)').matches
    ? LIGHT_SYSTEM_THEME
    : DARK_SYSTEM_THEME;
}

export function applyThemePreference(value: string | null | undefined): ThemePreferenceId {
  const preference = normalizeThemePreference(value);

  const applyResolvedTheme = () => {
    const resolvedTheme = resolveThemePreference(preference);
    document.documentElement.setAttribute('data-theme', resolvedTheme);
    document.documentElement.setAttribute('data-theme-preference', preference);
  };

  applyResolvedTheme();

  if (removeSystemThemeListener) {
    removeSystemThemeListener();
    removeSystemThemeListener = null;
  }

  if (preference === 'system' && typeof window !== 'undefined' && window.matchMedia) {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: light)');
    const handleSystemThemeChange = () => applyResolvedTheme();

    if (mediaQuery.addEventListener) {
      mediaQuery.addEventListener('change', handleSystemThemeChange);
      removeSystemThemeListener = () => mediaQuery.removeEventListener('change', handleSystemThemeChange);
    } else {
      mediaQuery.addListener(handleSystemThemeChange);
      removeSystemThemeListener = () => mediaQuery.removeListener(handleSystemThemeChange);
    }
  }

  return preference;
}
