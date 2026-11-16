<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import ActionBar from '@/components/action/ActionBar.vue'
import AppPrompt from '@/components/base/AppPrompt.vue'
import AppToast from '@/components/base/AppToast.vue'
import BoardSelector from '@/components/board/BoardSelector.vue'
import ConsoleOutput from '@/components/console/ConsoleOutput.vue'
import ArduinoUnoSvg from '@/components/diagram/ArduinoUnoSvg.vue'
import CodeEditor from '@/components/editor/CodeEditor.vue'
import EditorTabs from '@/components/editor/EditorTabs.vue'
import AppMenuBar from '@/components/layout/AppMenuBar.vue'
import MemoryBars from '@/components/memory/MemoryBars.vue'
import SketchPicker from '@/components/sketch/SketchPicker.vue'
import StatusBar from '@/components/status/StatusBar.vue'
import { useArduino } from '@/composables/useArduino'
import { useDiagram } from '@/composables/useDiagram'
import { useEditor } from '@/composables/useEditor'
import { useSketchFiles } from '@/composables/useSketchFiles'
import { useTheme } from '@/composables/useTheme'
import { useToast } from '@/composables/useToast'
import { useWindowControls } from '@/composables/useWindowControls'
import type { AppMenuDefinition } from '@/types/app'
import type { PinFunction } from '@/types/contract'

const { estadoPines, generatedCode, setPin } = useDiagram()
const {
  tabs,
  activeTab,
  activeTabId,
  editorText,
  addTab,
  closeTab,
  selectTab,
  renameTab,
  setCursor,
} = useEditor()
const { theme, resolvedTheme, config, setTheme } = useTheme()
const {
  busyAction,
  detecting,
  isBusy,
  canCancel,
  boards,
  selectedPort,
  hasDetected,
  boardError,
  status,
  statusKind,
  memoryError,
  memory,
  consoleText,
  consoleVariant,
  compilar,
  subir,
  detectar,
  cancelar,
  setSelectedPort,
  clearConsole,
} = useArduino()
const { show: showToast } = useToast()
const windowControls = useWindowControls()
const {
  sketches,
  loadingSketches,
  sketchListError,
  guardarTab,
  guardarActivo,
  guardarComo,
  guardarTodo,
  cargarSketches,
  abrirSketch,
} = useSketchFiles()

const editorRef = ref<{ focus: () => void } | null>(null)
const editorTabsRef = ref<{ startRenameActive: () => void } | null>(null)
const saveAsOpen = ref(false)
const saveAsName = ref('')
const pickerOpen = ref(false)

const menus = computed<AppMenuDefinition[]>(() => [
  {
    id: 'file',
    label: 'Archivo',
    items: [
      { id: 'file.new', label: 'Nuevo archivo', shortcut: 'Ctrl+N' },
      { id: 'file.open', label: 'Abrir sketch…', shortcut: 'Ctrl+O' },
      { separator: true },
      { id: 'file.save', label: 'Guardar', shortcut: 'Ctrl+S' },
      { id: 'file.saveAs', label: 'Guardar como…', shortcut: 'Ctrl+Shift+S' },
      { id: 'file.saveAll', label: 'Guardar todo', shortcut: 'Ctrl+Alt+S' },
      { separator: true },
      {
        id: 'file.close',
        label: 'Cerrar pestaña',
        shortcut: 'Ctrl+W',
        disabled: tabs.value.length <= 1,
      },
      { id: 'file.exit', label: 'Salir' },
    ],
  },
  {
    id: 'edit',
    label: 'Editar',
    items: [
      { id: 'edit.rename', label: 'Renombrar pestaña' },
      { id: 'edit.copy', label: 'Copiar código', disabled: editorText.value.length === 0 },
    ],
  },
  {
    id: 'boards',
    label: 'Placas',
    items: [
      { id: 'boards.detect', label: 'Detectar placas', disabled: isBusy.value },
      { id: 'boards.upload', label: 'Subir a la placa', disabled: isBusy.value },
    ],
  },
  {
    id: 'run',
    label: 'Ejecutar',
    items: [
      { id: 'run.compile', label: 'Compilar', disabled: isBusy.value },
      { id: 'run.cancel', label: 'Cancelar operación', disabled: !canCancel.value },
    ],
  },
  {
    id: 'terminal',
    label: 'Terminal',
    items: [
      { id: 'terminal.copy', label: 'Copiar salida', disabled: consoleText.value.length === 0 },
      { id: 'terminal.clear', label: 'Limpiar terminal', disabled: consoleText.value.length === 0 },
    ],
  },
  {
    id: 'window',
    label: 'Ventana',
    items: [
      { id: 'window.theme.system', label: 'Tema del sistema', checked: theme.value === 'system' },
      { id: 'window.theme.dark', label: 'Tema oscuro', checked: theme.value === 'dark' },
      { id: 'window.theme.light', label: 'Tema claro', checked: theme.value === 'light' },
      { separator: true },
      { id: 'window.minimize', label: 'Minimizar' },
      { id: 'window.maximize', label: 'Maximizar o restaurar' },
      { id: 'window.close', label: 'Cerrar ventana' },
    ],
  },
  {
    id: 'about',
    label: 'Acerca de',
    items: [{ id: 'about.show', label: 'Información de KOR' }],
  },
])

