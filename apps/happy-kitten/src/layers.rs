//! Text and graphic layers: turn a scene layer into a frame-sized premultiplied picture that the
//! GPU overlay pass can animate. Shared by the preview and the export.

use hk_core::{Id, Layer, MediaSource, Project};
use hk_media::Rgba;
use std::collections::HashMap;
use std::sync::Arc;

/// Decoded graphics (PNG logos and so on), loaded once per media entry.
#[derive(Default)]
pub struct ImageCache {
    map: HashMap<Id, Option<Arc<Rgba>>>,
}

/// Longest side kept of a graphic: sharp enough for 4K, small enough in memory.
const MAX_GRAPHIC: u32 = 3072;

impl ImageCache {
    pub fn get(&mut self, project: &Project, media: Id) -> Option<Arc<Rgba>> {
        self.map
            .entry(media)
            .or_insert_with(|| {
                let m = project.media(media)?;
                match &m.source {
                    MediaSource::File(p) => hk_media::load_image(p, MAX_GRAPHIC).ok().map(Arc::new),
                    MediaSource::Sample(i) => Some(Arc::new(hk_media::sample_frame(*i, 1280, 720))),
                }
            })
            .clone()
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }
}

/// What the animation shader needs: effect id, progress and the centre the effect works around.
pub fn animation(layer: &Layer) -> (u32, f32, [f32; 2]) {
    match layer {
        Layer::Text(t) => (t.fx.shader_id(), t.progress, t.spec.pos),
        Layer::Image(i) => (i.fx.shader_id(), i.progress, i.spec.pos),
        // effects have their own pass, see `Compositor::overlay_fx`
        Layer::Effect(_) => (0, 1.0, [0.5, 0.5]),
    }
}

/// Identifies the rendered picture of a layer (not its animation) for caching.
pub fn key(layer: &Layer) -> u64 {
    match layer {
        Layer::Text(t) => t.spec.raster_key(),
        Layer::Image(i) => {
            i.spec.raster_key()
                ^ i.media.wrapping_mul(0x9E37_79B9_7F4A_7C15)
                ^ 0x1F00_0000_0000_0000
        }
        Layer::Effect(e) => e.clip ^ 0x2F00_0000_0000_0000,
    }
}

/// The layer as a `w` x `h` premultiplied RGBA picture; `None` when it cannot be drawn
/// (no font, missing graphic file).
pub fn raster(
    layer: &Layer,
    project: &Project,
    images: &mut ImageCache,
    w: u32,
    h: u32,
) -> Option<Rgba> {
    match layer {
        Layer::Text(t) => hk_text::render(&t.spec, w, h).map(|r| Rgba {
            width: r.width,
            height: r.height,
            pixels: r.pixels,
        }),
        Layer::Image(i) => {
            let img = images.get(project, i.media)?;
            Some(hk_media::compose_overlay(&img, &i.spec, w, h))
        }
        Layer::Effect(_) => None,
    }
}
