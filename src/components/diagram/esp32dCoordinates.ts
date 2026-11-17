export interface PinCoordinate {
  pin: string
  cx: number
  cy: number
}

export const PINES_COORDENADAS_ESP32D: PinCoordinate[] = [
  // Fila izquierda (pines GPIO configurables)
  { pin: '23', cx: 35, cy: 94 },
  { pin: '22', cx: 35, cy: 112 },
  { pin: '21', cx: 35, cy: 130 },
  { pin: '19', cx: 35, cy: 148 },
  { pin: '18', cx: 35, cy: 166 },
  { pin: '5',  cx: 35, cy: 202 },
  { pin: '17', cx: 35, cy: 220 },
  { pin: '16', cx: 35, cy: 238 },
  { pin: '4',  cx: 35, cy: 256 },
  { pin: '0',  cx: 35, cy: 274 },
  { pin: '2',  cx: 35, cy: 292 },
  { pin: '15', cx: 35, cy: 310 },
  // Fila derecha (pines GPIO configurables)
  { pin: '25', cx: 205, cy: 94 },
  { pin: '26', cx: 205, cy: 112 },
  { pin: '27', cx: 205, cy: 130 },
  { pin: '32', cx: 205, cy: 148 },
  { pin: '33', cx: 205, cy: 166 },
  { pin: '34', cx: 205, cy: 184 },
  { pin: '35', cx: 205, cy: 202 },
  { pin: '14', cx: 205, cy: 238 },
  { pin: '12', cx: 205, cy: 256 },
  { pin: '13', cx: 205, cy: 274 },
]