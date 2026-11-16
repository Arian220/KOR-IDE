import type { MemoryState, MemoryUsage } from '@/types/contract'

function toUsage(match: RegExpMatchArray | null): MemoryUsage | null {
  if (!match) return null
  return {
    bytes: Number(match[1]),
    percent: Number(match[2]),
  }
}

export function parsearMemoria(stdout: string, stderr: string): MemoryState {
  const texto = `${stdout}\n${stderr}`
  const flashMatch = texto.match(/Sketch uses (\d+) bytes \((\d+)%\) of program storage space/)
  const sramMatch = texto.match(/Global variables use (\d+) bytes \((\d+)%\) of dynamic memory/)

  return {
    flash: toUsage(flashMatch),
    sram: toUsage(sramMatch),
  }
}
