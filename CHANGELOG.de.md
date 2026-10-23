# Änderungsprotokoll

[English](CHANGELOG.md) · [日本語](CHANGELOG.ja.md) · **Deutsch**

Alle wichtigen Änderungen an Happy-Kitten stehen hier. Das Format folgt [Keep a Changelog](https://keepachangelog.com/de/1.1.0/), die Versionen folgen [Semantic Versioning](https://semver.org/lang/de/).

## [1.1.0] - 2026-10-23

### Neu
- **Deutlicher „Gespeichert“-Hinweis.** Nach dem Speichern eines Projekts (oder eines Standbilds) zeigt ein grüner Hinweis „Gespeichert ✓“ zusammen mit dem vollständigen Dateipfad für ein paar Sekunden. Auch erneutes Speichern mit ⌘S gibt jetzt immer eine Rückmeldung.
- **Klang-Einstellungen** im Tab Audio („Klang“) für die gewählten Audio- oder Video-Clips:
  - Lautstärke von 0 bis 200 %,
  - Einblenden und Ausblenden mit weicher Kurve,
  - ein Effekt mit Stärke-Regler: **Echo**, **Hall**, **Dumpf** und **Telefon**,
  - mehrere Clips lassen sich gemeinsam ändern, ein Knopf setzt alles zurück.
  - Die Effekte werden pro Sample ohne Zustand berechnet, deshalb klingen Vorschau, Wiedergabe und Export genau gleich. Echo und Hall klingen nach dem Ende des Clips noch aus.
- **Tab Farbe** mit **Temperatur** (kalt bis warm) und **Farbton** (grün bis magenta) für die gewählten Video-Clips und Bilder. Die Korrektur läuft auf der GPU, ist live in der Vorschau zu sehen und steckt im Export. Die Helligkeit bleibt erhalten, und sie funktioniert mit jeder Blende (jede Seite einer Blende behält ihre eigene Farbe).
- Neues Symbol für den Tab Farbe, Übersetzungen (Englisch, Japanisch, Koreanisch, Spanisch) für alle neuen Texte.
- Entwickler-Hilfen: `HK_BANNER`, `HK_FX` / `HK_VOLUME` / `HK_FADE` / `HK_TEMP` / `HK_TINT`, `HK_SELECT_PICTURE`, `HK_TAB=color` (siehe README).

### Geändert
- Der Pool hat jetzt sechs Tabs (Dateien, Text, Grafik, Blenden, Farbe, Audio); lange Beschriftungen werden kleiner, damit sie passen.
- Beim Teilen eines Clips bleibt das Einblenden bei der ersten Hälfte und das Ausblenden geht an die zweite. Lautstärke und Farbkorrektur bleiben auf beiden Hälften.
- README, japanisches und deutsches README beschreiben die neuen Funktionen. Die Version ist jetzt 1.1.

### Kompatibilität
- Mit 1.0 gespeicherte Projekte öffnen sich in 1.1 unverändert (Klang und Farbe sind neutral). Mit 1.1 gespeicherte Projekte enthalten pro Clip die neuen Felder `audio` und `grade`; Version 1.0 ignoriert sie.

### Behoben
- Der Knopf „gerade“ im Grafik-Editor ist jetzt übersetzt.

## [1.0.0] - 2026-10-22

Erste öffentliche Version: lokaler, offline arbeitender Videoschnitt für macOS in Rust mit wgpu/Metal, Pfoten-Blenden und 13 klassischen Blenden, Text-Clips mit Effekten, Untertitel und Lyrics (SRT/LRC/TXT), mehrere Video- und Audiospuren, Grafiken mit Transparenz, 9:16-Export, Exportoptionen (Bildrate, Bitrate, Audio-Samplerate) und eine Oberfläche auf Deutsch, Englisch, Japanisch, Koreanisch und Spanisch.

[1.1.0]: https://github.com/Gottpot/Happy-Kitten/releases/tag/v1.1.0
[1.0.0]: https://github.com/Gottpot/Happy-Kitten/releases/tag/V1.0.0
