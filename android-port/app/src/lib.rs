mod about;
mod app;
mod audio;
mod audio_ui;
mod autosave;
mod color_ui;
mod devscript;
mod engine;
mod export;
mod export_ui;
mod files;
mod fx_ui;
mod graphics;
mod i18n;
mod i18n_data;
mod import;
mod kit;
mod layers;
mod platform;
mod phone;
mod pool;
mod pool_files;
mod presets;
mod preview;
mod theme;
mod timeline;

use eframe::egui;

/// Entry point called by `android-activity` (see `java/MainActivity.java`).
// The attribute is required by the Android loader; no other unsafe code lives in this crate.
#[unsafe(no_mangle)]
fn android_main(app: android_activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("HappyKitten"),
    );
    platform::init(&app);
    files::init_app(&app);
    autosave::install_panic_log();
    // Vulkan only: the emulator's GL translator cannot create window surfaces, and every
    // phone that can run the editor has Vulkan.
    let mut setup = eframe::egui_wgpu::WgpuSetupCreateNew::without_display_handle();
    setup.instance_descriptor.backends = eframe::wgpu::Backends::VULKAN;
    let options = eframe::NativeOptions {
        android_app: Some(app),
        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            wgpu_setup: eframe::egui_wgpu::WgpuSetup::CreateNew(setup),
            ..Default::default()
        },
        ..Default::default()
    };
    if let Err(e) = eframe::run_native(
        "Happy-Kitten",
        options,
        Box::new(|cc| Ok(Box::new(app::HappyKitten::new(cc)))),
    ) {
        log::error!("Happy-Kitten beendet: {e}");
    }
}

#[allow(dead_code)]
fn _keep_egui_in_scope(_: &egui::Context) {}