const statusDetail = computed(() => {
  const count = tabs.value.length
  const files = `${count} archivo${count === 1 ? '' : 's'}`
  const estado = activeTab.value?.sketchId ? 'guardado' : 'borrador'
  return `${files} · ${estado} · ${config.value.autosave ? 'Sesión autoguardada' : 'Sesión sin autoguardado'}`
})

watch(generatedCode, (code) => {
  editorText.value = code
})

function onPinChange(payload: { pin: string; func: PinFunction | null }) {
  setPin(payload.pin, payload.func)
}

function closeActiveTab() {
  const tab = activeTab.value
  if (!tab) return
  if (tabs.value.length <= 1) {
    showToast('No se puede cerrar la última pestaña.', 'info')
    return
  }
  if (tab.dirty) {
    const confirmed = window.confirm(`¿Cerrar "${tab.name}"? El contenido se descartará.`)
    if (!confirmed) return
  }
  closeTab(tab.id)
}

async function copyEditor() {
  try {
    await navigator.clipboard.writeText(editorText.value)
    showToast('Código copiado.', 'success')
  } catch {
    showToast('No se pudo copiar el código.', 'error')
  }
}

async function copyTerminal() {
  try {
    await navigator.clipboard.writeText(consoleText.value)
    showToast('Salida de la terminal copiada.', 'success')
  } catch {
    showToast('No se pudo copiar la salida.', 'error')
  }
}

function onCommand(id: string) {
  switch (id) {
    case 'file.new':
      void onAddTab()
      break
    case 'file.open':
      void onOpenSketches()
      break
    case 'file.save':
      onSave()
      break
    case 'file.saveAs':
      openSaveAs()
      break
    case 'file.saveAll':
      void guardarTodo()
      break
    case 'file.close':
      closeActiveTab()
      break
    case 'file.exit':
      windowControls.close()
      break
    case 'edit.rename':
      editorTabsRef.value?.startRenameActive()
      break
    case 'edit.copy':
      void copyEditor()
      break
    case 'boards.detect':
      onDetect()
      break
    case 'boards.upload':
      onUpload()
      break
    case 'run.compile':
      onCompile()
      break
    case 'run.cancel':
      onCancel()
      break
    case 'terminal.copy':
      void copyTerminal()
      break
    case 'terminal.clear':
      clearConsole()
      break
    case 'window.theme.system':
      setTheme('system')
      break
    case 'window.theme.dark':
      setTheme('dark')
      break
    case 'window.theme.light':
      setTheme('light')
      break
    case 'window.minimize':
      windowControls.minimize()
      break
    case 'window.maximize':
      windowControls.toggleMaximize()
      break
    case 'window.close':
      windowControls.close()
      break
    case 'about.show':
      showToast('KOR 0.1.0 — IDE visual para Arduino.', 'info')
      break
    default:
      break
  }
}

async function onAddTab() {
  addTab()
  await nextTick()
  editorRef.value?.focus()
}

function onCloseTab(id: string) {
  closeTab(id)
}

function onRenameTab(payload: { id: string; name: string }) {
  renameTab(payload.id, payload.name)
}

function openSaveAs() {
  const tab = activeTab.value
  if (!tab) {
    showToast('No hay un archivo activo para guardar.', 'info')
    return
  }
  saveAsName.value = tab.name.replace(/\.ino$/i, '') || 'sketch'
  saveAsOpen.value = true
}

function onSave() {
  const tab = activeTab.value
  if (!tab) {
    showToast('No hay un archivo activo para guardar.', 'info')
    return
  }
  if (!tab.sketchId) {
    openSaveAs()
    return
  }
  void guardarActivo()
}

async function onSaveAsConfirm(name: string) {
  saveAsOpen.value = false
  await guardarComo(name)
}

async function onOpenSketches() {
  pickerOpen.value = true
  await cargarSketches()
}

async function onSelectSketch(id: string) {
  pickerOpen.value = false
  await abrirSketch(id)
}

async function onRefreshSketches() {
  await cargarSketches()
}

async function onCompile() {
  const tab = activeTab.value
  if (!tab) return
  if (tab.sketchId) {
    const saved = await guardarTab(tab)
    if (!saved) return
  }
  await compilar(editorText.value, {
    sketchId: tab.sketchId,
    sketchName: tab.name,
  })
}

async function onUpload() {
  const tab = activeTab.value
  if (tab?.sketchId) {
    const saved = await guardarTab(tab)
    if (!saved) return
  }
  await subir(editorText.value, {
    sketchId: tab?.sketchId ?? null,
    sketchName: tab?.name ?? 'sketch',
  })
}

