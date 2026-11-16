<script setup lang="ts">
import { computed } from 'vue'
import type { BoardInfo } from '@/types/app'

const props = withDefaults(
  defineProps<{
    boards: BoardInfo[]
    selectedPort: string | null
    hasDetected: boolean
    detecting: boolean
    error?: string | null
  }>(),
  { error: null },
)

const emit = defineEmits<{
  'update:selectedPort': [port: string | null]
}>()

const selectedBoard = computed(() => {
  return props.boards.find((board) => board.port === props.selectedPort) ?? props.boards[0] ?? null
})

const hasMultiple = computed(() => props.boards.length > 1)

function onChange(event: Event) {
  const port = (event.target as HTMLSelectElement).value
  emit('update:selectedPort', port || null)
}
</script>

<template>
  <div v-if="hasDetected || detecting" class="board" data-testid="board-selector">
    <span class="board__status" aria-live="polite">
      <template v-if="detecting">Buscando placas...</template>
      <template v-else-if="error">Error de detección</template>
      <template v-else-if="boards.length === 0">Sin placas detectadas</template>
      <template v-else>{{ boards.length }} placa{{ boards.length === 1 ? '' : 's' }}</template>
    </span>

    <template v-if="!detecting && boards.length > 0">
      <label v-if="hasMultiple" class="board__field">
        <span class="board__caption">Puerto</span>
        <select
          class="board__select"
          :value="selectedBoard?.port ?? ''"
          aria-label="Puerto de la placa"
          @change="onChange"
        >
          <option v-for="board in boards" :key="board.port" :value="board.port">
            {{ board.port }} — {{ board.label }}
          </option>
        </select>
      </label>

      <div v-else-if="selectedBoard" class="board__single">
        <span class="board__port">{{ selectedBoard.port }}</span>
        <span class="board__label">{{ selectedBoard.label }}</span>
      </div>

      <span v-if="selectedBoard?.fqbn" class="board__fqbn">{{ selectedBoard.fqbn }}</span>
    </template>

    <span v-else-if="!detecting && error" class="board__guide board__guide--error">
      {{ error }}
    </span>
    <span v-else-if="!detecting" class="board__guide">
      Revisa el cable USB, los drivers y los permisos del puerto.
    </span>
  </div>
</template>

<style scoped>
.board {
  display: flex;
  align-items: center;
  gap: var(--kor-space-3);
  flex-wrap: wrap;
  padding: var(--kor-space-2) var(--kor-space-4);
  background-color: var(--kor-surface-2);
  border-bottom: 1px solid var(--kor-surface-3);
  font-size: var(--kor-text-sm);
}

.board__status {
  color: var(--kor-text-muted);
  white-space: nowrap;
}

.board__field {
  display: flex;
  align-items: center;
  gap: var(--kor-space-2);
}

.board__caption {
  color: var(--kor-text-muted);
}

.board__select {
  height: 32px;
  min-width: 220px;
  padding: 0 var(--kor-space-2);
  color: var(--kor-text);
  background-color: var(--kor-surface-3);
  border: 1px solid var(--kor-border);
  border-radius: var(--kor-radius-md);
}

.board__single {
  display: flex;
  align-items: center;
  gap: var(--kor-space-2);
}

.board__port {
  color: var(--kor-text-strong);
  font-weight: var(--kor-weight-medium);
}

.board__label {
  color: var(--kor-text);
}

.board__fqbn {
  color: var(--kor-text-muted);
  font-family: var(--kor-font-mono);
  font-size: var(--kor-text-xs);
}

.board__guide {
  color: var(--kor-warning);
}

.board__guide--error {
  color: var(--kor-danger-text);
}
</style>
