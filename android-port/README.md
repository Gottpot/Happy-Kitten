# Happy-Kitten für Android (Handy und Tablet)

Eigener Cargo-Workspace im Unterordner. Der Mac-Workspace im Hauptordner wird nicht angefasst.

| Teil | Woher |
| --- | --- |
| `hk-core`, `hk-gpu`, `hk-audio` | unverändert aus `../crates` |
| `app/` | Kopie der UI (`apps/happy-kitten`) plus Android-Teile: `lib.rs` (Einstieg), `platform.rs` (Ordner, Sprache, Ränder, Handy/Tablet), `files.rs` (JNI: Dateiauswahl, Galerie), `phone.rs` (Handy-Layout) |
| `hk-media/` | Kopie von `crates/hk-media`; Video über NDK **MediaExtractor/MediaCodec/MediaMuxer** (`ndk.rs` = aller `unsafe`-Code, `decode.rs`, `encode.rs`) |
| `hk-touch/` | Gestenverteiler (Wischen scrollt, langes Drücken greift), mit Tests am Mac: `cargo test -p hk-touch` |
| `hk-text/` | Kopie von `crates/hk-text` mit Schriften aus `/system/fonts` |
| `java/` | `MainActivity` (NativeActivity): System-Dateiauswahl, Speichern in die Galerie, Fensterränder |

## Bauen und starten

Voraussetzung: Android-SDK mit NDK 27 und Build-Tools 36 (Pfade in `scripts/env.sh`), `rustup target add aarch64-linux-android`.
Ohne Gradle: cargo → javac → d8 → aapt2 → zipalign → apksigner.

```bash
android-port/scripts/build.sh debug install     # baut, installiert, startet (Gerät oder Emulator per adb)
android-port/scripts/build.sh release           # APK nach android-port/build/happy-kitten.apk
```

Auf dem Gerät: Entwickleroptionen → USB-Debugging, dann `adb devices` muss das Gerät zeigen.

## Layout

* **Tablet** (kürzere Seite ≥ 720 pt): wie am Mac (Pool links, Player rechts oben, Timeline unten).
* **Handy**: Kopfzeile mit Export und Menü (⋯: Neu, Öffnen, Speichern, Standbild, Bildformat, Sprache, Über), darunter der Player, die Timeline und ganz unten der Asset-Bereich. Die Werkzeug-Tabs (Dateien, Text, Grafik, Blenden, Effekte, Farbe, Audio) und die Datei-Optionen (Dieses Projekt/Global, Dateien importieren, Suche, Filter) liegen hinter dem Kätzchen oben links. Im Asset-Bereich bleibt nur eine schmale Leiste mit dem Werkzeugnamen, der Dateizahl und dem Ansichtswechsel (Kacheln, Liste, Tabelle). Die Trennlinien zwischen Player, Timeline und Assets lassen sich ziehen; die Höhen werden in `layout.txt` im App-Ordner gemerkt (`phone.rs`). Ein Tippen auf eine Dateikarte hängt das Medium an die Timeline an.

**Gesten (Handy und Tablet):** Auf den Spuren der Timeline und auf den Dateikarten scrollt ein Wischen die Liste, auch wenn der Finger auf einem Clip oder einer Karte beginnt (mit Auslaufen nach dem Loslassen). Erst nach etwa 0,4 s Halten (kurzes Rütteln und ein Ring) greift der Clip oder die Karte: dann lässt er sich verschieben, trimmen oder auf die Timeline ziehen; hält man noch länger still, erscheint das Menü (Teilen, Löschen). Lineal, Zeiger, Regler und Tasten reagieren sofort wie bisher. Die Zonen meldet die UI mit `hk_touch::zone(...)` an.

## Wie Android sich vom Mac unterscheidet

* Dateien kommen über die System-Dateiauswahl und werden in den App-Ordner kopiert (`files/medien/…`). Projekte liegen in `files/projekte/`.
* Export: erst nach `files/export/`, dann in die Galerie (`Movies/Happy-Kitten`, Standbilder `Pictures/Happy-Kitten`). Kein „Speichern unter“-Dialog.
* Ordner-Import gibt es nicht.
* Mindestens Android 10 (API 29), nur arm64. Kein Netzwerk, keine Berechtigungen.

## Stand und bekannte Lücken

Getestet im Emulator (Pixel 6, Android 14, nur Software-Vulkan): Start, Handy-Layout, Import eines H.264-Videos, Vorschau, Timeline, 1080p-Export mit Ton (Datei geprüft). Noch nicht getestet: echtes Gerät, Tablet-Layout, HEVC, hochkant gedrehte Videos, Audio-Wiedergabe in der Vorschau, Projekt öffnen/speichern über die Oberfläche.

* Die Bildschirmtastatur kann über NativeActivity nur einfache Tasten liefern (kein Japanisch-Eingabemodus in Textfeldern).
* Die Videodecodierung liest die Ausgabe als Byte-Puffer (NV12/I420). Geräte mit herstellerspezifischem Kachelformat brauchen später einen zweiten Weg (ImageReader).
* Neue UI-Texte stehen in `app/src/i18n_data.rs`; Änderungen am Mac-Code müssen bei Bedarf von Hand herübergezogen werden (Kopie, keine gemeinsame Bibliothek).
* Geräteprüfungen von `hk-media` ohne App: `cargo test --target aarch64-linux-android -p hk-media --no-run`, Testdatei per `adb push`, dann mit `HK_TEST_VIDEO=… --ignored` bzw. `HK_TEST_OUT=… --ignored` ausführen.