function onShortcut(event: KeyboardEvent) {
  if (saveAsOpen.value || pickerOpen.value) return

  const mod = event.ctrlKey || event.metaKey
  if (!mod) return

  const key = event.key.toLowerCase()

  if (key === 's') {
    event.preventDefault()
    if (event.shiftKey) openSaveAs()
    else if (event.altKey) void guardarTodo()
    else onSave()
    return
  }

  if (event.altKey || event.shiftKey) return

  if (key === 'n') {
    event.preventDefault()
    void onAddTab()
  } else if (key === 'o') {
    event.preventDefault()
    void onOpenSketches()
  } else if (key === 'w') {
    event.preventDefault()
    closeActiveTab()
  }
}

onMounted(() => {
  document.addEventListener('keydown', onShortcut)
})

onBeforeUnmount(() => {
  document.removeEventListener('keydown', onShortcut)
})

function onDetect() {
  void detectar()
}

function onCancel() {
  void cancelar()
}
</script>

<template>
  <div class="ide">
    <AppMenuBar
      :menus="menus"
      :theme="theme"
      @command="onCommand"
      @update:theme="setTheme"
    />

    <div class="ide__layout">
      <section class="panel panel--left">
        <h2 class="panel__header">
          Arduino UNO
          <span class="panel__hint">(Rueda del ratón para zoom)</span>
        </h2>
        <ArduinoUnoSvg :pin-states="estadoPines" @change="onPinChange" />

        <MemoryBars :flash="memory.flash" :sram="memory.sram" :error="memoryError" />
      </section>

      <section class="panel panel--right">
        <div class="editor-top">
          <EditorTabs
            ref="editorTabsRef"
            :tabs="tabs"
            :active-id="activeTabId"
            @select="selectTab"
            @add="onAddTab"
            @close="onCloseTab"
            @rename="onRenameTab"
          />

          <ActionBar
            class="editor-top__actions"
            :busy-action="busyAction"
            :upload-port="selectedPort"
            :is-busy="isBusy"
            :can-cancel="canCancel"
            @detect="onDetect"
            @compile="onCompile"
            @upload="onUpload"
            @cancel="onCancel"
          />
        </div>

        <BoardSelector
          :boards="boards"
          :selected-port="selectedPort"
          :has-detected="hasDetected"
          :detecting="detecting"
          :error="boardError"
          @update:selected-port="setSelectedPort"
        />

        <CodeEditor
          v-if="activeTab"
          ref="editorRef"
          v-model="editorText"
          :tab-id="activeTab.id"
          :theme="resolvedTheme"
          :cursor="activeTab.cursor"
          @update:cursor="setCursor"
        />

        <ConsoleOutput
          :text="consoleText"
          :variant="consoleVariant"
          @clear="clearConsole"
        />
      </section>
    </div>

    <StatusBar :label="status" :kind="statusKind" :detail="statusDetail" />
    <AppToast />

    <AppPrompt
      :open="saveAsOpen"
      title="Guardar como"
      label="Nombre del sketch"
      :initial="saveAsName"
      placeholder="blink.ino"
      confirm-label="Guardar"
      @confirm="onSaveAsConfirm"
      @cancel="saveAsOpen = false"
    />

    <SketchPicker
      :open="pickerOpen"
      :sketches="sketches"
      :loading="loadingSketches"
      :error="sketchListError"
      @select="onSelectSketch"
      @close="pickerOpen = false"
      @refresh="onRefreshSketches"
    />
  </div>
</template>

<style scoped>
.ide {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background-color: var(--kor-bg);
}

.ide__layout {
  display: flex;
  gap: var(--kor-space-4);
  flex: 1;
  min-height: 0;
  padding: var(--kor-space-4) var(--kor-space-5);
}

.panel {
  display: flex;
  flex-direction: column;
  background-color: var(--kor-surface);
  border: 1px solid var(--kor-surface-3);
  border-radius: var(--kor-radius-lg);
  overflow: hidden;
}

.panel--left {
  flex: 0 0 clamp(320px, 34vw, 460px);
  padding: var(--kor-space-4);
}

.panel--right {
  flex: 1;
  min-width: 0;
  min-height: 0;
}

.editor-top {
  display: flex;
  align-items: stretch;
  flex: none;
  min-height: calc(var(--kor-titlebar-height) - 1px);
  background-color: var(--kor-surface-2);
  border-bottom: 1px solid var(--kor-surface-3);
}

.editor-top :deep(.action-bar) {
  align-items: center;
  flex: none;
  flex-wrap: nowrap;
  padding: 0 var(--kor-space-2);
}

.panel__header {
  display: flex;
  align-items: baseline;
  gap: var(--kor-space-2);
  margin: 0 0 var(--kor-space-2);
  font-size: var(--kor-text-md);
  font-weight: var(--kor-weight-bold);
}

.panel__hint {
  color: var(--kor-text-muted);
  font-size: var(--kor-text-xs);
  font-weight: var(--kor-weight-normal);
}

@media (max-width: 900px) {
  .ide__layout {
    flex-direction: column;
  }

  .panel--left {
    flex: 0 0 40%;
  }
}
</style>
