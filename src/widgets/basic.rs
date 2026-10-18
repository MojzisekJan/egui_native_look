//! Push buttons, switches, check boxes and radio buttons.

use egui::{Color32, CornerRadius, Rect, Response, Sense, Stroke, Ui, Widget, WidgetText};

use super::Ctx;
use crate::{lerp_color, Look};

/// A push button, as tall as the controls around it and as wide as its label (or
/// [`Button::width`]).
pub struct Button {
    label: String,
    accent: bool,
    width: Option<f32>,
}

impl Button {
    /// A button reading `label`.
    pub fn new(label: impl Into<String>) -> Self {
        Button { label: label.into(), accent: false, width: None }
    }
    /// The default action of a dialog, or a toggle that is on: in the accent.
    pub fn accent(mut self) -> Self {
        self.accent = true;
        self
    }
    /// In the accent when `on`.
    pub fn selected(mut self, on: bool) -> Self {
        self.accent = on;
        self
    }
    /// A fixed width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl Widget for Button {
    fn ui(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        let height = ui.spacing().interact_size.y;
        let text_width = ui.fonts_mut(|f| f.layout_no_wrap(self.label.clone(), x.m.body.clone(), x.c.fg).size().x);
        let width = self.width.unwrap_or((text_width + 2.0 * ui.spacing().button_padding.x.max(10.0)).max(height));
        let enabled = ui.is_enabled();
        let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), if enabled { Sense::click() } else { Sense::hover() });
        if ui.is_rect_visible(rect) {
            crate::paint::button(ui.painter(), &x.paint(), rect, &response, &self.label, x.m.body.clone(), enabled, self.accent);
        }
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, &self.label));
        response
    }
}

/// An on/off switch. The value has changed when `response.changed()`.
pub struct Switch<'a> {
    on: &'a mut bool,
    label: String,
}

impl<'a> Switch<'a> {
    /// A switch showing and flipping `on`.
    pub fn new(on: &'a mut bool) -> Self {
        Switch { on, label: String::new() }
    }
    /// What it switches, for a screen reader: the label of its row, which a switch does
    /// not draw itself.
    pub fn accessible_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
}

/// The width a [`Switch`] takes in `look`, for a row that leaves room for one at its end.
pub fn switch_width(look: Look) -> f32 {
    switch_size(look).x + 2.0
}

/// The size of a switch's pill in `look`.
pub(crate) fn switch_size(look: Look) -> egui::Vec2 {
    match look {
        Look::Mac => egui::vec2(32.0, 18.0),
        Look::Windows => egui::vec2(36.0, 18.0),
        Look::Gtk => egui::vec2(40.0, 22.0),
        Look::Ios => egui::vec2(51.0, 31.0),
        Look::Material => egui::vec2(52.0, 32.0),
        Look::Plain => egui::vec2(34.0, 18.0),
    }
}

