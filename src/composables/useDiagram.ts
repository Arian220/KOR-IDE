import { computed, ref } from 'vue'
import type { PinFunction } from '@/types/contract'
import { generarCodigo } from '@/utils/codegen'

export function useDiagram() {
  const estadoPines = ref<Record<string, PinFunction>>({})

  const generatedCode = computed(() => generarCodigo(estadoPines.value))

  function setPin(pin: string, func: PinFunction | null) {
    const next = { ...estadoPines.value }
    if (func) next[pin] = func
    else delete next[pin]
    estadoPines.value = next
  }

  return { estadoPines, generatedCode, setPin }
}
