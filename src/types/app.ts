export type ThemeMode = 'system' | 'dark' | 'light'
export type ResolvedTheme = 'dark' | 'light'
export type ToastKind = 'info' | 'success' | 'error'

export interface EditorCursor {
  line: number
  column: number
}

export interface EditorTab {
  id: string
  name: string
  content: string
  cursor: EditorCursor | null
  dirty: boolean
  sketchId: string | null
}

export interface KorConfig {
  theme: ThemeMode
  autosave: boolean
  autosaveDelayMs: number
}

export interface KorSession {
  version: 1
  activeTabId: string
  tabs: EditorTab[]
}

export interface SketchSummary {
  id: string
  name: string
  updated_at: string
  fqbn: string | null
}

export interface BoardInfo {
  port: string
  label: string
  protocol: string | null
  fqbn: string | null
}

export interface AppMenuItem {
  id: string
  label: string
  shortcut?: string
  disabled?: boolean
  checked?: boolean
}

export interface AppMenuSeparator {
  separator: true
}

export type AppMenuEntry = AppMenuItem | AppMenuSeparator

export interface AppMenuDefinition {
  id: string
  label: string
  items: AppMenuEntry[]
}
