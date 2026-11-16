<script setup lang="ts">
import { ref, type ComponentPublicInstance } from 'vue'
import type { EditorTab } from '@/types/app'

const props = defineProps<{
  tabs: EditorTab[]
  activeId: string
}>()

const emit = defineEmits<{
  select: [id: string]
  add: []
  close: [id: string]
  rename: [payload: { id: string; name: string }]
}>()

const renamingId = ref<string | null>(null)
const renameValue = ref('')

function isActive(tab: EditorTab) {
  return tab.id === props.activeId
}

function onClose(tab: EditorTab) {
  if (props.tabs.length <= 1) return
  if (tab.dirty) {
    const confirmed = window.confirm(`¿Cerrar "${tab.name}"? El contenido se descartará.`)
    if (!confirmed) return
  }
  emit('close', tab.id)
}

function startRename(tab: EditorTab) {
  renamingId.value = tab.id
  renameValue.value = tab.name
}

function focusRenameInput(el: Element | ComponentPublicInstance | null) {
  if (el instanceof HTMLInputElement) {
    el.focus()
    el.select()
  }
}

function commitRename() {
  if (!renamingId.value) return
  const id = renamingId.value
  const name = renameValue.value.trim()
  renamingId.value = null
  if (name) emit('rename', { id, name })
}

function cancelRename() {
  renamingId.value = null
}

function onTabKeydown(event: KeyboardEvent, tab: EditorTab) {
  if (event.key === 'Enter' || event.key === ' ') {
    event.preventDefault()
    emit('select', tab.id)
  }
}

function startRenameActive() {
  const tab = props.tabs.find((candidate) => candidate.id === props.activeId)
  if (tab) startRename(tab)
}

defineExpose({ startRenameActive })
</script>

<template>
  <div class="tabs" role="tablist" aria-label="Archivos abiertos">
    <div
      v-for="tab in tabs"
      :key="tab.id"
      class="tab"
      :class="{ 'tab--active': isActive(tab) }"
      role="tab"
      :aria-selected="isActive(tab)"
      tabindex="0"
      :data-testid="`tab-${tab.name}`"
      @click="emit('select', tab.id)"
      @keydown="onTabKeydown($event, tab)"
    >
      <input
        v-if="renamingId === tab.id"
        :ref="focusRenameInput"
        v-model="renameValue"
        class="tab__rename"
        type="text"
        aria-label="Nuevo nombre de archivo"
        @click.stop
        @blur="commitRename"
        @keydown.enter.prevent="commitRename"
        @keydown.esc.prevent="cancelRename"
      />
      <button
        v-else
        type="button"
        class="tab__name"
        :title="`${tab.name} — doble clic para renombrar`"
        @dblclick.stop="startRename(tab)"
      >
        {{ tab.name }}
      </button>
      <span
        v-if="tab.dirty || !tab.sketchId"
        class="tab__dirty"
        :title="tab.sketchId ? 'Cambios sin guardar' : 'Borrador sin guardar'"
        :aria-label="tab.sketchId ? 'Cambios sin guardar' : 'Borrador sin guardar'"
      >●</span>
      <button
        type="button"
        class="tab__close"
        :disabled="tabs.length <= 1"
        :aria-label="`Cerrar ${tab.name}`"
        title="Cerrar pestaña"
        @click.stop="onClose(tab)"
      >
        ×
      </button>
    </div>

    <button
      type="button"
      class="tabs__add"
      aria-label="Nueva pestaña"
      title="Nueva pestaña"
      data-testid="tab-add"
      @click="emit('add')"
    >
      +
    </button>
  </div>
</template>

<style scoped>
.tabs {
  display: flex;
  align-items: stretch;
  flex: 1;
  gap: 1px;
  min-width: 0;
  height: calc(var(--kor-titlebar-height) - 1px);
  padding: 0 var(--kor-space-2);
  background-color: transparent;
  overflow-x: auto;
  scrollbar-width: thin;
}

.tab {
  display: flex;
  align-items: center;
  gap: var(--kor-space-1);
  min-width: 0;
  max-width: 220px;
  padding: 0 var(--kor-space-2);
  border: 1px solid transparent;
  border-bottom: none;
  border-radius: var(--kor-radius-md) var(--kor-radius-md) 0 0;
  color: var(--kor-text-muted);
  cursor: pointer;
  user-select: none;
  transition:
    background-color var(--kor-dur-fast) var(--kor-ease-out),
    color var(--kor-dur-fast) var(--kor-ease-out);
}

.tab:hover {
  color: var(--kor-text);
  background-color: var(--kor-surface-3);
}

.tab:focus-visible {
  outline: none;
  box-shadow: var(--kor-shadow-focus);
}

.tab--active {
  color: var(--kor-text-strong);
  background-color: var(--kor-surface);
  border-color: var(--kor-surface-3);
  box-shadow: inset 0 -2px 0 var(--kor-accent);
}

.tab__name {
  flex: 1;
  min-width: 0;
  padding: 0;
  background: none;
  border: none;
  color: inherit;
  font-size: var(--kor-text-sm);
  text-align: left;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  cursor: pointer;
}

.tab__rename {
  flex: 1;
  min-width: 60px;
  height: 24px;
  padding: 0 var(--kor-space-1);
  color: var(--kor-text-strong);
  background-color: var(--kor-surface-3);
  border: 1px solid var(--kor-accent);
  border-radius: var(--kor-radius-sm);
}

.tab__dirty {
  flex: none;
  color: var(--kor-warning);
  font-size: 10px;
  line-height: 1;
}

.tab__close {
  flex: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  padding: 0;
  background: none;
  border: none;
  border-radius: var(--kor-radius-sm);
  color: var(--kor-text-muted);
  font-size: 16px;
  line-height: 1;
  cursor: pointer;
  transition:
    background-color var(--kor-dur-fast) var(--kor-ease-out),
    color var(--kor-dur-fast) var(--kor-ease-out);
}

.tab__close:hover:not(:disabled) {
  color: var(--kor-text-strong);
  background-color: var(--kor-surface-3);
}

.tab__close:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.tabs__add {
  flex: none;
  align-self: center;
  width: 28px;
  height: 28px;
  margin-left: var(--kor-space-1);
  padding: 0;
  background: none;
  border: none;
  border-radius: var(--kor-radius-md);
  color: var(--kor-text-muted);
  font-size: 20px;
  line-height: 1;
  cursor: pointer;
  transition:
    background-color var(--kor-dur-fast) var(--kor-ease-out),
    color var(--kor-dur-fast) var(--kor-ease-out);
}

.tabs__add:hover {
  color: var(--kor-text-strong);
  background-color: var(--kor-surface-3);
}

.tabs__add:active {
  transform: scale(0.96);
}
</style>