impl Widget for Switch<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        let c = &x.c;
        let size = switch_size(x.look);
        let height = ui.spacing().interact_size.y.max(size.y);
        let (outer, mut response) = ui.allocate_exact_size(egui::vec2(size.x + 2.0, height), Sense::click());
        let rect = Rect::from_center_size(outer.center(), size);
        if response.clicked() {
            *self.on = !*self.on;
            response.mark_changed();
        }
        let on = *self.on;
        let enabled = ui.is_enabled();
        response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, enabled, on, &self.label));
        if !ui.is_rect_visible(rect) {
            return response;
        }
        let t = ui.ctx().animate_bool_responsive(response.id, on);
        let painter = ui.painter();
        let round = CornerRadius::same((size.y / 2.0) as u8);
        let hovered = enabled && response.hovered();
        match x.look {
            Look::Gtk => {
                // A pill in a tint of the text colour, the accent when on, with a white
                // knob inset by three points and a soft shadow under it.
                let off = if hovered { c.button_hover } else { c.track };
                let mut fill = lerp_color(off, c.accent, t);
                if !enabled {
                    fill = fill.gamma_multiply(0.5);
                }
                painter.rect_filled(rect, round, fill);
                knob(painter, rect, t, 3.0, enabled, c.dark, 1.0);
            }
            Look::Ios => {
                // UIKit's switch: systemGreen when on, a white knob inset by two points.
                let green = crate::colors::rgb(if c.dark { 0x30d158 } else { 0x34c759 });
                let mut fill = lerp_color(c.track, green, t);
                if !enabled {
                    fill = fill.gamma_multiply(0.5);
                }
                painter.rect_filled(rect, round, fill);
                knob(painter, rect, t, 2.0, enabled, c.dark, 1.5);
            }
            Look::Material => {
                // Material 3: an outlined track whose thumb grows from 16 to 24 points and
                // takes the primary's text colour when on.
                let mut fill = lerp_color(c.track, c.accent, t);
                if !enabled {
                    fill = fill.gamma_multiply(0.38);
                }
                painter.rect_filled(rect, round, fill);
                if t < 1.0 {
                    painter.rect_stroke(rect, round, Stroke::new(2.0, c.control_strong.gamma_multiply(1.0 - t)), egui::StrokeKind::Inside);
                }
                let r = egui::lerp(8.0..=12.0, t) + if hovered { 1.0 } else { 0.0 };
                let x = egui::lerp((rect.left() + 16.0)..=(rect.right() - 16.0), t);
                painter.circle_filled(egui::pos2(x, rect.center().y), r, lerp_color(c.control_strong, c.on_accent, t));
            }
            Look::Mac => {
                // A white knob all but filling the height, over a light grey track when
                // off and the accent when on.
                let mut fill = lerp_color(c.track, c.accent, t);
                if !enabled {
                    fill = fill.gamma_multiply(0.55);
                }
                painter.rect_filled(rect, round, fill);
                if t < 1.0 {
                    let edge = if c.dark { Color32::from_white_alpha(14) } else { Color32::from_black_alpha(14) };
                    painter.rect_stroke(rect, round, Stroke::new(1.0, edge.gamma_multiply(1.0 - t)), egui::StrokeKind::Inside);
                }
                knob(painter, rect, t, 1.5, enabled, c.dark, 0.5);
            }
            Look::Windows => {
                // An outlined pill with a small knob in the strong stroke's colour when
                // off, the accent with a knob in the accent's text colour when on; the
                // knob grows under the pointer.
                let strong = if enabled { c.control_strong } else { c.disabled };
                if t > 0.0 {
                    let fill = if hovered { c.accent_hover } else { c.accent };
                    painter.rect_filled(rect, round, if enabled { fill } else { c.disabled }.gamma_multiply(t));
                }
                if t < 1.0 {
                    painter.rect_filled(rect, round, c.track.gamma_multiply(1.0 - t));
                    painter.rect_stroke(rect, round, Stroke::new(1.0, strong.gamma_multiply(1.0 - t)), egui::StrokeKind::Inside);
                }
                let r = if hovered { 7.0 } else { 6.0 };
                let x = egui::lerp((rect.left() + 9.0)..=(rect.right() - 9.0), t);
                painter.circle_filled(egui::pos2(x, rect.center().y), r, lerp_color(strong, c.on_accent, t));
            }
            Look::Plain => {
                let off = if hovered { c.button_hover } else { c.button };
                let on_fill = if hovered { c.accent_hover } else { c.accent };
                let mut fill = lerp_color(off, on_fill, t);
                if !enabled {
                    fill = fill.gamma_multiply(0.5);
                }
                painter.rect_filled(rect, round, fill);
                if t < 0.5 {
                    painter.rect_stroke(rect, round, Stroke::new(1.0, c.grid), egui::StrokeKind::Inside);
                }
                let r = size.y / 2.0 - 3.0;
                let x = egui::lerp((rect.left() + 3.0 + r)..=(rect.right() - 3.0 - r), t);
                let fill = lerp_color(c.dim, Color32::WHITE, t);
                painter.circle_filled(egui::pos2(x, rect.center().y), r, if enabled { fill } else { c.disabled });
            }
        }
        if response.has_focus() {
            crate::paint::ring(painter, &x.paint(), rect, round);
        }
        response
    }
}

/// A white switch knob inset by `inset`, with a shadow `drop` points below it.
fn knob(painter: &egui::Painter, rect: Rect, t: f32, inset: f32, enabled: bool, dark: bool, drop: f32) {
    let r = rect.height() / 2.0 - inset;
    let x = egui::lerp((rect.left() + inset + r)..=(rect.right() - inset - r), t);
    let c = egui::pos2(x, rect.center().y);
    painter.circle_filled(c + egui::vec2(0.0, drop), r + 0.5, Color32::from_black_alpha(if enabled { 48 } else { 20 }));
    painter.circle_filled(c, r, if enabled { Color32::WHITE } else { Color32::from_gray(if dark { 145 } else { 245 }) });
}

