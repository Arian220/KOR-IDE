<script setup lang="ts">
import { computed } from 'vue'
import AppDialog from '@/components/base/AppDialog.vue'
import { useAppInfo } from '@/composables/useAppInfo'
import logoUrl from '@/assets/kor-logo.png'

defineProps<{ open: boolean }>()

const emit = defineEmits<{ close: [] }>()

const { app, os } = useAppInfo()

const releaseDateLabel = computed(() => {
  const date = new Date(`${app.releaseDate}T00:00:00Z`)
  if (Number.isNaN(date.getTime())) return app.releaseDate
  return new Intl.DateTimeFormat('es-ES', {
    dateStyle: 'long',
    timeZone: 'UTC',
  }).format(date)
})

const osLabel = computed(() => {
  const parts = [os.value.platform, os.value.version].filter(Boolean)
  return parts.length > 0 ? parts.join(' ') : 'No disponible'
})
</script>

<template>
  <AppDialog :open="open" title="Información de KOR" @close="emit('close')">
    <div class="about">
      <header class="about__identity">
        <img class="about__logo" :src="logoUrl" alt="Logotipo de KOR" width="72" height="72" />
        <div>
          <h3 class="about__name">{{ app.name }}</h3>
          <p class="about__description">{{ app.description }}</p>
          <p class="about__version">Versión {{ app.version }}</p>
        </div>
      </header>

      <dl class="about__details">
        <div class="about__row">
          <dt>Versión</dt>
          <dd>{{ app.version }}</dd>
        </div>
        <div class="about__row">
          <dt>Fecha de publicación</dt>
          <dd>{{ releaseDateLabel }}</dd>
        </div>
        <div class="about__row">
          <dt>Licencia</dt>
          <dd>{{ app.license }}</dd>
        </div>
        <div class="about__row">
          <dt>Stack</dt>
          <dd>
            <ul class="about__stack">
              <li v-for="item in app.stack" :key="item">{{ item }}</li>
            </ul>
          </dd>
        </div>
        <div class="about__row">
          <dt>Sistema operativo</dt>
          <dd>{{ osLabel }}</dd>
        </div>
        <div class="about__row">
          <dt>Arquitectura</dt>
          <dd>{{ os.architecture || 'No disponible' }}</dd>
        </div>
        <div class="about__row">
          <dt>Idioma</dt>
          <dd>{{ os.locale || 'No disponible' }}</dd>
        </div>
        <div class="about__row">
          <dt>Resolución</dt>
          <dd>{{ os.screen || 'No disponible' }}</dd>
        </div>
      </dl>
    </div>

    <template #footer>
      <button type="button" class="about__button" @click="emit('close')">Cerrar</button>
    </template>
  </AppDialog>
</template>

<style scoped>
.about {
  display: flex;
  flex-direction: column;
  gap: var(--kor-space-4);
}

.about__identity {
  display: flex;
  align-items: center;
  gap: var(--kor-space-4);
  padding-bottom: var(--kor-space-4);
  border-bottom: 1px solid var(--kor-border-subtle);
}

.about__logo {
  flex: none;
  border-radius: var(--kor-radius-lg);
  box-shadow: var(--kor-shadow-sm);
}

.about__name {
  margin: 0;
  color: var(--kor-text-strong);
  font-size: var(--kor-text-xl);
  line-height: 1.1;
}

.about__description {
  margin: var(--kor-space-1) 0 0;
  color: var(--kor-text);
  font-size: var(--kor-text-sm);
}

.about__version {
  margin: var(--kor-space-1) 0 0;
  color: var(--kor-text-muted);
  font-size: var(--kor-text-xs);
  font-variant-numeric: tabular-nums;
}

.about__details {
  display: grid;
  gap: var(--kor-space-2);
  margin: 0;
}

.about__row {
  display: grid;
  grid-template-columns: minmax(130px, 0.8fr) 1.2fr;
  gap: var(--kor-space-3);
  align-items: baseline;
}

.about__row dt {
  color: var(--kor-text-muted);
  font-size: var(--kor-text-xs);
}

.about__row dd {
  margin: 0;
  color: var(--kor-text-strong);
  font-size: var(--kor-text-sm);
  overflow-wrap: anywhere;
}

.about__stack {
  display: flex;
  flex-wrap: wrap;
  gap: var(--kor-space-1) var(--kor-space-2);
  margin: 0;
  padding: 0;
  list-style: none;
}

.about__stack li {
  padding: 1px var(--kor-space-2);
  color: var(--kor-text);
  background-color: var(--kor-surface);
  border: 1px solid var(--kor-border-subtle);
  border-radius: var(--kor-radius-pill);
  font-size: var(--kor-text-xs);
}

.about__button {
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

.about__button:hover {
  color: var(--kor-text-strong);
  background-color: var(--kor-surface-3);
}
</style>
