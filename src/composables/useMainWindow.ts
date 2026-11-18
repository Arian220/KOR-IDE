import { useRouter } from 'vue-router'

function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

/**
 * Abre la ventana principal (Hub/IDE). En Wayland el compositor ignora el
 * reposicionamiento, así que en lugar de redimensionar la ventana del splash se
 * crea una ventana nueva con el tamaño definitivo desde Rust.
 */
export function useMainWindow() {
  const router = useRouter()

  async function openMainWindow(): Promise<void> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('open_main_window')
        return
      } catch (error) {
        console.warn('No se pudo abrir la ventana principal, usando el router:', error)
      }
    }

    // Respaldo para ejecución en navegador (sin Tauri).
    await router.replace('/hub')
  }

  return { openMainWindow }
}