<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { useWindowControls } from '@/composables/useWindowControls'
import { useAppInfo } from '@/composables/useAppInfo'
import { useAppBoot } from '@/composables/useAppBoot'
import { useToast } from '@/composables/useToast'

defineProps<{
  title?: string
}>()

const { app } = useAppInfo()
const { minimize, toggleMaximize, close } = useWindowControls()
const { runDiagnostico } = useAppBoot()
const { show: showToast } = useToast()

const menuOpen = ref(false)
const menuRef = ref<HTMLElement | null>(null)

function toggleMenu() {
  menuOpen.value = !menuOpen.value
}

async function onDiagnostico() {
  menuOpen.value = false
  showToast('Iniciando diagnóstico...', 'info')
  await runDiagnostico()
  showToast('Diagnóstico completado.', 'success')
}

function handleClickOutside(event: MouseEvent) {
  if (menuRef.value && !menuRef.value.contains(event.target as Node)) {
    menuOpen.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <header class="hub-header" data-tauri-drag-region>
    <div class="hub-header__brand" data-tauri-drag-region>
      <span class="hub-header__title">{{ app.name }}</span>
      
      <nav class="hub-nav">
        <div class="hub-menu" ref="menuRef">
          <button class="hub-menu__trigger" @click="toggleMenu">Ejecutar</button>
          <Transition name="fade">
            <div v-if="menuOpen" class="hub-menu__dropdown">
              <button class="hub-menu__item" @click="onDiagnostico">
                <span class="hub-menu__item-label">Diagnóstico completo</span>
                <span class="hub-menu__item-shortcut">Ctrl+Shift+I</span>
              </button>
            </div>
          </Transition>
        </div>
      </nav>
    </div>

    <div class="hub-header__actions">
      <button
        type="button"
        class="window-control"
        aria-label="Minimizar"
        @click="minimize"
      >
        <svg width="10" height="10" viewBox="0 0 10 10">
          <path d="M0 5h10v1H0z" fill="currentColor" />
        </svg>
      </button>
      <button
        type="button"
        class="window-control"
        aria-label="Maximizar"
        @click="toggleMaximize"
      >
        <svg width="10" height="10" viewBox="0 0 10 10">
          <path d="M0 0h10v10H0V0zm1 1v8h8V1H1z" fill="currentColor" />
        </svg>
      </button>
      <button
        type="button"
        class="window-control window-control--close"
        aria-label="Cerrar"
        @click="close"
      >
        <svg width="10" height="10" viewBox="0 0 10 10">
          <path d="M0 0l10 10M0 10L10 0" stroke="currentColor" fill="none" />
        </svg>
      </button>
    </div>
  </header>
</template>

<style scoped>
.hub-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--kor-titlebar-height);
  padding: 0 var(--kor-space-1) 0 var(--kor-space-4);
  background-color: var(--kor-surface);
  border-bottom: 1px solid var(--kor-surface-3);
  -webkit-user-select: none;
  user-select: none;
  z-index: 100;
}

.hub-header__brand {
  display: flex;
  align-items: center;
  gap: var(--kor-space-4);
  flex: 1;
  height: 100%;
}

.hub-header__logo {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  color: var(--kor-bg);
  background-color: var(--kor-accent);
  border-radius: var(--kor-radius-sm);
  font-size: 14px;
  font-weight: var(--kor-weight-bold);
  pointer-events: none;
}

.hub-header__title {
  color: var(--kor-text-strong);
  font-size: var(--kor-text-sm);
  font-weight: var(--kor-weight-bold);
  letter-spacing: 0.05em;
  pointer-events: none;
}

.hub-nav {
  display: flex;
  align-items: center;
  height: 100%;
  margin-left: var(--kor-space-2);
}

.hub-menu {
  position: relative;
  height: 100%;
}

.hub-menu__trigger {
  height: 100%;
  padding: 0 var(--kor-space-3);
  color: var(--kor-text);
  background: transparent;
  border: none;
  font-size: var(--kor-text-xs);
  cursor: pointer;
}

.hub-menu__trigger:hover {
  color: var(--kor-text-strong);
  background-color: var(--kor-surface-3);
}

.hub-menu__dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  min-width: 240px;
  padding: var(--kor-space-1) 0;
  background-color: var(--kor-surface-2);
  border: 1px solid var(--kor-border);
  border-radius: 0 0 var(--kor-radius-md) var(--kor-radius-md);
  box-shadow: var(--kor-shadow-md);
}

.hub-menu__item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: var(--kor-space-2) var(--kor-space-4);
  color: var(--kor-text);
  background: transparent;
  border: none;
  font-size: var(--kor-text-xs);
  text-align: left;
  cursor: pointer;
}

.hub-menu__item:hover {
  color: var(--kor-text-strong);
  background-color: var(--kor-accent);
}

.hub-menu__item-shortcut {
  font-size: 10px;
  opacity: 0.6;
}

.hub-header__actions {
  display: flex;
  align-items: stretch;
  height: 100%;
}

.window-control {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 100%;
  color: var(--kor-text-muted);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: background-color var(--kor-dur-fast) var(--kor-ease-out);
}

.window-control:hover {
  color: var(--kor-text-strong);
  background-color: var(--kor-surface-3);
}

.window-control--close:hover {
  background-color: var(--kor-danger);
}

.window-control svg {
  flex: none;
}

.fade-enter-active, .fade-leave-active {
  transition: opacity 0.15s ease;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}
</style>