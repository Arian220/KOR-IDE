<script setup lang="ts">
import { onMounted } from 'vue'
import { useAppBoot } from '@/composables/useAppBoot'
import { useMainWindow } from '@/composables/useMainWindow'

const { ensureBoot } = useAppBoot()
const { openMainWindow } = useMainWindow()

onMounted(async () => {
  await ensureBoot()
  await openMainWindow()
})
</script>

<template>
  <div class="splash">
    <div class="splash__content">
      <h1 class="splash__title">KOR</h1>
      <p class="splash__subtitle">by Loop-i</p>
      <p class="splash__status">Cargando proyecto…</p>
    </div>
  </div>
</template>

<style scoped>
.splash {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, var(--kor-bg) 0%, var(--kor-surface) 50%, var(--kor-accent) 150%);
  background-size: 200% 200%;
  animation: splash-shift 18s ease infinite;
  color: var(--kor-text-strong);
  font-family: var(--kor-font-ui);
}

@keyframes splash-shift {
  0% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
  100% { background-position: 0% 50%; }
}

.splash__content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--kor-space-2);
}

.splash__title {
  margin: 0;
  font-size: clamp(48px, 12vw, 96px);
  font-weight: var(--kor-weight-bold);
  letter-spacing: -0.02em;
  text-wrap: balance;
  background: linear-gradient(120deg, var(--kor-accent), var(--kor-run), var(--kor-success));
  -webkit-background-clip: text;
  background-clip: text;
  -webkit-text-fill-color: transparent;
}

.splash__subtitle {
  margin: 0;
  color: var(--kor-text-muted);
  font-size: var(--kor-text-lg);
  font-weight: var(--kor-weight-medium);
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.splash__status {
  margin: var(--kor-space-4) 0 0;
  color: var(--kor-text);
  font-size: var(--kor-text-sm);
  font-variant-numeric: tabular-nums;
}

@media (prefers-reduced-motion: reduce) {
  .splash {
    animation: none;
  }
}
</style>