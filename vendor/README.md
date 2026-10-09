# Eingebundene Originalquellen

Die exakten Upstream-Commits stehen in `versions.json`. Die ADV-Werksfirmware stammt von M5Stack; Grafik, Fonts und UI-Abhängigkeiten werden aus den jeweiligen Originalquellen kompiliert. Copyright- und Lizenzhinweise in den Quellen bleiben erhalten. Insbesondere tragen der Tastatur-Code GPL-Hinweise und TinyGPS++ LGPL-Hinweise; die gesamte eingebundene Sammlung ist nicht pauschal MIT-lizenziert.

Die Sammlung enthält nur benötigte Quellen. Beispiele, Dokumentationsbilder, ungenutzte Hardwaretreiber und ungenutzte Fonttabellen wurden entfernt. Die originale CN16-Fonttabelle bleibt erhalten.

Port-Anpassungen: virtuelle Uhr statt Host-Systemzeit, deterministische Zufallsinitialisierung, portable Integer-Konvertierung sowie die native Plattformauswahl in M5GFX. Die Hardware- und Dateizugriffadapter liegen unter `crates/m5fxx-factory/native`. Die originale Anwendungslogik und Grafikressourcen bleiben Grundlage der UI.
