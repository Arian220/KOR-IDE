<script setup lang="ts">
import { computed } from 'vue'
import type { MemoryUsage } from '@/types/contract'

const props = withDefaults(
  defineProps<{
    visible: boolean
    flash: MemoryUsage | null
    sram: MemoryUsage | null
    error?: boolean
  }>(),
  { error: false },
)

function label(usage: MemoryUsage | null) {
  if (props.error) return 'Error'
  if (!usage) return '0%'
  return `${usage.percent}% (${usage.bytes} b)`
}

function fillStyle(usage: MemoryUsage | null, normalColor: string) {
  if (props.error || !usage) {
    return { width: '0%', backgroundColor: normalColor }
  }
  return {
    width: `${usage.percent}%`,
    backgroundColor: usage.percent > 80 ? 'var(--kor-danger)' : normalColor,
  }
}

const flashText = computed(() => label(props.flash))
const sramText = computed(() => label(props.sram))
const flashStyle = computed(() => fillStyle(props.flash, 'var(--kor-success)'))
const sramStyle = computed(() => fillStyle(props.sram, 'var(--kor-accent)'))
</script>

<template>
  <div v-show="visible" class="memory-panel">
    <div class="memory-row">
      <span class="memory-row__label">Flash (Programa):</span>
      <div
        class="progress-bar"
        role="progressbar"
        aria-label="Uso de Flash"
        :aria-valuenow="flash?.percent ?? 0"
        aria-valuemin="0"
        aria-valuemax="100"
      >
        <div class="progress-fill" :style="flashStyle" />
      </div>
      <span class="memory-row__value">{{ flashText }}</span>
    </div>
    <div class="memory-row">
      <span class="memory-row__label">SRAM (Variables):</span>
      <div
        class="progress-bar"
        role="progressbar"
        aria-label="Uso de SRAM"
        :aria-valuenow="sram?.percent ?? 0"
        aria-valuemin="0"
        aria-valuemax="100"
      >
        <div class="progress-fill" :style="sramStyle" />
      </div>
      <span class="memory-row__value">{{ sramText }}</span>
    </div>
    <slot />
  </div>
</template>

<style scoped>
.memory-panel {
  padding: var(--kor-space-3) var(--kor-space-4) var(--kor-space-4);
  background-color: var(--kor-surface-2);
  border-bottom: 1px solid var(--kor-surface-3);
}

.memory-row {
  display: flex;
  align-items: center;
  gap: var(--kor-space-2);
  margin-bottom: var(--kor-space-2);
  font-size: var(--kor-text-sm);
}

.memory-row:last-of-type {
  margin-bottom: 0;
}

.memory-row__label {
  width: 120px;
  flex: none;
  color: var(--kor-text-muted);
}

.memory-row__value {
  width: 90px;
  flex: none;
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.progress-bar {
  flex: 1;
  height: 10px;
  background-color: var(--kor-bg);
  border-radius: var(--kor-radius-pill);
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  width: 0%;
  background-color: var(--kor-success);
  border-radius: inherit;
  transition: width var(--kor-dur-slow) var(--kor-ease-out);
}

@media (prefers-reduced-motion: reduce) {
  .progress-fill {
    transition: none;
  }
}
</style>
