<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { basicSetup } from 'codemirror'
import { Compartment, EditorState } from '@codemirror/state'
import { EditorView } from '@codemirror/view'
import { cpp } from '@codemirror/lang-cpp'
import { redo, undo } from '@codemirror/commands'
import { openSearchPanel } from '@codemirror/search'
import { oneDark } from '@codemirror/theme-one-dark'
import type { EditorCursor, ResolvedTheme } from '@/types/app'

const props = defineProps<{
  modelValue: string
  tabId: string
  theme: ResolvedTheme
  cursor: EditorCursor | null
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'update:cursor': [cursor: EditorCursor]
}>()

const container = ref<HTMLDivElement | null>(null)
const themeCompartment = new Compartment()
const states = new Map<string, EditorState>()

let view: EditorView | null = null
let syncing = false
let restoringCursor = false
let currentTabId = props.tabId

const baseTheme = EditorView.theme({
  '&': {
    height: '100%',
    backgroundColor: 'var(--kor-surface)',
    color: 'var(--kor-text)',
  },
  '.cm-scroller': {
    fontFamily: 'var(--kor-font-mono)',
    fontSize: 'var(--kor-text-sm)',
  },
  '.cm-gutters': {
    backgroundColor: 'var(--kor-surface-2)',
    color: 'var(--kor-text-muted)',
    border: 'none',
  },
  '.cm-activeLine': {
    backgroundColor: 'color-mix(in srgb, var(--kor-accent) 10%, transparent)',
  },
  '.cm-activeLineGutter': {
    backgroundColor: 'color-mix(in srgb, var(--kor-accent) 12%, transparent)',
  },
  '.cm-selectionBackground, &.cm-focused .cm-selectionBackground': {
    backgroundColor: 'color-mix(in srgb, var(--kor-accent) 30%, transparent)',
  },
})

function themeExtension(theme: ResolvedTheme) {
  return theme === 'dark' ? [oneDark, baseTheme] : [baseTheme]
}

function cursorFromState(state: EditorState): EditorCursor {
  const head = state.selection.main.head
  const line = state.doc.lineAt(head)
  return { line: line.number, column: head - line.from + 1 }
}

function selectionForCursor(state: EditorState, cursor: EditorCursor | null) {
  if (!cursor) return undefined
  const lineNumber = Math.min(Math.max(1, cursor.line), state.doc.lines)
  const line = state.doc.line(lineNumber)
  const position = Math.min(line.to, line.from + Math.max(0, cursor.column - 1))
  return { anchor: position }
}

function createExtensions() {
  return [
    basicSetup,
    cpp(),
    themeCompartment.of(themeExtension(props.theme)),
    EditorView.updateListener.of((update) => {
      if (update.docChanged && !syncing) {
        emit('update:modelValue', update.state.doc.toString())
      }
      if (update.selectionSet && !restoringCursor) {
        emit('update:cursor', cursorFromState(update.state))
      }
    }),
  ]
}

function restoreCursor() {
  if (!view || !props.cursor) return
  const selection = selectionForCursor(view.state, props.cursor)
  if (!selection) return
  if (view.state.selection.main.head === selection.anchor && view.state.selection.main.empty) {
    return
  }
  restoringCursor = true
  view.dispatch({ selection })
  restoringCursor = false
}

function createState(doc: string, cursor: EditorCursor | null) {
  const state = EditorState.create({ doc, extensions: createExtensions() })
  const selection = selectionForCursor(state, cursor)
  if (!selection) return state
  return state.update({ selection }).state
}

onMounted(() => {
  if (!container.value) return
  view = new EditorView({
    state: createState(props.modelValue, props.cursor),
    parent: container.value,
  })
})

watch(
  () => props.tabId,
  (tabId) => {
    if (!view || tabId === currentTabId) return
    states.set(currentTabId, view.state)
    currentTabId = tabId

    const stored = states.get(tabId)
    if (stored && stored.doc.toString() === props.modelValue) {
      view.setState(stored)
      return
    }
    view.setState(createState(props.modelValue, props.cursor))
  },
)

watch(
  () => props.modelValue,
  (value) => {
    if (!view) return
    if (view.state.doc.toString() === value) return
    syncing = true
    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: value },
    })
    syncing = false
    restoreCursor()
  },
)

watch(
  () => props.cursor,
  () => restoreCursor(),
  { deep: true },
)

watch(
  () => props.theme,
  (theme) => {
    view?.dispatch({ effects: themeCompartment.reconfigure(themeExtension(theme)) })
  },
)

onBeforeUnmount(() => {
  view?.destroy()
  view = null
  states.clear()
})

function selectedText(): string {
  if (!view) return ''
  const { from, to } = view.state.selection.main
  return view.state.sliceDoc(from, to)
}

async function copySelection(): Promise<boolean> {
  if (!view) return false
  const text = selectedText() || view.state.doc.toString()
  try {
    await navigator.clipboard.writeText(text)
    return true
  } catch {
    return false
  }
}

async function cutSelection(): Promise<boolean> {
  if (!view) return false
  const { from, to } = view.state.selection.main
  if (from === to) return false
  const text = view.state.sliceDoc(from, to)
  try {
    await navigator.clipboard.writeText(text)
  } catch {
    return false
  }
  view.dispatch({ changes: { from, to, insert: '' } })
  view.focus()
  return true
}

async function pasteClipboard(): Promise<boolean> {
  if (!view) return false
  try {
    const text = await navigator.clipboard.readText()
    if (!text) return false
    const { from, to } = view.state.selection.main
    view.dispatch({
      changes: { from, to, insert: text },
      selection: { anchor: from + text.length },
    })
    view.focus()
    return true
  } catch {
    return false
  }
}

function runUndo() {
  if (!view) return
  undo(view)
  view.focus()
}

function runRedo() {
  if (!view) return
  redo(view)
  view.focus()
}

function openFind() {
  if (!view) return
  openSearchPanel(view)
  view.focus()
}

defineExpose({
  getText: () => view?.state.doc.toString() ?? props.modelValue,
  focus: () => view?.focus(),
  copySelection,
  cutSelection,
  pasteClipboard,
  undo: runUndo,
  redo: runRedo,
  find: openFind,
})
</script>

<template>
  <div ref="container" class="code-editor" />
</template>

<style scoped>
.code-editor {
  flex: 1;
  overflow: auto;
  min-height: 0;
}

.code-editor :deep(.cm-editor) {
  height: 100%;
}

.code-editor :deep(.cm-scroller) {
  font-family: var(--kor-font-mono) !important;
}
</style>
