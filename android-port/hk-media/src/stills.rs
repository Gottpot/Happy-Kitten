use crate::{MediaError, Rgba};
use std::path::Path;

/// Decode an image, downscaling so the longer side is at most `max_dim`.
pub fn load_image(path: &Path, max_dim: u32) -> Result<Rgba, MediaError> {
    let img = image::open(path).map_err(|e| MediaError::Decode(e.to_string()))?;
    let img = if img.width().max(img.height()) > max_dim {
        img.resize(max_dim, max_dim, image::imageops::FilterType::Triangle)
    } else {
        img
    };
    let rgba = img.to_rgba8();
    Ok(Rgba {
        width: rgba.width(),
        height: rgba.height(),
        pixels: rgba.into_raw(),
    })
}
