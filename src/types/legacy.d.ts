import type { CommandResult } from './contract'

export interface ArduinoLegacyApi {
  detectarPlacas(rutaCli?: string): Promise<string>
  compilar(codigo: string, rutaCli?: string): Promise<CommandResult>
  subirAPlaca(codigo: string, rutaCli?: string, puerto?: string): Promise<CommandResult>
}

export interface ElectronWindowControls {
  minimize(): void
  toggleMaximize(): void
  close(): void
}

declare global {
  interface Window {
    api?: ArduinoLegacyApi
    windowControls?: ElectronWindowControls
  }
}

export {}
