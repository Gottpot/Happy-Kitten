#!/usr/bin/env python3
"""Writes THIRD-PARTY-LICENSES.md from `cargo metadata` (only what is built on macOS arm64).
Run from the project root:  python3 scripts/third_party_licenses.py"""
import json, subprocess

meta = json.loads(subprocess.run(
    ["cargo", "metadata", "--format-version", "1", "--filter-platform", "aarch64-apple-darwin"],
    check=True, capture_output=True, text=True).stdout)
workspace = set(meta["workspace_members"])
packages = {p["id"]: p for p in meta["packages"]}
used = [packages[n["id"]] for n in meta["resolve"]["nodes"] if n["id"] not in workspace]
used.sort(key=lambda p: (p["name"], p["version"]))

NOTES = """\
## Hinweise

- **MPL-2.0 (symphonia):** Datei-weises Copyleft. Happy-Kitten nutzt die Bibliothek unverändert als
  Abhängigkeit; ihr Quelltext ist unter https://github.com/pdeljanov/Symphonia und auf crates.io
  verfügbar. Änderungen an *deren* Dateien müssten wieder unter MPL-2.0 veröffentlicht werden.
- **`self_cell` (Apache-2.0 OR GPL-2.0-only):** Wir wählen Apache-2.0.
- **Eingebaute Schriften von egui (`epaint_default_fonts`):** SIL Open Font License 1.1 und Ubuntu Font
  Licence 1.0 (Ubuntu-Light, Hack, Noto Emoji u. a.). Sie stecken im Programm und dürfen mit diesen
  Lizenzhinweisen weitergegeben werden.
- **Schriften der Textclips:** Arial Rounded, Comic Sans, Georgia, Hiragino u. a. werden *nicht*
  mitgeliefert, sondern zur Laufzeit aus dem macOS-Schriftordner gelesen.
- **Apple-Frameworks** (AVFoundation, Metal, CoreAudio …) sind Teil von macOS und werden nicht mitgeliefert.
- Wer ein fertiges Programm (`.app`) weitergibt, legt die Lizenztexte der unten genannten Bibliotheken bei
  (sie liegen in deren Quellpaketen auf crates.io).

"""

lines = ["# Lizenzen von Drittbibliotheken", "",
         f"Happy-Kitten selbst: MIT oder Apache-2.0, © 2026 Gottpot. Diese {len(used)} Bibliotheken werden beim Bau für macOS verwendet:", "",
         NOTES, "| Bibliothek | Version | Lizenz | Quelle |", "|---|---|---|---|"]
for p in used:
    lic = p.get("license") or "siehe Lizenzdatei"
    repo = p.get("repository") or f"https://crates.io/crates/{p['name']}"
    lines.append(f"| {p['name']} | {p['version']} | {lic} | {repo} |")
open("THIRD-PARTY-LICENSES.md", "w").write("\n".join(lines) + "\n")
print(f"{len(used)} Bibliotheken geschrieben")
