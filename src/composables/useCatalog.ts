import { ref } from 'vue'
import { CATALOG_API } from '@/api/types'
import type { CoreInfo } from '@/types/contract'

export function useCatalog() {
  const cores = ref<CoreInfo[]>([])
  const loading = ref(false)
  const installing = ref<string | null>(null)
  const installError = ref<string | null>(null)

  async function loadCores() {
    loading.value = true
    try {
      cores.value = await CATALOG_API.listCores()
    } finally {
      loading.value = false
    }
  }

  async function ensureCore(coreId: string): Promise<boolean> {
    const existing = cores.value.find((c: CoreInfo) => c.id === coreId)
    if (existing?.installed) return true

    installError.value = null
    installing.value = coreId
    try {
      await CATALOG_API.installCore(coreId, (_stage: string) => {
        // Progress event handled by channel listener
      })
      await loadCores()
      return true
    } catch (err) {
      installError.value = err instanceof Error ? err.message : 'Error al instalar core'
      return false
    } finally {
      installing.value = null
    }
  }

  function isCoreInstalled(coreId: string): boolean {
    return cores.value.some((c: CoreInfo) => c.id === coreId && c.installed)
  }

  return {
    cores,
    loading,
    installing,
    installError,
    loadCores,
    ensureCore,
    isCoreInstalled,
  }
}