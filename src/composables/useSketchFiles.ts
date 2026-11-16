import { ref } from 'vue'
import { sketchApi } from '@/api'
import { mensajeDeError } from '@/api/errors'
import { useEditor } from '@/composables/useEditor'
import { useToast } from '@/composables/useToast'
import type { EditorTab, SketchSummary } from '@/types/app'

function nombreBase(name: string): string {
  const base = name.trim().replace(/\.ino$/i, '').trim()
  return base.length > 0 ? base : 'sketch'
}

function nombreArchivo(name: string): string {
  return `${nombreBase(name)}.ino`
}

export function useSketchFiles() {
  const { tabs, activeTab, markSaved, openSketch, uniqueTabName } = useEditor()
  const { show } = useToast()

  const sketches = ref<SketchSummary[]>([])
  const loadingSketches = ref(false)
  const sketchListError = ref<string | null>(null)

  async function guardarTab(tab: EditorTab): Promise<boolean> {
    const base = nombreBase(tab.name)

    try {
      if (tab.sketchId) {
        const saved = await sketchApi.save({
          id: tab.sketchId,
          name: base,
          content: tab.content,
          fqbn: null,
          updated_at: new Date().toISOString(),
        })
        markSaved(tab.id, tab.sketchId, nombreArchivo(saved.name))
        return true
      }

      const destino = nombreBase(uniqueTabName(nombreArchivo(base), tab.id))
      const created = await sketchApi.create(destino)

      try {
        const saved = await sketchApi.save({
          id: created.id,
          name: created.name,
          content: tab.content,
          fqbn: null,
          updated_at: new Date().toISOString(),
        })
        markSaved(tab.id, created.id, nombreArchivo(saved.name))
        return true
      } catch (error) {
        await sketchApi.remove(created.id).catch(() => undefined)
        throw error
      }
    } catch (error) {
      show(`No se pudo guardar "${tab.name}": ${mensajeDeError(error)}`, 'error')
      return false
    }
  }

  async function guardarActivo(): Promise<boolean> {
    const tab = activeTab.value
    if (!tab) {
      show('No hay un archivo activo para guardar.', 'info')
      return false
    }

    const ok = await guardarTab(tab)
    if (ok) show(`"${tab.name}" guardado.`, 'success')
    return ok
  }

  async function guardarComo(name: string): Promise<boolean> {
    const tab = activeTab.value
    if (!tab) {
      show('No hay un archivo activo para guardar.', 'info')
      return false
    }

    const destino = nombreBase(uniqueTabName(nombreArchivo(name), tab.id))

    try {
      const created = await sketchApi.create(destino)

      try {
        const saved = await sketchApi.save({
          id: created.id,
          name: created.name,
          content: tab.content,
          fqbn: null,
          updated_at: new Date().toISOString(),
        })
        markSaved(tab.id, created.id, nombreArchivo(saved.name))
        show(`Guardado como "${tab.name}".`, 'success')
        return true
      } catch (error) {
        await sketchApi.remove(created.id).catch(() => undefined)
        throw error
      }
    } catch (error) {
      show(`No se pudo guardar como: ${mensajeDeError(error)}`, 'error')
      return false
    }
  }

  async function guardarTodo(): Promise<boolean> {
    if (tabs.value.length === 0) {
      show('No hay archivos para guardar.', 'info')
      return false
    }

    const pendientes = [...tabs.value]
    const resultados: boolean[] = []
    for (const tab of pendientes) {
      resultados.push(await guardarTab(tab))
    }

    const guardados = resultados.filter(Boolean).length
    if (guardados === pendientes.length) {
      show(
        pendientes.length === 1
          ? 'Archivo guardado.'
          : `${pendientes.length} archivos guardados.`,
        'success',
      )
      return true
    }

    show(`Se guardaron ${guardados} de ${pendientes.length} archivos.`, 'error')
    return false
  }

  async function cargarSketches(): Promise<SketchSummary[]> {
    loadingSketches.value = true
    sketchListError.value = null

    try {
      sketches.value = await sketchApi.list()
      return sketches.value
    } catch (error) {
      sketches.value = []
      sketchListError.value = mensajeDeError(error)
      return []
    } finally {
      loadingSketches.value = false
    }
  }

  async function abrirSketch(id: string): Promise<boolean> {
    try {
      const sketch = await sketchApi.read(id)
      openSketch(sketch.id, nombreArchivo(sketch.name), sketch.content)
      show(`"${nombreArchivo(sketch.name)}" abierto.`, 'success')
      return true
    } catch (error) {
      show(`No se pudo abrir el sketch: ${mensajeDeError(error)}`, 'error')
      return false
    }
  }

  return {
    sketches,
    loadingSketches,
    sketchListError,
    guardarTab,
    guardarActivo,
    guardarComo,
    guardarTodo,
    cargarSketches,
    abrirSketch,
  }
}
