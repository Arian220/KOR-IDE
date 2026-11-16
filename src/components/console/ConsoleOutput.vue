<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watch } from 'vue'

const props = withDefaults(
  defineProps<{
    visible: boolean
    text: string
    variant?: 'info' | 'error' | 'success'
  }>(),
  { variant: 'info' },
)

const emit = defineEmits<{ clear: [] }>()

const scroller = ref<HTMLElement | null>(null)
const autoScroll = ref(true)
const copied = ref(false)

let copiedTimer: ReturnType<typeof setTimeout> | null = null

function scrollToBottom() {
  const element = scroller.value
  if (!element) return
  element.scrollTop = element.scrollHeight
  autoScroll.value = true
}

function onScroll() {
  const element = scroller.value
  if (!element) return
  const distance = element.scrollHeight - element.scrollTop - element.clientHeight
  autoScroll.value = distance < 24
}

async function copy() {
  if (!props.text) return
  try {
    await navigator.clipboard.writeText(props.text)
  } catch {
    const element = scroller.value
    if (!element) return
    const selection = window.getSelection()
    const range = document.createRange()
    range.selectNodeContents(element)
    selection?.removeAllRanges()
    selection?.addRange(range)
    document.execCommand('copy')
    selection?.removeAllRanges()
  }
  copied.value = true
  if (copiedTimer) clearTimeout(copiedTimer)
  copiedTimer = setTimeout(() => {
    copied.value = false
  }, 1500)
}

watch(
  () => [props.text, props.visible] as const,
  () => {
    if (!props.visible) return
    void nextTick(() => {
      if (autoScroll.value) scrollToBottom()
    })
  },
  { immediate: true },
)

onBeforeUnmount(() => {
  if (copiedTimer) clearTimeout(copiedTimer)
})
</script>

<template>
  <section v-show="visible" class="console" data-testid="console">
    <header class="console__bar">
      <span class="console__title">Salida</span>
      <div class="console__actions">
        <button type="button" class="console__action" :disabled="!text" @click="copy">
          {{ copied ? 'Copiado' : 'Copiar' }}
        </button>
        <button type="button" class="console__action" :disabled="!text" @click="emit('clear')">
          Limpiar
        </button>
      </div>
    </header>

    <pre
      ref="scroller"
      class="console__pre"
      :class="`console__pre--${variant}`"
      @scroll="onScroll"
    >{{ text }}</pre>

    <button
      v-if="!autoScroll"
      type="button"
      class="console__resume"
      @click="scrollToBottom"
    >
      Ir al final
    </button>
  </section>
</template>

<style scoped>
.console {
  position: relative;
  margin-top: var(--kor-space-2);
  background-color: var(--kor-console-bg);
  border: 1px solid var(--kor-border-subtle);
  border-radius: var(--kor-radius-md);
  overflow: hidden;
}

.console__bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--kor-space-2);
  padding: var(--kor-space-1) var(--kor-space-2);
  background-color: var(--kor-surface-2);
  border-bottom: 1px solid var(--kor-border-subtle);
}

.console__title {
  color: var(--kor-text-muted);
  font-size: var(--kor-text-xs);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.console__actions {
  display: flex;
  gap: var(--kor-space-1);
}

.console__action {
  padding: 2px 8px;
  background: none;
  border: 1px solid transparent;
  border-radius: var(--kor-radius-sm);
  color: var(--kor-text-muted);
  font-size: var(--kor-text-xs);
  cursor: pointer;
  transition:
    background-color var(--kor-dur-fast) var(--kor-ease-out),
    color var(--kor-dur-fast) var(--kor-ease-out);
}

.console__action:hover:not(:disabled) {
  color: var(--kor-text-strong);
  background-color: var(--kor-surface-3);
}

.console__action:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.console__pre {
  margin: 0;
  padding: var(--kor-space-2) var(--kor-space-3);
  max-height: 140px;
  overflow-y: auto;
  color: var(--kor-text);
  font-family: var(--kor-font-mono);
  font-size: var(--kor-text-xs);
  line-height: var(--kor-leading);
  white-space: pre-wrap;
  word-break: break-word;
}

.console__pre--error {
  color: var(--kor-danger-text);
}

.console__pre--success {
  color: var(--kor-success);
}

.console__resume {
  position: sticky;
  right: var(--kor-space-2);
  bottom: var(--kor-space-2);
  float: right;
  margin: 0 var(--kor-space-2) var(--kor-space-2);
  padding: 2px 8px;
  color: var(--kor-text-strong);
  background-color: var(--kor-surface-3);
  border: 1px solid var(--kor-border);
  border-radius: var(--kor-radius-pill);
  font-size: var(--kor-text-xs);
  cursor: pointer;
}
</style>
