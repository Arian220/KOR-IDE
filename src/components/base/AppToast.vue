<script setup lang="ts">
import { useToast } from '@/composables/useToast'

const { toasts, dismiss } = useToast()
</script>

<template>
  <TransitionGroup name="toast" tag="div" class="toasts" aria-live="polite">
    <div
      v-for="toast in toasts"
      :key="toast.id"
      class="toast"
      :class="`toast--${toast.kind}`"
      role="status"
      data-testid="toast"
    >
      <span class="toast__message">{{ toast.message }}</span>
      <button
        type="button"
        class="toast__close"
        aria-label="Cerrar aviso"
        @click="dismiss(toast.id)"
      >
        ×
      </button>
    </div>
  </TransitionGroup>
</template>

<style scoped>
.toasts {
  position: fixed;
  right: var(--kor-space-5);
  bottom: var(--kor-space-5);
  z-index: 100;
  display: flex;
  flex-direction: column;
  gap: var(--kor-space-2);
  width: min(420px, calc(100vw - 40px));
  pointer-events: none;
}

.toast {
  display: flex;
  align-items: flex-start;
  gap: var(--kor-space-3);
  padding: var(--kor-space-3) var(--kor-space-4);
  background-color: var(--kor-surface-3);
  border: 1px solid var(--kor-border);
  border-left: 4px solid var(--kor-accent);
  border-radius: var(--kor-radius-md);
  box-shadow: var(--kor-shadow-md);
  pointer-events: auto;
}

.toast--success {
  border-left-color: var(--kor-success);
}

.toast--error {
  border-left-color: var(--kor-danger);
}

.toast__message {
  flex: 1;
  color: var(--kor-text);
  font-size: var(--kor-text-sm);
  text-wrap: pretty;
}

.toast__close {
  flex: none;
  width: 24px;
  height: 24px;
  padding: 0;
  background: none;
  border: none;
  border-radius: var(--kor-radius-sm);
  color: var(--kor-text-muted);
  font-size: 16px;
  line-height: 1;
  cursor: pointer;
}

.toast__close:hover {
  color: var(--kor-text-strong);
  background-color: var(--kor-surface-2);
}

.toast-enter-active,
.toast-leave-active {
  transition:
    opacity var(--kor-dur-base) var(--kor-ease-out),
    transform var(--kor-dur-base) var(--kor-ease-out);
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateY(8px);
}

@media (prefers-reduced-motion: reduce) {
  .toast-enter-active,
  .toast-leave-active {
    transition: none;
  }
}
</style>
