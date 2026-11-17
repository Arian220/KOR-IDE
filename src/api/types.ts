import type { BoardListResponse, CliStatus, CommandResult, CoreInfo, LibraryInfo } from '@/types/contract'

export interface ArduinoRunOptions {
  fqbn?: string
  sketchId?: string | null
  sketchName?: string
  onJobId?: (jobId: string) => void
  onProgress?: (stage: string) => void
  onOutput?: (chunk: string, stream: 'stdout' | 'stderr') => void
}

export interface CatalogApi {
  listCores(): Promise<CoreInfo[]>
  installCore(id: string, onProgress?: (stage: string) => void): Promise<CoreInfo>
  uninstallCore(id: string): Promise<boolean>
  listLibraries(query?: string): Promise<LibraryInfo[]>
  installLibrary(name: string, version?: string, onProgress?: (stage: string) => void): Promise<LibraryInfo>
  uninstallLibrary(name: string): Promise<boolean>
}

export interface ArduinoApi {
  readonly puedeCancelar: boolean
  detectarPlacas(): Promise<BoardListResponse>
  verificarCli(): Promise<CliStatus>
  compilar(codigo: string, options?: ArduinoRunOptions): Promise<CommandResult>
  subirAPlaca(
    codigo: string,
    puerto: string,
    options?: ArduinoRunOptions,
  ): Promise<CommandResult>
  cancelar(jobId: string): Promise<boolean>
}

export const CATALOG_API: CatalogApi = {
  async listCores(): Promise<CoreInfo[]> {
    const { invoke } = await import('@tauri-apps/api/core')
    return invoke<CoreInfo[]>('list_cores')
  },
  async installCore(id: string, onProgress?: (stage: string) => void): Promise<CoreInfo> {
    const { invoke } = await import('@tauri-apps/api/core')
    const { listen } = await import('@tauri-apps/api/event')
    const prefix = `core-install-${id}`
    const unlisten = await listen(`${prefix}:progress`, (event: any) => {
      onProgress?.(event.payload)
    })
    try {
      return await invoke<CoreInfo>('install_core', { id, channel: prefix })
    } finally {
      unlisten()
    }
  },
  async uninstallCore(id: string): Promise<boolean> {
    const { invoke } = await import('@tauri-apps/api/core')
    return invoke<boolean>('uninstall_core', { id })
  },
  async listLibraries(query?: string): Promise<LibraryInfo[]> {
    const { invoke } = await import('@tauri-apps/api/core')
    return invoke<LibraryInfo[]>('list_libraries', { query })
  },
  async installLibrary(name: string, version?: string, onProgress?: (stage: string) => void): Promise<LibraryInfo> {
    const { invoke } = await import('@tauri-apps/api/core')
    const { listen } = await import('@tauri-apps/api/event')
    const prefix = `lib-install-${name}`
    const unlisten = await listen(`${prefix}:progress`, (event: any) => {
      onProgress?.(event.payload)
    })
    try {
      return await invoke<LibraryInfo>('install_library', { name, version, channel: prefix })
    } finally {
      unlisten()
    }
  },
  async uninstallLibrary(name: string): Promise<boolean> {
    const { invoke } = await import('@tauri-apps/api/core')
    return invoke<boolean>('uninstall_library', { name })
  },
}