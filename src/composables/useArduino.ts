import { computed, onMounted, ref } from 'vue'
import {
  CLI_NO_INSTALADO,
  arduinoApi,
  esCancelado,
  esErrorCliAusente,
  mensajeDeError,
} from '@/api'
import type { ArduinoRunOptions } from '@/api'
import { useToast } from '@/composables/useToast'
import type { BoardListResponse, CommandResult, MemoryState } from '@/types/contract'
import type { BoardInfo } from '@/types/app'
import { parsearMemoria } from '@/utils/memory'

export type BusyAction = 'compile' | 'upload' | null
export type ConsoleVariant = 'info' | 'error' | 'success'
export type StatusKind = 'idle' | 'busy' | 'success' | 'error' | 'warning'

const MAX_CONSOLE_LINES = 5000
const DEFAULT_FQBN = 'arduino:avr:uno'

function emptyMemory(): MemoryState {
  return { flash: null, sram: null }
}

function limitLines(text: string): string {
  const lines = text.split('\n')
  if (lines.length <= MAX_CONSOLE_LINES) return text
  return lines.slice(lines.length - MAX_CONSOLE_LINES).join('\n')
}

function capitalizar(value: string): string {
  if (!value) return value
  return value.charAt(0).toUpperCase() + value.slice(1)
}

function salidaDe(resultado: CommandResult): string {
  return [resultado.stdout, resultado.stderr]
    .map((value) => value.trim())
    .filter(Boolean)
    .join('\n')
}

function mapBoards(response: BoardListResponse): BoardInfo[] {
  return (response.detected_ports ?? [])
    .filter((item) => typeof item.port?.address === 'string' && item.port.address.length > 0)
    .map((item, index) => {
      const candidate = item.matching_boards?.[0]
      return {
        port: item.port.address,
        label: candidate?.name ?? item.port.label ?? `Placa ${index + 1}`,
        protocol: item.port.protocol ?? null,
        fqbn: candidate?.fqbn ?? null,
      }
    })
}

