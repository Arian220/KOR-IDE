import type { EditorCursor, EditorTab, KorConfig, KorSession } from '@/types/app'

export const CONFIG_KEY = 'kor.config.v1'
export const SESSION_KEY = 'kor.session.v1'

export const DEFAULT_CONFIG: KorConfig = {
  theme: 'system',
  autosave: true,
  autosaveDelayMs: 800,
}

function readJson<T>(key: string): T | null {
  try {
    const raw = window.localStorage.getItem(key)
    if (!raw) return null
    return JSON.parse(raw) as T
  } catch {
    return null
  }
}

function writeJson(key: string, value: unknown): void {
  try {
    window.localStorage.setItem(key, JSON.stringify(value))
  } catch {
    // Almacenamiento no disponible: la sesión sigue en memoria.
  }
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value))
}

function sanitizeCursor(cursor: unknown): EditorCursor | null {
  if (!cursor || typeof cursor !== 'object') return null
  const candidate = cursor as Partial<EditorCursor>
  if (typeof candidate.line !== 'number' || typeof candidate.column !== 'number') return null
  return {
    line: Math.max(1, Math.floor(candidate.line)),
    column: Math.max(1, Math.floor(candidate.column)),
  }
}

export function loadConfig(): KorConfig {
  const stored = readJson<Partial<KorConfig>>(CONFIG_KEY)
  if (!stored) return { ...DEFAULT_CONFIG }

  const theme =
    stored.theme === 'dark' || stored.theme === 'light' || stored.theme === 'system'
      ? stored.theme
      : DEFAULT_CONFIG.theme

  return {
    theme,
    autosave: typeof stored.autosave === 'boolean' ? stored.autosave : DEFAULT_CONFIG.autosave,
    autosaveDelayMs:
      typeof stored.autosaveDelayMs === 'number'
        ? clamp(Math.floor(stored.autosaveDelayMs), 200, 5000)
        : DEFAULT_CONFIG.autosaveDelayMs,
  }
}

export function saveConfig(config: KorConfig): void {
  writeJson(CONFIG_KEY, config)
}

export function loadSession(): KorSession | null {
  const stored = readJson<Partial<KorSession>>(SESSION_KEY)
  if (!stored || !Array.isArray(stored.tabs)) return null

  const tabs: EditorTab[] = []
  const seen = new Set<string>()

  for (const item of stored.tabs.slice(0, 50)) {
    if (!item || typeof item !== 'object') continue
    const candidate = item as Partial<EditorTab>
    if (
      typeof candidate.id !== 'string' ||
      candidate.id.length === 0 ||
      typeof candidate.name !== 'string' ||
      candidate.name.trim().length === 0 ||
      typeof candidate.content !== 'string' ||
      seen.has(candidate.id)
    ) {
      continue
    }

    seen.add(candidate.id)
    const sketchId =
      typeof candidate.sketchId === 'string' && candidate.sketchId.length > 0
        ? candidate.sketchId
        : null
    const dirty =
      sketchId === null
        ? candidate.content.trim().length > 0
        : typeof candidate.dirty === 'boolean'
          ? candidate.dirty
          : false
    tabs.push({
      id: candidate.id,
      name: candidate.name,
      content: candidate.content,
      cursor: sanitizeCursor(candidate.cursor),
      dirty,
      sketchId,
    })
  }

  if (tabs.length === 0) return null

  const activeTabId =
    typeof stored.activeTabId === 'string' && tabs.some((tab) => tab.id === stored.activeTabId)
      ? stored.activeTabId
      : tabs[0].id

  return { version: 1, activeTabId, tabs }
}

export function saveSession(session: KorSession): void {
  writeJson(SESSION_KEY, {
    version: 1,
    activeTabId: session.activeTabId,
    tabs: session.tabs.map((tab) => ({
      id: tab.id,
      name: tab.name,
      content: tab.content,
      cursor: tab.cursor,
      dirty: tab.dirty,
      sketchId: tab.sketchId,
    })),
  })
}

export function clearSession(): void {
  try {
    window.localStorage.removeItem(SESSION_KEY)
  } catch {
    // Sin almacenamiento disponible.
  }
}
