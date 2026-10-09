# Öffentlicher Cardputer-QEMU

Der normale Espressif-QEMU enthält die benötigte Cardputer-LCD-/Tastenbrücke nicht. Dieses Verzeichnis baut eine angepasste Version aus dem öffentlichen, festgeschriebenen Quellstand. Ein unveränderter QEMU ersetzt diesen Build nicht.

## Voraussetzungen

Benötigt werden ein C/C++-Compiler, Python 3 mit `venv`, `curl`, `tar`, `pkg-config`, GLib, Pixman und libgcrypt. Auf macOS sind Xcode Command Line Tools erforderlich; die Bibliotheken lassen sich beispielsweise mit Homebrew installieren:

```bash
brew install pkg-config glib pixman libgcrypt
./tools/qemu/build.sh
```

Das Skript lädt Commit `40edccac415693c5130f91c01d84176ae6008566` von [espressif/qemu](https://github.com/espressif/qemu), wendet die lokalen Anpassungen an und installiert Ninja 1.13.2 in einer eigenen Python-Umgebung. Quelle und Build liegen unter `target/`; ausführbare Datei und S3-ROM landen im ignorierten Verzeichnis `tools/qemu/bin/`.

```bash
M5FXX_BUILD_JOBS=2 ./tools/qemu/build.sh
cargo run -p m5fxx-desktop
```

Der Laufzeitpfad wurde auf macOS Apple Silicon geprüft. Linux- und Windows-Ausführung dieses neuen Firmwarepfads sind noch nicht verifiziert; das Buildskript benötigt Bash.

## Eigenen Build verwenden

```bash
M5FXX_QEMU=/absoluter/pfad/qemu-system-xtensa cargo run -p m5fxx-desktop
```

Der Emulator muss dieselbe Cardputer-Brücke enthalten. `esp32s3_rev0_rom.bin` muss neben der ausführbaren Datei, in deren `pc-bios`-Unterordner oder im benachbarten `../pc-bios` liegen.

Bei „Cardputer QEMU missing“ zuerst das Buildskript ausführen oder den Pfad prüfen. Bei fehlender Verbindung zur Cardputer-Brücke sicherstellen, dass der angepasste Build verwendet wird. Bei einem abgebrochenen Erstbuild meldet das Skript einen unvollständigen Quellbaum; diesen unter `target/cardputer-qemu-src` umbenennen und erneut starten. Ein fertiger Build wird beim nächsten Aufruf wiederverwendet.

## Änderungen und Lizenz

`patch.py` ergänzt das Board-Modell und korrigiert SPI-Transferlängen, mehrspurige Flash-Lesezugriffe, GD25Q64-Statusregister und die deaktivierte Slirp-Abhängigkeit. `m5fxx_cardputer.c` ergänzt GPSPI-/LCD-Übertragung, GPIO-Tasten, LEDC-Helligkeit und synthetische ADC-Werte.

QEMU und das Board-Modell stehen unter **GPL-2.0-or-later**; die ursprünglichen Quellen behalten ihre Lizenzhinweise. Es werden keine Velxio-Downloads oder Schlüssel benötigt. Firmwaredateien werden nicht mitgeliefert. Unterstützte Funktionen und Grenzen stehen in [ESP32-Emulation](../../docs/esp32-emulation.md).
