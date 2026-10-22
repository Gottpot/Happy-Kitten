//! The file lists of the media pool: project files and the global library, as tiles, a list or
//! a table, with search, filters, sorting and a clear marker for everything already on the timeline.

use crate::app::{HappyKitten, Payload};
use crate::i18n::tr;
use crate::import::Scope;
use crate::kit::{self, Icon};
use crate::pool::ellipsize;
use crate::theme::*;
use crate::trf;
use egui::{Align2, Color32, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui, pos2, vec2};
use hk_core::{Id, MediaKind, MediaSource, Tick};
use std::path::PathBuf;
use std::time::{Duration, Instant};

const CARD_W: f32 = 150.0;
const THUMB_H: f32 = 84.0;
const USED_FILL: Color32 = Color32::from_rgb(255, 244, 204);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Tiles,
    List,
    Table,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KindFilter {
    All,
    Video,
    Image,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UsageFilter {
    All,
    Used,
    Unused,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Added,
    Name,
    Duration,
    Kind,
}

/// Search, filter, sort and view settings of the file lists.
pub struct PoolView {
    pub query: String,
    pub kind: KindFilter,
    pub usage: UsageFilter,
    pub sort: SortKey,
    pub desc: bool,
    pub mode: ViewMode,
}

impl Default for PoolView {
    fn default() -> Self {
        PoolView {
            query: String::new(),
            kind: KindFilter::All,
            usage: UsageFilter::All,
            sort: SortKey::Added,
            desc: false,
            mode: ViewMode::Tiles,
        }
    }
}

/// One file as the lists show it, project or global.
struct Row {
    id: Id,
    name: String,
    /// Name with extension, as on disk.
    file: String,
    folder: String,
    path: Option<PathBuf>,
    kind: MediaKind,
    secs: f64,
    size: (u32, u32),
    alpha: bool,
    used: bool,
    global: bool,
    missing: bool,
}

impl Row {
    fn kind_label(&self) -> &'static str {
        match self.kind {
            MediaKind::Video => tr("Video"),
            MediaKind::Image if self.alpha => tr("Grafik"),
            MediaKind::Image => tr("Bild"),
            MediaKind::Audio => tr("Ton"),
        }
    }

    fn details(&self) -> String {
        let size = if self.size.0 > 0 {
            format!(" · {}×{}", self.size.0, self.size.1)
        } else {
            String::new()
        };
        if self.kind == MediaKind::Image {
            format!("{}{}", self.kind_label(), size)
        } else {
            format!("{} · {}{}", self.kind_label(), clock(self.secs), size)
        }
    }
}

fn clock(secs: f64) -> String {
    let t = Tick::from_secs(secs.max(0.0)).secs();
    let (m, s) = ((t / 60.0) as u32, t % 60.0);
    format!("{m}:{s:04.1}")
}

/// Gold check badge: this file is already used on the timeline.
fn used_badge(p: &egui::Painter, c: egui::Pos2, r: f32) {
    p.circle_filled(c, r, GOLD);
    p.circle_stroke(c, r, ink(1.8));
    kit::draw_icon(
        p,
        Icon::Check,
        Rect::from_center_size(c, vec2(r * 1.3, r * 1.3)),
        BROWN,
    );
}

fn type_badge(p: &egui::Painter, kind: MediaKind, rect: Rect) {
    let (icon, fill) = match kind {
        MediaKind::Audio => (Icon::Note, AUDIO_COLOR),
        MediaKind::Image => (Icon::Sparkle, PEACH),
        MediaKind::Video => (Icon::Film, PINK_SOFT),
    };
    p.rect_filled(rect, round(5), fill);
    kit::draw_icon(p, icon, rect.shrink(rect.width() * 0.2), BROWN);
}

