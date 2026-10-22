//! Audio for Happy-Kitten: decoding (symphonia, pure Rust), waveform peaks, a
//! small timeline mixer and playback through the system output (cpal).
//! Everything runs locally; nothing is streamed or uploaded.

mod decode;
mod library;
mod mix;
mod player;

pub use decode::{PEAK_BLOCK, Pcm, decode_file, probe};
pub use library::Library;
pub use mix::{ClipPlan, mix_block};
pub use player::Player;
