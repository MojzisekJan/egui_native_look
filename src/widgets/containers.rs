//! Sections, list rows and menu items.

use egui::{CornerRadius, Rect, Response, Sense, Stroke, Ui, Widget};

use super::Ctx;
use crate::Look;

/// A section that folds: a header with the look's disclosure chevron (turning on macOS
/// and in libadwaita, a caret on Windows) and the title, and below it `contents` while it
/// is open. Returns what `contents` returned, while open.
pub fn section<R>(ui: &mut Ui, id: impl std::hash::Hash + std::fmt::Debug, title: &str, default_open: bool, contents: impl FnOnce(&mut Ui) -> R) -> Option<R> {
    let x = Ctx::of(ui);
    let c = x.c;
    let id = ui.make_persistent_id(id);
    let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, default_open);
    let height = ui.spacing().interact_size.y;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(ui.available_width(), height), Sense::click());
    if response.clicked() {
        state.toggle(ui);
    }
    let openness = state.openness(ui.ctx());
    if response.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(x.m.radius), c.row_hover);
    }
    let m = egui::pos2(rect.left() + 12.0, rect.center().y);
    let colour = if response.hovered() { c.fg } else { c.dim };
    // A chevron pointing right, turned down as the section opens.
    let angle = openness * std::f32::consts::FRAC_PI_2;
    let turn = |v: egui::Vec2| egui::vec2(v.x * angle.cos() - v.y * angle.sin(), v.x * angle.sin() + v.y * angle.cos());
    let size = if x.look == Look::Windows { 3.5 } else { 3.0 };
    let points = [egui::vec2(-size * 0.6, -size), egui::vec2(size * 0.6, 0.0), egui::vec2(-size * 0.6, size)].map(|v| m + turn(v));
    ui.painter().add(egui::Shape::line(points.to_vec(), Stroke::new(1.5, colour)));
    crate::text(ui.painter(), egui::pos2(rect.left() + 26.0, rect.center().y), egui::Align2::LEFT_CENTER, title, x.m.bold.clone(), c.fg);
    if response.has_focus() {
        crate::paint::ring(ui.painter(), &x.paint(), rect, CornerRadius::same(x.m.radius));
    }
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::CollapsingHeader, ui.is_enabled(), title));
    state.store(ui.ctx());
    let shown = state.show_body_unindented(ui, contents);
    shown.map(|body| body.inner)
}

/// A row of a list, the width available and `height` tall: picked rows in the look's
/// selection (the accent on macOS, a tint with an accent bar on Windows, a tint in
/// libadwaita and the plain look), the row under the pointer lit. Draw the row's content
/// into `response.rect`.
pub struct ListRow {
    picked: bool,
    height: f32,
}

impl ListRow {
    pub fn new(picked: bool) -> Self {
        ListRow { picked, height: 26.0 }
    }
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl Widget for ListRow {
    fn ui(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        let c = x.c;
        let (rect, response) = ui.allocate_exact_size(egui::vec2(ui.available_width(), self.height), Sense::click());
        let round = CornerRadius::same(if x.look == Look::Plain { 3 } else { 5 });
        if self.picked {
            match x.look {
                Look::Mac => {
                    ui.painter().rect_filled(rect, round, c.accent);
                }
                Look::Windows => {
                    ui.painter().rect_filled(rect, round, c.button_hover);
                    let bar = Rect::from_center_size(egui::pos2(rect.left() + 2.0, rect.center().y), egui::vec2(3.0, rect.height() * 0.5));
                    ui.painter().rect_filled(bar, CornerRadius::same(2), c.accent);
                }
                Look::Material => {
                    // Material's chosen item: the tonal container, rounded to a pill.
                    ui.painter().rect_filled(rect, CornerRadius::same((rect.height() / 2.0) as u8), c.button);
                }
                Look::Gtk | Look::Ios | Look::Plain => {
                    ui.painter().rect_filled(rect, round, c.accent.gamma_multiply(if c.dark { 0.38 } else { 0.24 }));
                }
            }
        } else if response.hovered() {
            ui.painter().rect_filled(rect, round, c.row_hover);
        }
        response
    }
}

/// The colour of text on a picked [`ListRow`] in the look in force: white on macOS's
/// accent, the ordinary text colour elsewhere.
pub fn picked_text(ui: &Ui) -> egui::Color32 {
    let x = Ctx::of(ui);
    if x.look == Look::Mac { x.c.on_accent } else { x.c.fg }
}

/// An item of a menu: an icon or a tick at its start, its label, and its shortcut at its
/// end; the row under the pointer in the accent on macOS and in the plain look, in a tint
/// on Windows and in libadwaita.
pub struct MenuItem {
    label: String,
    shortcut: String,
    icon: Option<(String, egui::FontId)>,
    checked: bool,
}

impl MenuItem {
    pub fn new(label: impl Into<String>) -> Self {
        MenuItem { label: label.into(), shortcut: String::new(), icon: None, checked: false }
    }
    /// The shortcut, as text (see [`crate::shortcut_text`]).
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = shortcut.into();
        self
    }
    /// An icon glyph in `font`, an icon font's.
    pub fn icon(mut self, glyph: impl Into<String>, font: egui::FontId) -> Self {
        self.icon = Some((glyph.into(), font));
        self
    }
    /// A tick at its start.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }
}

impl Widget for MenuItem {
    fn ui(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        let c = x.c;
        let height = (ui.spacing().interact_size.y - 4.0).max(22.0);
        let font = x.m.body.clone();
        let label_width = ui.fonts_mut(|f| f.layout_no_wrap(self.label.clone(), font.clone(), c.fg).size().x);
        let shortcut_width = if self.shortcut.is_empty() { 0.0 } else { ui.fonts_mut(|f| f.layout_no_wrap(self.shortcut.clone(), font.clone(), c.fg).size().x) + 24.0 };
        let width = ui.available_width().max(28.0 + label_width + shortcut_width + 12.0);
        let enabled = ui.is_enabled();
        let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), if enabled { Sense::click() } else { Sense::hover() });
        let hot = enabled && response.hovered();
        let accent_row = matches!(x.look, Look::Mac | Look::Plain);
        if hot {
            ui.painter().rect_filled(rect, CornerRadius::same(4), if accent_row { c.accent } else { c.button_hover });
        }
        let text = if !enabled { c.disabled } else if hot && accent_row { c.on_accent } else { c.fg };
        let weak = if !enabled { c.disabled } else if hot && accent_row { c.on_accent.gamma_multiply(0.8) } else { c.dim };
        let column = egui::pos2(rect.left() + 13.0, rect.center().y);
        if self.checked {
            super::basic::check_mark(ui.painter(), column, text);
        } else if let Some((glyph, icon_font)) = &self.icon {
            crate::text(ui.painter(), column, egui::Align2::CENTER_CENTER, glyph, icon_font.clone(), weak);
        }
        crate::text(ui.painter(), egui::pos2(rect.left() + 28.0, rect.center().y), egui::Align2::LEFT_CENTER, &self.label, font.clone(), text);
        if !self.shortcut.is_empty() {
            crate::text(ui.painter(), egui::pos2(rect.right() - 10.0, rect.center().y), egui::Align2::RIGHT_CENTER, &self.shortcut, font, weak);
        }
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, &self.label));
        response
    }
}
