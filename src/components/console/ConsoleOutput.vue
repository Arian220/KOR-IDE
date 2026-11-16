<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'

const props = withDefaults(
  defineProps<{
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

const empty = computed(() => props.text.length === 0)

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
  () => props.text,
  () => {
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
  <section class="terminal" data-testid="console">
    <header class="terminal__bar">
      <span class="terminal__title">Terminal</span>
      <div class="terminal__actions">
        <button
          type="button"
          class="terminal__action"
          :disabled="empty"
          @click="copy"
        >
          {{ copied ? 'Copiado' : 'Copiar' }}
        </button>
        <button
          type="button"
          class="terminal__action"
          :disabled="empty"
          @click="emit('clear')"
        >
          Limpiar terminal
        </button>
      </div>
    </header>

    <pre
      ref="scroller"
      class="terminal__pre"
      :class="[`terminal__pre--${variant}`, { 'terminal__pre--empty': empty }]"
      @scroll="onScroll"
    >{{ empty ? 'La salida de compilación y carga aparecerá aquí.' : text }}</pre>

    <button
      v-if="!autoScroll && !empty"
      type="button"
      class="terminal__resume"
      @click="scrollToBottom"
    >
      Ir al final
    </button>
  </section>
</template>

<style scoped>
.terminal {
  position: relative;
  display: flex;
  flex-direction: column;
  flex: none;
  height: var(--kor-terminal-height);
  min-height: 120px;
  background-color: var(--kor-console-bg);
  border-top: 1px solid var(--kor-border-subtle);
}

.terminal__bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex: none;
  gap: var(--kor-space-2);
  min-height: 34px;
  padding: 0 var(--kor-space-3);
  background-color: var(--kor-surface-2);
  border-bottom: 1px solid var(--kor-border-subtle);
}

.terminal__title {
  color: var(--kor-text);
  font-size: var(--kor-text-xs);
  font-weight: var(--kor-weight-medium);
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.terminal__actions {
  display: flex;
  gap: var(--kor-space-1);
}

.terminal__action {
  min-height: 26px;
  padding: 2px var(--kor-space-2);
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

.terminal__action:hover:not(:disabled) {
  color: var(--kor-text-strong);
  background-color: var(--kor-surface-3);
}

.terminal__action:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.terminal__pre {
  flex: 1;
  min-height: 0;
  margin: 0;
  padding: var(--kor-space-2) var(--kor-space-3);
  overflow-y: auto;
  color: var(--kor-text);
  font-family: var(--kor-font-mono);
  font-size: var(--kor-text-xs);
  line-height: var(--kor-leading);
  white-space: pre-wrap;
  word-break: break-word;
}

.terminal__pre--empty {
  color: var(--kor-text-muted);
}

.terminal__pre--error {
  color: var(--kor-danger-text);
}

.terminal__pre--success {
  color: var(--kor-success);
}

.terminal__resume {
  position: absolute;
  right: var(--kor-space-3);
  bottom: var(--kor-space-2);
  padding: 2px 8px;
  color: var(--kor-text-strong);
  background-color: var(--kor-surface-3);
  border: 1px solid var(--kor-border);
  border-radius: var(--kor-radius-pill);
  font-size: var(--kor-text-xs);
  cursor: pointer;
}

.terminal__resume:active {
  transform: scale(0.96);
}

@media (prefers-reduced-motion: reduce) {
  .terminal__resume:active {
    transform: none;
  }
}
</style>
