export interface TauriPortInfo {
  address: string
  label?: string | null
  protocol?: string | null
  protocol_label?: string | null
  properties?: Record<string, string>
}

export interface TauriBoardCandidate {
  name: string
  fqbn: string
}

export interface TauriBoardInfo {
  port: TauriPortInfo
  matching_boards: TauriBoardCandidate[]
  protocol: string | null
  protocol_label: string | null
}

export interface TauriCliInfo {
  path: string
  version: string
  source: 'managed' | 'manual' | 'path'
}

export interface TauriCliStatus {
  available: boolean
  info: TauriCliInfo | null
  required_version: string
}

export interface TauriMemoryUsage {
  flash_used: number
  flash_max: number
  flash_percent: number
  sram_used: number
  sram_max: number
  sram_percent: number
}

export interface TauriCompileResult {
  success: boolean
  stdout: string
  stderr: string
  diagnostics: unknown[]
  memory: TauriMemoryUsage | null
  build_dir: string | null
}

export interface TauriUploadResult {
  success: boolean
  stdout: string
  stderr: string
  port: string
  fqbn: string
}

export interface TauriSketch {
  id: string
  name: string
  content: string
  fqbn: string | null
  updated_at: string
}

export interface TauriSketchSummary {
  id: string
  name: string
  updated_at: string
  fqbn: string | null
}

export interface TauriAppError {
  kind: string
  data?: unknown
}

export type TauriJobEvent =
  | {
      type: 'started'
      payload: {
        job_id: string
        kind: 'compile' | 'upload' | 'install_core' | 'install_library' | 'download_cli'
      }
    }
  | { type: 'progress'; payload: { job_id: string; stage: string; percent: number | null } }
  | {
      type: 'output'
      payload: { job_id: string; stream: 'stdout' | 'stderr'; chunk: string }
    }
  | { type: 'finished'; payload: { job_id: string; success: boolean } }
  | { type: 'failed'; payload: { job_id: string; error: TauriAppError } }
