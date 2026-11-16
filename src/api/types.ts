import type { BoardListResponse, CliStatus, CommandResult } from '@/types/contract'

export interface ArduinoRunOptions {
  fqbn?: string
  sketchId?: string | null
  sketchName?: string
  onJobId?: (jobId: string) => void
  onProgress?: (stage: string) => void
  onOutput?: (chunk: string, stream: 'stdout' | 'stderr') => void
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