impl HappyKitten {
    pub fn pool_files(&mut self, ui: &mut Ui, audio: bool) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(5.0, 5.0);
            self.scope_switch_inline(ui);
            ui.add_space(4.0);
            if kit::pill(ui, Some(Icon::Plus), tr("Dateien"), false, true)
                .on_hover_text(tr("Mehrere Dateien auf einmal wählen (⌘I)"))
                .clicked()
            {
                self.import_dialog();
            }
            if kit::pill(ui, Some(Icon::Folder), tr("Ordner"), false, true)
                .on_hover_text(tr(
                    "Alle Videos, Bilder, Audios und Untertitel eines Ordners",
                ))
                .clicked()
            {
                self.import_folder_dialog();
            }
        });
        self.filter_bar(ui, audio);

        let (rows, total) = self.build_rows(audio);
        ui.label(
            egui::RichText::new(trf!("{n} von {total}", n = rows.len(), total = total))
                .size(12.0)
                .color(BROWN_SOFT),
        );
        if rows.is_empty() {
            self.empty_hint(ui, total > 0);
            return;
        }
        let mode = self.pool.mode;
        match mode {
            ViewMode::Tiles => {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
                            for row in &rows {
                                self.tile(ui, row);
                            }
                        });
                    });
            }
            ViewMode::List => {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 5.0;
                        for row in &rows {
                            self.list_row(ui, row);
                        }
                    });
            }
            ViewMode::Table => {
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.table(ui, &rows);
                    });
            }
        }
    }

    /// Two kinds of imports: "Projekt" (only this project) and "Global" (every project).
    fn scope_switch_inline(&mut self, ui: &mut Ui) {
        {
            if kit::pill(ui, Some(Icon::Film), tr("Dieses Projekt"), self.scope == Scope::Project, true).on_hover_text(tr("Dateien, die nur zu diesem Projekt gehören. Goldener Haken = schon auf der Timeline verwendet")).clicked() {
                self.scope = Scope::Project;
            }
            if kit::pill(ui, Some(Icon::Globe), tr("Global"), self.scope == Scope::Global, true)
                .on_hover_text(tr("Bibliothek für alle Projekte: Dateien bleiben, wo sie sind, und sind überall verfügbar"))
                .clicked()
            {
                self.scope = Scope::Global;
            }
        }
    }

    fn filter_bar(&mut self, ui: &mut Ui, audio: bool) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            ui.add(
                egui::TextEdit::singleline(&mut self.pool.query)
                    .hint_text(tr("Suchen …"))
                    .desired_width(140.0),
            );
            if !self.pool.query.is_empty()
                && kit::round_button(ui, Icon::Trash, 26.0, false, true)
                    .on_hover_text(tr("Suche leeren"))
                    .clicked()
            {
                self.pool.query.clear();
            }
            ui.add_space(2.0);
            for (mode, icon, tip) in [
                (ViewMode::Tiles, Icon::Grid, tr("Kacheln")),
                (ViewMode::List, Icon::List, tr("Liste mit ganzem Namen")),
                (ViewMode::Table, Icon::Table, tr("Tabelle")),
            ] {
                if kit::round_button(ui, icon, 30.0, self.pool.mode == mode, true)
                    .on_hover_text(tip)
                    .clicked()
                {
                    self.pool.mode = mode;
                }
            }
        });
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            if !audio {
                egui::ComboBox::from_id_salt("pool-kind")
                    .selected_text(match self.pool.kind {
                        KindFilter::All => tr("Alle Arten"),
                        KindFilter::Video => tr("Videos"),
                        KindFilter::Image => tr("Bilder"),
                    })
                    .width(96.0)
                    .show_ui(ui, |ui| {
                        for (k, label) in [
                            (KindFilter::All, tr("Alle Arten")),
                            (KindFilter::Video, tr("Videos")),
                            (KindFilter::Image, tr("Bilder")),
                        ] {
                            ui.selectable_value(&mut self.pool.kind, k, label);
                        }
                    });
            }
            egui::ComboBox::from_id_salt("pool-usage")
                .selected_text(match self.pool.usage {
                    UsageFilter::All => tr("Alles"),
                    UsageFilter::Used => tr("Verwendet"),
                    UsageFilter::Unused => tr("Noch frei"),
                })
                .width(96.0)
                .show_ui(ui, |ui| {
                    for (u, label) in [
                        (UsageFilter::All, tr("Alles")),
                        (UsageFilter::Used, tr("Verwendet")),
                        (UsageFilter::Unused, tr("Noch frei")),
                    ] {
                        ui.selectable_value(&mut self.pool.usage, u, label);
                    }
                });
            egui::ComboBox::from_id_salt("pool-sort")
                .selected_text(match self.pool.sort {
                    SortKey::Added => tr("Reihenfolge"),
                    SortKey::Name => tr("Name"),
                    SortKey::Duration => tr("Dauer"),
                    SortKey::Kind => tr("Art"),
                })
                .width(100.0)
                .show_ui(ui, |ui| {
                    for (k, label) in [
                        (SortKey::Added, tr("Reihenfolge")),
                        (SortKey::Name, tr("Name")),
                        (SortKey::Duration, tr("Dauer")),
                        (SortKey::Kind, tr("Art")),
                    ] {
                        ui.selectable_value(&mut self.pool.sort, k, label);
                    }
                });
            if kit::round_button(ui, Icon::Redo, 28.0, self.pool.desc, true)
                .on_hover_text(if self.pool.desc {
                    tr("Absteigend")
                } else {
                    tr("Aufsteigend")
                })
                .clicked()
            {
                self.pool.desc = !self.pool.desc;
            }
        });
    }

    /// The filtered and sorted rows plus how many there are before filtering.
    fn build_rows(&mut self, audio: bool) -> (Vec<Row>, usize) {
        let want = |k: MediaKind| (k == MediaKind::Audio) == audio;
        let used_ids = self.doc.project.used_media();
        let mut rows: Vec<Row> = Vec::new();
        if self.scope == Scope::Project {
            for m in self.doc.project.media.iter().filter(|m| want(m.kind)) {
                let path = match &m.source {
                    MediaSource::File(p) => Some(p.clone()),
                    MediaSource::Sample(_) => None,
                };
                rows.push(Row {
                    id: m.id,
                    name: m.name.clone(),
                    file: path
                        .as_ref()
                        .and_then(|p| p.file_name())
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| m.name.clone()),
                    folder: folder_of(path.as_deref()),
                    path,
                    kind: m.kind,
                    secs: m.duration.secs(),
                    size: m.size,
                    alpha: m.alpha,
                    used: used_ids.contains(&m.id),
                    global: false,
                    missing: false,
                });
            }
        } else {
            // a library entry counts as used when the project uses a media with the same file
            let used_paths: std::collections::HashSet<PathBuf> = self
                .doc
                .project
                .media
                .iter()
                .filter(|m| used_ids.contains(&m.id))
                .filter_map(|m| {
                    if let MediaSource::File(p) = &m.source {
                        Some(p.clone())
                    } else {
                        None
                    }
                })
                .collect();
            let items: Vec<_> = self
                .lib
                .items
                .iter()
                .filter(|i| want(i.kind))
                .cloned()
                .collect();
            for i in items {
                self.engine
                    .want_thumb_item(i.id, MediaSource::File(i.path.clone()), i.kind);
                let missing = {
                    let e = self
                        .lib_exists
                        .entry(i.id)
                        .or_insert_with(|| (i.path.exists(), Instant::now()));
                    if e.1.elapsed() > Duration::from_secs(3) {
                        *e = (i.path.exists(), Instant::now());
                    }
                    !e.0
                };
                rows.push(Row {
                    id: i.id,
                    name: i.name.clone(),
                    file: i
                        .path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| i.name.clone()),
                    folder: folder_of(Some(&i.path)),
                    used: used_paths.contains(&i.path),
                    path: Some(i.path),
                    kind: i.kind,
                    secs: i.duration.secs(),
                    size: i.size,
                    alpha: i.alpha,
                    global: true,
                    missing,
                });
            }
        }
        let total = rows.len();
        let q = self.pool.query.trim().to_lowercase();
        rows.retain(|r| {
            (q.is_empty()
                || r.file.to_lowercase().contains(&q)
                || r.name.to_lowercase().contains(&q))
                && match self.pool.kind {
                    KindFilter::All => true,
                    KindFilter::Video => r.kind == MediaKind::Video,
                    KindFilter::Image => r.kind == MediaKind::Image,
                }
                && match self.pool.usage {
                    UsageFilter::All => true,
                    UsageFilter::Used => r.used,
                    UsageFilter::Unused => !r.used,
                }
        });
        match self.pool.sort {
            SortKey::Added => {}
            SortKey::Name => rows.sort_by_key(|r| r.file.to_lowercase()),
            SortKey::Duration => rows.sort_by(|a, b| a.secs.total_cmp(&b.secs)),
            SortKey::Kind => rows.sort_by_key(|r| (r.kind_label(), r.file.to_lowercase())),
        }
        if self.pool.desc {
            rows.reverse();
        }
        (rows, total)
    }

    fn empty_hint(&self, ui: &mut Ui, filtered: bool) {
        ui.add_space(24.0);
        ui.vertical_centered(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(60.0, 60.0), Sense::hover());
            kit::paw(ui.painter(), rect.center(), 26.0, PEACH_DEEP);
            let msg = if filtered { tr("Nichts passt zu Suche und Filtern.") } else { tr("Noch nichts da – importiere Dateien oder einen ganzen Ordner, oder zieh sie ins Fenster.") };
            ui.label(egui::RichText::new(msg).color(BROWN_SOFT));
        });
    }

    // ----- the three views ---------------------------------------------

    fn thumb_into(&self, ui: &mut Ui, row: &Row, rect: Rect, radius: u8) {
        let p = ui.painter();
        match (row.kind, self.engine.thumb(row.id)) {
            (MediaKind::Audio, _) => {
                p.rect_filled(rect, round(radius), AUDIO_COLOR);
                kit::draw_icon(
                    p,
                    Icon::Note,
                    Rect::from_center_size(rect.center(), rect.size().min_elem() * vec2(0.4, 0.4)),
                    BROWN,
                );
            }
            (_, Some(tex)) => {
                egui::Image::new(tex)
                    .corner_radius(round(radius))
                    .paint_at(ui, rect);
            }
            _ => {
                p.rect_filled(rect, round(radius), BEIGE);
                kit::paw(p, rect.center(), rect.size().min_elem() * 0.18, PEACH_DEEP);
            }
        }
        if row.missing {
            let p = ui.painter();
            p.rect_filled(
                rect,
                round(radius),
                Color32::from_rgba_unmultiplied(255, 240, 240, 215),
            );
        }
    }

    fn tile(&mut self, ui: &mut Ui, row: &Row) {
        let (rect, resp) =
            ui.allocate_exact_size(vec2(CARD_W, THUMB_H + 50.0), Sense::click_and_drag());
        let thumb = Rect::from_min_size(rect.min + vec2(6.0, 6.0), vec2(CARD_W - 12.0, THUMB_H));
        let hovered = resp.hovered();
        {
            let p = ui.painter();
            let fill = if row.used {
                USED_FILL
            } else if hovered {
                PINK_SOFT
            } else {
                CREAM
            };
            p.rect_filled(rect, round(14), fill);
            p.rect_stroke(
                rect,
                round(14),
                if row.used {
                    Stroke::new(3.0, GOLD)
                } else {
                    ink(if hovered { 2.0 } else { 1.5 })
                },
                StrokeKind::Inside,
            );
        }
        self.thumb_into(ui, row, thumb, 10);
        let p = ui.painter();
        // full name, wrapped on two lines
        let galley = p.layout(
            row.file.clone(),
            egui::FontId::proportional(12.5),
            BROWN,
            CARD_W - 18.0,
        );
        p.with_clip_rect(Rect::from_min_max(
            pos2(rect.left() + 4.0, thumb.bottom() + 3.0),
            pos2(rect.right() - 4.0, rect.bottom() - 2.0),
        ))
        .galley(pos2(rect.left() + 9.0, thumb.bottom() + 5.0), galley, BROWN);
        if row.kind != MediaKind::Image {
            let br = Rect::from_min_size(
                pos2(thumb.right() - 48.0, thumb.bottom() - 21.0),
                vec2(44.0, 17.0),
            );
            p.rect_filled(br, round(8), Color32::from_black_alpha(130));
            p.text(
                br.center(),
                Align2::CENTER_CENTER,
                format!("{:.1}s", row.secs),
                egui::FontId::proportional(11.0),
                Color32::WHITE,
            );
        }
        if row.global {
            let g = thumb.right_top() + vec2(-15.0, 15.0);
            p.circle_filled(g, 11.0, CREAM);
            p.circle_stroke(g, 11.0, ink(1.4));
            kit::draw_icon(
                p,
                Icon::Globe,
                Rect::from_center_size(g, vec2(14.0, 14.0)),
                BROWN,
            );
        }
        if row.used {
            // a clearly visible sticker, bottom left of the picture
            let used = tr("verwendet");
            let used_w = p
                .layout_no_wrap(used.to_string(), egui::FontId::proportional(11.5), BROWN)
                .size()
                .x;
            let pill = Rect::from_min_size(
                pos2(thumb.left() + 4.0, thumb.bottom() - 22.0),
                vec2((used_w + 32.0).max(60.0), 18.0),
            );
            p.rect_filled(pill, round(9), GOLD);
            p.rect_stroke(pill, round(9), ink(1.4), StrokeKind::Inside);
            kit::draw_icon(
                p,
                Icon::Check,
                Rect::from_center_size(pill.left_center() + vec2(11.0, 0.0), vec2(12.0, 12.0)),
                BROWN,
            );
            p.text(
                pill.left_center() + vec2(21.0, 0.0),
                Align2::LEFT_CENTER,
                used,
                egui::FontId::proportional(11.5),
                BROWN,
            );
        }
        if row.missing {
            p.text(
                thumb.center(),
                Align2::CENTER_CENTER,
                tr("Datei fehlt"),
                egui::FontId::proportional(13.0),
                Color32::from_rgb(180, 60, 60),
            );
        }
        let plus = Rect::from_center_size(thumb.left_top() + vec2(18.0, 18.0), vec2(26.0, 26.0));
        if hovered && !row.missing {
            p.circle_filled(plus.center(), 13.0, PINK);
            p.circle_stroke(plus.center(), 13.0, ink(1.8));
            kit::draw_icon(
                p,
                Icon::Plus,
                Rect::from_center_size(plus.center(), vec2(15.0, 15.0)),
                BROWN,
            );
        }
        self.row_interact(resp, row, Some(plus));
    }

    fn list_row(&mut self, ui: &mut Ui, row: &Row) {
        let (rect, resp) =
            ui.allocate_exact_size(vec2(ui.available_width(), 46.0), Sense::click_and_drag());
        let hovered = resp.hovered();
        {
            let p = ui.painter();
            let fill = if row.used {
                USED_FILL
            } else if hovered {
                PINK_SOFT
            } else {
                CREAM
            };
            p.rect_filled(rect, round(12), fill);
            p.rect_stroke(
                rect,
                round(12),
                if row.used {
                    Stroke::new(2.5, GOLD)
                } else {
                    ink(if hovered { 1.8 } else { 1.2 })
                },
                StrokeKind::Inside,
            );
        }
        let thumb = Rect::from_min_size(rect.min + vec2(30.0, 5.0), vec2(56.0, 36.0));
        self.thumb_into(ui, row, thumb, 6);
        let p = ui.painter();
        if row.used {
            used_badge(p, rect.left_center() + vec2(16.0, 0.0), 9.0);
        }
        if row.global {
            kit::draw_icon(
                p,
                Icon::Globe,
                Rect::from_center_size(thumb.right_top() + vec2(-2.0, 2.0), vec2(13.0, 13.0)),
                BROWN,
            );
        }
        let x = thumb.right() + 10.0;
        let clip = p.with_clip_rect(Rect::from_min_max(
            pos2(x, rect.top()),
            pos2(rect.right() - 6.0, rect.bottom()),
        ));
        clip.text(
            pos2(x, rect.top() + 14.0),
            Align2::LEFT_CENTER,
            &row.file,
            egui::FontId::proportional(13.5),
            BROWN,
        );
        let sub = if row.missing {
            tr("Datei fehlt").to_string()
        } else {
            row.details()
        };
        clip.text(
            pos2(x, rect.top() + 32.0),
            Align2::LEFT_CENTER,
            sub,
            egui::FontId::proportional(11.5),
            if row.missing {
                Color32::from_rgb(180, 60, 60)
            } else {
                BROWN_SOFT
            },
        );
        self.row_interact(resp, row, None);
    }

    fn table(&mut self, ui: &mut Ui, rows: &[Row]) {
        let width = ui.available_width().max(640.0);
        // column left edges
        let name_w = (width - 86.0 - 56.0 - 64.0 - 92.0 - 140.0).max(170.0);
        let cols = [
            34.0,
            86.0,
            86.0 + name_w,
            86.0 + name_w + 56.0,
            86.0 + name_w + 120.0,
            86.0 + name_w + 212.0,
        ];
        let total_w = cols[5] + 140.0;
        let (head, _) = ui.allocate_exact_size(vec2(total_w, 28.0), Sense::hover());
        ui.painter().rect_filled(head, round(8), BEIGE);
        // headers: clickable to sort
        let header_cells = [
            (cols[1], cols[2], tr("Name"), Some(SortKey::Name)),
            (cols[2], cols[3], tr("Art"), Some(SortKey::Kind)),
            (cols[3], cols[4], tr("Dauer"), Some(SortKey::Duration)),
            (cols[4], cols[5], tr("Größe"), None),
            (cols[5], total_w, tr("Ordner"), None),
        ];
        for (i, (x0, x1, label, key)) in header_cells.into_iter().enumerate() {
            let cell = Rect::from_min_max(
                pos2(head.left() + x0, head.top()),
                pos2(head.left() + x1, head.bottom()),
            );
            let resp = ui.interact(
                cell,
                egui::Id::new(("table-head", i)),
                if key.is_some() {
                    Sense::click()
                } else {
                    Sense::hover()
                },
            );
            let active = key.is_some_and(|k| k == self.pool.sort);
            let p = ui.painter();
            p.text(
                cell.left_center() + vec2(4.0, 0.0),
                Align2::LEFT_CENTER,
                label,
                egui::FontId::proportional(12.5),
                BROWN,
            );
            if active {
                let c = cell.right_center() - vec2(10.0, 0.0);
                let dy = if self.pool.desc { 4.0 } else { -4.0 };
                p.add(Shape::convex_polygon(
                    vec![c + vec2(-5.0, -dy), c + vec2(5.0, -dy), c + vec2(0.0, dy)],
                    BROWN,
                    Stroke::NONE,
                ));
            }
            if let (true, Some(k)) = (resp.clicked(), key) {
                if self.pool.sort == k {
                    self.pool.desc = !self.pool.desc;
                } else {
                    self.pool.sort = k;
                    self.pool.desc = false;
                }
            }
        }
        ui.spacing_mut().item_spacing.y = 3.0;
        for row in rows {
            let (rect, resp) = ui.allocate_exact_size(vec2(total_w, 32.0), Sense::click_and_drag());
            let hovered = resp.hovered();
            {
                let p = ui.painter();
                let fill = if row.used {
                    USED_FILL
                } else if hovered {
                    PINK_SOFT
                } else {
                    CREAM
                };
                p.rect_filled(rect, round(8), fill);
                p.rect_stroke(
                    rect,
                    round(8),
                    if row.used {
                        Stroke::new(2.0, GOLD)
                    } else {
                        ink(1.0)
                    },
                    StrokeKind::Inside,
                );
            }
            let thumb = Rect::from_min_size(rect.min + vec2(cols[0] + 2.0, 4.0), vec2(44.0, 24.0));
            self.thumb_into(ui, row, thumb, 4);
            let p = ui.painter();
            if row.used {
                used_badge(p, rect.left_center() + vec2(15.0, 0.0), 9.0);
            }
            let cell = |x0: f32, x1: f32| {
                p.with_clip_rect(Rect::from_min_max(
                    pos2(rect.left() + x0, rect.top()),
                    pos2(rect.left() + x1 - 4.0, rect.bottom()),
                ))
            };
            let font = egui::FontId::proportional(13.0);
            cell(cols[1], cols[2]).text(
                rect.left_center() + vec2(cols[1] + 4.0, 0.0),
                Align2::LEFT_CENTER,
                &row.file,
                font.clone(),
                BROWN,
            );
            type_badge(
                p,
                row.kind,
                Rect::from_center_size(
                    rect.left_center() + vec2(cols[2] + 12.0, 0.0),
                    vec2(20.0, 20.0),
                ),
            );
            cell(cols[2], cols[3]).text(
                rect.left_center() + vec2(cols[2] + 28.0, 0.0),
                Align2::LEFT_CENTER,
                row.kind_label(),
                font.clone(),
                BROWN_SOFT,
            );
            let dur = if row.kind == MediaKind::Image {
                "–".to_string()
            } else {
                clock(row.secs)
            };
            cell(cols[3], cols[4]).text(
                rect.left_center() + vec2(cols[3] + 4.0, 0.0),
                Align2::LEFT_CENTER,
                dur,
                font.clone(),
                BROWN,
            );
            let size = if row.size.0 > 0 {
                format!("{}×{}", row.size.0, row.size.1)
            } else {
                "–".into()
            };
            cell(cols[4], cols[5]).text(
                rect.left_center() + vec2(cols[4] + 4.0, 0.0),
                Align2::LEFT_CENTER,
                size,
                font.clone(),
                BROWN,
            );
            let folder = if row.missing {
                tr("Datei fehlt").to_string()
            } else {
                row.folder.clone()
            };
            cell(cols[5], total_w).text(
                rect.left_center() + vec2(cols[5] + 4.0, 0.0),
                Align2::LEFT_CENTER,
                folder,
                font,
                if row.missing {
                    Color32::from_rgb(180, 60, 60)
                } else {
                    BROWN_SOFT
                },
            );
            self.row_interact(resp, row, None);
        }
    }

    /// Drag, double-click, "+" click and the context menu shared by all three views.
    fn row_interact(&mut self, resp: Response, row: &Row, plus: Option<Rect>) {
        if !row.missing {
            resp.dnd_set_drag_payload(if row.global {
                Payload::LibMedia(row.id)
            } else {
                Payload::Media(row.id)
            });
            let on_plus = plus.is_some_and(|pl| {
                resp.clicked() && resp.interact_pointer_pos().is_some_and(|p| pl.contains(p))
            });
            if on_plus || resp.double_clicked() {
                let pid = if row.global {
                    self.use_lib_item(row.id)
                } else {
                    Some(row.id)
                };
                if let Some(pid) = pid {
                    self.append_media(pid);
                }
            }
        }
        let (id, global, path, name, kind, secs, size, alpha) = (
            row.id,
            row.global,
            row.path.clone(),
            row.name.clone(),
            row.kind,
            row.secs,
            row.size,
            row.alpha,
        );
        resp.context_menu(|ui| {
            if global {
                if ui.button(tr("In dieses Projekt übernehmen")).clicked() {
                    self.use_lib_item(id);
                    ui.close();
                }
                if kind == MediaKind::Image
                    && ui.button(tr("Als Grafik über das Video legen")).clicked()
                {
                    if let Some(pid) = self.use_lib_item(id) {
                        self.add_overlay_at_playhead(pid);
                    }
                    ui.close();
                }
                if ui.button(tr("Aus der Bibliothek entfernen")).clicked() {
                    self.lib.remove_item(id);
                    self.save_library();
                    ui.close();
                }
            } else {
                if let Some(path) = path.clone()
                    && ui
                        .button(tr("In die globale Bibliothek kopieren"))
                        .clicked()
                {
                    let lid =
                        self.lib
                            .add_item(name.clone(), path, kind, Tick::from_secs(secs), size);
                    self.lib.set_alpha(lid, alpha);
                    self.save_library();
                    self.say("In der globalen Bibliothek gespeichert");
                    ui.close();
                }
                if kind == MediaKind::Image
                    && ui.button(tr("Als Grafik über das Video legen")).clicked()
                {
                    self.add_overlay_at_playhead(id);
                    ui.close();
                }
                if ui.button(tr("Aus dem Projekt entfernen")).clicked() {
                    if let Err(e) = self.doc.edit(|p| p.remove_media(id)) {
                        self.say(e.to_string());
                    }
                    ui.close();
                }
            }
        });
        let tip = match &row.path {
            Some(p) => format!("{}\n{}", row.file, p.display()),
            None => row.file.clone(),
        };
        resp.on_hover_text(tip);
        let _ = ellipsize;
    }
}

fn folder_of(path: Option<&std::path::Path>) -> String {
    let Some(parent) = path.and_then(|p| p.parent()) else {
        return tr("Beispiel").into();
    };
    let parts: Vec<String> = parent
        .components()
        .rev()
        .take(2)
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    parts.into_iter().rev().collect::<Vec<_>>().join("/")
}
