<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import AppDialog from '@/components/base/AppDialog.vue'
import AppButton from '@/components/base/AppButton.vue'
import HubMenuBar from '@/components/layout/HubMenuBar.vue'
import { useAppBoot } from '@/composables/useAppBoot'
import { useSketchFiles } from '@/composables/useSketchFiles'
import { useArduino, BOARDS_PREDEFINIDOS } from '@/composables/useArduino'
import type { SketchProject, ProjectType } from '@/types/app'

const router = useRouter()
const { result, booted } = useAppBoot()
const { abrirSketch, cargarSketches } = useSketchFiles()
const { setFqbn } = useArduino()

const showNewProjectModal = ref(false)
const newProjectForm = ref({
  name: '',
  fqbn: BOARDS_PREDEFINIDOS[0].fqbn,
  type: 'code' as ProjectType,
})

const recentProjects = ref<SketchProject[]>([])

async function refreshProjects() {
  const sketches = await cargarSketches()
  recentProjects.value = sketches.map(s => ({ ...s, type: 'code' }))
}

onMounted(async () => {
  if (result.value) {
    recentProjects.value = result.value.recentProjects.map((project) => ({ ...project }))
  } else {
    refreshProjects()
  }
})

async function openProject(id: string) {
  const ok = await abrirSketch(id)
  if (ok) {
    router.push('/ide')
  }
}

function startNewProject() {
  showNewProjectModal.value = true
}

async function finalizeNewProject() {
  if (!newProjectForm.value.name.trim()) {
    return
  }
  
  // Configurar placa
  setFqbn(newProjectForm.value.fqbn)
  
  // En una versión futura aquí se llamaría a sketchApi.create con newProjectForm.value.name
  showNewProjectModal.value = false
  router.push('/ide')
}

function formatDate(iso: string) {
  const date = new Date(iso)
  return date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}
</script>

<template>
  <div class="hub">
    <HubMenuBar />

    <main v-if="booted" class="hub__content">
      <section class="hub__panel hub__panel--left">
        <h2 class="hub__section-title">Comenzar</h2>
        
        <div class="hub__actions">
          <button class="hub__action-card" @click="startNewProject">
            <div class="hub__action-icon">＋</div>
            <div class="hub__action-info">
              <span class="hub__action-name">Nuevo proyecto</span>
              <span class="hub__action-desc">Crea un sketch desde cero</span>
            </div>
          </button>

          <button class="hub__action-card hub__action-card--ghost" @click="router.push('/ide')">
            <div class="hub__action-icon">↗</div>
            <div class="hub__action-info">
              <span class="hub__action-name">Abrir editor</span>
              <span class="hub__action-desc">Ir directo al entorno de desarrollo</span>
            </div>
          </button>
        </div>

        <div v-if="result?.updateAvailable" class="hub__update">
          <span class="hub__update-icon">🔔</span>
          <div class="hub__update-info">
            <p class="hub__update-text">Nueva versión disponible: <strong>{{ result.latestVersion }}</strong></p>
            <button class="hub__update-link">Más información</button>
          </div>
        </div>
      </section>

      <section class="hub__panel hub__panel--right">
        <h2 class="hub__section-title">Proyectos recientes</h2>
        
        <div v-if="recentProjects.length === 0" class="hub__empty">
          <p>No tienes proyectos recientes todavía.</p>
          <AppButton @click="startNewProject">Crear mi primer proyecto</AppButton>
        </div>

        <ul v-else class="hub__projects">
          <li v-for="project in recentProjects" :key="project.id">
            <button class="project-card" @click="openProject(project.id)">
              <div class="project-card__main">
                <span class="project-card__name">{{ project.name.replace(/\.ino$/i, '') }}</span>
                <span class="project-card__meta">{{ project.fqbn || 'Placa no definida' }}</span>
              </div>
              <div class="project-card__side">
                <span class="project-card__type">{{ project.type === 'code' ? 'Código' : 'Blocky' }}</span>
                <span class="project-card__date">{{ formatDate(project.updated_at) }}</span>
              </div>
            </button>
          </li>
        </ul>
      </section>
    </main>

    <!-- Modal Nuevo Proyecto -->
    <AppDialog :open="showNewProjectModal" title="Configurar nuevo proyecto" @close="showNewProjectModal = false">
      <div class="new-project">
        <div class="field">
          <label for="p-name" class="field__label">Nombre del proyecto</label>
          <input id="p-name" v-model="newProjectForm.name" type="text" class="field__input" placeholder="MiProyecto" data-autofocus />
        </div>

        <div class="field">
          <label for="p-board" class="field__label">Placa</label>
          <select id="p-board" v-model="newProjectForm.fqbn" class="field__select">
            <option v-for="board in BOARDS_PREDEFINIDOS" :key="board.fqbn" :value="board.fqbn">
              {{ board.nombre }}
            </option>
          </select>
        </div>

        <div class="field">
          <span class="field__label">Tipo de proyecto</span>
          <div class="type-selector">
            <label class="type-option">
              <input v-model="newProjectForm.type" type="radio" value="code" name="p-type" />
              <div class="type-option__box">
                <span class="type-option__title">Código C++</span>
                <span class="type-option__desc">Programación estándar Arduino</span>
              </div>
            </label>
            <label class="type-option type-option--disabled">
              <input type="radio" value="blocky" name="p-type" disabled title="Próximamente" />
              <div class="type-option__box">
                <span class="type-option__title">Blocky (Próximamente)</span>
                <span class="type-option__desc">Programación visual por bloques</span>
              </div>
            </label>
          </div>
        </div>
      </div>

      <template #footer>
        <AppButton variant="ghost" @click="showNewProjectModal = false">Cancelar</AppButton>
        <AppButton @click="finalizeNewProject">Finalizar</AppButton>
      </template>
    </AppDialog>
  </div>
