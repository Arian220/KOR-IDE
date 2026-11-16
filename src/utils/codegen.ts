import type { PinFunction } from '@/types/contract'

export function generarCodigo(estadoPines: Record<string, PinFunction>): string {
  const pinesConfigurados = Object.keys(estadoPines)
  if (pinesConfigurados.length === 0) {
    return '// Esperando configuración...\n'
  }

  let codigo = '// === BEGIN PIN CONFIG ===\n'
  pinesConfigurados.forEach((pin) => {
    codigo += `const int PIN_${pin} = ${pin};\n`
  })

  codigo += '\nvoid setup() {\n'
  pinesConfigurados.forEach((pin) => {
    const func = estadoPines[pin]
    if (func === 'salida') codigo += `  pinMode(PIN_${pin}, OUTPUT);\n`
    if (func === 'entrada') codigo += `  pinMode(PIN_${pin}, INPUT_PULLUP);\n`
    if (func === 'pwm') codigo += `  pinMode(PIN_${pin}, OUTPUT);\n`
  })

  codigo += '}\n\nvoid loop() {\n'
  pinesConfigurados.forEach((pin) => {
    const func = estadoPines[pin]
    if (func === 'salida') {
      codigo += `  // Parpadear Pin ${pin}\n  digitalWrite(PIN_${pin}, HIGH);\n  delay(500);\n  digitalWrite(PIN_${pin}, LOW);\n  delay(500);\n`
    }
    if (func === 'entrada') {
      codigo += `  // Leer Pin ${pin}\n  int valor_${pin} = digitalRead(PIN_${pin});\n`
    }
    if (func === 'pwm') {
      codigo += `  // Atenuar Pin ${pin}\n  for(int i=0; i<=255; i+=5) { analogWrite(PIN_${pin}, i); delay(10); }\n`
    }
  })

  codigo += '}\n\n// === END PIN CONFIG ===\n\n// --- ESCRIBE TU CÓDIGO PERSONALIZADO AQUÍ ABAJO ---\n'
  return codigo
}
