<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import AppDialog from '@/components/base/AppDialog.vue'

const props = withDefaults(
  defineProps<{
    open: boolean
    title: string
    label?: string
    initial?: string
    placeholder?: string
    confirmLabel?: string
    cancelLabel?: string
  }>(),
  {
    label: '',
    initial: '',
    placeholder: '',
    confirmLabel: 'Guardar',
    cancelLabel: 'Cancelar',
  },
)

const emit = defineEmits<{
  confirm: [value: string]
  cancel: []
}>()

const value = ref('')
const input = ref<HTMLInputElement | null>(null)

watch(
  () => props.open,
  async (open) => {
    if (!open) return
    value.value = props.initial
    await nextTick()
    input.value?.focus()
    input.value?.select()
  },
  { immediate: true },
)

function confirm() {
  const name = value.value.trim()
  if (!name) return
  emit('confirm', name)
}
</script>

<template>
  <AppDialog :open="open" :title="title" @close="emit('cancel')">
    <label class="prompt">
      <span v-if="label" class="prompt__label">{{ label }}</span>
      <input
        ref="input"
        v-model="value"
        class="prompt__input"
        data-autofocus
        type="text"
        :placeholder="placeholder"
        @keydown.enter.prevent="confirm"
      />
    </label>

    <template #footer>
      <button type="button" class="prompt__button" @click="emit('cancel')">
        {{ cancelLabel }}
      </button>
      <button
        type="button"
        class="prompt__button prompt__button--primary"
        :disabled="!value.trim()"
        @click="confirm"
      >
        {{ confirmLabel }}
      </button>
    </template>
  </AppDialog>
</template>

<style scoped>
.prompt {
  display: flex;
  flex-direction: column;
  gap: var(--kor-space-2);
}

.prompt__label {
  color: var(--kor-text-muted);
  font-size: var(--kor-text-sm);
}

.prompt__input {
  width: 100%;
  height: 34px;
  padding: 0 var(--kor-space-3);
  color: var(--kor-text-strong);
  background-color: var(--kor-surface);
  border: 1px solid var(--kor-border);
  border-radius: var(--kor-radius-md);
  font-size: var(--kor-text-sm);
  outline: none;
}

.prompt__input:focus {
  border-color: var(--kor-accent);
  box-shadow: var(--kor-shadow-focus);
}

.prompt__button {
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

.prompt__button:hover:not(:disabled) {
  color: var(--kor-text-strong);
  background-color: var(--kor-surface-3);
}

.prompt__button--primary {
  color: var(--kor-text-strong);
  background-color: var(--kor-accent);
  border-color: var(--kor-accent);
}

.prompt__button--primary:hover:not(:disabled) {
  background-color: var(--kor-accent-hover);
}

.prompt__button:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}
</style>
