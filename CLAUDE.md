# Happy-Kitten – Hinweise für die Arbeit am Code

Lokaler macOS-Videoschnitt in Rust (wgpu/Metal, egui, AVFoundation). Komplett offline, kein Netzwerkcode, keine Telemetrie. Sprache der Oberfläche: Deutsch.

## Befehle
- Starten: `cargo run --release -p happy-kitten` · Paket: `./scripts/bundle.sh`
- Prüfen: `cargo test --workspace` und `cargo clippy --workspace --all-targets`
- Blenden ansehen ohne App: `cargo run -p hk-gpu --example contact_sheet -- out.png`
- Oberfläche prüfen: `HK_SCREENSHOT=out.png target/debug/happy-kitten` (siehe README.de.md „Entwickler-Hilfen“)

## Regeln
- Schichten: `hk-core` kennt weder GPU noch UI noch Dateien außer Projekt-JSON. UI-Code liegt nur in `apps/happy-kitten`.
- Zeit ist immer `hk_core::Tick`, nie `f64`-Sekunden in Schnittlogik.
- Zahlen aus Dateien sind feindlich: kein `unwrap`/Indexzugriff auf Mediendaten, stattdessen `get`, `clamp`, `Result`.
- `unsafe` nur in `crates/hk-media` (Apple-Frameworks), mit `// SAFETY:`-Kommentar.
- Text: `TextSpec` (hk-core) → `hk-text` zeichnet ein frame-großes Bild (premultiplied) → `text_overlay.wgsl` blendet es mit `TextFx` ein/aus. Neuer Effekt: `TextFx` + `case` im Shader + `text_sheet` ansehen.
- Szene: `Project::scene_at` liefert Bild (`video_frame_at`, Text-Clips zählen nie als Bild) plus Text-Ebenen; Vorschau und Export benutzen dieselbe Szene.
- Globale Bibliothek: `GlobalLibrary` (hk-core), Ids mit `LIB_ID_BIT`; Einträge werden beim Benutzen per `ensure_media` ins Projekt übernommen.
- Auswahl: `selection` (primär) + `extra` (weitere); immer `set_selection`/`select_many` benutzen, nie `selection =` direkt. Gruppen bewegen mit `Project::shift_clips` (zustandslos, aus `group_origins`). Während Ziehen hält `Document::begin_gesture` die Blenden (`hold_transitions`), erst `end_gesture` räumt auf.
- Bildformat: `Project.size` + `Project.fit` (Contain/Cover/Blur); Compositor `set_fit`; Export-Größe aus dem Format-Dialog (kurze Seite = Auflösung).
- Grafiken: `Clip.overlay: Option<OverlaySpec>` (Bild mit Größe/Position/Drehung/Deckkraft), Szene hat `layers` (Text und Bild in Spurreihenfolge); `layers.rs` rastert jede Ebene frame-groß (premultiplied) für den GPU-Overlay-Pass, Vorschau und Export teilen das. `MediaItem.alpha` wird beim Import erkannt und macht Bilder zu Grafiken.
- Neue Blende: `TransitionKind` in `hk-core/src/transition.rs`, Funktion + `case` in `hk-gpu/src/transitions.wgsl`, danach `contact_sheet` ansehen.
- UI-Stil: Farben nur aus `theme.rs`, Icons handgezeichnet in `kit.rs`; Texte deutsch.
- Sprachen: deutscher Text ist der Schlüssel. UI-Texte immer `tr("…")`, mit Platzhaltern `trf!("{n} …", n = x)` (nie `format!` mit Übersetzung mischen), Laufzeitmeldungen gehen durch `say()` → `tr_dyn` (Schlüssel mit `:` am Ende übersetzt den Kopf einer Fehlermeldung). Einträge in `i18n_data.rs` (en, ja, ko, es); `cargo test` prüft Vollständigkeit und Platzhalter. In Konstanten steht der deutsche Schlüssel, `tr` wird erst beim Zeichnen aufgerufen. Ansehen: `HK_LANG=ja HK_SCREENSHOT=out.png …`.
- READMEs: `README.md` (Englisch, Startseite auf GitHub), `README.ja.md`, `README.de.md`. Änderungen an Bedienung, Befehlen oder Download-Hinweisen in allen drei nachziehen.
- Klang pro Clip: `Clip.audio: AudioSpec` (Lautstärke, Ein-/Ausblenden, ein `AudioFx`); der Mixer (`hk-audio/src/mix.rs`) rechnet Effekte zustandslos pro Sample (Echo/Hall als Abgriffe, Filter als Mittelwert), damit Blockgrenzen, Vorschau und Export identisch klingen. Neuer Effekt: `AudioFx` + `Voice::sample` + Test „unabhängig von Blockgrenzen“.
- Farbe pro Clip: `Clip.grade: Grade` (Temperatur, Farbton) → `SourceRef.grade` → `Compositor::set_grades` (vor jedem Render aufrufen) → Verstärkung in `transitions.wgsl` (`sa`/`sb`).
- Änderungsprotokoll: `CHANGELOG.md` (Englisch), `.ja.md`, `.de.md` bei jeder Änderung nachführen; Version in `Cargo.toml`.
- Blenden-Shader: nie `pow(x, 2.0)` (für negatives x undefiniert), sondern `sq(x)`; große skalierte Koordinaten vor Potenzen absichern (Überlauf). Jede neue Blende kommt in `TransitionKind` (`name`, `blurb`, `shader_id` lückenlos, Gruppe, ggf. `card_progress`), in den Shader-`switch`, in `i18n_data.rs` und wird mit `contact_sheet` angesehen; der GPU-Test „beginnt auf A, endet auf B“ muss grün bleiben.
- Neues Projekt ist leer (`Project::default()`); die Beispielszenen gibt es nur noch mit `HK_DEMO=1` und für die Effektkarten.
- Overlay-Effekte: `Clip.fx: Option<FxSpec>` (hk-core `fx.rs`; Clip ohne Medium, zählt als Ebene wie Text/Grafik) → `Scene.layers` enthält `Layer::Effect` (mit Fade-`alpha`, Zeit, Seed) → `Compositor::overlay_fx` (Shader `fx.wgsl`, liest eine Kopie des bisherigen Bildes). Neuer Effekt: `FxKind` (`name`, `blurb`, `shader_id` lückenlos ab 1, Gruppe, `params`), Funktion + `case` in `fx.wgsl`, `i18n_data.rs`, mit `cargo run -p hk-gpu --example fx_sheet -- out.png` ansehen; der GPU-Test „zeichnet, verschwindet bei Alpha 0“ muss grün bleiben.
- Ton für Effekte: `hk_audio::analyze(plan, pcm, t)` → `AudioLevels` (32 Bänder, Bass/Mitten/Höhen, 8 Beat-Werte im 50-ms-Abstand), ohne Vorgeschichte, damit Vorschau und Export gleich sind. Vorschau rechnet nur, wenn `Project::needs_levels_at` wahr ist; der Export pro Bild. Equalizer-Effekte und `beat > 0` brauchen den Ton (`FxSpec::needs_audio`).
- Dev: `HK_TAB=fx HK_ADD_FX=EqBars` (Effekt am Zeiger einfügen, Zeiger 1,2 s danach), `HK_DESELECT=1`.
