# Lizenzen von Drittbibliotheken

Happy-Kitten selbst: MIT oder Apache-2.0, © 2026 Gottpot. Diese 215 Bibliotheken werden beim Bau für macOS verwendet:

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


| Bibliothek | Version | Lizenz | Quelle |
|---|---|---|---|
| accesskit | 0.24.1 | MIT OR Apache-2.0 | https://github.com/AccessKit/accesskit |
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | https://github.com/oyvindln/adler2 |
| ahash | 0.8.12 | MIT OR Apache-2.0 | https://github.com/tkaitchuck/ahash |
| allocator-api2 | 0.2.21 | MIT OR Apache-2.0 | https://github.com/zakarumych/allocator-api2 |
| arboard | 3.7.0 | MIT OR Apache-2.0 | https://github.com/1Password/arboard |
| arrayvec | 0.7.8 | MIT OR Apache-2.0 | https://github.com/bluss/arrayvec |
| ash | 0.38.0+1.3.281 | MIT OR Apache-2.0 | https://github.com/ash-rs/ash |
| autocfg | 1.5.1 | Apache-2.0 OR MIT | https://github.com/cuviper/autocfg |
| bit-set | 0.10.0 | Apache-2.0 OR MIT | https://github.com/contain-rs/bit-set |
| bit-vec | 0.9.1 | Apache-2.0 OR MIT | https://github.com/contain-rs/bit-vec |
| bitflags | 1.3.2 | MIT/Apache-2.0 | https://github.com/bitflags/bitflags |
| bitflags | 2.13.2 | MIT OR Apache-2.0 | https://github.com/bitflags/bitflags |
| block2 | 0.5.1 | MIT | https://github.com/madsmtm/objc2 |
| block2 | 0.6.2 | MIT | https://github.com/madsmtm/objc2 |
| bytemuck | 1.25.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/bytemuck |
| bytemuck_derive | 1.12.1 | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/bytemuck |
| byteorder-lite | 0.1.0 | Unlicense OR MIT | https://github.com/image-rs/byteorder-lite |
| cc | 1.9.0 | MIT OR Apache-2.0 | https://github.com/rust-lang/cc-rs |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | https://github.com/rust-lang/cfg-if |
| cfg_aliases | 0.2.2 | MIT | https://github.com/katharostech/cfg_aliases |
| codespan-reporting | 0.13.1 | Apache-2.0 | https://github.com/brendanzab/codespan |
| color | 0.3.3 | Apache-2.0 OR MIT | https://github.com/linebender/color |
| color_quant | 1.1.0 | MIT | https://github.com/image-rs/color_quant.git |
| core-foundation | 0.9.4 | MIT OR Apache-2.0 | https://github.com/servo/core-foundation-rs |
| core-foundation-sys | 0.8.7 | MIT OR Apache-2.0 | https://github.com/servo/core-foundation-rs |
| core-graphics | 0.23.2 | MIT OR Apache-2.0 | https://github.com/servo/core-foundation-rs |
| core-graphics-types | 0.1.3 | MIT OR Apache-2.0 | https://github.com/servo/core-foundation-rs |
| core_maths | 0.1.1 | MIT | https://github.com/robertbastian/core_maths |
| coreaudio-rs | 0.14.2 | MIT/Apache-2.0 | https://github.com/RustAudio/coreaudio-rs.git |
| cpal | 0.17.3 | Apache-2.0 | https://github.com/RustAudio/cpal |
| crc32fast | 1.5.3 | MIT OR Apache-2.0 | https://github.com/srijs/rust-crc32fast |
| cursor-icon | 1.2.0 | MIT OR Apache-2.0 OR Zlib | https://github.com/rust-windowing/cursor-icon |
| dasp_sample | 0.11.0 | MIT OR Apache-2.0 | https://github.com/rustaudio/sample.git |
| dispatch | 0.2.0 | MIT | http://github.com/SSheldon/rust-dispatch |
| dispatch2 | 0.3.1 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| dlib | 0.5.3 | MIT | https://github.com/elinorbgr/dlib |
| document-features | 0.2.12 | MIT OR Apache-2.0 | https://github.com/slint-ui/document-features |
| dpi | 0.1.2 | Apache-2.0 AND MIT | https://github.com/rust-windowing/winit |
| ecolor | 0.36.2 | MIT OR Apache-2.0 | https://github.com/emilk/egui |
| eframe | 0.36.2 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/eframe |
| egui | 0.36.2 | MIT OR Apache-2.0 | https://github.com/emilk/egui |
| egui-wgpu | 0.36.2 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/egui-wgpu |
| egui-winit | 0.36.2 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/egui-winit |
| either | 1.19.0 | MIT OR Apache-2.0 | https://github.com/rayon-rs/either |
| emath | 0.36.2 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/emath |
| encoding_rs | 0.8.42 | (Apache-2.0 OR MIT) AND BSD-3-Clause | https://github.com/hsivonen/encoding_rs |
| epaint | 0.36.2 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/epaint |
| epaint_default_fonts | 0.36.2 | (MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0 | https://github.com/emilk/egui/tree/main/crates/epaint_default_fonts |
| equivalent | 1.0.3 | Apache-2.0 OR MIT | https://github.com/indexmap-rs/equivalent |
| euclid | 0.22.14 | MIT OR Apache-2.0 | https://github.com/servo/euclid |
| extended | 0.1.0 | MIT | https://github.com/depp/extended-rs |
| fax | 0.2.7 | MIT | https://github.com/pdf-rs/fax |
| fdeflate | 0.3.7 | MIT OR Apache-2.0 | https://github.com/image-rs/fdeflate |
| fearless_simd | 0.4.1 | Apache-2.0 OR MIT | https://github.com/linebender/fearless_simd |
| find-msvc-tools | 0.1.15 | MIT OR Apache-2.0 | https://github.com/rust-lang/cc-rs |
| flate2 | 1.1.10 | MIT OR Apache-2.0 | https://github.com/rust-lang/flate2-rs |
| foldhash | 0.1.5 | Zlib | https://github.com/orlp/foldhash |
| foldhash | 0.2.0 | Zlib | https://github.com/orlp/foldhash |
| font-types | 0.12.7 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| fontdue | 0.9.5 | MIT OR Apache-2.0 OR Zlib | https://github.com/mooman219/fontdue |
| foreign-types | 0.5.0 | MIT/Apache-2.0 | https://github.com/sfackler/foreign-types |
| foreign-types-macros | 0.2.4 | MIT/Apache-2.0 | https://github.com/sfackler/foreign-types |
| foreign-types-shared | 0.3.1 | MIT/Apache-2.0 | https://github.com/sfackler/foreign-types |
| gif | 0.14.2 | MIT OR Apache-2.0 | https://github.com/image-rs/image-gif |
| glifo | 0.2.0 | Apache-2.0 OR MIT | https://github.com/linebender/vello |
| glow | 0.17.0 | MIT OR Apache-2.0 OR Zlib | https://github.com/grovesNL/glow |
| gpu-allocator | 0.28.0 | MIT OR Apache-2.0 | https://github.com/Traverse-Research/gpu-allocator |
| guillotiere | 0.7.0 | MIT/Apache-2.0 | https://github.com/nical/guillotiere |
| half | 2.7.1 | MIT OR Apache-2.0 | https://github.com/VoidStarKat/half-rs |
| harfrust | 0.12.0 | MIT | https://github.com/harfbuzz/harfrust |
| hashbrown | 0.15.5 | MIT OR Apache-2.0 | https://github.com/rust-lang/hashbrown |
| hashbrown | 0.16.1 | MIT OR Apache-2.0 | https://github.com/rust-lang/hashbrown |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 | https://github.com/rust-lang/hashbrown |
| image | 0.25.10 | MIT OR Apache-2.0 | https://github.com/image-rs/image |
| image-webp | 0.2.4 | MIT OR Apache-2.0 | https://github.com/image-rs/image-webp |
| indexmap | 2.14.2 | Apache-2.0 OR MIT | https://github.com/indexmap-rs/indexmap |
| itertools | 0.15.0 | MIT OR Apache-2.0 | https://github.com/rust-itertools/itertools |
| itoa | 1.0.18 | MIT OR Apache-2.0 | https://github.com/dtolnay/itoa |
| jobserver | 0.1.35 | MIT OR Apache-2.0 | https://github.com/rust-lang/jobserver-rs |
| khronos-egl | 6.0.0 | MIT/Apache-2.0 | https://github.com/timothee-haudebourg/khronos-egl |
| kurbo | 0.13.1 | Apache-2.0 OR MIT | https://github.com/linebender/kurbo |
| lazy_static | 1.5.1 | MIT OR Apache-2.0 | https://github.com/rust-lang-nursery/lazy-static.rs |
| libc | 0.2.191 | MIT OR Apache-2.0 | https://github.com/rust-lang/libc |
| libloading | 0.8.9 | ISC | https://github.com/nagisa/rust_libloading/ |
| libm | 0.2.16 | MIT | https://github.com/rust-lang/compiler-builtins |
| linebender_resource_handle | 0.1.1 | Apache-2.0 OR MIT | https://github.com/linebender/raw_resource_handle |
| litrs | 1.0.1 | MIT OR Apache-2.0 | https://github.com/LukasKalbertodt/litrs |
| lock_api | 0.4.14 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot |
| log | 0.4.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/log |
| mach2 | 0.5.0 | BSD-2-Clause OR MIT OR Apache-2.0 | https://github.com/JohnTitor/mach2 |
| memchr | 2.8.3 | Unlicense OR MIT | https://github.com/BurntSushi/memchr |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 | https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide |
| miniz_oxide | 0.9.1 | MIT OR Zlib OR Apache-2.0 | https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide |
| moxcms | 0.8.1 | BSD-3-Clause OR Apache-2.0 | https://github.com/awxkee/moxcms.git |
| multiversion_no_op | 1.0.0 | Apache-2.0 OR MIT | https://github.com/hsivonen/multiversion_no_op |
| naga | 30.0.1 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| naga-types | 30.0.1 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| nohash-hasher | 0.2.0 | Apache-2.0 OR MIT | https://github.com/paritytech/nohash-hasher |
| num-traits | 0.2.19 | MIT OR Apache-2.0 | https://github.com/rust-num/num-traits |
| objc-sys | 0.3.5 | MIT | https://github.com/madsmtm/objc2 |
| objc2 | 0.5.3 | MIT | https://github.com/madsmtm/objc2 |
| objc2 | 0.6.5 | MIT | https://github.com/madsmtm/objc2 |
| objc2-app-kit | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| objc2-app-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| objc2-audio-toolbox | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| objc2-av-foundation | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| objc2-avf-audio | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| objc2-core-audio | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| objc2-core-audio-types | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| objc2-core-data | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| objc2-core-foundation | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| objc2-core-graphics | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| objc2-core-image | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| objc2-core-image | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| objc2-core-media | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| objc2-core-video | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| objc2-encode | 4.1.0 | MIT | https://github.com/madsmtm/objc2 |
| objc2-exception-helper | 0.1.1 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| objc2-foundation | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| objc2-foundation | 0.3.2 | MIT | https://github.com/madsmtm/objc2 |
| objc2-image-io | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| objc2-io-surface | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| objc2-media-toolbox | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| objc2-metal | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| objc2-metal | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| objc2-quartz-core | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| objc2-quartz-core | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| once_cell | 1.21.4 | MIT OR Apache-2.0 | https://github.com/matklad/once_cell |
| ordered-float | 5.5.0 | MIT | https://github.com/reem/rust-ordered-float |
| parking_lot | 0.12.5 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot |
| parking_lot_core | 0.9.12 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot |
| peniko | 0.6.1 | Apache-2.0 OR MIT | https://github.com/linebender/peniko |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT | https://github.com/taiki-e/pin-project-lite |
| pkg-config | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/pkg-config-rs |
| png | 0.18.1 | MIT OR Apache-2.0 | https://github.com/image-rs/image-png |
| pollster | 0.4.0 | Apache-2.0/MIT | https://github.com/zesterer/pollster |
| pollster | 1.0.1 | Apache-2.0/MIT | https://github.com/zesterer/pollster |
| polycool | 0.4.0 | MIT OR Apache-2.0 | https://github.com/linebender/kurbo |
| presser | 0.3.1 | MIT OR Apache-2.0 | https://github.com/EmbarkStudios/presser |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | https://github.com/dtolnay/proc-macro2 |
| profiling | 1.0.18 | MIT OR Apache-2.0 | https://github.com/aclysma/profiling |
| pxfm | 0.1.30 | BSD-3-Clause OR Apache-2.0 | https://github.com/awxkee/pxfm |
| quick-error | 2.0.1 | MIT/Apache-2.0 | http://github.com/tailhook/quick-error |
| quote | 1.0.47 | MIT OR Apache-2.0 | https://github.com/dtolnay/quote |
| raw-window-handle | 0.6.2 | MIT OR Apache-2.0 OR Zlib | https://github.com/rust-windowing/raw-window-handle |
| raw-window-metal | 1.1.0 | MIT OR Apache-2.0 | https://github.com/rust-windowing/raw-window-metal |
| read-fonts | 0.41.0 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| renderdoc-sys | 1.1.0 | MIT OR Apache-2.0 | https://github.com/ebkalderon/renderdoc-rs |
| rfd | 0.17.2 | MIT | https://github.com/PolyMeilex/rfd |
| rustc-hash | 1.1.0 | Apache-2.0/MIT | https://github.com/rust-lang-nursery/rustc-hash |
| rustc-hash | 2.1.3 | Apache-2.0 OR MIT | https://github.com/rust-lang/rustc-hash |
| rustversion | 1.0.23 | MIT OR Apache-2.0 | https://github.com/dtolnay/rustversion |
| scopeguard | 1.2.0 | MIT OR Apache-2.0 | https://github.com/bluss/scopeguard |
| self_cell | 1.3.0 | Apache-2.0 OR GPL-2.0-only | https://github.com/Voultapher/self_cell |
| serde | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | https://github.com/serde-rs/json |
| shlex | 2.0.1 | MIT OR Apache-2.0 | https://github.com/comex/rust-shlex |
| simd-adler32 | 0.3.10 | MIT | https://github.com/mcountryman/simd-adler32 |
| simdutf8 | 0.1.5 | MIT OR Apache-2.0 | https://github.com/rusticstuff/simdutf8 |
| skrifa | 0.44.0 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| smallvec | 1.16.3 | MIT OR Apache-2.0 | https://github.com/servo/rust-smallvec |
| smol_str | 0.2.2 | MIT OR Apache-2.0 | https://github.com/rust-analyzer/smol_str |
| spirv | 0.4.0+sdk-1.4.341.0 | Apache-2.0 | https://github.com/gfx-rs/rspirv |
| static_assertions | 1.1.0 | MIT OR Apache-2.0 | https://github.com/nvzqz/static-assertions-rs |
| symphonia | 0.5.5 | MPL-2.0 | https://github.com/pdeljanov/Symphonia |
| symphonia-bundle-flac | 0.5.5 | MPL-2.0 | https://github.com/pdeljanov/Symphonia |
| symphonia-bundle-mp3 | 0.5.5 | MPL-2.0 | https://github.com/pdeljanov/Symphonia |
| symphonia-codec-aac | 0.5.5 | MPL-2.0 | https://github.com/pdeljanov/Symphonia |
| symphonia-codec-alac | 0.5.5 | MPL-2.0 | https://github.com/pdeljanov/Symphonia |
| symphonia-codec-pcm | 0.5.5 | MPL-2.0 | https://github.com/pdeljanov/Symphonia |
| symphonia-codec-vorbis | 0.5.5 | MPL-2.0 | https://github.com/pdeljanov/Symphonia |
| symphonia-core | 0.5.5 | MPL-2.0 | https://github.com/pdeljanov/Symphonia |
| symphonia-format-caf | 0.5.5 | MPL-2.0 | https://github.com/pdeljanov/Symphonia |
| symphonia-format-isomp4 | 0.5.5 | MPL-2.0 | https://github.com/pdeljanov/Symphonia |
| symphonia-format-mkv | 0.5.5 | MPL-2.0 | https://github.com/pdeljanov/Symphonia |
| symphonia-format-ogg | 0.5.5 | MPL-2.0 | https://github.com/pdeljanov/Symphonia |
| symphonia-format-riff | 0.5.5 | MPL-2.0 | https://github.com/pdeljanov/Symphonia |
| symphonia-metadata | 0.5.5 | MPL-2.0 | https://github.com/pdeljanov/Symphonia |
| symphonia-utils-xiph | 0.5.5 | MPL-2.0 | https://github.com/pdeljanov/Symphonia |
| syn | 2.0.119 | MIT OR Apache-2.0 | https://github.com/dtolnay/syn |
| syn | 3.0.9 | MIT OR Apache-2.0 | https://github.com/dtolnay/syn |
| termcolor | 1.4.1 | Unlicense OR MIT | https://github.com/BurntSushi/termcolor |
| thiserror | 2.0.21 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| tiff | 0.11.3 | MIT | https://github.com/image-rs/image-tiff |
| tracing | 0.1.44 | MIT | https://github.com/tokio-rs/tracing |
| tracing-core | 0.1.36 | MIT | https://github.com/tokio-rs/tracing |
| ttf-parser | 0.25.1 | MIT OR Apache-2.0 | https://github.com/harfbuzz/ttf-parser |
| type-map | 0.5.1 | MIT/Apache-2.0 | https://github.com/kardeiz/type-map |
| unicode-general-category | 1.1.0 | Apache-2.0 | https://github.com/yeslogic/unicode-general-category |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | https://github.com/dtolnay/unicode-ident |
| unicode-segmentation | 1.13.3 | MIT OR Apache-2.0 | https://github.com/unicode-rs/unicode-segmentation |
| unicode-width | 0.2.2 | MIT OR Apache-2.0 | https://github.com/unicode-rs/unicode-width |
| uuid | 1.28.0 | Apache-2.0 OR MIT | https://github.com/uuid-rs/uuid |
| vello_common | 0.1.0 | Apache-2.0 OR MIT | https://github.com/linebender/vello |
| vello_cpu | 0.1.0 | Apache-2.0 OR MIT | https://github.com/linebender/vello |
| version_check | 0.9.5 | MIT/Apache-2.0 | https://github.com/SergioBenitez/version_check |
| wayland-sys | 0.31.11 | MIT | https://github.com/smithay/wayland-rs |
| web-time | 1.1.0 | MIT OR Apache-2.0 | https://github.com/daxpedda/web-time |
| weezl | 0.1.12 | MIT OR Apache-2.0 | https://github.com/image-rs/weezl |
| wgpu | 30.0.1 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| wgpu-core | 30.0.1 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| wgpu-core-deps-apple | 30.0.1 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| wgpu-hal | 30.0.1 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| wgpu-naga-bridge | 30.0.1 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| wgpu-types | 30.0.1 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| winit | 0.30.13 | Apache-2.0 | https://github.com/rust-windowing/winit |
| zerocopy | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | https://github.com/google/zerocopy |
| zerocopy-derive | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | https://github.com/google/zerocopy |
| zlib-rs | 0.6.8 | Zlib | https://github.com/trifectatechfoundation/zlib-rs |
| zmij | 1.0.23 | MIT | https://github.com/dtolnay/zmij |
| zune-core | 0.5.3 | MIT OR Apache-2.0 OR Zlib | https://github.com/etemesi254/zune-image |
| zune-jpeg | 0.5.15 | MIT OR Apache-2.0 OR Zlib | https://github.com/etemesi254/zune-image/tree/dev/crates/zune-jpeg |
