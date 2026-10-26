# Changelog

**English** · [日本語](CHANGELOG.ja.md) · [Deutsch](CHANGELOG.de.md)

All notable changes to Happy-Kitten are written down here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/).

## [1.3.0] - 2026-10-26

### Added
- **New Effects tab** with 13 overlay effects, drawn by the GPU completely from maths over the picture below them:
  - *Frames:* fire frame, ice frame, lightning frame, neon frame,
  - *Equalizers:* bar equalizer, circle equalizer, wave equalizer, beat rings,
  - *Glitch and cyberpunk:* glitch, cyber grid, laser show,
  - *Light:* lens flare, light leak.
- An effect is a clip: drag it onto a video track, or click it to put it at the playhead on a free video track (or on a new one). Its length is changed by its edges on the timeline, and it can be moved, split, deleted and copied like any other clip. Several effects can lie over each other.
- Settings per effect (as far as they make sense): width, height or size, speed, strength, colour (hue shift), beat reaction, position, mirror, fade in and out. Several effects can be edited together, and a reset button restores the defaults.
- **The equalizers and the beat reaction follow the music.** The sound of the timeline is analysed at the current moment (FFT, 32 bands from 40 Hz to 14 kHz, a beat detector on the bass). The analysis needs no history, so preview, playback and export show exactly the same.
- The cards in the tab are live: hover one to see the effect move.
- Tests: audio analysis (right band for a tone, beat only when the bass jumps), effect clips (fade in/out, split, tracks), and a GPU test that every effect draws, disappears at 0 % and that the equalizers react to the sound. The `fx_sheet` example renders all effects into one picture.
- English, Japanese, Korean and Spanish translations for everything new.

### Compatibility
- Projects that contain an effect clip cannot be opened with version 1.2 or older, because those versions do not know effect clips. Projects from 1.0 to 1.2 open in 1.3 unchanged.

## [1.2.0] - 2026-10-24

### Added
- **25 new transitions** in five groups in the Transitions tab, all computed on the GPU:
  - *Noise and cells:* paper burn, shatter, ink in water,
  - *Waves and optics:* raindrop, fisheye, zoom blur, spin blur, colour fringe, light leak,
  - *Curves and shapes:* sakura, heart iris, spiral, paw bloom, honeycomb, kaleidoscope,
  - *Manga look:* halftone dots, retro dither, tabby stripes, calico cat,
  - *3D and physics:* page curl, cube, flip, melt, Droste zoom, fractal.
- A GPU test checks that **every** transition starts on the old picture and ends on the new one, without leftovers of its own effect.
- `contact_sheet` can render a slice of the transitions (`… out.png 19 6`), `HK_DEMO=1` starts with the sample scenes.
- English, Japanese, Korean and Spanish translations for all new names and descriptions, with a test that no transition text is untranslated.

### Changed
- **A new project is empty.** The four sample scenes (Sonnenwiese, Wollknäuel, Abendkatze, Minzgarten) are no longer added, so you can start with your own files at once. Projects that already contain them open unchanged.
- The picture cards in the Transitions tab show each blend at a progress where it is easy to recognise (for example the flip, which is edge-on at 50 %).

### Fixed
- Rims and glows of several shaders (Claw Wipe, Gentle Paw Fade, Kitty Dash) used `pow(x, 2.0)`, which is undefined for negative `x`. They now use `x * x`, so the rim shows on both sides of the edge.
- Claw Wipe no longer jumps in at the very first frame.

### Compatibility
- Projects that use one of the new transitions cannot be opened with version 1.1 or older, because those versions do not know the transition. Everything else is unchanged; projects from 1.0 and 1.1 open in 1.2.

## [1.1.0] - 2026-10-23

### Added
- **Clear “saved” notice.** After saving a project (or a still frame) a green notice shows “Saved ✓” together with the full file path for a few seconds. Saving again with ⌘S now always gives feedback.
- **Sound settings** in the Audio tab (“Sound”), for the selected audio or video clips:
  - volume from 0 to 200 %,
  - fade in and fade out, with a smooth curve,
  - one effect with an amount slider: **echo**, **reverb**, **muffled** and **telephone**,
  - several clips can be edited together, and a reset button restores the defaults.
  - The effects are computed per sample without any state, so preview, playback and export sound exactly the same. Echo and reverb ring out after the end of the clip.
- **Colour tab** with **temperature** (cool to warm) and **tint** (green to magenta) for the selected video clips and images. The grade runs on the GPU, shows live in the preview and is part of the export. It keeps the brightness of the picture and works together with every transition (each side of a transition keeps its own grade).
- New icon for the Colour tab, English/Japanese/Korean/Spanish translations for all new texts.
- Developer helpers: `HK_BANNER`, `HK_FX` / `HK_VOLUME` / `HK_FADE` / `HK_TEMP` / `HK_TINT`, `HK_SELECT_PICTURE`, `HK_TAB=color` (see the README).

### Changed
- The pool now has six tabs (Files, Text, Graphic, Transitions, Colour, Audio); long labels shrink to fit.
- When a clip is split, the fade-in stays with the first half and the fade-out goes to the second half. Volume and colour grade are kept on both halves.
- The README, the Japanese README and the German README describe the new features. The version is now 1.1.

### Compatibility
- Projects saved with 1.0 open in 1.1 unchanged (sound and colour start neutral). Projects saved with 1.1 contain the new fields `audio` and `grade` per clip; version 1.0 ignores them.

### Fixed
- The “straight” button in the graphic editor is now translated.

## [1.0.0] - 2026-10-22

First public version: local, offline video editor for macOS in Rust with wgpu/Metal, paw transitions and 13 classic transitions, text clips with effects, subtitles and lyrics (SRT/LRC/TXT), several video and audio tracks, graphics with transparency, 9:16 export, export options (frame rate, bitrate, audio sample rate) and an interface in German, English, Japanese, Korean and Spanish.

[1.3.0]: https://github.com/Gottpot/Happy-Kitten/releases/tag/v1.3.0
[1.2.0]: https://github.com/Gottpot/Happy-Kitten/releases/tag/v1.2.0
[1.1.0]: https://github.com/Gottpot/Happy-Kitten/releases/tag/v1.1.0
[1.0.0]: https://github.com/Gottpot/Happy-Kitten/releases/tag/V1.0.0
