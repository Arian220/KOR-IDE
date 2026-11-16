export { arduinoApi } from './client'
export { sketchApi } from './tauri'
export { CLI_NO_INSTALADO, esCancelado, esErrorCliAusente, mensajeDeError } from './errors'
export type { ArduinoApi, ArduinoRunOptions } from './types'
export {
  CONFIG_KEY,
  SESSION_KEY,
  DEFAULT_CONFIG,
  loadConfig,
  saveConfig,
  loadSession,
  saveSession,
  clearSession,
} from './storage'