</template>

<style scoped>
.hub {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background-color: var(--kor-bg);
  color: var(--kor-text);
  font-family: var(--kor-font-ui);
}

.hub__content {
  display: flex;
  flex: 1;
  min-height: 0;
  gap: var(--kor-space-6);
  padding: var(--kor-space-6);
}

.hub__panel {
  display: flex;
  flex-direction: column;
}

.hub__panel--left {
  flex: 0 0 320px;
}

.hub__panel--right {
  flex: 1;
  min-width: 0;
}

.hub__section-title {
  margin: 0 0 var(--kor-space-4);
  color: var(--kor-text-strong);
  font-size: var(--kor-text-lg);
  font-weight: var(--kor-weight-bold);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.hub__actions {
  display: flex;
  flex-direction: column;
  gap: var(--kor-space-3);
}

.hub__action-card {
  display: flex;
  align-items: center;
  gap: var(--kor-space-4);
  padding: var(--kor-space-4);
  background-color: var(--kor-surface-2);
  border: 1px solid var(--kor-border-subtle);
  border-radius: var(--kor-radius-lg);
  color: var(--kor-text);
  text-align: left;
  cursor: pointer;
  transition: all var(--kor-dur-base) var(--kor-ease-out);
}

.hub__action-card:hover {
  background-color: var(--kor-surface-3);
  border-color: var(--kor-accent);
  transform: translateY(-2px);
}

.hub__action-card--ghost {
  background-color: transparent;
  border-style: dashed;
}

.hub__action-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  background-color: var(--kor-surface-3);
  border-radius: var(--kor-radius-md);
  font-size: 20px;
  color: var(--kor-accent);
}

.hub__action-info {
  display: flex;
  flex-direction: column;
}

.hub__action-name {
  color: var(--kor-text-strong);
  font-weight: var(--kor-weight-bold);
}

.hub__action-desc {
  font-size: var(--kor-text-xs);
  color: var(--kor-text-muted);
}

