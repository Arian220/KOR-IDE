<script setup lang="ts">
import AppDialog from '@/components/base/AppDialog.vue'
import { APP_INFO, MIT_LICENSE } from '@/config/appInfo'

defineProps<{ open: boolean }>()

const emit = defineEmits<{ close: [] }>()
</script>

<template>
  <AppDialog :open="open" title="Licencia" @close="emit('close')">
    <div class="license">
      <p class="license__summary">
        KOR se distribuye bajo la licencia
        <strong>{{ APP_INFO.license }}</strong>.
      </p>
      <pre class="license__text">{{ MIT_LICENSE }}</pre>
    </div>

    <template #footer>
      <button type="button" class="license__button" @click="emit('close')">Cerrar</button>
    </template>
  </AppDialog>
</template>

<style scoped>
.license {
  display: flex;
  flex-direction: column;
  gap: var(--kor-space-3);
}

.license__summary {
  margin: 0;
  color: var(--kor-text);
  font-size: var(--kor-text-sm);
}

.license__text {
  margin: 0;
  padding: var(--kor-space-3);
  color: var(--kor-text);
  background-color: var(--kor-surface);
  border: 1px solid var(--kor-border-subtle);
  border-radius: var(--kor-radius-md);
  font-family: var(--kor-font-mono);
  font-size: var(--kor-text-xs);
  line-height: var(--kor-leading);
  white-space: pre-wrap;
}

.license__button {
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

.license__button:hover {
  color: var(--kor-text-strong);
  background-color: var(--kor-surface-3);
}
</style>
