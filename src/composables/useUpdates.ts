import { ref } from 'vue'
import { APP_INFO } from '@/config/appInfo'

export interface UpdateCheck {
  checking: boolean
  available: boolean
  latestVersion: string | null
  error: string | null
}

const state = ref<UpdateCheck>({
  checking: false,
  available: false,
  latestVersion: null,
  error: null,
})

let initialized = false

export function useUpdates() {
  if (!initialized) {
    initialized = true
  }

  async function check(): Promise<UpdateCheck> {
    if (state.value.checking) return { ...state.value }
    state.value.checking = true
    state.value.error = null

    try {
      const response = await fetch(APP_INFO.releasesApi, {
        headers: { Accept: 'application/vnd.github+json' },
      })
      if (response.status === 404) {
        state.value.available = false
        state.value.latestVersion = null
        return { ...state.value }
      }
      if (!response.ok) throw new Error(' respuesta no válida')

      const data = (await response.json()) as { tag_name?: string }
      const latest = data.tag_name?.replace(/^v/i, '') ?? null
      state.value.latestVersion = latest
      state.value.available = !!latest && latest !== APP_INFO.version
      return { ...state.value }
    } catch (error) {
      state.value.error = error instanceof Error ? error.message : 'Error desconocido'
      return { ...state.value }
    } finally {
      state.value.checking = false
    }
  }

  return { state, check }
}