/// A check box with its label: on macOS a small rounded box, on Windows WinUI's larger
/// outlined one, in libadwaita a box outlined in a tint of the text colour, each filled
/// with the accent and ticked when on; egui's own in the plain look. An indeterminate one
/// shows a dash, for a mixed selection.
pub struct Checkbox<'a> {
    checked: &'a mut bool,
    text: WidgetText,
    indeterminate: bool,
}

impl<'a> Checkbox<'a> {
    /// A check box showing and flipping `checked`, with `text` after it.
    pub fn new(checked: &'a mut bool, text: impl Into<WidgetText>) -> Self {
        Checkbox { checked, text: text.into(), indeterminate: false }
    }
    /// Shows a dash instead of a tick or nothing: some of what it stands for is on.
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }
}

impl Widget for Checkbox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        if !x.look.native() {
            return ui.add(egui::Checkbox::new(self.checked, self.text).indeterminate(self.indeterminate));
        }
        let (mut response, mark) = mark_row(ui, &x, self.text, egui::WidgetType::Checkbox, *self.checked);
        if response.clicked() {
            *self.checked = !*self.checked;
            response.mark_changed();
        }
        let painter = ui.painter();
        let round = match x.look {
            Look::Ios => CornerRadius::same((mark.width() / 2.0) as u8),
            Look::Material => CornerRadius::same(2),
            _ => CornerRadius::same(4),
        };
        let enabled = ui.is_enabled();
        let c = &x.c;
        if *self.checked || self.indeterminate {
            painter.rect_filled(mark, round, if enabled { c.accent } else { c.accent.gamma_multiply(0.5) });
            if self.indeterminate {
                let m = mark.center();
                painter.line_segment([m - egui::vec2(3.5, 0.0), m + egui::vec2(3.5, 0.0)], Stroke::new(1.8, c.on_accent));
            } else {
                check_mark(painter, mark.center(), c.on_accent);
            }
        } else {
            paint_mark_box(painter, &x, mark, round, enabled, response.hovered());
        }
        if response.has_focus() {
            crate::paint::ring(painter, &x.paint(), mark, round);
        }
        response
    }
}

/// A radio button, chosen or not: in the system looks a circle filled with the accent and
/// a dot in the accent's text colour when chosen; egui's own in the plain look. See
/// [`radio_value`] for the usual way to use it.
pub struct Radio {
    selected: bool,
    text: WidgetText,
}

impl Radio {
    /// A radio button, chosen when `selected`, with `text` after it; see also
    /// [`radio_value`].
    pub fn new(selected: bool, text: impl Into<WidgetText>) -> Self {
        Radio { selected, text: text.into() }
    }
}

impl Widget for Radio {
    fn ui(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        if !x.look.native() {
            return ui.radio(self.selected, self.text);
        }
        let (response, mark) = mark_row(ui, &x, self.text, egui::WidgetType::RadioButton, self.selected);
        let painter = ui.painter();
        let round = CornerRadius::same((mark.width() / 2.0) as u8);
        let enabled = ui.is_enabled();
        let c = &x.c;
        if self.selected && x.look == Look::Material {
            // Material 3: a ring and a dot, both in the primary.
            let colour = if enabled { c.accent } else { c.disabled };
            painter.circle_stroke(mark.center(), mark.width() / 2.0 - 1.0, Stroke::new(2.0, colour));
            painter.circle_filled(mark.center(), 5.0, colour);
        } else if self.selected && x.look == Look::Ios {
            // iOS marks a chosen option with a tick in an accent circle.
            painter.circle_filled(mark.center(), mark.width() / 2.0, if enabled { c.accent } else { c.accent.gamma_multiply(0.5) });
            check_mark(painter, mark.center(), c.on_accent);
        } else if self.selected {
            painter.circle_filled(mark.center(), mark.width() / 2.0, if enabled { c.accent } else { c.accent.gamma_multiply(0.5) });
            // WinUI's dot is wide and grows under the pointer; the others keep a small one.
            let dot = if x.look == Look::Windows { if response.hovered() { 5.0 } else { 4.5 } } else { 3.0 };
            painter.circle_filled(mark.center(), dot, c.on_accent);
        } else {
            paint_mark_box(painter, &x, mark, round, enabled, response.hovered());
        }
        if response.has_focus() {
            crate::paint::ring(painter, &x.paint(), mark, round);
        }
        response
    }
}

