// ESP32-C6 UART to USB bridge
void setup() {
  Serial.begin(115200);       // USB to PC
  Serial1.begin(115200, SERIAL_8N1, 20, 21);  // UART: RX=20, TX=21

  while (!Serial) {}

  Serial.println("Hello, world!");
}

void loop() {
  // Forward data from ESP UART (Serial1) to USB serial (Serial)
  if (Serial1.available()) {
    Serial.write(Serial1.read());
  }

  // Echo PC commands back to Pi
  if (Serial.available()) {
    Serial1.write(Serial.read());
  }
}
