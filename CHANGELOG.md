# Changelog

Alle nennenswerten Änderungen an diesem Projekt werden in dieser Datei dokumentiert.

Das Format basiert auf [Keep a Changelog](https://keepachangelog.com/de/1.1.0/) und dieses Projekt folgt den Regeln des [Semantic Versioning](https://semver.org/lang/de/).

---

## [0.2.0] - 2026-10-09

### Hinzugefügt (Added)
- **Echte ESP32-S3 Binärausführung via QEMU (`crates/m5fxx-esp32`):**
  - Integrierte QEMU-Child-Prozess-Anbindung mit lokalem Cardputer-Board-Treiber (`tools/qemu/m5fxx_cardputer.c`).
  - Echte Xtensa/ESP32-S3 Maschinencode-Ausführung von zusammengeführten `.bin`-Images (Bootloader @ 0x0, Partitionstabelle @ 0x8000, App @ 0x10000).
  - Verifiziert und getestet mit **Bruce 1.8** Firmware: Boot-Logo, Live-Menü und Tasten-Navigation (WiFi zu BLE).
  - Sichere temporäre Flash-Kopien (Originaldateien werden niemals modifiziert).
  - GPSPI2/3- und DMA-Transferbrücke zum ST7789V2-Displaycontroller.
  - Reproduzierbares QEMU-Buildskript (`tools/qemu/build.sh`) basierend auf Espressifs öffentlichem QEMU (`esp-develop-9.2.2-20260417`).
- **Nativer M5Stack Cardputer ADV Werksfirmware-Port (`crates/m5fxx-factory`):**
  - Vollständiger nativer C/C++17-Port der offiziellen ADV V0.3 Werksfirmware mit Mooncake UI-Framework und M5GFX.
  - Pixel-perfekte deterministische Frame-Tests für alle 13 integrierten Mooncake-Apps.
  - Simulierbare Peripherieeingaben (IMU Beschleunigung/Gyroskop, Akku, Wi-Fi/BLE-Status, Mikrofon-Pegel, Caps).
- **Firmware-Runtime & Boot-Screen im Demo-Modus:**
  - Nach Flash-Vorgängen startet die geladene Firmware direkt in einer dedizierten Runtime-Ansicht mit Betriebszeit, interaktivem seriellem Log (`UART0`) und Live-Tastenanzeige.
  - Automatisches Umschalten in den ADV-Factory-Modus bei M5Stack-Werksimages.
- **Vollständige Modifier- & Host-Tastatur-Unterstützung:**
  - Echtzeit-Übertragung von Host-Modifiern (`Shift`, `Ctrl`, `Alt`, `Fn`) auf die Cardputer 4×14 Matrix.
  - Direkte Navigation mit Host-Pfeiltasten, `Enter`, `Esc`, `Tab` und `Backspace`.
  - Fokusverlust-Schutz (`reset_all`) verhindert hängende Tastenzustände.
- **Drag & Drop Firmware-Installation:**
  - Drop beliebiger Firmware-Binärdateien direkt auf das Simulatorfenster ohne Fokuszwang.
  - Visuelles Drop-Overlay und animierte Flash-Fortschrittsanzeige im LCD.
- **Dokumentation & Mockups:**
  - Hochauflösende Screenshots der Stock-Werksfirmware (`boot.png`, `launcher.png`, Gesamtansicht) und der Bruce-Firmware in der README.
  - Ausführliche Spezifikation der Emulationsarchitektur in `docs/esp32-emulation.md` und `docs/display-fidelity.md`.

### Geändert (Changed)
- Versionierung aller Crates auf Semantic Versioning `0.2.0` vereinheitlicht.
- Optimierung der ST7789V2-Displaypipeline auf Nearest-Neighbor GPU-Filterung für gestochen scharfe Pixel-Darstellung.
- Developer Panel um Firmware-Reboot-Buttons (`▶ <name>`), Pause/Resume und detaillierte Systemmetriken erweitert.

---

## [0.1.0] - 2026-10-09

### Hinzugefügt (Added)
- Initiales Release des `m5fxx` M5Stack Cardputer Simulators in Rust (`eframe`/`egui`).
- Authentisches 840×540 Vektordesign des Cardputer-Gehäuses im hellgrauen Industrie-Look.
- Sandboxed MicroSD-Kartenspeicher mit Directory-Traversal-Schutz.
- ST7789V2 240×135 RGB565 IPS-Displaytreiber mit Display-Only Zoom-Modus.
- 56-Tasten Tastenmatrix mit visuellen Tastenkappen und taktilem BtnG0-Taster.
- Integriertes Developer-Panel zur Echtzeit-Überwachung von FPS, Eingaben und virtueller Hardware.
