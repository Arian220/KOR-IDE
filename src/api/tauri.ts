import { Channel, invoke } from '@tauri-apps/api/core'
import type { BoardListResponse, CliStatus, CommandResult } from '@/types/contract'
import type {
  TauriAppError,
  TauriBoardInfo,
  TauriCliStatus,
  TauriCompileResult,
  TauriJobEvent,
  TauriSketch,
  TauriSketchSummary,
  TauriUploadResult,
} from '@/types/tauri'
import type { ArduinoApi, ArduinoRunOptions } from './types'

const MANAGED_SKETCH_KEY = 'kor.managedSketchId'
const MANAGED_SKETCH_NAME = 'KOR Sketch'
const DEFAULT_FQBN = 'arduino:avr:uno'

let managedSketchId: string | null = null

function esAppError(value: unknown): value is TauriAppError {
  return (
    !!value &&
    typeof value === 'object' &&
    typeof (value as { kind?: unknown }).kind === 'string'
  )
}

function leerSketchGestionado(): string | null {
  if (managedSketchId) return managedSketchId
  try {
    managedSketchId = window.localStorage.getItem(MANAGED_SKETCH_KEY)
  } catch {
    managedSketchId = null
  }
  return managedSketchId
}

function guardarSketchGestionado(id: string): void {
  managedSketchId = id
  try {
    window.localStorage.setItem(MANAGED_SKETCH_KEY, id)
  } catch {
    // Sin almacenamiento: el id sigue en memoria.
  }
}

function limpiarSketchGestionado(): void {
  managedSketchId = null
  try {
    window.localStorage.removeItem(MANAGED_SKETCH_KEY)
  } catch {
    // Sin almacenamiento disponible.
  }
}

async function crearSketchGestionado(): Promise<string> {
  const sketch = await invoke<TauriSketch>('create_sketch', { name: MANAGED_SKETCH_NAME })
  guardarSketchGestionado(sketch.id)
  return sketch.id
}

async function prepararSketch(codigo: string, fqbn: string): Promise<string> {
  for (let intento = 0; intento < 2; intento += 1) {
    let id = leerSketchGestionado()
    if (!id) id = await crearSketchGestionado()

    try {
      await invoke<TauriSketchSummary>('save_sketch', {
        sketch: {
          id,
          name: MANAGED_SKETCH_NAME,
          content: codigo,
          fqbn,
          updated_at: new Date().toISOString(),
        },
      })
      return id
    } catch (error) {
      if (intento === 0 && esAppError(error) && error.kind === 'sketch_not_found') {
        limpiarSketchGestionado()
        continue
      }
      throw error
    }
  }

  throw new Error('No se pudo preparar el sketch para la operación.')
}

function desenvolverEvento(message: unknown): TauriJobEvent | null {
  if (!message || typeof message !== 'object') return null
  const directo = message as { type?: unknown }
  if (typeof directo.type === 'string') return message as TauriJobEvent

  const envuelto = message as { data?: unknown }
  if (envuelto.data && typeof envuelto.data === 'object') {
    const candidato = envuelto.data as { type?: unknown }
    if (typeof candidato.type === 'string') return envuelto.data as TauriJobEvent
  }

  return null
}

function crearCanal(options?: ArduinoRunOptions): Channel<TauriJobEvent> {
  const channel = new Channel<TauriJobEvent>()
  channel.onmessage = (message) => {
    const evento = desenvolverEvento(message)
    if (!evento) return

    if (evento.type === 'started') {
      options?.onJobId?.(evento.payload.job_id)
      return
    }

    if (evento.type === 'progress') {
      options?.onProgress?.(evento.payload.stage)
      return
    }

    if (evento.type === 'output') {
      options?.onOutput?.(evento.payload.chunk, evento.payload.stream)
    }
  }
  return channel
}

function resultadoDeError(error: TauriAppError, mensaje: string): CommandResult {
  const data = error.data as { stdout?: string; stderr?: string } | undefined
  return {
    success: false,
    stdout: data?.stdout ?? '',
    stderr: data?.stderr ?? '',
    error: mensaje,
  }
}

export const tauriApi: ArduinoApi = {
  puedeCancelar: true,

  async detectarPlacas(): Promise<BoardListResponse> {
    const boards = await invoke<TauriBoardInfo[]>('detect_boards')
    return {
      detected_ports: boards.map((board) => ({
        port: {
          address: board.port.address,
          label: board.port.label ?? undefined,
          protocol: board.port.protocol ?? undefined,
        },
        matching_boards: board.matching_boards.map((candidate) => ({
          name: candidate.name,
          fqbn: candidate.fqbn,
        })),
      })),
    }
  },

  async verificarCli(): Promise<CliStatus> {
    const status = await invoke<TauriCliStatus>('get_cli_status')
    return {
      available: status.available,
      info: status.info,
      required_version: status.required_version,
    }
  },

  async compilar(codigo: string, options?: ArduinoRunOptions): Promise<CommandResult> {
    const fqbn = options?.fqbn || DEFAULT_FQBN
    const sketchId = await prepararSketch(codigo, fqbn)
    const events = crearCanal(options)

    try {
      const resultado = await invoke<TauriCompileResult>('compile', {
        request: {
          sketch_path: sketchId,
          fqbn,
          build_properties: [],
          warnings: 'all',
          clean: false,
        },
        events,
      })

      return {
        success: true,
        stdout: resultado.stdout,
        stderr: resultado.stderr,
      }
    } catch (error) {
      if (esAppError(error) && error.kind === 'compile_failed') {
        return resultadoDeError(error, 'La compilación falló.')
      }
      throw error
    }
  },

  async subirAPlaca(
    codigo: string,
    puerto: string,
    options?: ArduinoRunOptions,
  ): Promise<CommandResult> {
    const fqbn = options?.fqbn || DEFAULT_FQBN
    const sketchId = await prepararSketch(codigo, fqbn)
    const events = crearCanal(options)

    try {
      const resultado = await invoke<TauriUploadResult>('upload', {
        request: {
          sketch_path: sketchId,
          fqbn,
          port: puerto,
          compile_first: true,
          verify: false,
          programmer: null,
          baudrate: null,
          build_properties: [],
        },
        events,
      })

      return {
        success: true,
        stdout: resultado.stdout,
        stderr: resultado.stderr,
      }
    } catch (error) {
      if (esAppError(error) && error.kind === 'upload_failed') {
        return resultadoDeError(error, 'No se pudo cargar el programa.')
      }
      throw error
    }
  },

  async cancelar(jobId: string): Promise<boolean> {
    return invoke<boolean>('cancel_job', { jobId })
  },
}
