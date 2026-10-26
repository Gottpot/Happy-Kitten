# Änderungsprotokoll

[English](CHANGELOG.md) · [日本語](CHANGELOG.ja.md) · **Deutsch**

Alle wichtigen Änderungen an Happy-Kitten stehen hier. Das Format folgt [Keep a Changelog](https://keepachangelog.com/de/1.1.0/), die Versionen folgen [Semantic Versioning](https://semver.org/lang/de/).

## [1.3.0] - 2026-10-26

### Neu
- **Neuer Tab Effekte** mit 13 Overlay-Effekten, die die GPU komplett aus Mathematik über das darunterliegende Bild zeichnet:
  - *Umrandungen:* Feuer-Rahmen, Eis-Rahmen, Blitz-Rahmen, Neon-Rahmen,
  - *Equalizer:* Balken-Equalizer, Kreis-Equalizer, Wellen-Equalizer, Beat-Ringe,
  - *Glitch und Cyberpunk:* Glitch, Cyber-Raster, Laser-Show,
  - *Licht:* Linsenreflex, Lichtleck.
- Ein Effekt ist ein Clip: auf eine Videospur ziehen oder anklicken, dann liegt er am Zeiger auf einer freien Videospur (sonst auf einer neuen). Die Länge stellst du an den Rändern in der Timeline ein, er lässt sich wie jeder Clip verschieben, teilen und löschen. Mehrere Effekte können übereinanderliegen.
- Einstellungen pro Effekt (soweit sinnvoll): Breite, Höhe oder Größe, Geschwindigkeit, Stärke, Farbe (Farbverschiebung), Beat-Reaktion, Position, Spiegeln, Ein- und Ausblenden. Mehrere Effekte lassen sich gemeinsam ändern, ein Knopf setzt alles zurück.
- **Die Equalizer und die Beat-Reaktion folgen der Musik.** Der Ton der Timeline wird für den jeweiligen Moment analysiert (FFT, 32 Bänder von 40 Hz bis 14 kHz, ein Beat-Erkenner auf dem Bass). Die Analyse braucht keine Vorgeschichte, deshalb zeigen Vorschau, Wiedergabe und Export genau dasselbe.
- Die Karten im Tab sind lebendig: Mit der Maus darüber siehst du den Effekt laufen.
- Tests: Audio-Analyse (richtiges Band für einen Ton, Beat nur wenn der Bass springt), Effekt-Clips (Ein-/Ausblenden, Teilen, Spuren) und ein GPU-Test, dass jeder Effekt zeichnet, bei 0 % verschwindet und die Equalizer auf den Ton reagieren. Das Beispiel `fx_sheet` rendert alle Effekte in ein Bild.
- Übersetzungen (Englisch, Japanisch, Koreanisch, Spanisch) für alles Neue.

### Kompatibilität
- Projekte mit einem Effekt-Clip lassen sich mit Version 1.2 oder älter nicht öffnen, weil diese Versionen Effekt-Clips nicht kennen. Projekte aus 1.0 bis 1.2 öffnen sich in 1.3 unverändert.

## [1.2.0] - 2026-10-24

### Neu
- **25 neue Blenden** in fünf Gruppen im Tab Blenden, alle auf der GPU berechnet:
  - *Rauschen und Zellen:* Papierbrand, Scherben, Tinte im Wasser,
  - *Wellen und Optik:* Wassertropfen, Fischauge, Zoom-Unschärfe, Dreh-Unschärfe, Farbsaum, Lichtleck,
  - *Kurven und Formen:* Sakura, Herz öffnen, Spirale, Pfotenblüte, Waben, Kaleidoskop,
  - *Manga-Look:* Rasterpunkte, Retro-Raster, Tigerstreifen, Glückskatze,
  - *3D und Physik:* Seite umblättern, Würfel, Umklappen, Schmelzen, Droste-Zoom, Fraktal.
- Ein GPU-Test prüft, dass **jede** Blende auf dem alten Bild beginnt und auf dem neuen endet, ohne Reste ihres eigenen Effekts.
- `contact_sheet` kann einen Ausschnitt der Blenden rendern (`… out.png 19 6`), `HK_DEMO=1` startet mit den Beispielszenen.
- Übersetzungen (Englisch, Japanisch, Koreanisch, Spanisch) für alle neuen Namen und Beschreibungen, mit einem Test, dass kein Blendentext unübersetzt bleibt.

### Geändert
- **Ein neues Projekt ist leer.** Die vier Beispielszenen (Sonnenwiese, Wollknäuel, Abendkatze, Minzgarten) werden nicht mehr hinzugefügt, du kannst also sofort mit eigenen Dateien beginnen. Projekte, die sie schon enthalten, öffnen sich unverändert.
- Die Bildkarten im Tab Blenden zeigen jede Blende bei einem Fortschritt, bei dem man sie gut erkennt (zum Beispiel das Umklappen, das bei 50 % genau von der Kante zu sehen ist).

### Behoben
- Ränder und Leuchten mehrerer Shader (Claw Wipe, Gentle Paw Fade, Kitty Dash) benutzten `pow(x, 2.0)`, das für negatives `x` undefiniert ist. Jetzt steht dort `x * x`, der Rand zeigt sich auf beiden Seiten der Kante.
- Claw Wipe springt nicht mehr im allerersten Bild herein.

### Kompatibilität
- Projekte, die eine der neuen Blenden benutzen, lassen sich mit Version 1.1 oder älter nicht öffnen, weil diese Versionen die Blende nicht kennen. Sonst ändert sich nichts; Projekte aus 1.0 und 1.1 öffnen sich in 1.2.

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

[1.3.0]: https://github.com/Gottpot/Happy-Kitten/releases/tag/v1.3.0
[1.2.0]: https://github.com/Gottpot/Happy-Kitten/releases/tag/v1.2.0
[1.1.0]: https://github.com/Gottpot/Happy-Kitten/releases/tag/v1.1.0
[1.0.0]: https://github.com/Gottpot/Happy-Kitten/releases/tag/V1.0.0
