import { ref } from 'vue'
import type { ToastKind } from '@/types/app'

export interface ToastItem {
  id: number
  message: string
  kind: ToastKind
}

const toasts = ref<ToastItem[]>([])
const timers = new Map<number, ReturnType<typeof setTimeout>>()
let nextId = 1

export function useToast() {
  function dismiss(id: number) {
    const timer = timers.get(id)
    if (timer) {
      clearTimeout(timer)
      timers.delete(id)
    }
    toasts.value = toasts.value.filter((toast) => toast.id !== id)
  }

  function show(message: string, kind: ToastKind = 'info', duration = 4000): number {
    const id = nextId
    nextId += 1
    toasts.value = [...toasts.value, { id, message, kind }]
    if (duration > 0) {
      timers.set(
        id,
        setTimeout(() => dismiss(id), duration),
      )
    }
    return id
  }

  return { toasts, show, dismiss }
}