/// A [`Radio`] setting `current` to `value` when clicked.
pub fn radio_value<T: PartialEq>(ui: &mut Ui, current: &mut T, value: T, text: impl Into<WidgetText>) -> Response {
    let mut response = ui.add(Radio::new(*current == value, text));
    if response.clicked() && *current != value {
        *current = value;
        response.mark_changed();
    }
    response
}

/// A tick centred on `c`.
pub fn check_mark(painter: &egui::Painter, c: egui::Pos2, color: Color32) {
    painter.add(egui::Shape::line(vec![c + egui::vec2(-4.0, 0.0), c + egui::vec2(-1.2, 3.0), c + egui::vec2(4.0, -3.5)], Stroke::new(1.6, color)));
}

/// The row of a check box or radio button: the mark (14 points, 18 on Windows), then the
/// label, all of it one click target. The label wraps as the `Ui` wraps its text, the mark
/// beside its first line. Returns the response and the mark's square.
fn mark_row(ui: &mut Ui, x: &Ctx, text: WidgetText, kind: egui::WidgetType, selected: bool) -> (Response, Rect) {
    let (side, gap) = match x.look {
        Look::Windows => (18.0, 8.0),
        Look::Ios => (22.0, 10.0),
        Look::Material => (18.0, 12.0),
        _ => (14.0, 6.0),
    };
    let galley = text.into_galley(ui, None, ui.available_width() - side - gap, egui::FontSelection::FontId(x.m.body.clone()));
    let width = side + gap + galley.size().x;
    let line = ui.spacing().interact_size.y.max(side);
    let first_row = galley.rows.first().map_or(galley.size().y, |row| row.height());
    let height = line.max(galley.size().y + line - first_row);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), Sense::click());
    let middle = rect.top() + line / 2.0;
    let mark = Rect::from_center_size(egui::pos2(rect.left() + side / 2.0, middle), egui::vec2(side, side));
    let enabled = ui.is_enabled();
    let colour = if enabled { x.c.fg } else { x.c.disabled };
    let top = crate::galley_top(ui.ctx(), &galley, middle);
    response.widget_info(|| egui::WidgetInfo::selected(kind, enabled, selected, galley.text()));
    ui.painter().galley(egui::pos2(mark.right() + gap, top), galley, colour);
    (response, mark)
}

/// An unchecked mark: on macOS raised and white on the light scheme, a faint well on the
/// dark; on Windows a faint fill inside the strong stroke; in libadwaita a two-point
/// outline in a tint of the text colour.
fn paint_mark_box(painter: &egui::Painter, x: &Ctx, mark: Rect, round: CornerRadius, enabled: bool, hovered: bool) {
    let c = &x.c;
    match x.look {
        Look::Gtk => {
            let tint = if hovered { c.button_pressed } else { c.button_hover };
            painter.rect_stroke(mark, round, Stroke::new(2.0, if enabled { tint } else { c.button_disabled }), egui::StrokeKind::Inside);
        }
        Look::Ios | Look::Material => {
            // An outline in the strong stroke: two points thick in Material, one and a half
            // on iOS; Material lays a state layer over it under the pointer.
            if hovered && x.look == Look::Material {
                painter.circle_filled(mark.center(), mark.width() * 1.1, c.fg.gamma_multiply(0.08));
            }
            let width = if x.look == Look::Material { 2.0 } else { 1.5 };
            painter.rect_stroke(mark, round, Stroke::new(width, if enabled { c.control_strong } else { c.disabled }), egui::StrokeKind::Inside);
        }
        Look::Windows => {
            let fill = if c.dark { Color32::from_black_alpha(if hovered { 40 } else { 26 }) } else { Color32::from_black_alpha(if hovered { 14 } else { 6 }) };
            painter.rect_filled(mark, round, fill);
            painter.rect_stroke(mark, round, Stroke::new(1.0, if enabled { c.control_strong } else { c.disabled }), egui::StrokeKind::Inside);
        }
        _ if c.dark => {
            painter.rect_filled(mark, round, Color32::from_white_alpha(if enabled { 22 } else { 10 }));
            painter.rect_stroke(mark, round, Stroke::new(1.0, Color32::from_white_alpha(if enabled { 30 } else { 14 })), egui::StrokeKind::Inside);
        }
        _ => crate::paint::raised(painter, x.c.dark, mark, round, if enabled { Color32::WHITE } else { c.button_disabled }, enabled),
    }
}
