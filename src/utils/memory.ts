import type { MemoryState, MemoryUsage } from '@/types/contract'

function toUsage(match: RegExpMatchArray | null): MemoryUsage | null {
  if (!match) return null
  return {
    bytes: Number(match[1]),
    percent: Number(match[2]),
  }
}

function firstMatch(text: string, patterns: RegExp[]): RegExpMatchArray | null {
  for (const pattern of patterns) {
    const match = text.match(pattern)
    if (match) return match
  }
  return null
}

interface SectionSize {
  name?: string
  size?: number
  maxSize?: number
  max_size?: number
}

interface BuilderResult {
  executableSectionsSize?: SectionSize[]
  executable_sections_size?: SectionSize[]
}

interface CompileJson {
  builderResult?: BuilderResult
  builder_result?: BuilderResult
}

function sectionUsage(section: SectionSize | undefined): MemoryUsage | null {
  if (!section || typeof section.size !== 'number') return null
  const max = section.maxSize ?? section.max_size ?? 0
  const percent = max > 0 ? Math.round((section.size / max) * 1000) / 10 : 0
  return { bytes: section.size, percent }
}

function matchesName(name: string | undefined, candidates: string[]): boolean {
  if (!name) return false
  const normalized = name.toLowerCase()
  return candidates.some((candidate) => normalized === candidate)
}

function parseJson(stdout: string): MemoryState | null {
  const start = stdout.indexOf('{')
  const end = stdout.lastIndexOf('}')
  if (start === -1 || end <= start) return null

  let parsed: CompileJson
  try {
    parsed = JSON.parse(stdout.slice(start, end + 1)) as CompileJson
  } catch {
    return null
  }

  const builder = parsed.builderResult ?? parsed.builder_result
  const sections = builder?.executableSectionsSize ?? builder?.executable_sections_size
  if (!sections || sections.length === 0) return null

  const flash = sectionUsage(
    sections.find((section) => matchesName(section.name, ['flash', 'text'])),
  )
  const sram = sectionUsage(
    sections.find((section) => matchesName(section.name, ['data', 'sram', 'bss'])),
  )

  if (!flash && !sram) return null
  return { flash, sram }
}

export function parsearMemoria(stdout: string, stderr: string): MemoryState {
  const json = parseJson(stdout)
  if (json) return json

  const texto = `${stdout}\n${stderr}`
  const flashMatch = firstMatch(texto, [
    /Sketch uses (\d+) bytes \((\d+)%\) of program storage space/i,
    /El Sketch usa (\d+) bytes \((\d+)%\) del espacio de almacenamiento de programa/i,
  ])
  const sramMatch = firstMatch(texto, [
    /Global variables use (\d+) bytes \((\d+)%\) of dynamic memory/i,
    /Las variables Globales usan (\d+) bytes \((\d+)%\) de la memoria dinámica/i,
  ])

  return {
    flash: toUsage(flashMatch),
    sram: toUsage(sramMatch),
  }
}
