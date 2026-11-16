<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import svgMarkup from '@/assets/arduino-uno.svg?raw'
import type { PinFunction } from '@/types/contract'
import { PINES_COORDENADAS } from './pinCoordinates'

const props = defineProps<{ pinStates: Record<string, PinFunction> }>()
const emit = defineEmits<{ change: [payload: { pin: string; func: PinFunction | null }] }>()

type PinSelection = 'ninguno' | PinFunction

const FILLS: Record<'none' | PinFunction, string> = {
  none: 'color-mix(in srgb, var(--kor-focus) 12%, transparent)',
  salida: 'color-mix(in srgb, var(--kor-success) 60%, transparent)',
  entrada: 'color-mix(in srgb, var(--kor-accent) 60%, transparent)',
  pwm: 'color-mix(in srgb, var(--kor-warning) 60%, transparent)',
}

const HOVER_FILL = 'color-mix(in srgb, var(--kor-focus) 55%, transparent)'

const root = ref<HTMLDivElement | null>(null)
const container = ref<HTMLDivElement | null>(null)
const menuVisible = ref(false)
const menuTop = ref(0)
const menuLeft = ref(0)
const activePin = ref<string | null>(null)
const selectedFunc = ref<PinSelection>('ninguno')

let escala = 1.0
let posX = 0
let posY = 0
let isDragging = false
let startX = 0
let startY = 0

const circles = new Map<string, SVGCircleElement>()

function fillFor(func?: PinFunction) {
  return FILLS[func ?? 'none']
}

function applyTransform() {
  const svg = container.value?.querySelector('svg')
  if (svg) svg.style.transform = `translate(${posX}px, ${posY}px) scale(${escala})`
}

function updateVisual() {
  circles.forEach((circle, pin) => {
    circle.style.fill = fillFor(props.pinStates[pin])
  })
}

function onWheel(event: WheelEvent) {
  event.preventDefault()
  const factorZoom = event.deltaY < 0 ? 1.1 : 0.9
  escala = Math.min(5, Math.max(0.5, escala * factorZoom))
  applyTransform()
}

function onMouseDown(event: MouseEvent) {
  if ((event.target as Element).tagName.toLowerCase() === 'circle') return
  isDragging = true
  startX = event.clientX - posX
  startY = event.clientY - posY
}

function onMouseMove(event: MouseEvent) {
  if (!isDragging) return
  posX = event.clientX - startX
  posY = event.clientY - startY
  applyTransform()
}

function onMouseUp() {
  isDragging = false
}

function hideMenu() {
  menuVisible.value = false
}

function onDocumentClick() {
  hideMenu()
}

function openMenu(pin: string, circle: SVGCircleElement) {
  activePin.value = pin
  selectedFunc.value = props.pinStates[pin] ?? 'ninguno'
  const rootRect = root.value?.getBoundingClientRect()
  const circleRect = circle.getBoundingClientRect()
  if (rootRect) {
    menuTop.value = circleRect.top - rootRect.top + 10
    menuLeft.value = circleRect.left - rootRect.left + 10
  }
  menuVisible.value = true
}

function onSelectChange() {
  if (!activePin.value) return
  const func = selectedFunc.value === 'ninguno' ? null : selectedFunc.value
  emit('change', { pin: activePin.value, func })
  hideMenu()
}

function mountDiagram() {
  const host = container.value
  if (!host) return

  host.innerHTML = svgMarkup
  const svg = host.querySelector('svg')
  if (!svg) {
    host.textContent = 'Error al cargar arduino-uno.svg'
    return
  }

  const layer = document.createElementNS('http://www.w3.org/2000/svg', 'g')
  svg.appendChild(layer)

  PINES_COORDENADAS.forEach((p) => {
    const circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle')
    circle.setAttribute('cx', String(p.cx))
    circle.setAttribute('cy', String(p.cy))
    circle.setAttribute('r', '4.5')
    circle.style.fill = fillFor(props.pinStates[p.pin])
    circle.style.stroke = 'var(--kor-focus)'
    circle.style.strokeWidth = '0.7'
    circle.style.cursor = 'pointer'
    circle.setAttribute('data-pin', p.pin)

    circle.addEventListener('mouseenter', () => {
      circle.style.fill = HOVER_FILL
    })
    circle.addEventListener('mouseleave', () => {
      circle.style.fill = fillFor(props.pinStates[p.pin])
    })
    circle.addEventListener('click', (event) => {
      event.stopPropagation()
      openMenu(p.pin, circle)
    })

    layer.appendChild(circle)
    circles.set(p.pin, circle)
  })
}

onMounted(() => {
  mountDiagram()
  container.value?.addEventListener('wheel', onWheel, { passive: false })
  container.value?.addEventListener('mousedown', onMouseDown)
  window.addEventListener('mousemove', onMouseMove)
  window.addEventListener('mouseup', onMouseUp)
  document.addEventListener('click', onDocumentClick)
})

onBeforeUnmount(() => {
  container.value?.removeEventListener('wheel', onWheel)
  container.value?.removeEventListener('mousedown', onMouseDown)
  window.removeEventListener('mousemove', onMouseMove)
  window.removeEventListener('mouseup', onMouseUp)
  document.removeEventListener('click', onDocumentClick)
  circles.clear()
})

watch(() => props.pinStates, updateVisual, { deep: true })
</script>

<template>
  <div ref="root" class="diagram">
    <div ref="container" class="diagram__canvas">Cargando placa...</div>
    <div
      v-show="menuVisible"
      class="menu-pin"
      :style="{ top: `${menuTop}px`, left: `${menuLeft}px` }"
      @click.stop
    >
      <select v-model="selectedFunc" @change="onSelectChange">
        <option value="ninguno">-- Quitar configuración --</option>
        <option value="salida">Salida Digital (Parpadear)</option>
        <option value="entrada">Entrada Digital (Leer)</option>
        <option value="pwm">Salida PWM (Atenuar)</option>
      </select>
    </div>
  </div>
</template>

<style scoped>
.diagram {
  position: relative;
  flex: 1;
  display: flex;
  min-height: 0;
}

.diagram__canvas {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: var(--kor-bg);
  border-radius: var(--kor-radius-lg);
  padding: 10px;
  margin-top: 10px;
  overflow: hidden;
  cursor: grab;
}

.diagram__canvas:active {
  cursor: grabbing;
}

.diagram__canvas :deep(svg) {
  max-width: none;
  max-height: none;
  transform-origin: 0 0;
  transition: transform 0.1s ease-out;
}

.menu-pin {
  position: absolute;
  background-color: var(--kor-surface-3);
  border: 1px solid var(--kor-border);
  border-radius: var(--kor-radius-md);
  padding: 5px;
  z-index: 10;
  box-shadow: var(--kor-shadow-md);
}

.menu-pin select {
  background-color: var(--kor-bg);
  color: var(--kor-text-strong);
  border: 1px solid var(--kor-border-subtle);
  padding: 5px;
  border-radius: var(--kor-radius-sm);
  width: 100%;
  cursor: pointer;
}
</style>
