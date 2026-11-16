<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

const props = defineProps<{
  open: boolean
  title: string
}>()

const emit = defineEmits<{
  close: []
}>()

const panel = ref<HTMLElement | null>(null)

function onKeydown(event: KeyboardEvent) {
  if (!props.open) return
  if (event.key === 'Escape') {
    event.stopPropagation()
    emit('close')
  }
}

watch(
  () => props.open,
  async (open) => {
    if (typeof document === 'undefined') return
    document.body.style.overflow = open ? 'hidden' : ''
    if (!open) return
    await nextTick()
    panel.value?.querySelector<HTMLElement>('[data-autofocus]')?.focus()
  },
  { immediate: true },
)

onMounted(() => {
  document.addEventListener('keydown', onKeydown)
})

onBeforeUnmount(() => {
  document.removeEventListener('keydown', onKeydown)
  document.body.style.overflow = ''
})
</script>

<template>
  <Teleport to="body">
    <Transition name="dialog">
      <div v-if="open" class="dialog-overlay" @pointerdown.self="emit('close')">
        <div
          ref="panel"
          class="dialog"
          role="dialog"
          aria-modal="true"
          :aria-label="title"
        >
          <header class="dialog__header">
            <h2 class="dialog__title">{{ title }}</h2>
            <button
              type="button"
              class="dialog__close"
              aria-label="Cerrar"
              @click="emit('close')"
            >
              ×
            </button>
          </header>

          <div class="dialog__body">
            <slot />
          </div>

          <footer v-if="$slots.footer" class="dialog__footer">
            <slot name="footer" />
          </footer>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--kor-space-4);
  background-color: rgba(0, 0, 0, 0.55);
}

.dialog {
  display: flex;
  flex-direction: column;
  width: min(520px, 100%);
  max-height: min(70vh, 640px);
  background-color: var(--kor-surface-2);
  border: 1px solid var(--kor-border);
  border-radius: var(--kor-radius-lg);
  box-shadow: var(--kor-shadow-md);
  overflow: hidden;
}

.dialog__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--kor-space-3);
  padding: var(--kor-space-3) var(--kor-space-4);
  border-bottom: 1px solid var(--kor-border-subtle);
}

.dialog__title {
  margin: 0;
  color: var(--kor-text-strong);
  font-size: var(--kor-text-md);
  font-weight: var(--kor-weight-bold);
}

.dialog__close {
  flex: none;
  width: 26px;
  height: 26px;
  padding: 0;
  color: var(--kor-text-muted);
  background: transparent;
  border: none;
  border-radius: var(--kor-radius-sm);
  font-size: 18px;
  line-height: 1;
  cursor: pointer;
}

.dialog__close:hover {
  color: var(--kor-text-strong);
  background-color: var(--kor-surface-3);
}

.dialog__body {
  flex: 1;
  min-height: 0;
  padding: var(--kor-space-4);
  overflow-y: auto;
}

.dialog__footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--kor-space-2);
  padding: var(--kor-space-3) var(--kor-space-4);
  border-top: 1px solid var(--kor-border-subtle);
}

.dialog-enter-active,
.dialog-leave-active {
  transition: opacity var(--kor-dur-base) var(--kor-ease-out);
}

.dialog-enter-from,
.dialog-leave-to {
  opacity: 0;
}

@media (prefers-reduced-motion: reduce) {
  .dialog-enter-active,
  .dialog-leave-active {
    transition: none;
  }
}
</style>
