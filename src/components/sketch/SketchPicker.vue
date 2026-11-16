<script setup lang="ts">
import AppDialog from '@/components/base/AppDialog.vue'
import type { SketchSummary } from '@/types/app'

defineProps<{
  open: boolean
  sketches: SketchSummary[]
  loading: boolean
  error: string | null
}>()

const emit = defineEmits<{
  select: [id: string]
  close: []
  refresh: []
}>()

function nombreArchivo(name: string): string {
  return /\.ino$/i.test(name) ? name : `${name}.ino`
}

function fechaLegible(value: string): string {
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? '' : date.toLocaleString()
}
</script>

<template>
  <AppDialog :open="open" title="Abrir sketch" @close="emit('close')">
    <p v-if="loading" class="picker__status">Cargando sketches…</p>
    <p v-else-if="error" class="picker__status picker__status--error">{{ error }}</p>
    <p v-else-if="sketches.length === 0" class="picker__status">
      Todavía no hay sketches guardados.
    </p>

    <ul v-else class="picker">
      <li v-for="sketch in sketches" :key="sketch.id">
        <button type="button" class="picker__item" @click="emit('select', sketch.id)">
          <span class="picker__name">{{ nombreArchivo(sketch.name) }}</span>
          <span class="picker__date">{{ fechaLegible(sketch.updated_at) }}</span>
        </button>
      </li>
    </ul>

    <template #footer>
      <button type="button" class="picker__button" :disabled="loading" @click="emit('refresh')">
        Actualizar
      </button>
      <button type="button" class="picker__button" @click="emit('close')">Cerrar</button>
    </template>
  </AppDialog>
</template>

<style scoped>
.picker {
  display: flex;
  flex-direction: column;
  gap: var(--kor-space-1);
  margin: 0;
  padding: 0;
  list-style: none;
}

.picker__item {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--kor-space-3);
  width: 100%;
  padding: var(--kor-space-2) var(--kor-space-3);
  color: var(--kor-text);
  background: transparent;
  border: 1px solid transparent;
  border-radius: var(--kor-radius-md);
  text-align: left;
  cursor: pointer;
}

.picker__item:hover,
.picker__item:focus-visible {
  color: var(--kor-text-strong);
  background-color: var(--kor-surface-3);
  border-color: var(--kor-border-subtle);
  outline: none;
}

.picker__name {
  font-size: var(--kor-text-sm);
  font-weight: var(--kor-weight-medium);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.picker__date {
  flex: none;
  color: var(--kor-text-muted);
  font-size: var(--kor-text-xs);
}

.picker__status {
  margin: 0;
  padding: var(--kor-space-4) 0;
  color: var(--kor-text-muted);
  font-size: var(--kor-text-sm);
  text-align: center;
}

.picker__status--error {
  color: var(--kor-danger-text);
}

.picker__button {
  min-height: 32px;
  padding: 6px 14px;
  color: var(--kor-text);
  background-color: transparent;
  border: 1px solid var(--kor-border);
  border-radius: var(--kor-radius-md);
  font-size: var(--kor-text-sm);
  font-weight: var(--kor-weight-bold);
  cursor: pointer;
}

.picker__button:hover:not(:disabled) {
  color: var(--kor-text-strong);
  background-color: var(--kor-surface-3);
}

.picker__button:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}
</style>
