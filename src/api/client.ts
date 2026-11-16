import type { ArduinoLegacyApi } from '@/types/legacy'
import type { BoardListResponse, CliStatus, CommandResult } from '@/types/contract'
import { normalizarResultado } from './errors'
import { tauriApi } from './tauri'
import type { ArduinoApi } from './types'

const isWindows = typeof navigator !== 'undefined' && /Windows/i.test(navigator.userAgent)
const CLI_PATH =
  import.meta.env.VITE_ARDUINO_CLI_PATH || (isWindows ? 'C:\\ArduinoCLI\\arduino-cli.exe' : 'arduino-cli')

function getLegacyApi(): ArduinoLegacyApi {
  if (!window.api) {
    throw new Error('El backend de escritorio no está disponible. Ejecuta KOR dentro de Electron o Tauri.')
  }
  return window.api
}

const electronApi: ArduinoApi = {
  puedeCancelar: false,

  async detectarPlacas(): Promise<BoardListResponse> {
    const raw = await getLegacyApi().detectarPlacas(CLI_PATH)
    return JSON.parse(raw) as BoardListResponse
  },

  async verificarCli(): Promise<CliStatus> {
    return { available: true, info: null, required_version: '' }
  },

  async compilar(codigo: string): Promise<CommandResult> {
    return normalizarResultado(await getLegacyApi().compilar(codigo, CLI_PATH))
  },

  async subirAPlaca(codigo: string, puerto: string): Promise<CommandResult> {
    return normalizarResultado(await getLegacyApi().subirAPlaca(codigo, CLI_PATH, puerto))
  },

  async cancelar(): Promise<boolean> {
    throw new Error('La cancelación no está disponible en el backend de Electron.')
  },
}

const isTauri =
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

export const arduinoApi: ArduinoApi = isTauri ? tauriApi : electronApi
