import { computed, ref } from 'vue'
import { loadConfig, loadSession, saveSession } from '@/api'
import type { EditorCursor, EditorTab, KorSession } from '@/types/app'

const INITIAL_TEXT = '// Esperando configuración...\n'
const DEFAULT_TAB_NAME = 'blink.ino'

function createId(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID()
  }
  return `tab-${Date.now()}-${Math.random().toString(16).slice(2)}`
}

function createTab(name: string, content = '', cursor: EditorCursor | null = null): EditorTab {
  return {
    id: createId(),
    name,
    content,
    cursor,
    dirty: content.trim().length > 0,
    sketchId: null,
  }
}

const storedSession = loadSession()
const initialTabs =
  storedSession && storedSession.tabs.length > 0
    ? storedSession.tabs
    : [createTab(DEFAULT_TAB_NAME, INITIAL_TEXT)]

const tabs = ref<EditorTab[]>(initialTabs)
const activeTabId = ref<string>(storedSession?.activeTabId ?? initialTabs[0].id)
const config = loadConfig()

const activeTab = computed<EditorTab | null>(() => {
  return tabs.value.find((tab) => tab.id === activeTabId.value) ?? tabs.value[0] ?? null
})

const editorText = computed<string>({
  get: () => activeTab.value?.content ?? '',
  set: (value: string) => {
    const tab = activeTab.value
    if (!tab || tab.content === value) return
    tab.content = value
    tab.dirty = true
    scheduleSave()
  },
})

let saveTimer: ReturnType<typeof setTimeout> | null = null

function snapshot(): KorSession {
  return { version: 1, activeTabId: activeTabId.value, tabs: tabs.value }
}

export function saveNow(): void {
  if (saveTimer) {
    clearTimeout(saveTimer)
    saveTimer = null
  }
  saveSession(snapshot())
}

export function scheduleSave(): void {
  if (!config.autosave) return
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(saveNow, config.autosaveDelayMs)
}

function uniqueName(base: string, ignoreId?: string): string {
  const normalized = base.trim() || DEFAULT_TAB_NAME
  const withExtension = normalized.includes('.') ? normalized : `${normalized}.ino`
  const existing = new Set(tabs.value.filter((tab) => tab.id !== ignoreId).map((tab) => tab.name))
  if (!existing.has(withExtension)) return withExtension

  const dot = withExtension.lastIndexOf('.')
  const stem = dot === -1 ? withExtension : withExtension.slice(0, dot)
  const extension = dot === -1 ? '' : withExtension.slice(dot)
  let counter = 2
  while (existing.has(`${stem}-${counter}${extension}`)) counter += 1
  return `${stem}-${counter}${extension}`
}

function nextTabName(): string {
  let counter = 2
  while (tabs.value.some((tab) => tab.name === `sketch${counter}.ino`)) counter += 1
  return `sketch${counter}.ino`
}

function selectTab(id: string): void {
  if (!tabs.value.some((tab) => tab.id === id)) return
  if (activeTabId.value === id) return
  activeTabId.value = id
  scheduleSave()
}

function addTab(): EditorTab {
  const tab = createTab(nextTabName())
  tabs.value = [...tabs.value, tab]
  activeTabId.value = tab.id
  scheduleSave()
  return tab
}

function closeTab(id: string): boolean {
  if (tabs.value.length <= 1) return false
  const index = tabs.value.findIndex((tab) => tab.id === id)
  if (index === -1) return false

  tabs.value = tabs.value.filter((tab) => tab.id !== id)
  if (activeTabId.value === id) {
    const next = tabs.value[Math.min(index, tabs.value.length - 1)]
    activeTabId.value = next.id
  }
  scheduleSave()
  return true
}

function renameTab(id: string, name: string): void {
  const tab = tabs.value.find((candidate) => candidate.id === id)
  if (!tab) return
  const nextName = uniqueName(name, id)
  if (tab.name === nextName) return
  tab.name = nextName
  tab.dirty = true
  scheduleSave()
}

function uniqueTabName(name: string, ignoreId?: string): string {
  return uniqueName(name, ignoreId)
}

function markSaved(id: string, sketchId: string, name?: string): void {
  const tab = tabs.value.find((candidate) => candidate.id === id)
  if (!tab) return
  tab.sketchId = sketchId
  tab.dirty = false
  if (name && tab.name !== name) tab.name = name
  scheduleSave()
}

function openSketch(sketchId: string, name: string, content: string): EditorTab {
  const existing = tabs.value.find((tab) => tab.sketchId === sketchId)
  if (existing) {
    activeTabId.value = existing.id
    scheduleSave()
    return existing
  }

  const tab = createTab(name, content)
  tab.sketchId = sketchId
  tab.dirty = false
  tabs.value = [...tabs.value, tab]
  activeTabId.value = tab.id
  scheduleSave()
  return tab
}

function setCursor(cursor: EditorCursor): void {
  const tab = activeTab.value
  if (!tab) return
  if (tab.cursor?.line === cursor.line && tab.cursor?.column === cursor.column) return
  tab.cursor = {
    line: Math.max(1, Math.floor(cursor.line)),
    column: Math.max(1, Math.floor(cursor.column)),
  }
  scheduleSave()
}

if (typeof window !== 'undefined') {
  window.addEventListener('beforeunload', saveNow)
  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'hidden') saveNow()
  })
}

export function useEditor() {
  return {
    tabs,
    activeTab,
    activeTabId,
    editorText,
    addTab,
    closeTab,
    selectTab,
    renameTab,
    uniqueTabName,
    markSaved,
    openSketch,
    setCursor,
    saveNow,
  }
}
