import { computed, ref, watch } from 'vue'
import { loadConfig, saveConfig } from '@/api'
import type { KorConfig, ResolvedTheme, ThemeMode } from '@/types/app'

const config = ref<KorConfig>(loadConfig())
const systemTheme = ref<ResolvedTheme>('dark')
const resolvedTheme = computed<ResolvedTheme>(() =>
  config.value.theme === 'system' ? systemTheme.value : config.value.theme,
)

let initialized = false
let mediaQuery: MediaQueryList | null = null

function applyTheme(theme: ResolvedTheme) {
  document.documentElement.dataset.theme = theme
}

function readSystemTheme() {
  if (typeof window === 'undefined' || !window.matchMedia) {
    systemTheme.value = 'dark'
    return
  }
  systemTheme.value = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

function handleSystemChange() {
  readSystemTheme()
  if (config.value.theme === 'system') applyTheme(systemTheme.value)
}

export function initTheme() {
  readSystemTheme()
  if (!initialized && typeof window !== 'undefined' && window.matchMedia) {
    mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    mediaQuery.addEventListener('change', handleSystemChange)
    initialized = true
  }
  applyTheme(resolvedTheme.value)
}

export function useTheme() {
  watch(
    resolvedTheme,
    (theme) => {
      applyTheme(theme)
    },
    { immediate: true },
  )

  function setTheme(theme: ThemeMode) {
    config.value = { ...config.value, theme }
    saveConfig(config.value)
    if (theme !== 'system') applyTheme(theme)
  }

  function toggleTheme() {
    setTheme(resolvedTheme.value === 'dark' ? 'light' : 'dark')
  }

  return {
    config,
    theme: computed(() => config.value.theme),
    resolvedTheme,
    isDark: computed(() => resolvedTheme.value === 'dark'),
    setTheme,
    toggleTheme,
    mediaQuery,
  }
}
