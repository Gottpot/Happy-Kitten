//! "Über Happy-Kitten": who made it and under which licence.

use crate::app::HappyKitten;
use crate::kit;
use crate::theme::*;

impl HappyKitten {
    pub fn about_ui(&mut self, ctx: &egui::Context) {
        if !self.about {
            return;
        }
        let mut open = true;
        egui::Window::new("Über Happy-Kitten")
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(kit::card_frame(CREAM).inner_margin(egui::Margin::same(22)))
            .show(ctx, |ui| {
                ui.set_width(380.0);
                ui.vertical_centered(|ui| {
                    ui.add(
                        egui::Image::new(&self.logo)
                            .corner_radius(round(22))
                            .fit_to_exact_size(egui::vec2(150.0, 150.0)),
                    );
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(format!("Happy-Kitten v{}", short_version())).size(28.0));
                    ui.label(egui::RichText::new("Erstellt von Gottpot").size(20.0).color(PINK_DEEP));
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("Ein lokaler, werbefreier Videoschnitt für den Mac: ohne Cloud, ohne Konto, ohne Abo.")
                            .color(BROWN_SOFT),
                    );
                    ui.add_space(8.0);
                    ui.label("Quelloffen unter MIT oder Apache-2.0.");
                    ui.label(
                        egui::RichText::new("Mit Dank an egui, wgpu, symphonia, cpal, fontdue, image und viele weitere Open-Source-Bibliotheken (siehe THIRD-PARTY-LICENSES.md).")
                            .size(12.5)
                            .color(BROWN_SOFT),
                    );
                    ui.add_space(12.0);
                    if kit::pill(ui, Some(kit::Icon::Check), "Schön!", true, true).clicked() {
                        open = false;
                    }
                });
            });
        if !open || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.about = false;
        }
    }
}

/// "1.0" from the 1.0.0 package version.
pub fn short_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
        .rsplit_once('.')
        .map_or("1.0", |(a, _)| a)
}
