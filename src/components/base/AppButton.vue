<script setup lang="ts">
type Variant = 'compile' | 'upload' | 'ghost'

withDefaults(
  defineProps<{
    variant?: Variant
    disabled?: boolean
  }>(),
  {
    variant: 'compile',
    disabled: false,
  },
)
</script>

<template>
  <button
    type="button"
    class="app-button"
    :class="`app-button--${variant}`"
    :disabled="disabled"
  >
    <slot />
  </button>
</template>

<style scoped>
.app-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 32px;
  padding: 6px 12px;
  color: var(--kor-text-strong);
  border: none;
  border-radius: var(--kor-radius-md);
  cursor: pointer;
  font-weight: var(--kor-weight-bold);
  white-space: nowrap;
  transition:
    background-color var(--kor-dur-fast) var(--kor-ease-out),
    transform var(--kor-dur-fast) var(--kor-ease-out),
    opacity var(--kor-dur-fast) var(--kor-ease-out);
}

.app-button:hover:not(:disabled) {
  filter: brightness(1.08);
}

.app-button:active:not(:disabled) {
  transform: scale(0.96);
}

.app-button:focus-visible {
  outline: none;
  box-shadow: var(--kor-shadow-focus);
}

.app-button:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.app-button--compile {
  background-color: var(--kor-accent);
}

.app-button--compile:hover:not(:disabled) {
  background-color: var(--kor-accent-hover);
}

.app-button--upload {
  background-color: var(--kor-run);
}

.app-button--upload:hover:not(:disabled) {
  background-color: var(--kor-run-hover);
}

.app-button--ghost {
  color: var(--kor-text);
  background-color: transparent;
  border: 1px solid var(--kor-border);
}

.app-button--ghost:hover:not(:disabled) {
  color: var(--kor-text-strong);
  background-color: var(--kor-surface-3);
}

@media (prefers-reduced-motion: reduce) {
  .app-button {
    transition: none;
  }

  .app-button:active:not(:disabled) {
    transform: none;
  }
}
</style>
