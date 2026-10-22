# Mitmachen

Schön, dass du helfen magst! Kurz und knapp:

- **Fehler und Wünsche** als Issue melden, am besten mit einer kurzen Beschreibung, was du gemacht hast, und dem Eintrag aus `~/Library/Application Support/Happy-Kitten/crash.log`, falls die App abgestürzt ist.
- **Änderungen** als Pull Request. Vorher bitte `cargo fmt --all`, `cargo clippy --workspace --all-targets` und `cargo test --workspace`. Hinweise zum Aufbau stehen in [CLAUDE.md](CLAUDE.md) und der [README](README.md).
- **Stil:** Oberfläche auf Deutsch, Farben nur aus `theme.rs`, Zeit immer als `Tick`, keine `unwrap`s auf Mediendaten, `unsafe` nur in `crates/hk-media`.
- **Lizenz:** Mit deinem Beitrag erklärst du dich einverstanden, dass er wie das Projekt unter MIT oder Apache-2.0 steht (so wie in Apache-2.0 Abschnitt 5 beschrieben). Bitte keinen Code einreichen, den du nicht selbst geschrieben hast oder der unter einer anderen Lizenz steht (zum Beispiel GPL).
- Das Logo (`assets/logo.png`) gehört Gottpot und ist von der freien Lizenz ausgenommen.
