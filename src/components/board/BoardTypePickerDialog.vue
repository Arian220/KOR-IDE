<script setup lang="ts">
import AppDialog from '@/components/base/AppDialog.vue'
import { BOARDS_PREDEFINIDOS } from '@/composables/useArduino'

defineProps<{ open: boolean }>()
const emit = defineEmits<{
  close: []
  select: [fqbn: string]
}>()
</script>

<template>
  <AppDialog :open="open" title="Seleccionar placa" @close="emit('close')">
    <div class="board-list">
      <button
        v-for="board in BOARDS_PREDEFINIDOS"
        :key="board.fqbn"
        class="board-item"
        @click="emit('select', board.fqbn)"
      >
        <div class="board-name">{{ board.nombre }}</div>
        <div class="board-fqbn">{{ board.fqbn }}</div>
      </button>
    </div>
  </AppDialog>
</template>

<style scoped>
.board-list {
  display: flex;
  flex-direction: column;
  gap: var(--kor-space-2);
}

.board-item {
  display: flex;
  flex-direction: column;
  padding: var(--kor-space-3);
  text-align: left;
  background-color: var(--kor-surface-3);
  border: 1px solid var(--kor-border);
  border-radius: var(--kor-radius-md);
  cursor: pointer;
  transition: background-color 0.2s;
}

.board-item:hover {
  background-color: var(--kor-surface-4);
}

.board-name {
  font-weight: var(--kor-weight-bold);
  color: var(--kor-text-strong);
}

.board-fqbn {
  font-family: var(--kor-font-mono);
  font-size: var(--kor-text-xs);
  color: var(--kor-text-muted);
}
</style>