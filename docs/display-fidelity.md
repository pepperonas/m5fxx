# Display-Fidelity und ADV Factory

## Referenz

Der [M5Stack Cardputer ADV](https://docs.m5stack.com/en/core/Cardputer-Adv) hat ein 1,14-Zoll-IPS-Display mit 240 × 135 sichtbaren Pixeln und ST7789V2-Controller. Der Firmware-Pfad nutzt RGB565. Die M5GFX-Konfiguration verwendet ein 135 × 240-Panel, Offsets 52/40, Rotation 1, Inversion und 40 MHz SPI-Schreibtakt.

Die Anwendungen stammen aus dem [offiziellen CardputerADV-Zweig](https://github.com/m5stack/M5Cardputer-UserDemo/tree/CardputerADV), Commit `b549eac0a3c65bc108186c276b8fac0a214aaa4e`. M5GFX zeichnet die originalen Fonts, Icons, Launcher, Statusleisten und 13 Anwendungen. Native Adapter ersetzen ESP-IDF/Arduino-Hardwarezugriffe.

## Darstellung

Der Rust-Controller hält 240 × 320 Pixel RAM und unterstützt Adressfenster, RGB565/RGB666-Schreibdaten, MADCTL-Rotation/Farbfolge, Display an/aus, Schlaf und Inversion. Der native Firmware-Framebuffer wird über den Landscape-RGB565-Pfad übertragen. Dies ist kein elektrisches SPI-Modell und keine vollständige Emulation sämtlicher ST7789-Register.

Die Zoomansicht zeigt jedes logische Pixel als ganzzahligen Block physischer Monitorpixel; Nearest-Neighbor verhindert Filterunschärfe. Informationen stehen außerhalb des LCD. In der Geräteansicht wird das Display mit dem Gehäuse skaliert.

Die 25 × 15 mm große Öffnung ist aus der [offiziellen ADV-CAD-Datei](https://raw.githubusercontent.com/m5stack/M5_Hardware/master/Products/K132-Adv_Cardputer-Adv/Structures/Cardputer-Adv.stl) abgeleitet. Die aktive 16:9-Fläche wird darin zentriert. Der schmale schwarze Rahmen ist eine visuelle Annäherung: Die gesamte obere CAD-Fläche enthält auch Gehäuseteile und darf nicht als schwarzes Displayglas gezeichnet werden.

Die physische aktive Fläche, Panel-Gamma, Blickwinkel, Reflexionen und reale Hintergrundbeleuchtung sind nicht am Gerät vermessen. Eine optisch vermessene 1:1-Kopie wird daher nicht behauptet.

## Simulierte Hardware

Zeit, Batterie, Funkzustände, IMU und Audioeingaben sind kontrollierte Simulationen. Es gibt keine echte Host-Funkverbindung oder Mikrofonaufnahme. SD-Dateizugriffe bleiben innerhalb des gewählten lokalen Verzeichnisses. G0 und Matrix-Tasten werden an die originale Firmware weitergereicht. Diese Angaben betreffen den nativen ADV-Port. Das separate Profil **ESP32 Binary** führt zusammengeführte ESP32-S3-Images in QEMU aus und speist echte Firmware-SPI-Daten in denselben Displaycontroller; siehe [ESP32-Emulation](esp32-emulation.md). Seine Peripheriegrenzen unterscheiden sich vom nativen Port.

## Prüfung

`cargo test --workspace` prüft Controller-Pixel, Tasteneingaben, SD-Sandbox, HiDPI-Skalierung und native Firmware. Die PNG-Fixtures in `crates/m5fxx-factory/tests/frames` vergleichen Boot, Launcher und alle 13 Apps pixelgenau. Sie sind deterministische Aufnahmen des nativen Ports, keine Aufnahmen realer Hardware.

Neue Referenzen lassen sich mit `cargo run -p m5fxx-factory --example capture` unter `target/factory-captures` erzeugen. Referenzen nur nach Prüfung einer beabsichtigten Darstellungsänderung ersetzen.
