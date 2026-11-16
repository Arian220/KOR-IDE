<script setup lang="ts">
import type { StatusKind } from '@/composables/useArduino'

withDefaults(
  defineProps<{
    label: string
    kind?: StatusKind
    detail?: string
  }>(),
  { kind: 'idle', detail: '' },
)
</script>

<template>
  <footer class="status" :class="`status--${kind}`" role="status" aria-live="polite">
    <span class="status__dot" aria-hidden="true" />
    <span class="status__label">{{ label }}</span>
    <span v-if="detail" class="status__detail">{{ detail }}</span>
  </footer>
</template>

<style scoped>
.status {
  display: flex;
  align-items: center;
  gap: var(--kor-space-2);
  min-height: 30px;
  padding: 0 var(--kor-space-3);
  background-color: var(--kor-surface-2);
  border-top: 1px solid var(--kor-surface-3);
  color: var(--kor-text-muted);
  font-size: var(--kor-text-xs);
}

.status__dot {
  width: 8px;
  height: 8px;
  flex: none;
  border-radius: 50%;
  background-color: var(--kor-text-muted);
}

.status--busy .status__dot {
  background-color: var(--kor-accent);
}

.status--success .status__dot {
  background-color: var(--kor-success);
}

.status--error .status__dot {
  background-color: var(--kor-danger);
}

.status--warning .status__dot {
  background-color: var(--kor-warning);
}

.status__label {
  color: var(--kor-text);
}

.status__detail {
  margin-left: auto;
  color: var(--kor-text-muted);
  font-variant-numeric: tabular-nums;
}
</style>
