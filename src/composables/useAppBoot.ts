import { readonly, ref } from 'vue'
import { arduinoApi, sketchApi } from '@/api'
import { loadConfig } from '@/api/storage'
import { useUpdates } from '@/composables/useUpdates'
import type { BoardInfo, BootResult, SketchProject } from '@/types/app'

const DEFAULT_CLI_ERROR = 'arduino-cli no está disponible.'

/**
 * La ventana del splash y la ventana principal (Hub/IDE) son contextos distintos.
 * Guardamos el resultado de la carga en localStorage para que el Hub lo muestre
 * al instante sin repetir la detección de placas ni la lectura de proyectos.
 */
const BOOT_CACHE_KEY = 'kor.boot.v1'

let bootPromise: Promise<BootResult> | null = null

function emptyBootResult(): BootResult {
  return {
    cliAvailable: false,
    cliPath: null,
    cliError: null,
    boards: [],
    boardError: null,
    preferences: loadConfig(),
    recentProjects: [],
    updateAvailable: false,
    latestVersion: null,
  }
}

function readCache(): BootResult | null {
  if (typeof window === 'undefined') return null
  try {
    const raw = window.localStorage.getItem(BOOT_CACHE_KEY)
    if (!raw) return null
    const data = JSON.parse(raw) as Partial<BootResult> | null
    if (!data || !Array.isArray(data.boards) || !Array.isArray(data.recentProjects)) {
      return null
    }
    return { ...emptyBootResult(), ...data }
  } catch {
    return null
  }
}

function writeCache(data: BootResult): void {
  if (typeof window === 'undefined') return
  try {
    window.localStorage.setItem(BOOT_CACHE_KEY, JSON.stringify(data))
  } catch {
    // Sin almacenamiento disponible: el Hub repetirá la carga.
  }
}

const cached = readCache()
const booted = ref(cached !== null)
const bootError = ref<string | null>(null)
const result = ref<BootResult | null>(cached)

function projectTypeOf(): 'code' | 'blocky' {
  return 'code'
}

async function loadBootResult(): Promise<BootResult> {
  const { check } = useUpdates()

  const preferences = loadConfig()

  // 1. arduino-cli
  let cliAvailable = false
  let cliPath: string | null = null
  let cliError: string | null = null
  try {
    const cli = await arduinoApi.verificarCli()
    cliAvailable = cli.available
    cliPath = cli.info?.path ?? null
    if (!cli.available) {
      cliError = DEFAULT_CLI_ERROR
    }
  } catch (error) {
    cliError = error instanceof Error ? error.message : DEFAULT_CLI_ERROR
  }

  // 2. placas conectadas (llamada directa a la API)
  let boards: BoardInfo[] = []
  let boardError: string | null = null
  try {
    const respuesta = await arduinoApi.detectarPlacas()
    boards = (respuesta.detected_ports ?? [])
      .filter((item) => typeof item.port?.address === 'string' && item.port.address.length > 0)
      .map((item, index) => {
        const candidate = item.matching_boards?.[0]
        return {
          port: item.port.address,
          label: candidate?.name ?? item.port.label ?? `Placa ${index + 1}`,
          protocol: item.port.protocol ?? null,
          fqbn: candidate?.fqbn ?? null,
        }
      })
  } catch (error) {
    boardError = error instanceof Error ? error.message : 'No se pudieron detectar placas.'
  }

  // 3. proyectos recientes
  let recentProjects: SketchProject[] = []
  try {
    const sketches = await sketchApi.list()
    recentProjects = sketches.map((sketch) => ({
      ...sketch,
      type: projectTypeOf(),
    }))
  } catch {
    recentProjects = []
  }

  // 4. actualizaciones (con límite de tiempo para no retrasar el arranque)
  const update = await Promise.race([
    check().catch(() => null),
    new Promise<null>((resolve) => setTimeout(() => resolve(null), 2500)),
  ])

  return {
    cliAvailable,
    cliPath,
    cliError,
    boards,
    boardError,
    preferences,
    recentProjects,
    updateAvailable: update?.available ?? false,
    latestVersion: update?.latestVersion ?? null,
  }
}

export function useAppBoot() {
  if (!bootPromise) {
    bootPromise = loadBootResult()
      .then((data) => {
        result.value = data
        booted.value = true
        bootError.value = null
        writeCache(data)
        return data
      })
      .catch((error) => {
        bootError.value = error instanceof Error ? error.message : 'Error al iniciar KOR.'
        result.value = {
          ...emptyBootResult(),
          cliError: bootError.value,
          boardError: bootError.value,
        }
        booted.value = true
        writeCache(result.value)
        return result.value
      })
  }

  async function runDiagnostico(): Promise<BootResult> {
    bootPromise = null
    booted.value = false
    bootError.value = null
    result.value = null
    return useAppBoot().ensureBoot()
  }

  return {
    booted: readonly(booted),
    bootError: readonly(bootError),
    result: readonly(result),
    ensureBoot: () => bootPromise!,
    runDiagnostico,
  }
}