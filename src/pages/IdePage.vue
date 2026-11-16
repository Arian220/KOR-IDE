<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import ActionBar from '@/components/action/ActionBar.vue'
import AppToast from '@/components/base/AppToast.vue'
import BoardSelector from '@/components/board/BoardSelector.vue'
import ConsoleOutput from '@/components/console/ConsoleOutput.vue'
import ArduinoUnoSvg from '@/components/diagram/ArduinoUnoSvg.vue'
import CodeEditor from '@/components/editor/CodeEditor.vue'
import EditorTabs from '@/components/editor/EditorTabs.vue'
import MemoryBars from '@/components/memory/MemoryBars.vue'
import StatusBar from '@/components/status/StatusBar.vue'
import { useArduino } from '@/composables/useArduino'
import { useDiagram } from '@/composables/useDiagram'
import { useEditor } from '@/composables/useEditor'
import { useTheme } from '@/composables/useTheme'
import type { ThemeMode } from '@/types/app'
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
  memoryVisible,
  memoryError,
  memory,
  consoleVisible,
  consoleText,
  consoleVariant,
  compilar,
  subir,
  detectar,
  cancelar,
  setSelectedPort,
  clearConsole,
} = useArduino()

const editorRef = ref<{ focus: () => void } | null>(null)

const statusDetail = computed(() => {
  const count = tabs.value.length
  const files = `${count} archivo${count === 1 ? '' : 's'}`
  return `${files} · ${config.value.autosave ? 'Autoguardado' : 'Sin autoguardado'}`
})

watch(generatedCode, (code) => {
  editorText.value = code
})

function onPinChange(payload: { pin: string; func: PinFunction | null }) {
  setPin(payload.pin, payload.func)
}

function onThemeChange(event: Event) {
  setTheme((event.target as HTMLSelectElement).value as ThemeMode)
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

function onCompile() {
  void compilar(editorText.value)
}

function onUpload() {
  void subir(editorText.value)
}

function onDetect() {
  void detectar()
}

function onCancel() {
  void cancelar()
}
</script>

<template>
  <div class="ide">
    <header class="ide__header">
      <div class="ide__brand">
        <h1 class="ide__title">KOR</h1>
        <span class="ide__subtitle">Generador Visual + Compilador</span>
      </div>

      <div class="ide__header-actions">
        <label class="theme-control">
          <span>Tema</span>
          <select :value="theme" aria-label="Tema de la interfaz" @change="onThemeChange">
            <option value="system">Sistema</option>
            <option value="dark">Oscuro</option>
            <option value="light">Claro</option>
          </select>
        </label>

        <ActionBar
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
    </header>

    <div class="ide__layout">
      <section class="panel panel--left">
        <h2 class="panel__header">
          Arduino UNO
          <span class="panel__hint">(Rueda del ratón para zoom)</span>
        </h2>
        <ArduinoUnoSvg :pin-states="estadoPines" @change="onPinChange" />
      </section>

      <section class="panel panel--right">
        <EditorTabs
          :tabs="tabs"
          :active-id="activeTabId"
          @select="selectTab"
          @add="onAddTab"
          @close="onCloseTab"
          @rename="onRenameTab"
        />

        <BoardSelector
          :boards="boards"
          :selected-port="selectedPort"
          :has-detected="hasDetected"
          :detecting="detecting"
          :error="boardError"
          @update:selected-port="setSelectedPort"
        />

        <MemoryBars
          :visible="memoryVisible"
          :flash="memory.flash"
          :sram="memory.sram"
          :error="memoryError"
        >
          <ConsoleOutput
            :visible="consoleVisible"
            :text="consoleText"
            :variant="consoleVariant"
            @clear="clearConsole"
          />
        </MemoryBars>

        <CodeEditor
          v-if="activeTab"
          ref="editorRef"
          v-model="editorText"
          :tab-id="activeTab.id"
          :theme="resolvedTheme"
          :cursor="activeTab.cursor"
          @update:cursor="setCursor"
        />
      </section>
    </div>

    <StatusBar :label="status" :kind="statusKind" :detail="statusDetail" />
    <AppToast />
  </div>
</template>

<style scoped>
.ide {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background-color: var(--kor-bg);
}

.ide__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--kor-space-4);
  padding: var(--kor-space-2) var(--kor-space-5);
  background-color: var(--kor-surface);
  border-bottom: 1px solid var(--kor-surface-3);
}

.ide__brand {
  display: flex;
  align-items: baseline;
  gap: var(--kor-space-3);
  min-width: 0;
}

.ide__title {
  margin: 0;
  font-size: var(--kor-text-xl);
  font-weight: var(--kor-weight-bold);
  letter-spacing: 0.02em;
}

.ide__subtitle {
  color: var(--kor-text-muted);
  font-size: var(--kor-text-sm);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.ide__header-actions {
  display: flex;
  align-items: center;
  gap: var(--kor-space-4);
  flex-wrap: wrap;
  justify-content: flex-end;
}

.theme-control {
  display: flex;
  align-items: center;
  gap: var(--kor-space-2);
  color: var(--kor-text-muted);
  font-size: var(--kor-text-sm);
  white-space: nowrap;
}

.theme-control select {
  height: 32px;
  padding: 0 var(--kor-space-2);
  color: var(--kor-text);
  background-color: var(--kor-surface-2);
  border: 1px solid var(--kor-border);
  border-radius: var(--kor-radius-md);
  cursor: pointer;
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

@media (max-width: 1100px) {
  .ide__subtitle {
    display: none;
  }
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
