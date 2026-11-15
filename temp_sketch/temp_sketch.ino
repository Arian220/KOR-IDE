// === BEGIN PIN CONFIG ===
const int PIN_13 = 13;

void setup() {
  pinMode(PIN_13, OUTPUT);
}

void loop() {
  // Parpadear Pin 13
  digitalWrite(PIN_13, HIGH);
  delay(500);
  digitalWrite(PIN_13, LOW);
  delay(500);
}

// === END PIN CONFIG ===

// --- ESCRIBE TU CÓDIGO PERSONALIZADO AQUÍ ABAJO ---
