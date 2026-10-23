# 🐾 Happy-Kitten v1.1

[English](README.md) · [日本語](README.ja.md) · **Deutsch**

**Von Gottpot** · quelloffen unter MIT oder Apache-2.0

Ein lokaler, blitzschneller Video-Schnitt für den Mac, geschrieben in Rust. Die Bildverarbeitung läuft über **wgpu/Metal** auf der GPU, die Oberfläche in zuckersüßem Pastellrosa statt Adobe-Grau.
Alles passiert **offline**: keine Cloud, kein Konto, keine Telemetrie, kein Abo.

📺 YouTube: [@Gottpot](https://www.youtube.com/@Gottpot)

![Logo](assets/logo.png)

## Bilder

![Der Editor mitten in einer Pfoten-Blende](docs/screenshots/main.png)

| | |
|---|---|
| ![Text und Effekte](docs/screenshots/text.png) | ![Blenden](docs/screenshots/transitions.png) |
| ![Export](docs/screenshots/export.png) | ![Japanische Oberfläche](docs/screenshots/japanese.png) |

## Download und erster Start

Die fertige App liegt unter [Releases](../../releases) als `Happy-Kitten-….zip` (Apple-Silicon-Mac, macOS 13 oder neuer): entpacken und in den Ordner *Programme* ziehen. Ich bin **kein von Apple zertifizierter Entwickler**, die App ist deshalb nicht notarisiert. macOS kann den Start mit „ist beschädigt“ oder „Entwickler kann nicht überprüft werden“ ablehnen. Dann hilft eines von beiden:

1. Rechtsklick auf die App → *Öffnen*. Ab macOS 15 einmal versuchen zu öffnen und danach unter *Systemeinstellungen → Datenschutz & Sicherheit* auf *Trotzdem öffnen* klicken.
2. Im Terminal das Quarantäne-Kennzeichen entfernen, das beim Herunterladen angehängt wird:

```bash
xattr -dr com.apple.quarantine /Applications/Happy-Kitten.app
```

Wer die App selbst baut (siehe unten), bekommt die Warnung nie. Der Quelltext ist offen und enthält keinen Netzwerkcode.

## Starten

```bash
# einmalig: Rust installieren (https://rustup.rs), dann
cargo run --release -p happy-kitten
# oder als Programm-Paket:
./scripts/bundle.sh      # erzeugt dist/Happy-Kitten.app
```

Beim ersten Start liegt ein kleines Demo-Projekt auf der Timeline (drei Szenen, zwei Pfoten-Blenden). Die vier Beispielszenen sind im Media Pool immer vorhanden, es braucht also keine eigenen Dateien zum Ausprobieren.

## Bedienung

| Aktion | So geht's |
|---|---|
| Medien importieren | „Import“ (⌘I) oder „Dateien“ im Pool: **mehrere Dateien auf einmal** wählen; „Ordner“ holt alle Videos, Bilder, Audios und Untertitel eines Ordners; oder einfach ins Fenster ziehen |
| Zwei Arten von Import | Oben im Pool: **Dieses Projekt** (Dateien gehören nur zu diesem Projekt) oder **Global** (Bibliothek für alle Projekte, gespeichert in `~/Library/Application Support/Happy-Kitten/library.json`; die Dateien bleiben, wo sie sind). Globale Clips lassen sich wie normale auf die Timeline ziehen, Rechtsklick: ins Projekt übernehmen / entfernen |
| Text einfügen | Tab **Text**: Vorlage auf eine Spur ziehen oder ＋ anklicken. Den Text-Clip wählen, dann links Text, Schrift, Größe, Farbe, Kontur, Hintergrund, Schatten, Ausrichtung, Position und Ein-/Ausblenden ändern |
| Untertitel und Lyrics | Tab **Text** → „SRT · LRC · TXT“ (oder Datei ins Fenster ziehen): jede Zeile wird ein Text-Clip auf einer neuen Spur. „Stil für alle Texte der Spur“ und „Zeilen bunt färben“ ändern alle auf einmal |
| Grafiken über dem Video | Tab **Grafik** → „Grafik (PNG)“, oder ein PNG mit Transparenz einfach auf eine Videospur ziehen (Bilder mit durchsichtigen Teilen werden automatisch zur Grafik; Rechtsklick „Als Grafik über das Video legen“ geht bei jedem Bild). Den Clip wählen, dann **Größe, Position, Drehung, Deckkraft** und Ein-/Ausblenden (Überblenden, Verpixeln, Verwirbeln, Zoom, …) einstellen. Mehrere Grafiken gemeinsam ändern geht wie bei Text |
| Stil-Vorlage auf mehrere Texte | Texte markieren, dann im Tab **Text** oben eine Vorlage wählen und „Auf N Texte anwenden“, oder den **goldenen Haken** auf einer Vorlagenkarte, oder **Rechtsklick** auf die Karte bzw. auf einen Text-Clip in der Timeline („Vorlage anwenden“). Wortlaut und (ohne Haken „Position mitnehmen“) Position bleiben |
| Mehr Spuren | „Videospur“ / „Audiospur“ in der Timeline (bis 8 je Art). Obere Videospuren liegen über unteren, Text liegt über dem Bild. Rechtsklick auf den Spurkopf löscht leere Spuren |
| Auf die Timeline legen | Karte auf eine Spur ziehen, doppelklicken oder auf das ＋ klicken |
| Verschieben | Clip ziehen (rastet an Kanten und am Zeiger ein, kann nie überlappen) |
| Mehrere Clips markieren | **Auf freier Fläche der Spuren ziehen** (Auswahlrahmen), oder ⇧/⌘-Klick, oder ⌘A. Ziehen verschiebt alle gemeinsam, Entf löscht alle, ⇧Entf schließt die Lücken |
| Mehrere Texte einheitlich ändern | Texte markieren, im Tab **Text** ändern: jede Änderung gilt für alle gewählten Texte (nur der Wortlaut bleibt je Text) |
| Blende an vielen Schnittpunkten | Clips markieren, im Tab **Blenden** eine Blende anklicken: sie landet an jedem Schnittpunkt zwischen zwei gewählten Clips |
| Dauer einer Blende | Im Tab Blenden „Eigene Dauer“ für neue Blenden, an der Blende in der Timeline die Ränder des rosa Bandes ziehen, oder die Blende anklicken und im Feld über der Timeline einstellen |
| Bildformat | Über der Vorschau: **16:9**, **9:16** (Shorts, Reels, TikTok), 1:1, 4:5. „Einpassen“, „Füllen“ (zuschneiden) oder „Unscharf füllen“ legt fest, was mit Bildern anderer Form passiert |
| Kürzen / Trimmen | Ränder des Clips ziehen |
| Teilen | Zeiger setzen, **S** (oder ⌘B) – teilt den gewählten Clip, sonst alle unter dem Zeiger |
| Löschen | **Entf**; mit **⇧** schließt sich die Lücke (oder Rechtsklick) |
| Blende einfügen | Im Tab „Blenden“ eine Blende zwischen zwei Clips ziehen; Dauer und Art im Feld über der Timeline ändern |
| Abspielen | **Leertaste**; ←/→ springen ein Bild, Pos1/Ende zum Anfang/Ende |
| Rückgängig / Wiederholen | ⌘Z / ⇧⌘Z |
| Klang einstellen | Audio- oder Video-Clip wählen, Tab **Audio** → „Klang“: **Lautstärke** (0 bis 200 %), **Einblenden**, **Ausblenden** und ein **Effekt**: Echo, Hall, Dumpf oder Telefon, mit Stärke-Regler. Mehrere Clips lassen sich gemeinsam ändern. Vorschau, Wiedergabe und Export klingen gleich |
| Farbe anpassen | Video-Clip oder Bild wählen, Tab **Farbe**: **Temperatur** (kalt bis warm) und **Farbton** (grün bis magenta). Sofort in der Vorschau und im Export, mehrere Clips gemeinsam änderbar |
| Speichern / Öffnen | ⌘S / ⌘O (`.kitten`-Dateien, reines JSON). Nach dem Speichern zeigt ein grüner Hinweis „Gespeichert ✓“ und den Dateipfad für ein paar Sekunden |
| Film exportieren | „Exportieren“ → MP4 (H.264 oder HEVC) mit Ton. Format wie das Projekt oder 16:9, **9:16 Short**, 1:1, 4:5; Auflösung = kurze Seite (720 / 1080 / 2160 Pixel); **Bildrate** (23,976 · 24 · 25 · 29,97 · 30 · 50 · 59,94 · 60, zum Beispiel 24 für Midjourney-Material); **Bitrate** automatisch oder selbst festgelegt (1 bis 150 Mbit/s); **Audio** 32 / 44,1 / 48 kHz (AAC) und 128 bis 320 kbit/s |
| Standbild | „Standbild“ speichert das aktuelle Vorschaubild als PNG |
| Absturzschutz | ungespeicherte Arbeit wird alle 30 s zur Seite kopiert (`~/Library/Application Support/Happy-Kitten`) und nach einem Absturz beim Start zurückgeholt; vor „Neu“, „Öffnen“ und dem Schließen wird gefragt |

## Dateiliste

Im Tab **Dateien** (und Audio) gibt es Suche, Filter (Art, verwendet / noch frei) und Sortierung. Drei Ansichten: **Kacheln**, **Liste** (ganzer Dateiname) und **Tabelle** (Name, Art, Dauer, Größe, Ordner; Klick auf die Spaltenköpfe sortiert). Alles, was schon auf der Timeline liegt, trägt in jeder Ansicht einen **goldenen Haken** mit goldener Umrandung. Ein Tooltip zeigt den vollen Pfad.

## Schrift, Japanisch und andere Sprachen

Die Schriftliste zeigt jede Schrift in ihrer eigenen Gestalt. Fehlt einer Schrift ein Zeichen (Japanisch, Chinesisch, Koreanisch), springt automatisch eine passende Mac-Schrift ein (Hiragino, Apple SD Gothic); japanische Texte ohne Leerzeichen werden zwischen den Zeichen umbrochen. Im Hochformat richtet sich die Textgröße nach der Bildbreite.

## Sprachen

Oben rechts sitzt der Sprachknopf (Weltkugel mit Länderkürzel): **DE · EN · JP · KR · ES**. Beim ersten Start nimmt Happy-Kitten die Sprache von macOS (sonst Englisch), die Wahl wird in `settings.json` im Programmordner gemerkt. Beim Entwickeln erzwingt `HK_LANG=ja` eine Sprache.

Übersetzt wird mit dem deutschen Text als Schlüssel: `tr("Speichern")`, mit Platzhaltern `trf!("{n} Clips ausgewählt", n = 3)`. Alle Übersetzungen stehen in `apps/happy-kitten/src/i18n_data.rs` (Spalten: English, 日本語, 한국어, Español); ein Test prüft, dass nichts leer ist und die Platzhalter übereinstimmen. Neuer Text: Schlüssel im Code mit `tr(…)` umgeben, eine Zeile in die Tabelle, fertig. Eine weitere Sprache braucht einen Eintrag in `Lang` und eine Spalte in der Tabelle. Fehlt eine Übersetzung, erscheint der deutsche Text.

## Text mit Effekten

Text wird mit den Schriften des Macs gezeichnet (Umbruch, Kontur, Schatten, Box) und von der GPU animiert. Ein- und Ausblenden: Überblenden, **Verpixeln**, **Verwirbeln**, Weichzeichnen, Hochgleiten, Heranzoomen, Wellen, Glitch.
`cargo run -p hk-gpu --example text_sheet -- text.png` zeigt alle Effekte in fünf Stufen.
Eigene Stile lassen sich als **globale Vorlage** speichern und in jedem Projekt benutzen.

## Die Pfoten-Blenden

Alle Blenden sind WGSL-Shader (`crates/hk-gpu/src/transitions.wgsl`), die Pfoten und Krallen **mathematisch aus Distanzfeldern** berechnen. Es gibt keine vorgerenderten Bilder.

| Blende | Was passiert |
|---|---|
| **Claw Wipe** | sechs geschwungene Kratzer reißen das Bild auf, die Mitte zuerst |
| **Furry Blur** | haarartig verschmierter Fell-Nebel, das Bild wechselt entlang der Fellsträhnen |
| **Paw Spin** | eine Riesenpfote wirbelt aus der Mitte, das neue Bild liegt in ihr |
| **Gentle Paw Fade** | Pfotenabdrücke tapsen diagonal übers Bild und öffnen Fenster zum nächsten Clip |
| **Kitty Dash** | ein galoppierendes Kätzchen rennt durchs Bild, mit Tempolinien und Pfotenspur |
| **Meow Glitch** | zerrissene Zeilen, RGB-Versatz, Blockwechsel und Neon-Katzenköpfe |

Dazu **13 klassische Blenden**: Überblenden, Schwarzblende, Weißblende, Wischen →, Wischen ↓, Schieben, Zoom, Kreis öffnen, Uhrzeiger, Flügeltür, Verpixeln, Verwirbeln, Weichzeichner.

`cargo run -p hk-gpu --example contact_sheet -- blenden.png` rendert alle Blenden in fünf Stufen in ein Bild, `--example single` ein einzelnes Bild in beliebiger Größe.

## Aufbau

```
crates/
  hk-core    Zeit (exakte Ticks), Projektmodell, Schnittoperationen, Undo/Redo, Speichern. Keine GPU, keine UI.
  hk-gpu     wgpu-Compositor und die Blenden-Shader. Rendert in eine Rgba8-Textur (Vorschau, Export, Tests).
  hk-media   Bilder (inkl. Grafiken mit Transparenz), Demo-Szenen, Video über AVFoundation (sequenzielles Hardware-Decoding, MP4-Export per AVAssetWriter).
  hk-text    Text-Rasterizer: Schriften, Zeilenumbruch, Kontur, Schatten, Hintergrundbox.
  hk-audio   Ton-Decoding (symphonia), Waveform, Mixer und Wiedergabe (cpal). Die Audio-Uhr führt das Bild.
apps/
  happy-kitten  egui-Oberfläche (eigenes Pastell-Theme, handgezeichnete Icons), Vorschau-Engine, Export.
```

Entwurfsregeln:

- **Exakte Zeit.** Alles läuft in ganzzahligen Ticks (705 600 000 pro Sekunde, teilbar durch alle üblichen Bild- und Abtastraten), nie in `f64`-Sekunden.
- **Eine Pipeline für Vorschau und Export.** Beide benutzen denselben Compositor, was du siehst, ist was exportiert wird.
- **Der Kern ist testbar ohne Fenster.** `cargo test --workspace`.
- **Nur eine Stelle mit `unsafe`:** die Apple-Framework-Anbindung in `hk-media` (Video lesen/schreiben).

## Entwickler-Hilfen

Umgebungsvariablen für automatische Prüfungen ohne Mausklick:

```bash
HK_LANG=en                   # Sprache erzwingen (de, en, ja, ko, es)
HK_SCREENSHOT=out.png        # speichert das Fenster nach kurzer Zeit und beendet sich
HK_PLAYHEAD=5.0 HK_TAB=effects HK_IMPORT=clip.mp4:song.wav  # Zeiger, Tab, Import vorbelegen
HK_PLAY=1                    # Wiedergabe starten und die Audio-Uhr ausgeben
HK_EXPORT=film.mp4 HK_HEVC=1 # Export in 720p ohne Dialog, dann beenden
HK_GLOBAL=1 HK_IMPORT=ordner # Import in die globale Bibliothek (in Entwicklerläufen wird nichts gespeichert)
HK_TAB=color HK_SELECT_PICTURE=1   # Tab Farbe mit dem ersten Bild-Clip öffnen
HK_FX=echo HK_VOLUME=0.5 HK_FADE=1 HK_TEMP=0.6 HK_TINT=-0.3  # Klang und Farbe auf alle Clips setzen
HK_BANNER=1                  # den grünen „Gespeichert“-Hinweis zeigen
HK_SCRIPT=build|text         # spielt Mausbedienung ab (Ziehen, Trimmen, Blende, Text, Bibliothek); mit HK_DUMP=1 wird die Timeline ausgegeben
```

## Verhältnis zu filmcraft

Happy-Kitten wurde mit [storytold/filmcraft](https://github.com/storytold/filmcraft) (MIT OR Apache-2.0) als Referenz gebaut: wgpu-Compositor in linearer Pipeline, exakte Ganzzahl-Zeit, Befehle statt Spezialfälle, egui als UI. Es ist ein **eigener, schlanker Kern**; es wurde kein Code übernommen. Die Entscheidung dagegen, filmcraft als Abhängigkeit einzubinden: es zielt auf Premiere-Parität (50+ Crates, eigene Codecs), Happy-Kitten bewusst auf wenig Ballast. Die reinen Rust-Codecs von filmcraft (`h264`, `hevc`, `aac`, `isobmff` …) bleiben die naheliegende Option für eine spätere Windows/Linux-Portierung.

## Autor und Lizenz

Erstellt von **Gottpot**. Der Quelltext steht wahlweise unter der [MIT-Lizenz](LICENSE-MIT) oder der [Apache-Lizenz 2.0](LICENSE-APACHE), © 2026 Gottpot. Du darfst ihn nutzen, ändern und weitergeben, auch kommerziell, wenn du den Lizenzhinweis behältst.

- **Logo:** `assets/logo.png` ist © Gottpot und **nicht** Teil der freien Lizenz, siehe [assets/LICENSE.md](assets/LICENSE.md). Wer einen Fork veröffentlicht, ersetzt es durch ein eigenes.
- **Bibliotheken:** Alle Drittbibliotheken sind freie Lizenzen (MIT, Apache-2.0, Zlib, BSD, MPL-2.0, …), aufgelistet in [THIRD-PARTY-LICENSES.md](THIRD-PARTY-LICENSES.md) (erzeugt mit `python3 scripts/third_party_licenses.py`). Schriften der Textclips werden nicht mitgeliefert, sondern zur Laufzeit aus macOS gelesen.
- **Änderungsprotokoll:** Was sich in welcher Version geändert hat, steht in [CHANGELOG.de.md](CHANGELOG.de.md).
- Beiträge sind willkommen, siehe [CONTRIBUTING.md](CONTRIBUTING.md). Das ist ein Hobbyprojekt und keine Rechtsberatung.

## Als Nächstes

- Text frei auf dem Bild verschieben (bisher per Positionsfeld)
- Bild-in-Bild: Größe und Position von Video-Clips
- Geschwindigkeit pro Clip
- Audio-Überblendungen an Schnitten