.hub__projects {
  display: flex;
  flex-direction: column;
  gap: var(--kor-space-2);
  margin: 0;
  padding: 0;
  list-style: none;
}

.project-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: var(--kor-space-3) var(--kor-space-4);
  background-color: var(--kor-surface);
  border: 1px solid var(--kor-border-subtle);
  border-radius: var(--kor-radius-md);
  color: var(--kor-text);
  text-align: left;
  cursor: pointer;
  transition: background-color var(--kor-dur-fast) var(--kor-ease-out);
}

.project-card:hover {
  background-color: var(--kor-surface-2);
  border-color: var(--kor-border);
}

.project-card__main {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.project-card__name {
  color: var(--kor-text-strong);
  font-weight: var(--kor-weight-medium);
}

.project-card__meta {
  font-size: var(--kor-text-xs);
  color: var(--kor-text-muted);
  font-family: var(--kor-font-mono);
}

.project-card__side {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 2px;
}

.project-card__type {
  font-size: 10px;
  padding: 2px 6px;
  background-color: var(--kor-surface-3);
  border-radius: var(--kor-radius-pill);
  text-transform: uppercase;
}

.project-card__date {
  font-size: var(--kor-text-xs);
  color: var(--kor-text-muted);
}

.hub__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--kor-space-4);
  padding: var(--kor-space-6);
  background-color: var(--kor-surface);
  border: 1px dashed var(--kor-border);
  border-radius: var(--kor-radius-lg);
  color: var(--kor-text-muted);
}

.hub__update {
  margin-top: auto;
  display: flex;
  gap: var(--kor-space-3);
  padding: var(--kor-space-3);
  background-color: rgba(0, 122, 204, 0.1);
  border: 1px solid var(--kor-accent);
  border-radius: var(--kor-radius-md);
}

.hub__update-text {
  margin: 0;
  font-size: var(--kor-text-xs);
}

.hub__update-link {
  background: none;
  border: none;
  padding: 0;
  color: var(--kor-accent);
  font-size: var(--kor-text-xs);
  font-weight: var(--kor-weight-bold);
  cursor: pointer;
  text-decoration: underline;
}

/* Modal fields */
.new-project {
  display: flex;
  flex-direction: column;
  gap: var(--kor-space-4);
}

.field {
  display: flex;
  flex-direction: column;
  gap: var(--kor-space-2);
}

.field__label {
  color: var(--kor-text-muted);
  font-size: var(--kor-text-xs);
  font-weight: var(--kor-weight-bold);
  text-transform: uppercase;
}

.field__input, .field__select {
  height: 38px;
  padding: 0 var(--kor-space-3);
  background-color: var(--kor-surface-3);
  border: 1px solid var(--kor-border);
  border-radius: var(--kor-radius-md);
  color: var(--kor-text-strong);
}

.field__input:focus, .field__select:focus {
  outline: none;
  border-color: var(--kor-accent);
  box-shadow: 0 0 0 2px rgba(0, 122, 204, 0.2);
}

.type-selector {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--kor-space-3);
}

.type-option {
  position: relative;
  cursor: pointer;
}

.type-option input {
  position: absolute;
  opacity: 0;
}

.type-option__box {
  padding: var(--kor-space-3);
  background-color: var(--kor-surface-3);
  border: 2px solid transparent;
  border-radius: var(--kor-radius-md);
  transition: all var(--kor-dur-fast) var(--kor-ease-out);
}

.type-option input:checked + .type-option__box {
  border-color: var(--kor-accent);
  background-color: rgba(0, 122, 204, 0.1);
}

.type-option__title {
  display: block;
  color: var(--kor-text-strong);
  font-weight: var(--kor-weight-bold);
  margin-bottom: 4px;
}

.type-option__desc {
  display: block;
  font-size: 11px;
  color: var(--kor-text-muted);
}

.type-option--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

@media (max-width: 900px) {
  .hub__content {
    flex-direction: column;
  }
  .hub__panel--left {
    flex: none;
  }
}
</style>