export function useArduino() {
  const { show: showToast } = useToast()

  const busyAction = ref<BusyAction>(null)
  const detecting = ref(false)
  const currentJobId = ref<string | null>(null)

  const boards = ref<BoardInfo[]>([])
  const selectedPort = ref<string | null>(null)
  const hasDetected = ref(false)
  const boardError = ref<string | null>(null)

  const memoryVisible = ref(false)
  const memoryError = ref(false)
  const memory = ref<MemoryState>(emptyMemory())

  const consoleVisible = ref(false)
  const consoleText = ref('')
  const consoleVariant = ref<ConsoleVariant>('info')

  const status = ref('Listo')
  const statusKind = ref<StatusKind>('idle')

  const isBusy = computed(() => busyAction.value !== null || detecting.value)
  const canCancel = computed(
    () => busyAction.value !== null && currentJobId.value !== null && arduinoApi.puedeCancelar,
  )

  let streamBuffer = ''

  function setStatus(label: string, kind: StatusKind) {
    status.value = label
    statusKind.value = kind
  }

  function resetMemory() {
    memory.value = emptyMemory()
    memoryError.value = false
  }

  function showConsole(text: string, variant: ConsoleVariant) {
    consoleText.value = limitLines(text)
    consoleVariant.value = variant
    consoleVisible.value = true
  }

  function clearConsole() {
    streamBuffer = ''
    consoleText.value = ''
    consoleVisible.value = false
    consoleVariant.value = 'info'
  }

  function applyMemory(result: { stdout: string; stderr: string }) {
    memory.value = parsearMemoria(result.stdout, result.stderr)
    memoryError.value = false
    memoryVisible.value = true
  }

  function setSelectedPort(port: string | null) {
    selectedPort.value = port
    if (port) setStatus(`Placa seleccionada: ${port}`, 'success')
  }

  function fqbnSeleccionado(port: string | null): string {
    const board = boards.value.find((item) => item.port === port)
    return board?.fqbn || DEFAULT_FQBN
  }

  async function comprobarCli() {
    try {
      const cli = await arduinoApi.verificarCli()
      if (!cli.available) {
        boardError.value = CLI_NO_INSTALADO
        setStatus('arduino-cli no instalado', 'error')
      }
    } catch {
      // La detección manual mostrará el error correspondiente.
    }
  }

  onMounted(() => {
    void comprobarCli()
  })

  async function detectar(notifyEmpty = true): Promise<BoardInfo[]> {
    if (busyAction.value || detecting.value) return boards.value

    detecting.value = true
    setStatus('Buscando placas...', 'busy')

    try {
      const respuesta = await arduinoApi.detectarPlacas()
      boards.value = mapBoards(respuesta)
      hasDetected.value = true
      boardError.value = null

      if (boards.value.length === 0) {
        selectedPort.value = null
        setStatus('Sin placa', 'warning')
        if (notifyEmpty) {
          showToast('No se detectó ninguna placa. Revisa el cable USB y los drivers.', 'error')
        }
      } else {
        selectedPort.value = boards.value[0].port
        setStatus(
          boards.value.length === 1
            ? `Placa lista en ${boards.value[0].port}`
            : `${boards.value.length} placas detectadas`,
          'success',
        )
      }

      return boards.value
    } catch (error) {
      const mensaje = mensajeDeError(error)
      boards.value = []
      selectedPort.value = null
      hasDetected.value = true
      boardError.value = mensaje
      setStatus(esErrorCliAusente(error) ? 'arduino-cli no instalado' : 'CLI no disponible', 'error')
      showToast(mensaje, 'error')
      return []
    } finally {
      detecting.value = false
    }
  }

  async function compilar(codigo: string, options?: ArduinoRunOptions) {
    if (isBusy.value) return

    busyAction.value = 'compile'
    currentJobId.value = null
    setStatus('Compilando...', 'busy')
    memoryVisible.value = true
    resetMemory()
    clearConsole()

    try {
      const resultado = await arduinoApi.compilar(codigo, {
        ...options,
        fqbn: fqbnSeleccionado(selectedPort.value),
        onJobId: (jobId) => {
          currentJobId.value = jobId
        },
        onProgress: (stage) => setStatus(capitalizar(stage), 'busy'),
      })

      if (resultado.success) {
        applyMemory(resultado)
        showConsole(
          salidaDe(resultado) || 'Compilación completada sin salida del CLI.',
          'success',
        )
        setStatus('Compilación correcta', 'success')
      } else {
        const sinCli = esErrorCliAusente(resultado.error) || esErrorCliAusente(resultado.stderr)
        memoryError.value = true
        setStatus(sinCli ? 'arduino-cli no instalado' : 'Error de compilación', 'error')
        showConsole(
          salidaDe(resultado) || resultado.error || 'La compilación falló.',
          'error',
        )
      }
    } catch (error) {
      if (esCancelado(error)) {
        memoryError.value = true
        setStatus('Operación cancelada', 'warning')
        showConsole('Operación cancelada.', 'info')
        return
      }

      const sinCli = esErrorCliAusente(error)
      memoryError.value = true
      setStatus(sinCli ? 'arduino-cli no instalado' : 'Error de compilación', 'error')
      showConsole(mensajeDeError(error), 'error')
    } finally {
      currentJobId.value = null
      busyAction.value = null
    }
  }

  async function subir(codigo: string, options?: ArduinoRunOptions) {
    if (isBusy.value) return

    if (boards.value.length === 0) {
      await detectar(false)
      if (boards.value.length === 0) {
        if (!boardError.value) {
          showToast('Conecta una placa antes de subir el código.', 'error')
        }
        return
      }
    }

    const puerto = selectedPort.value ?? boards.value[0]?.port
    if (!puerto) {
      setStatus('Sin placa', 'warning')
      showToast('No hay un puerto seleccionado.', 'error')
      return
    }

    busyAction.value = 'upload'
    currentJobId.value = null
    selectedPort.value = puerto
    setStatus(`Subiendo a ${puerto}...`, 'busy')
    memoryVisible.value = true
    resetMemory()

    streamBuffer = `Subiendo a ${puerto}...\n`
    showConsole(streamBuffer, 'info')

    try {
      const resultado = await arduinoApi.subirAPlaca(codigo, puerto, {
        ...options,
        fqbn: fqbnSeleccionado(puerto),
        onJobId: (jobId) => {
          currentJobId.value = jobId
        },
        onProgress: (stage) => setStatus(capitalizar(stage), 'busy'),
        onOutput: (chunk) => {
          streamBuffer = limitLines(streamBuffer + chunk)
          consoleText.value = streamBuffer
          consoleVisible.value = true
        },
      })

      if (resultado.success) {
        setStatus(`Carga completada en ${puerto}`, 'success')
        showConsole(`Código subido con éxito a ${puerto}.`, 'success')
        applyMemory(resultado)
        showToast(`Código subido a ${puerto}.`, 'success')
      } else {
        const sinCli = esErrorCliAusente(resultado.error) || esErrorCliAusente(resultado.stderr)
        setStatus(sinCli ? 'arduino-cli no instalado' : 'Error al subir', 'error')
        showConsole(
          `Error al subir:\n${salidaDe(resultado) || resultado.error || ''}`,
          'error',
        )
        showToast(
          sinCli ? resultado.error || 'arduino-cli no instalado.' : 'No se pudo subir el código a la placa.',
          'error',
        )
      }
    } catch (error) {
      if (esCancelado(error)) {
        setStatus('Operación cancelada', 'warning')
        showConsole('Operación cancelada.', 'info')
        return
      }

      const sinCli = esErrorCliAusente(error)
      setStatus(sinCli ? 'arduino-cli no instalado' : 'Error al subir', 'error')
      showConsole(mensajeDeError(error), 'error')
      showToast(
        sinCli ? mensajeDeError(error) : 'Error al comunicarse con la placa.',
        'error',
      )
    } finally {
      currentJobId.value = null
      busyAction.value = null
    }
  }

  async function cancelar() {
    const jobId = currentJobId.value
    if (!jobId || !arduinoApi.puedeCancelar) return

    try {
      await arduinoApi.cancelar(jobId)
      setStatus('Cancelando...', 'busy')
    } catch (error) {
      showToast(mensajeDeError(error), 'error')
    }
  }

  return {
    busyAction,
    detecting,
    isBusy,
    canCancel,
    boards,
    selectedPort,
    hasDetected,
    boardError,
    status,
    statusKind,
    memoryVisible,
    memoryError,
    memory,
    consoleVisible,
    consoleText,
    consoleVariant,
    compilar,
    subir,
    detectar,
    cancelar,
    setSelectedPort,
    clearConsole,
  }
}
