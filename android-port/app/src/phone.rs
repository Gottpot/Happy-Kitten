//! Phone layout: slim header, player on top, timeline below it and the tool area at the bottom.
//! The tool tabs and the file options sit behind the kitten in the header, so the assets get the
//! whole tool area. The blocks are separated by handles that can be dragged.

use crate::app::{HappyKitten, PoolTab};
use crate::i18n::tr;
use crate::kit::{self, Icon};
use crate::pool_files::ViewMode;
use crate::theme::*;
use egui::{PopupCloseBehavior, Rect, Sense, Ui, pos2, vec2};

/// Height of a drag handle between two blocks.
const HANDLE_H: f32 = 26.0;
const HEADER_H: f32 = 56.0;
const MIN_TOOLS: f32 = 110.0;
const MIN_TIMELINE: f32 = 215.0;
const MIN_PLAYER: f32 = 240.0;
const DEFAULT_TOOLS: f32 = 0.28;
const DEFAULT_TIMELINE: f32 = 0.24;

/// Heights of the tool area and the timeline as a share of the screen height.
#[derive(Clone, Copy)]
pub struct PhoneLayout {
    pub tools: f32,
    pub timeline: f32,
}

impl Default for PhoneLayout {
    fn default() -> Self {
        PhoneLayout {
            tools: DEFAULT_TOOLS,
            timeline: DEFAULT_TIMELINE,
        }
    }
}

impl PhoneLayout {
    fn path() -> Option<std::path::PathBuf> {
        crate::platform::data_dir().map(|d| d.join("layout.txt"))
    }

    pub fn load() -> PhoneLayout {
        let text = Self::path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .unwrap_or_default();
        let mut it = text.split_whitespace().filter_map(|v| v.parse::<f32>().ok());
        match (it.next(), it.next()) {
            (Some(t), Some(l)) if (0.05..0.9).contains(&t) && (0.05..0.9).contains(&l) => {
                PhoneLayout {
                    tools: t,
                    timeline: l,
                }
            }
            _ => PhoneLayout::default(),
        }
    }

