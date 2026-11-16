<script setup lang="ts">
import type { AppMenuDefinition, AppMenuItem, ThemeMode } from '@/types/app'
import { useWindowControls } from '@/composables/useWindowControls'
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue'

const props = defineProps<{
  menus: AppMenuDefinition[]
  theme: ThemeMode
}>()

const emit = defineEmits<{
  command: [id: string]
  'update:theme': [theme: ThemeMode]
}>()

const { minimize, toggleMaximize, close } = useWindowControls()

const root = ref<HTMLElement | null>(null)
const openId = ref<string | null>(null)

function toggleMenu(id: string) {
  openId.value = openId.value === id ? null : id
}

function hoverMenu(id: string) {
  if (openId.value) openId.value = id
}

function closeMenu() {
  openId.value = null
}

function focusTrigger(id: string) {
  root.value
    ?.querySelector<HTMLButtonElement>(`[data-menu-trigger="${id}"]`)
    ?.focus()
}

function isItem(entry: AppMenuDefinition['items'][number]): entry is AppMenuItem {
  return !('separator' in entry)
}

function runItem(item: AppMenuItem) {
  if (item.disabled) return
  emit('command', item.id)
  closeMenu()
}

function onPointerDown(event: PointerEvent) {
  if (!root.value?.contains(event.target as Node)) closeMenu()
}

function onKeydown(event: KeyboardEvent) {
  if (!openId.value) return
  const current = openId.value

  if (event.key === 'Escape') {
    event.stopPropagation()
    closeMenu()
    focusTrigger(current)
    return
  }

  if (event.key !== 'ArrowDown' && event.key !== 'ArrowUp') return
  const items = Array.from(
    root.value?.querySelectorAll<HTMLButtonElement>('.menu__item:not(:disabled)') ?? [],
  )
  if (items.length === 0) return

  event.preventDefault()
  const index = items.indexOf(document.activeElement as HTMLButtonElement)
  const nextIndex =
    event.key === 'ArrowDown'
      ? (index + 1) % items.length
      : (index - 1 + items.length) % items.length
  items[nextIndex]?.focus()
}

function onThemeChange(event: Event) {
  emit('update:theme', (event.target as HTMLSelectElement).value as ThemeMode)
}

async function openWithKeyboard(id: string) {
  openId.value = id
  await nextTick()
  root.value?.querySelector<HTMLButtonElement>(`[data-menu="${id}"] .menu__item`)?.focus()
}

onMounted(() => {
  document.addEventListener('pointerdown', onPointerDown)
})

onBeforeUnmount(() => {
  document.removeEventListener('pointerdown', onPointerDown)
})
</script>

<template>
  <header ref="root" class="menubar" data-tauri-drag-region @keydown="onKeydown">
    <div class="menubar__brand" data-tauri-drag-region>
      <span class="menubar__logo">KOR</span>
    </div>

    <nav class="menubar__menus" aria-label="Menú principal">
      <div
        v-for="menu in props.menus"
        :key="menu.id"
        class="menu"
        :class="{ 'menu--open': openId === menu.id }"
        :data-menu="menu.id"
      >
        <button
          type="button"
          class="menu__trigger"
          :data-menu-trigger="menu.id"
          :aria-expanded="openId === menu.id"
          aria-haspopup="menu"
          @click="toggleMenu(menu.id)"
          @mouseenter="hoverMenu(menu.id)"
          @keydown.down.prevent="openWithKeyboard(menu.id)"
        >
          {{ menu.label }}
        </button>

        <div v-if="openId === menu.id" class="menu__dropdown" role="menu">
          <template v-for="(entry, index) in menu.items" :key="index">
            <div v-if="!isItem(entry)" class="menu__separator" role="separator" />
            <button
              v-else
              type="button"
              class="menu__item"
              role="menuitem"
              :disabled="entry.disabled"
              @click="runItem(entry)"
            >
              <span class="menu__check" aria-hidden="true">{{ entry.checked ? '✓' : '' }}</span>
              <span class="menu__label">{{ entry.label }}</span>
            </button>
          </template>
        </div>
      </div>
    </nav>

    <div class="menubar__spacer" data-tauri-drag-region />

    <div class="menubar__actions">
      <select
        class="menubar__theme"
        :value="props.theme"
        aria-label="Tema de la interfaz"
        @change="onThemeChange"
      >
        <option value="system">Tema: Sistema</option>
        <option value="dark">Tema: Oscuro</option>
        <option value="light">Tema: Claro</option>
      </select>

      <div class="window-controls" aria-label="Controles de ventana">
        <button
          type="button"
          class="window-control"
          aria-label="Minimizar ventana"
          title="Minimizar"
          @click="minimize"
        >
          <svg viewBox="0 0 10 10" aria-hidden="true">
            <path d="M2 5h6" />
          </svg>
        </button>
        <button
          type="button"
          class="window-control"
          aria-label="Maximizar o restaurar ventana"
          title="Maximizar o restaurar"
          @click="toggleMaximize"
        >
          <svg viewBox="0 0 10 10" aria-hidden="true">
            <rect x="2.5" y="2.5" width="5" height="5" />
          </svg>
        </button>
        <button
          type="button"
          class="window-control window-control--close"
          aria-label="Cerrar ventana"
          title="Cerrar"
          @click="close"
        >
          <svg viewBox="0 0 10 10" aria-hidden="true">
            <path d="M2 2l6 6M8 2L2 8" />
          </svg>
        </button>
      </div>
    </div>
  </header>
