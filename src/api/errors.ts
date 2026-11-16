import type { CommandResult } from '@/types/contract'

export const CLI_NO_INSTALADO =
  'No se encontró arduino-cli. Instálalo o configura VITE_ARDUINO_CLI_PATH.'

interface AppErrorShape {
  kind: string
  data?: unknown
}

function esFormaAppError(error: unknown): error is AppErrorShape {
  return (
    !!error &&
    typeof error === 'object' &&
    typeof (error as { kind?: unknown }).kind === 'string'
  )
}

function texto(error: unknown): string {
  if (error instanceof Error) return error.message
  if (typeof error === 'string') return error
  return ''
}

function textoDeDato(data: unknown): string {
  if (typeof data === 'string') return data
  if (data && typeof data === 'object' && 'message' in data) {
    return String((data as { message: unknown }).message)
  }
  return ''
}

function mensajeDeAppError(error: AppErrorShape): string {
  switch (error.kind) {
    case 'cli_not_found':
      return CLI_NO_INSTALADO
    case 'cli_download':
      return 'No se pudo descargar arduino-cli.'
    case 'cli_checksum_mismatch':
      return 'El binario de arduino-cli no es válido.'
    case 'cli_unsupported':
      return `La versión de arduino-cli no es compatible. ${textoDeDato(error.data)}`.trim()
    case 'board_not_found':
      return 'No se detectó ninguna placa.'
    case 'invalid_port':
      return 'El puerto seleccionado no es válido.'
    case 'invalid_fqbn':
      return 'La placa seleccionada no es válida.'
    case 'compile_failed':
      return 'La compilación falló.'
    case 'upload_failed':
      return 'No se pudo cargar el programa.'
    case 'cancelled':
      return 'Operación cancelada.'
    case 'job_not_found':
      return 'La operación ya no está en curso.'
    case 'sketch_not_found':
      return 'No se encontró el sketch.'
    case 'path_not_allowed':
      return 'Ruta no permitida.'
    case 'network':
      return 'Error de red.'
    case 'invalid_config':
      return `Configuración inválida. ${textoDeDato(error.data)}`.trim()
    case 'io':
      return `Error de entrada/salida. ${textoDeDato(error.data)}`.trim()
    case 'internal':
      return `Error interno. ${textoDeDato(error.data)}`.trim()
    default:
      return 'Error desconocido.'
  }
}

export function esErrorCliAusente(error: unknown): boolean {
  if (esFormaAppError(error)) {
    return error.kind === 'cli_not_found'
  }

  const message = texto(error).toLowerCase()
  if (!message) return false
  return (
    message.includes('enoent') ||
    message.includes('not found') ||
    message.includes('no such file') ||
    message.includes('orden no encontrada') ||
    message.includes('no se encontró arduino-cli') ||
    message.includes('no se encontro arduino-cli')
  )
}

export function esCancelado(error: unknown): boolean {
  return esFormaAppError(error) && error.kind === 'cancelled'
}

export function mensajeDeError(error: unknown): string {
  if (esFormaAppError(error)) return mensajeDeAppError(error)
  if (esErrorCliAusente(error)) return CLI_NO_INSTALADO
  if (error instanceof Error) return error.message
  if (typeof error === 'string') return error
  return 'Error desconocido.'
}

export function normalizarResultado(result: CommandResult): CommandResult {
  if (result.success) return result
  if (esErrorCliAusente(result.error) || esErrorCliAusente(result.stderr)) {
    return { ...result, error: CLI_NO_INSTALADO, stderr: '' }
  }
  return result
}