    pub fn save(&self) {
        if let Some(p) = Self::path() {
            let _ = std::fs::write(p, format!("{} {}\n", self.tools, self.timeline));
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Handle {
    /// Between the timeline and the tools.
    Tools,
    /// Between the player and the timeline.
    Timeline,
}

impl HappyKitten {
    /// Keeps the app out from under the status bar and the gesture bar.
    pub fn safe_area(&mut self, ui: &mut Ui, top: f32, bottom: f32) {
        self.safe_insets = (top, bottom);
        if top > 0.5 {
            egui::Panel::top("safe-top")
                .exact_size(top)
                .show_separator_line(false)
                .frame(egui::Frame::new().fill(PINK))
                .show(ui, |_| {});
        }
        if bottom > 0.5 {
            egui::Panel::bottom("safe-bottom")
                .exact_size(bottom)
                .show_separator_line(false)
                .frame(egui::Frame::new().fill(CREAM))
                .show(ui, |_| {});
        }
    }

    /// Height in points of the tool area and of the timeline, kept inside what the screen allows.
    pub fn phone_heights(&mut self, screen_h: f32) -> (f32, f32) {
        let (top, bottom) = self.safe_insets;
        let avail = screen_h - top - bottom - HEADER_H - 2.0 * HANDLE_H;
        let room = (avail - MIN_PLAYER).max(MIN_TOOLS + MIN_TIMELINE);
        let tools = (self.phone_layout.tools * screen_h).clamp(MIN_TOOLS, room - MIN_TIMELINE);
        let timeline = (self.phone_layout.timeline * screen_h).clamp(MIN_TIMELINE, room - tools);
        self.phone_layout.tools = tools / screen_h;
        self.phone_layout.timeline = timeline / screen_h;
        (tools, timeline)
    }

    /// The strip between two blocks: drag it up or down to give one block more room.
    pub fn split_handle(&mut self, ui: &mut Ui, which: Handle) {
        let screen_h = ui.ctx().content_rect().height();
        let id = match which {
            Handle::Tools => "split-tools",
            Handle::Timeline => "split-timeline",
        };
        egui::Panel::bottom(id)
            .exact_size(HANDLE_H)
            .show_separator_line(false)
            .frame(egui::Frame::new().fill(CREAM))
            .show(ui, |ui| {
                let rect = ui.max_rect();
                let resp = ui.interact(rect, egui::Id::new(id), Sense::drag());
                let active = resp.dragged();
                ui.painter().rect_filled(
                    Rect::from_center_size(rect.center(), vec2(54.0, 6.0)),
                    round(3),
                    if active { BROWN } else { PEACH_DEEP },
                );
                if resp.hovered() || active {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
                }
                let dy = resp.drag_delta().y / screen_h;
                match which {
                    // the boundary follows the finger: tools shrink, the timeline grows
                    Handle::Tools => {
                        self.phone_layout.tools -= dy;
                        self.phone_layout.timeline += dy;
                    }
                    Handle::Timeline => self.phone_layout.timeline -= dy,
                }
                if resp.drag_stopped() {
                    self.phone_layout.save();
                }
            });
    }

    /// The whole phone arrangement from the bottom up: tools, handle, timeline, handle, header;
    /// the player takes what is left.
    pub fn phone_layout_ui(&mut self, ui: &mut Ui) {
        self.pool_panel(ui);
        self.split_handle(ui, Handle::Tools);
        self.timeline_panel(ui);
        self.split_handle(ui, Handle::Timeline);
        self.header_phone(ui);
    }

    pub fn header_phone(&mut self, ui: &mut Ui) {
        let frame = egui::Frame::new()
            .fill(PINK)
            .inner_margin(egui::Margin::symmetric(10, 6));
        egui::Panel::top("header")
            .exact_size(HEADER_H)
            .show_separator_line(false)
            .frame(frame)
            .show(ui, |ui| {
                let full = ui.max_rect();
                ui.painter().line_segment(
                    [
                        full.left_bottom() + vec2(-20.0, 6.0),
                        full.right_bottom() + vec2(20.0, 6.0),
                    ],
                    ink(2.0),
                );
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    // The kitten opens the tools: tabs, scope, import, search and filters.
                    let (badge, resp) = ui.allocate_exact_size(vec2(52.0, 40.0), Sense::click());
                    let p = ui.painter();
                    p.rect_filled(badge, round(14), CREAM);
                    p.rect_stroke(badge, round(14), ink(1.8), egui::StrokeKind::Inside);
                    let face = Rect::from_min_size(badge.min, vec2(40.0, badge.height()));
                    egui::Image::new(&self.logo)
                        .uv(Rect::from_min_max(pos2(0.215, 0.07), pos2(0.795, 0.60)))
                        .corner_radius(round(12))
                        .paint_at(ui, face.shrink(2.0));
                    // a small arrow says "menu"
                    let c = pos2(badge.right() - 9.0, badge.center().y);
                    ui.painter().add(egui::Shape::convex_polygon(
                        vec![
                            c + vec2(-4.5, -2.5),
                            c + vec2(4.5, -2.5),
                            c + vec2(0.0, 3.0),
                        ],
                        BROWN,
                        egui::Stroke::NONE,
                    ));
                    self.tool_menu(&resp);

                    let name = if self.doc.is_dirty() {
                        format!("{} •", self.doc.project.name)
                    } else {
                        self.doc.project.name.clone()
                    };
                    ui.add(
                        egui::Label::new(egui::RichText::new(name).size(17.0).color(BROWN))
                            .truncate(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        self.menu_button(ui);
                        if kit::pill(ui, Some(Icon::Film), tr("Exportieren"), true, true).clicked()
                        {
                            self.open_export_dialog();
                        }
                    });
                });
            });
    }

    /// The popup behind the kitten: tool tabs and, for the file lists, scope, import and filters.
    fn tool_menu(&mut self, badge: &egui::Response) {
        egui::Popup::menu(badge)
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .width(340.0)
            .show(|ui| {
                ui.set_min_width(330.0);
                ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                ui.label(egui::RichText::new(tr("Werkzeuge")).size(13.0).color(BROWN_SOFT));
                ui.horizontal_wrapped(|ui| {
                    for (tab, icon, label) in Self::tab_defs() {
                        if self.tab_button(ui, tab, icon, label, 74.0) {
                            ui.close();
                        }
                    }
                });
                let files = match self.tab {
                    PoolTab::Files => Some(false),
                    PoolTab::Audio if !self.audio_settings => Some(true),
                    _ => None,
                };
                if let Some(audio) = files {
                    ui.separator();
                    ui.label(egui::RichText::new(tr("Dateien")).size(13.0).color(BROWN_SOFT));
                    if self.scope_import_row(ui) {
                        // the file picker takes over; the menu would still be there afterwards
                        ui.close();
                    }
                    self.filter_bar(ui, audio);
                }
            });
    }

    /// Title line of the tool area: which tool, how many files, and the view switch.
    pub fn phone_bar(&mut self, ui: &mut Ui, count: Option<String>, views: bool) {
        let (icon, label) = Self::tab_defs()
            .into_iter()
            .find(|(t, ..)| *t == self.tab)
            .map_or((Icon::Film, ""), |(_, i, l)| (i, l));
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            let (r, _) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
            kit::draw_icon(ui.painter(), icon, r, BROWN);
            ui.label(egui::RichText::new(label).size(16.0).color(BROWN).strong());
            if let Some(c) = count {
                ui.label(egui::RichText::new(c).size(12.0).color(BROWN_SOFT));
            }
            if views {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    for (mode, icon, tip) in [
                        (ViewMode::Table, Icon::Table, tr("Tabelle")),
                        (ViewMode::List, Icon::List, tr("Liste mit ganzem Namen")),
                        (ViewMode::Tiles, Icon::Grid, tr("Kacheln")),
                    ] {
                        if kit::round_button(ui, icon, 32.0, self.pool.mode == mode, true)
                            .on_hover_text(tip)
                            .clicked()
                        {
                            self.pool.mode = mode;
                        }
                    }
                });
            }
        });
        ui.add_space(2.0);
    }

    /// The "more" button: project actions that do not fit into the slim header.
    fn menu_button(&mut self, ui: &mut Ui) {
        let (rect, resp) = ui.allocate_exact_size(vec2(40.0, 40.0), Sense::click());
        let p = ui.painter();
        p.circle_filled(rect.center(), 19.0, CREAM);
        p.circle_stroke(rect.center(), 19.0, ink(1.8));
        for dy in [-7.0, 0.0, 7.0] {
            p.circle_filled(rect.center() + vec2(0.0, dy), 2.4, BROWN);
        }
        egui::Popup::menu(&resp).show(|ui| {
            ui.set_min_width(190.0);
            if ui.button(tr("Neu")).clicked() {
                self.new_project();
                ui.close();
            }
            if ui.button(tr("Öffnen")).clicked() {
                self.open_project();
                ui.close();
            }
            if ui.button(tr("Speichern")).clicked() {
                self.save_project(false);
                ui.close();
            }
            if ui.button(tr("Standbild")).clicked() {
                self.save_still();
                ui.close();
            }
            ui.separator();
            ui.label(egui::RichText::new(tr("Bildformat")).size(13.0).color(BROWN_SOFT));
            self.canvas_bar_row(ui);
            ui.separator();
            self.language_button(ui);
            if ui.button(tr("Über Happy-Kitten")).clicked() {
                self.about = true;
                ui.close();
            }
        });
    }

    /// Phone: the tools sit in a panel at the bottom; their height is set by the handle above.
    pub fn pool_panel_phone(&mut self, ui: &mut Ui) {
        let screen_h = ui.ctx().content_rect().height();
        let (tools, _) = self.phone_heights(screen_h);
        let frame = egui::Frame::new()
            .fill(CREAM)
            .inner_margin(egui::Margin::symmetric(6, 4));
        egui::Panel::bottom("pool")
            .exact_size(tools)
            .show_separator_line(false)
            .frame(frame)
            .show(ui, |ui| {
                kit::card_frame(PEACH).show(ui, |ui| {
                    ui.set_min_size(ui.available_size());
                    // The tools never grow the panel: what does not fit scrolls.
                    let room = ui.available_height().max(60.0);
                    egui::ScrollArea::vertical()
                        .id_salt("pool-phone")
                        .max_height(room)
                        .auto_shrink([false, false])
                        .show(ui, |ui| self.pool_content(ui));
                });
            });
    }
}
