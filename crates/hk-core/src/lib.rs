//! Happy-Kitten core: exact time, project model and the (very small) set of
//! editing operations. No GPU, no UI, no I/O besides project (de)serialisation.

mod document;
mod library;
mod overlay;
mod project;
mod subtitles;
mod text;
mod time;
mod transition;

pub use document::Document;
pub use library::{GlobalLibrary, LIB_ID_BIT, LibItem, TextPreset};
pub use overlay::OverlaySpec;
pub use project::{
    CANVAS_SHAPES, Clip, EditError, FitMode, Id, ImageLayer, Layer, MAX_TRACKS_PER_KIND, MediaItem,
    MediaKind, MediaSource, Project, Scene, SourceRef, TextLayer, Track, TrackKind, TransitionSpan,
    VideoFrame, preview_size,
};
pub use subtitles::{Cue, parse_by_extension, parse_lrc, parse_plain, parse_srt};
pub use text::{Align, FONTS, TextFx, TextSpec};
pub use time::{Fps, TICKS_PER_SECOND, Tick};
pub use transition::{Transition, TransitionKind};
