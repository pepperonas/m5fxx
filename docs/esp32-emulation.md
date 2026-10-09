# ESP32-S3-Binärfirmware

## Ablauf

`m5fxx-esp32` prüft das ESP32-S3-Image, legt eine private Flash-Kopie im temporären Verzeichnis an und startet einen separaten QEMU-Prozess. Die Eingabedatei wird nicht verändert. Das zusammengeführte Image enthält Bootloader bei `0x0000`, Partitionstabelle bei `0x8000` und Anwendung bei `0x10000`. Unterstützte deklarierte Flashgrößen: 2, 4, 8 und 16 MiB; fehlender Flashinhalt wird mit `0xff` aufgefüllt.

Eine lokale TCP-Brücke verbindet das Board-Modell mit Rust. QEMU überträgt LCD-Befehle, Pixeldaten und Helligkeit; Rust sendet Matrix-Tasten, G0, Pause und Reset. Der ST7789-Controller erzeugt die sichtbaren 240 × 135 Pixel. UART-Ausgaben kommen vom tatsächlichen Emulatorprozess.

Der temporäre Flash dient dieser Laufzeitsitzung. Änderungen durch die Firmware werden nicht in die originale `.bin` zurückgeschrieben; beim erneuten Import entsteht eine neue Kopie. Der QEMU-Prozess erhält keine Host-Funkverbindung und keine Verbindung zur nativen SD-Sandbox.

## Geprüfter Stand

Mit dem bereitgestellten zusammengeführten **Bruce-1.8-Image** wurden Bootlogo, Hauptmenü und der Wechsel von WiFi zu BLE über die emulierte Tastatur beobachtet. Die README zeigt Aufnahmen dieses Stands. Das ist keine Zusage für alle Versionen oder Funktionen von Bruce.

| Bereich | Implementierung / Grenze |
| --- | --- |
| CPU und Flash | Öffentlicher Espressif-QEMU für ESP32-S3; lokale SPI-Flash-Korrekturen |
| Display | GPSPI2/3, CPU-/DMA-Schreibdaten, LCD-Auswahl und ST7789-Befehlssatz |
| Eingabe | Original-Cardputer-GPIO-Matrix und G0; ADV-TCA8418 nicht modelliert |
| Hintergrundbeleuchtung | LEDC-Duty wird an die Darstellung übertragen |
| Batterie | Synthetischer ADC-Wert; keine reale Akkumessung |
| Wi-Fi, BLE, ESP-NOW | Keine funktionsfähige Funkemulation |
| SD, Audio, Mikrofon, Grove, externe RF-Module | Noch nicht vollständig emuliert bzw. nicht angebunden |
| I²C | Registermodell; keine vollständige Bus-/Geräteemulation |

Menüs hardwareabhängiger Funktionen können sichtbar sein, obwohl die zugehörige Hardware fehlt. Solche Funktionen können warten, Fehler melden oder die Firmware abbrechen lassen. Das Modell ist keine vollständige elektrische Nachbildung des Geräts.

## Quellen und Build

Basis ist [Espressifs öffentlicher QEMU](https://github.com/espressif/qemu), Version `esp-develop-9.2.2-20260417`, Commit `40edccac415693c5130f91c01d84176ae6008566`. [Espressifs ESP32-S3-Anleitung](https://docs.espressif.com/projects/esp-idf/en/v5.5/esp32s3/api-guides/tools/qemu.html) beschreibt den zugrunde liegenden Emulator.

Die lokalen Erweiterungen sind in `tools/qemu/m5fxx_cardputer.c` und `tools/qemu/patch.py` nachvollziehbar. [Velxio](https://github.com/davidmonterocrespo24/velxio) diente als Architekturhinweis; der Build verwendet keine dort separat bereitgestellten QEMU-Archive oder Lizenzschlüssel.

Installation und Fehlersuche: [tools/qemu/README.md](../tools/qemu/README.md). Displaymodell: [Display-Fidelity](display-fidelity.md).
