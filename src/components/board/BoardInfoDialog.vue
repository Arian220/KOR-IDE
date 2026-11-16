<script setup lang="ts">
import AppDialog from '@/components/base/AppDialog.vue'
import type { BoardInfo } from '@/types/app'

defineProps<{
  open: boolean
  board: BoardInfo | null
}>()

const emit = defineEmits<{ close: [] }>()
</script>

<template>
  <AppDialog :open="open" title="Información de la placa" @close="emit('close')">
    <div v-if="board" class="board-info">
      <dl class="board-info__details">
        <div class="board-info__row">
          <dt>Puerto</dt>
          <dd>{{ board.port }}</dd>
        </div>
        <div class="board-info__row">
          <dt>Modelo</dt>
          <dd>{{ board.label }}</dd>
        </div>
        <div class="board-info__row">
          <dt>FQBN</dt>
          <dd>{{ board.fqbn || 'No disponible' }}</dd>
        </div>
        <div class="board-info__row">
          <dt>Protocolo</dt>
          <dd>{{ board.protocol || 'No disponible' }}</dd>
        </div>
      </dl>
    </div>
    <p v-else class="board-info__empty">
      No hay una placa seleccionada. Detecta placas para consultar sus datos.
    </p>

    <template #footer>
      <button type="button" class="board-info__button" @click="emit('close')">Cerrar</button>
    </template>
  </AppDialog>
</template>

<style scoped>
.board-info__details {
  display: grid;
  gap: var(--kor-space-2);
  margin: 0;
}

.board-info__row {
  display: grid;
  grid-template-columns: minmax(110px, 0.7fr) 1.3fr;
  gap: var(--kor-space-3);
  align-items: baseline;
}

.board-info__row dt {
  color: var(--kor-text-muted);
  font-size: var(--kor-text-xs);
}

.board-info__row dd {
  margin: 0;
  color: var(--kor-text-strong);
  font-family: var(--kor-font-mono);
  font-size: var(--kor-text-sm);
  overflow-wrap: anywhere;
}

.board-info__empty {
  margin: 0;
  color: var(--kor-text-muted);
  font-size: var(--kor-text-sm);
}

.board-info__button {
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

.board-info__button:hover {
  color: var(--kor-text-strong);
  background-color: var(--kor-surface-3);
}
</style>
