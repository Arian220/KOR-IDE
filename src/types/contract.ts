export type PinFunction = 'salida' | 'entrada' | 'pwm'

export interface LegacyPortInfo {
  address: string
  label?: string
  protocol?: string
}

export interface LegacyBoardCandidate {
  name?: string
  fqbn?: string
}

export interface LegacyDetectedPort {
  port: LegacyPortInfo
  matching_boards?: LegacyBoardCandidate[]
}

export interface BoardListResponse {
  detected_ports?: LegacyDetectedPort[]
}

export interface CommandResult {
  success: boolean
  stdout: string
  stderr: string
  error?: string
  memory?: MemoryState | null
}

export interface CliInfo {
  path: string
  version: string
  source: 'managed' | 'manual' | 'path'
}

export interface CliStatus {
  available: boolean
  info: CliInfo | null
  required_version: string
}

export interface MemoryUsage {
  bytes: number
  percent: number
}

export interface MemoryState {
  flash: MemoryUsage | null
  sram: MemoryUsage | null
}

export interface CoreInfo {
  id: string
  installed: boolean
  version: string | null
  latest_version: string | null
}

export interface LibraryInfo {
  name: string
  installed_version: string | null
  latest_version: string | null
  author: string | null
  sentence: string | null
}
