# m5fxx - M5Stack Cardputer Desktop Simulator

Ein plattformübergreifender Desktop-Simulator für M5Stack Cardputer Firmware, geschrieben in **Rust** mit nativer Desktop-Oberfläche (`eframe`/`egui`).

Der Simulator läuft unter **macOS** (inkl. Apple Silicon & Intel), **Windows** und **Linux** und bildet den M5Stack Cardputer optisch sowie bei der Bedienung pixelgenau und maßstabsgetreu nach.

---

## 1. Features & Funktionsumfang

- **Realistische Hardware-Darstellung:**
  - Originalgetreues Gehäuse mit exakten Abmessungen (84.0 × 54.0 × 19.7 mm nach M5Stack-Spezifikation)
  - Vektorbasierte, stufenlos skalierbare Darstellung mit Retina-/HiDPI-Unterstützung
  - ST7789V2 240×135 IPS LCD-Display an exakter Position
  - Vollständige 56-Tasten-Tastatur (4 Zeilen × 14 Spalten) mit Originalbeschriftungen (Primary, Shift/Aa-Layer und Fn-Layer)
  - Interaktiver physischer G0-Taster (`BtnG0`) und PWR-Status-LED
  - Perforiertes Lautsprecher-Gitter und charakteristische Gehäuseschrauben
- **Display-Pipeline:**
  - Emulierter RGB565-Framebuffer mit echten ST7789-Farben
  - Nearest-Neighbor-Skalierung (keine unscharfe Pixelinterpolation)
  - Umschaltbarer **Zoomed Display Only Modus** (maximale ganzzahlige Pixelskalierung)
- **Eingabesystem:**
  - Klickbare virtuelle Tasten mit taktiler Hervorhebung
  - Volle Unterstützung physischer Host-Tastaturen (DE-QWERTZ, US-QWERTY, Sonderzeichen)
  - Trennung von Tastenzuständen (`Key-Down` / `Key-Up`) und Texteingabe
  - Focus-Loss-Schutz: Automatischer Reset aller Tasten bei Fokusverlust (verhindert hängende Tasten)
  - Maus-Tracking: Sauberes Loslassen bei Klicks außerhalb der Tastenfläche
- **Simulierte Hardware & Sandboxed SD-Karte:**
  - MicroSD-Dateisystem auf konfigurierbaren Host-Ordner gemappt
  - Pfad-Traversal-Schutz gegen Verzeichnis-Ausbrüche (`../` und absolute Pfade blockiert)
  - Monotone Systemzeit (`millis()`, `micros()`)
  - Statusanzeigen für Akku (mV, Prozent, Ladezustand)
  - Umschaltung zwischen **Original Cardputer** (Software-GPIO-Matrix) und **Cardputer ADV** (TCA8418 I²C Keypad Controller)
- **Entwickler-Panel:**
  - Simulation Pausieren / Fortsetzen
  - Firmware- und HAL-Reset
  - Live-Anzeige gedrückter Matrix-Tasten und aktiver Modifier (`Fn`, `Shift`, `Ctrl`, `Opt`, `Alt`)
  - SD-Karten-Ordnerauswahl per nativem Datei-Dialog (`rfd`)
  - Live-Protokollausgabe und FPS-Zähler

---

## 2. Hardware-Status: Unterstützt vs. Ausstehend

| Hardware-Komponente | Status | Implementierungsdetails |
| :--- | :--- | :--- |
| **ST7789V2 Display (240×135)** |  **Vollständig** | RGB565 Framebuffer, Primitiven, Font, GPU Nearest-Neighbor |
| **56-Tasten-Tastatur** |  **Vollständig** | 4×14 Matrix, Host-Keyboard-Mapping, Klickflächen, Modifier |
| **G0-Taster (BtnG0)** |  **Vollständig** | Klickbar, Download/Action-Button der Firmware |
| **MicroSD-Slot** |  **Vollständig** | Lokaler Sandboxed Ordner, Traversal-Protection |
| **System-Uhr & Timer** |  **Vollständig** | Monotone `millis()`, `micros()`, Frame-Delta `dt` |
| **Akku / Power-Status** |  **Simuliert** | Spannung (mV), Prozentwert, Ladeerkennung |
| **NS4168 1W Lautsprecher** |  **Vorbereitet** | HAL-Audio-Stubs vorhanden, DSP/Beep-Erweiterung möglich |
| **SPM1423 PDM Mikrofon** |  **Vorbereitet** | Virtuelle Audiopuffer-Schnittstelle vorbereitet |
| **Wi-Fi / ESP-NOW / BLE** | ❌ **Nicht implementiert** | Wird im UI explizit als nicht unterstützt deklariert |
| **Grove HY2.0-4P / GPIOs** | ❌ **Nicht implementiert** | Externe Hardware-Pins nicht simuliert |

