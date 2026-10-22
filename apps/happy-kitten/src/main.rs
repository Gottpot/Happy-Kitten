mod about;
mod app;
mod audio;
mod autosave;
mod devscript;
mod engine;
mod export;
mod export_ui;
mod graphics;
mod import;
mod kit;
mod layers;
mod pool;
mod pool_files;
mod presets;
mod preview;
mod theme;
mod timeline;

use eframe::egui;

fn icon() -> egui::IconData {
    let img = image::load_from_memory(include_bytes!("../../../assets/logo.png")).map(|i| {
        i.resize(256, 256, image::imageops::FilterType::Lanczos3)
            .to_rgba8()
    });
    match img {
        Ok(rgba) => egui::IconData {
            width: rgba.width(),
            height: rgba.height(),
            rgba: rgba.into_raw(),
        },
        Err(_) => egui::IconData::default(),
    }
}

fn main() -> eframe::Result {
    autosave::install_panic_log();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Happy-Kitten")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1100.0, 700.0])
            .with_icon(icon())
            // The pink header runs under the macOS traffic lights.
            .with_fullsize_content_view(true)
            .with_title_shown(false)
            .with_titlebar_shown(false),
        ..Default::default()
    };
    eframe::run_native(
        "Happy-Kitten",
        options,
        Box::new(|cc| Ok(Box::new(app::HappyKitten::new(cc)))),
    )
}
