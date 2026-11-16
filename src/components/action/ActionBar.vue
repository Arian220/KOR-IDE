<script setup lang="ts">
import { computed } from 'vue'
import AppButton from '@/components/base/AppButton.vue'
import type { BusyAction } from '@/composables/useArduino'

const props = defineProps<{
  busyAction: BusyAction
  uploadPort: string | null
  isBusy: boolean
  canCancel: boolean
}>()

const emit = defineEmits<{
  detect: []
  compile: []
  upload: []
  cancel: []
}>()

const compileLabel = computed(() =>
  props.busyAction === 'compile' ? 'Compilando...' : 'Compilar',
)

const uploadLabel = computed(() => {
  if (props.busyAction !== 'upload') return 'Subir a la placa'
  return props.uploadPort ? `Subiendo a ${props.uploadPort}...` : 'Buscando placa...'
})
</script>

<template>
  <div class="action-bar">
    <AppButton variant="ghost" :disabled="isBusy" @click="emit('detect')">
      Detectar
    </AppButton>
    <AppButton variant="compile" :disabled="isBusy" @click="emit('compile')">
      {{ compileLabel }}
    </AppButton>
    <AppButton variant="upload" :disabled="isBusy" @click="emit('upload')">
      {{ uploadLabel }}
    </AppButton>
    <AppButton v-if="canCancel" variant="ghost" @click="emit('cancel')">
      Cancelar
    </AppButton>
  </div>
</template>

<style scoped>
.action-bar {
  display: flex;
  align-items: center;
  gap: var(--kor-space-2);
  flex-wrap: wrap;
  justify-content: flex-end;
}
</style>
