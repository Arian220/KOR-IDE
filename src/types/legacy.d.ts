import type { CommandResult } from './contract'

export interface ArduinoLegacyApi {
  detectarPlacas(rutaCli?: string): Promise<string>
  compilar(codigo: string, rutaCli?: string): Promise<CommandResult>
  subirAPlaca(codigo: string, rutaCli?: string, puerto?: string): Promise<CommandResult>
}

declare global {
  interface Window {
    api?: ArduinoLegacyApi
  }
}

export {}
