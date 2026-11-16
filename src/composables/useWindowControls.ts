import { isTauri } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'

type WindowAction = 'minimize' | 'toggleMaximize' | 'close'

export function useWindowControls() {
  const tauri = isTauri()

  async function runTauri(action: WindowAction) {
    try {
      await getCurrentWindow()[action]()
    } catch {
      // La ventana no expone la operación: el control queda inerte.
    }
  }

  function runElectron(action: WindowAction) {
    window.windowControls?.[action]()
  }

  function run(action: WindowAction) {
    if (tauri) {
      void runTauri(action)
      return
    }
    runElectron(action)
  }

  return {
    minimize: () => run('minimize'),
    toggleMaximize: () => run('toggleMaximize'),
    close: () => run('close'),
  }
}