</template>

<style scoped>
.menubar {
  display: flex;
  align-items: stretch;
  flex: none;
  height: var(--kor-titlebar-height);
  background-color: var(--kor-surface);
  border-bottom: 1px solid var(--kor-surface-3);
  user-select: none;
  -webkit-app-region: drag;
}

.menubar__brand {
  display: flex;
  align-items: center;
  flex: none;
  padding: 0 var(--kor-space-4) 0 var(--kor-space-5);
}

.menubar__logo {
  color: var(--kor-text-strong);
  font-size: var(--kor-text-lg);
  font-weight: var(--kor-weight-bold);
  letter-spacing: 0.12em;
  line-height: 1;
}

.menubar__menus {
  display: flex;
  align-items: stretch;
  flex: none;
  -webkit-app-region: no-drag;
}

.menu {
  position: relative;
  display: flex;
  align-items: center;
}

.menu__trigger {
  height: 100%;
  padding: 0 var(--kor-space-3);
  color: var(--kor-text);
  background: transparent;
  border: none;
  font-size: var(--kor-text-sm);
  cursor: pointer;
  transition:
    background-color var(--kor-dur-fast) var(--kor-ease-out),
    color var(--kor-dur-fast) var(--kor-ease-out);
}

.menu__trigger:hover,
.menu--open .menu__trigger {
  color: var(--kor-text-strong);
  background-color: var(--kor-surface-3);
}

.menu__dropdown {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  z-index: 60;
  min-width: 224px;
  padding: var(--kor-space-1);
  background-color: var(--kor-surface-2);
  border: 1px solid var(--kor-border);
  border-radius: var(--kor-radius-md);
  box-shadow: var(--kor-shadow-md);
}

.menu__item {
  display: grid;
  grid-template-columns: 16px 1fr;
  align-items: center;
  gap: var(--kor-space-2);
  width: 100%;
  min-height: 30px;
  padding: 0 var(--kor-space-2);
  color: var(--kor-text);
  background: transparent;
  border: none;
  border-radius: var(--kor-radius-sm);
  font-size: var(--kor-text-sm);
  text-align: left;
  cursor: pointer;
}

.menu__item:hover:not(:disabled),
.menu__item:focus-visible {
  color: var(--kor-text-strong);
  background-color: var(--kor-surface-3);
  outline: none;
}

.menu__item:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.menu__check {
  color: var(--kor-accent);
  font-size: var(--kor-text-sm);
  line-height: 1;
}

.menu__label {
  min-width: 0;
  white-space: nowrap;
}

.menu__separator {
  height: 1px;
  margin: var(--kor-space-1) var(--kor-space-2);
  background-color: var(--kor-border-subtle);
}

.menubar__spacer {
  flex: 1;
  min-width: var(--kor-space-4);
}

.menubar__actions {
  display: flex;
  align-items: stretch;
  flex: none;
  -webkit-app-region: no-drag;
}

.menubar__theme {
  align-self: center;
  height: 26px;
  margin-right: var(--kor-space-2);
  padding: 0 var(--kor-space-2);
  color: var(--kor-text-muted);
  background-color: var(--kor-surface-2);
  border: 1px solid var(--kor-border-subtle);
  border-radius: var(--kor-radius-sm);
  font-size: var(--kor-text-xs);
  cursor: pointer;
}

.menubar__theme:hover {
  color: var(--kor-text);
  border-color: var(--kor-border);
}

.window-controls {
  display: flex;
  align-items: stretch;
}

.window-control {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 100%;
  padding: 0;
  color: var(--kor-text-muted);
  background: transparent;
  border: none;
  cursor: pointer;
  transition:
    background-color var(--kor-dur-fast) var(--kor-ease-out),
    color var(--kor-dur-fast) var(--kor-ease-out);
}

.window-control svg {
  width: 10px;
  height: 10px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.2;
  stroke-linecap: square;
}

.window-control:hover {
  color: var(--kor-text-strong);
  background-color: var(--kor-surface-3);
}

.window-control--close:hover {
  color: var(--kor-text-strong);
  background-color: var(--kor-danger);
}

@media (max-width: 900px) {
  .menubar__theme {
    display: none;
  }
}
</style>