---

## 3. Lauffähige Beispielanwendung

Die mitgelieferte Firmware-Anwendung (`m5fxx-app-demo`) demonstriert alle Kernaspekte:
1. **SplashScreen:** Animierter Startbildschirm mit Ladebalken und automatischer/tastenbasierter Weiterleitung
2. **Hauptmenü:** Navigation über Cardputer-Tastatur (`Fn+;` für Up, `Fn+.` für Down, `Enter` zum Auswählen, oder Direkttasten `1`–`4`)
3. **Text Editor:** Vollwertiger Texteditor mit blinkendem Cursor, Backspace, Zeilenumbruch und Zeichenzählung
4. **Grafikanimation:** Physikbasierte, mehrfarbige Bouncing Balls und flüssiges Sternenfeld (60 FPS)
5. **SD-Karten-Verwaltung:** Schreiben (`W`) und Lesen (`R`) einer Datei (`sample.txt`) auf der virtuellen SD-Karte
6. **System- & Hardware-Info:** Modell-, Display-, Controller- und Akkustatus

---

## 4. Schnellstart & Ausführung

### Voraussetzungen
- Rust 1.80+ (getestet mit Rust 1.98.0)
- Cargo

### Starten des Simulators
```bash
cargo run --release -p m5fxx-desktop
```

### Ausführen der Tests
```bash
cargo test --workspace
```

### Codeformatierung & Lints
```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
```

---

## 5. Plattform-Support & Build-Anweisungen

### macOS (Apple Silicon & Intel)
- **Status:** **Verifiziert** (Getestet auf Apple Silicon macOS)
- **Build:**
  ```bash
  cargo build --release -p m5fxx-desktop
  ```

### Linux (Ubuntu, Debian, Fedora, Arch)
- **Status:** **Quellcode-kompatibel** (nutzt Standard-winit/egui über Wayland/X11)
- **Benötigte Pakete (Ubuntu/Debian):**
  ```bash
  sudo apt-get update
  sudo apt-get install -y libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
  ```
- **Build:**
  ```bash
  cargo build --release -p m5fxx-desktop
  ```

### Windows (x86_64)
- **Status:** **Quellcode-kompatibel** (reine Rust-Standardbibliothek und native Windows-APIs)
- **Voraussetzung:** Visual Studio C++ Build Tools & Rust `x86_64-pc-windows-msvc`
- **Build:**
  ```powershell
  cargo build --release -p m5fxx-desktop
  ```

---

## 6. Integration eigener Firmware

### Fall A: Firmware in Rust
Wenn Ihre Firmware bereits in Rust geschrieben ist oder als Rust-Projekt aufgebaut wird:
1. Erstellen Sie eine Crate, die `m5fxx-core` als Abhängigkeit einbindet.
2. Nutzen Sie `CardputerHal` für Display- und Eingabezugriffe.
3. Auf dem Desktop übergeben Sie die Instanz an den Simulator (`m5fxx-desktop`).
4. Auf der echten Hardware implementieren Sie denselben Trait mit `esp-idf-hal` oder `esp-hal`.

### Fall B: Firmware in C oder C++
Wenn Ihre Firmware in C oder C++ (z. B. Arduino/ESP-IDF) vorliegt:
1. Nutzen Sie die schmale C-ABI-Headerdatei [`include/m5fxx_abi.h`](include/m5fxx_abi.h).
2. Kompilieren Sie Ihre firmware-eigene Geschäftslogik (Menüs, Parser, Grafiken) als statische Bibliothek (`.a`) für den Host.
3. Die Schnittstelle übergibt den Zeiger auf den 240×135 Framebuffer (`get_framebuffer()`) und empfängt Tastatur-Events (`m5fxx_key_event_t`).
4. Auf diese Weise bleibt Ihre bestehende C++-Firmware erhalten, ohne unvollständige Nachbauten komplexer Arduino-Libraries zu erfordern.
