export interface PinCoordinate {
  pin: string
  cx: number
  cy: number
}

export const PINES_COORDENADAS: PinCoordinate[] = [
  // Fila superior (digitales)
  { pin: '13', cx: 134.8, cy: 19 },
  { pin: '12', cx: 143.8, cy: 19 },
  { pin: '11', cx: 152.8, cy: 19 },
  { pin: '10', cx: 161.8, cy: 19 },
  { pin: '9', cx: 170.8, cy: 19 },
  { pin: '8', cx: 179.8, cy: 19 },
  { pin: '7', cx: 194.2, cy: 19 },
  { pin: '6', cx: 203.2, cy: 19 },
  { pin: '5', cx: 212.2, cy: 19 },
  { pin: '4', cx: 221.2, cy: 19 },
  { pin: '3', cx: 230.2, cy: 19 },
  { pin: '2', cx: 239.2, cy: 19 },
  { pin: '1', cx: 248.2, cy: 19 },
  { pin: '0', cx: 257.2, cy: 19 },
  // Fila inferior (analógicos)
  { pin: 'A0', cx: 212.2, cy: 190 },
  { pin: 'A1', cx: 221.2, cy: 190 },
  { pin: 'A2', cx: 230.2, cy: 190 },
  { pin: 'A3', cx: 239.2, cy: 190 },
  { pin: 'A4', cx: 248.2, cy: 190 },
  { pin: 'A5', cx: 257.2, cy: 190 },
]
