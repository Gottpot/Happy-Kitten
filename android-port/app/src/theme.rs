//! Happy-Kitten look: warm pastel pink, cream and peach panels, chocolate-brown ink.

use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle,
};

pub const PINK: Color32 = Color32::from_rgb(255, 190, 207);
pub const PINK_DEEP: Color32 = Color32::from_rgb(248, 150, 178);
pub const PINK_SOFT: Color32 = Color32::from_rgb(255, 226, 233);
pub const CREAM: Color32 = Color32::from_rgb(255, 249, 240);
pub const PEACH: Color32 = Color32::from_rgb(255, 228, 208);
pub const PEACH_DEEP: Color32 = Color32::from_rgb(252, 205, 175);
pub const BEIGE: Color32 = Color32::from_rgb(244, 230, 212);
pub const BROWN: Color32 = Color32::from_rgb(92, 56, 44);
pub const BROWN_SOFT: Color32 = Color32::from_rgb(150, 110, 92);
pub const YARN: Color32 = Color32::from_rgb(244, 168, 190);
pub const GOLD: Color32 = Color32::from_rgb(244, 196, 98);

/// Clip colours: soft blue, orange, pink, mint, lavender, yellow.
pub const CLIP_COLORS: [Color32; 6] = [
    Color32::from_rgb(176, 214, 250),
    Color32::from_rgb(255, 206, 158),
    Color32::from_rgb(255, 188, 208),
    Color32::from_rgb(178, 232, 208),
    Color32::from_rgb(216, 198, 246),
    Color32::from_rgb(255, 232, 150),
];

/// The "saved" notice: soft mint background, deep green text.
pub const SAVED_BG: Color32 = Color32::from_rgb(214, 244, 222);
pub const SAVED_TEXT: Color32 = Color32::from_rgb(26, 122, 66);

pub const AUDIO_COLOR: Color32 = Color32::from_rgb(214, 200, 244);

pub fn round(r: u8) -> CornerRadius {
    CornerRadius::same(r)
}

pub fn ink(width: f32) -> Stroke {
    Stroke::new(width, BROWN)
}

pub fn heading_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}



/// egui family for the preview of a text font ("hk-rounded", "hk-comic", ...).
pub fn text_family(id: &str) -> FontFamily {
    FontFamily::Name(format!("hk-{id}").into())
}

fn read_font(path: &str, index: u32) -> Option<FontData> {
    let bytes = std::fs::read(path).ok()?;
    let mut data = FontData::from_owned(bytes);
    data.index = index;
    Some(data)
}

/// The brand font, a Japanese/Chinese/Korean fallback so subtitles are readable everywhere in the
/// UI, and one family per text font so the font list can show every font in its own letters.
fn build_fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let add = |fonts: &mut FontDefinitions, key: &str, path: &str, index: u32| -> bool {
        match read_font(path, index) {
            Some(d) => {
                fonts.font_data.insert(key.into(), std::sync::Arc::new(d));
                true
            }
            None => false,
        }
    };
    let rounded = hk_text::font_path("rounded")
        .is_some_and(|(p, i)| add(&mut fonts, "rounded", &p, i));
    let mut cjk_keys = Vec::new();
    for (i, (path, index)) in hk_text::fallback_paths(false, false)
        .into_iter()
        .enumerate()
    {
        let key = format!("cjk{i}");
        if add(&mut fonts, &key, &path, index) {
            cjk_keys.push(key);
        }
    }
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        let list = fonts.families.entry(family.clone()).or_default();
        if rounded && family == FontFamily::Proportional {
            list.insert(0, "rounded".into());
        }
        list.extend(cjk_keys.iter().cloned());
    }
    for (id, _) in hk_core::FONTS {
        let Some((path, index)) =
            hk_text::font_path(id).filter(|(p, _)| std::path::Path::new(p).exists())
        else {
            continue;
        };
        let key = format!("hkfont-{id}");
        if add(&mut fonts, &key, &path, index) {
            let mut chain = vec![key];
            chain.extend(cjk_keys.iter().cloned());
            fonts.families.insert(text_family(id), chain);
        }
    }
    fonts
}

pub fn apply(ctx: &egui::Context) {
    ctx.set_fonts(build_fonts());

    ctx.set_theme(egui::ThemePreference::Light);
    ctx.all_styles_mut(|style| {
        let v = &mut style.visuals;
        v.dark_mode = false;
        v.override_text_color = Some(BROWN);
        v.panel_fill = CREAM;
        v.window_fill = CREAM;
        v.extreme_bg_color = PINK_SOFT;
        v.faint_bg_color = PEACH;
        v.window_corner_radius = round(16);
        v.menu_corner_radius = round(12);
        v.window_stroke = ink(1.5);
        v.selection.bg_fill = PINK;
        v.selection.stroke = ink(1.5);
        v.hyperlink_color = PINK_DEEP;

        let w = &mut v.widgets;
        w.noninteractive.bg_fill = CREAM;
        w.noninteractive.bg_stroke = Stroke::new(1.0, PEACH_DEEP);
        w.noninteractive.fg_stroke = ink(1.0);
        w.noninteractive.corner_radius = round(10);
        w.inactive.bg_fill = PEACH_DEEP;
        w.inactive.weak_bg_fill = PEACH;
        w.inactive.bg_stroke = ink(1.5);
        w.inactive.fg_stroke = ink(1.5);
        w.inactive.corner_radius = round(12);
        w.hovered.bg_fill = PINK;
        w.hovered.weak_bg_fill = PINK_SOFT;
        w.hovered.bg_stroke = ink(2.0);
        w.hovered.fg_stroke = ink(2.0);
        w.hovered.corner_radius = round(12);
        w.active.bg_fill = PINK;
        w.active.weak_bg_fill = PINK;
        w.active.bg_stroke = ink(2.0);
        w.active.fg_stroke = ink(2.0);
        w.active.corner_radius = round(12);
        w.open = w.active;

        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.spacing.slider_width = 120.0;
        style
            .text_styles
            .insert(TextStyle::Body, FontId::proportional(15.0));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::proportional(15.0));
        style
            .text_styles
            .insert(TextStyle::Small, FontId::proportional(12.0));
        style
            .text_styles
            .insert(TextStyle::Heading, FontId::proportional(24.0));
        style
            .text_styles
            .insert(TextStyle::Monospace, FontId::monospace(14.0));
    });
}
