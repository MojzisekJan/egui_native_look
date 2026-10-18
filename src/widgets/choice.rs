//! Choosing: segmented choices, pop-up buttons with their list, sliders and tabs.

use egui::{Color32, CornerRadius, Rect, Response, Sense, Stroke, Ui, Widget};

use super::{truncated, Ctx};
use crate::Look;

/// A choice of one of `items` in equal segments: in the system looks a grey track with the
/// chosen segment raised out of it; in the plain look the chosen segment in the accent.
/// `response.changed()` when the choice changed.
pub struct Segmented<'a> {
    id: egui::Id,
    selected: &'a mut usize,
    items: Vec<(String, String)>,
    width: Option<f32>,
}

impl<'a> Segmented<'a> {
    /// `items` are labels; [`Segmented::tooltips`] adds a tooltip to each.
    pub fn new(id: impl std::hash::Hash + std::fmt::Debug, selected: &'a mut usize, items: &[&str]) -> Self {
        Segmented { id: egui::Id::new(id), selected, items: items.iter().map(|s| (s.to_string(), String::new())).collect(), width: None }
    }
    /// A tooltip for each item, in order.
    pub fn tooltips(mut self, tips: &[&str]) -> Self {
        for (item, tip) in self.items.iter_mut().zip(tips) {
            item.1 = tip.to_string();
        }
        self
    }
    /// The whole control's width; by default the width available.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl Widget for Segmented<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        let c = &x.c;
        let before = *self.selected;
        let height = ui.spacing().interact_size.y;
        let width = self.width.unwrap_or_else(|| ui.available_width());
        let (rect, mut response) = ui.allocate_exact_size(egui::vec2(width, height), Sense::hover());
        let n = self.items.len().max(1);
        let each = rect.width() / n as f32;
        let id = ui.id().with(("segmented", self.id));
        let enabled = ui.is_enabled();
        let r = x.m.radius;
        if x.look == Look::Material {
            return material_segments(ui, &x, rect, response, id, self.selected, &self.items, before);
        }
        let system = x.look.native();
        if system {
            ui.painter().rect_filled(rect, CornerRadius::same(r + 1), c.track);
            for i in 1..n {
                if i != *self.selected && i != *self.selected + 1 {
                    let left = (rect.left() + each * i as f32).round() + 0.5;
                    ui.painter().line_segment([egui::pos2(left, rect.top() + 7.0), egui::pos2(left, rect.bottom() - 7.0)], Stroke::new(1.0, c.control_border));
                }
            }
        }
        for (i, (label, tip)) in self.items.iter().enumerate() {
            let slot = Rect::from_min_size(egui::pos2(rect.left() + each * i as f32, rect.top()), egui::vec2(each, rect.height()));
            let segment = ui.interact(slot, id.with(i), if enabled { Sense::click() } else { Sense::hover() });
            let on = i == *self.selected;
            let colour = if system {
                if on {
                    let fill = if enabled { c.raised } else { c.raised.gamma_multiply(0.6) };
                    let inner = CornerRadius::same(r.saturating_sub(1));
                    let body = slot.shrink(2.0);
                    match x.look {
                        Look::Windows => crate::paint::fluent(ui.painter(), &x.c, body, inner, fill, enabled),
                        Look::Gtk => {
                            // libadwaita's toggle group raises its active toggle without an edge.
                            ui.painter().rect_filled(body.translate(egui::vec2(0.0, 1.0)), inner, Color32::from_black_alpha(if c.dark { 50 } else { 20 }));
                            ui.painter().rect_filled(body, inner, fill);
                        }
                        _ => crate::paint::raised(ui.painter(), x.c.dark, body, inner, fill, enabled),
                    }
                } else if segment.hovered() && enabled {
                    ui.painter().rect_filled(slot.shrink(2.0), CornerRadius::same(r.saturating_sub(1)), c.row_hover);
                }
                if enabled { c.fg } else { c.disabled }
            } else {
                let round = CornerRadius { nw: if i == 0 { r } else { 0 }, sw: if i == 0 { r } else { 0 }, ne: if i + 1 == n { r } else { 0 }, se: if i + 1 == n { r } else { 0 } };
                let fill = match (on, segment.hovered() && enabled) {
                    (true, true) => c.accent_hover,
                    (true, false) => c.accent,
                    (false, true) => c.button_hover,
                    (false, false) => c.button,
                };
                ui.painter().rect_filled(slot, round, if enabled { fill } else { fill.gamma_multiply(0.5) });
                if !enabled { c.disabled } else if on { c.on_accent } else { c.fg }
            };
            let font = if on && system { x.m.bold.clone() } else { x.m.body.clone() };
            let (galley, cut) = truncated(ui, label, font, colour, slot.width() - 10.0);
            let top = crate::galley_top(ui.ctx(), &galley, slot.center().y);
            ui.painter().galley(egui::pos2(slot.center().x - galley.size().x / 2.0, top), galley, colour);
            let hover = match (cut, tip.is_empty()) {
                (true, false) => format!("{label}\n\n{tip}"),
                (true, true) => label.clone(),
                (false, _) => tip.clone(),
            };
            let segment = if hover.is_empty() { segment } else { segment.on_hover_text(&hover) };
            if segment.clicked() {
                *self.selected = i;
            }
            if segment.has_focus() {
                crate::paint::ring(ui.painter(), &x.paint(), slot.shrink(2.0), CornerRadius::same(r.saturating_sub(1)));
            }
        }
        if !system {
            ui.painter().rect_stroke(rect, CornerRadius::same(r), Stroke::new(1.0, c.border), egui::StrokeKind::Inside);
        }
        if *self.selected != before {
            response.mark_changed();
        }
        response
    }
}

/// Material 3's segmented buttons in `rect`: an outlined pill split by thin lines, the
/// chosen segment in the tonal container with a tick before its label.
#[allow(clippy::too_many_arguments)]
fn material_segments(ui: &mut Ui, x: &Ctx, rect: Rect, mut response: Response, id: egui::Id, selected: &mut usize, items: &[(String, String)], before: usize) -> Response {
    let c = &x.c;
    let n = items.len().max(1);
    let each = rect.width() / n as f32;
    let pill = (rect.height() / 2.0) as u8;
    let enabled = ui.is_enabled();
    for (i, (label, tip)) in items.iter().enumerate() {
        let slot = Rect::from_min_size(egui::pos2(rect.left() + each * i as f32, rect.top()), egui::vec2(each, rect.height()));
        let segment = ui.interact(slot, id.with(i), if enabled { Sense::click() } else { Sense::hover() });
        let round = CornerRadius { nw: if i == 0 { pill } else { 0 }, sw: if i == 0 { pill } else { 0 }, ne: if i + 1 == n { pill } else { 0 }, se: if i + 1 == n { pill } else { 0 } };
        let on = i == *selected;
        if on {
            ui.painter().rect_filled(slot, round, c.button);
        } else if segment.hovered() && enabled {
            ui.painter().rect_filled(slot, round, c.fg.gamma_multiply(0.08));
        }
        let colour = if enabled { c.fg } else { c.disabled };
        let (galley, _) = truncated(ui, label, x.m.body.clone(), colour, slot.width() - 30.0);
        let tick = if on { 18.0 } else { 0.0 };
        let left = slot.center().x - (galley.size().x + tick) / 2.0;
        if on {
            super::basic::check_mark(ui.painter(), egui::pos2(left + 7.0, slot.center().y), colour);
        }
        let top = crate::galley_top(ui.ctx(), &galley, slot.center().y);
        ui.painter().galley(egui::pos2(left + tick, top), galley, colour);
        if i > 0 {
            ui.painter().line_segment([egui::pos2(slot.left(), slot.top()), egui::pos2(slot.left(), slot.bottom())], Stroke::new(1.0, c.control_strong));
        }
        let segment = if tip.is_empty() { segment } else { segment.on_hover_text(tip) };
        if segment.clicked() {
            *selected = i;
        }
    }
    ui.painter().rect_stroke(rect, CornerRadius::same(pill), Stroke::new(1.0, c.control_strong), egui::StrokeKind::Inside);
    if *selected != before {
        response.mark_changed();
    }
    response
}

/// A pop-up button with its list: the chosen item on the button, the items in a list
/// under it (or over it, where there is more room). `response.changed()` when the choice
/// changed.
pub struct Dropdown<'a> {
    id: egui::Id,
    selected: &'a mut usize,
    items: Vec<String>,
    width: Option<f32>,
}

impl<'a> Dropdown<'a> {
    pub fn new(id: impl std::hash::Hash + std::fmt::Debug, selected: &'a mut usize, items: &[&str]) -> Self {
        Dropdown { id: egui::Id::new(id), selected, items: items.iter().map(|s| s.to_string()).collect(), width: None }
    }
    /// The button's width; by default its longest item's.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl Widget for Dropdown<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        let c = x.c;
        let before = *self.selected;
        let height = ui.spacing().interact_size.y;
        let font = x.m.body.clone();
        let longest = self.items.iter().map(|t| ui.fonts_mut(|f| f.layout_no_wrap(t.clone(), font.clone(), c.fg).size().x)).fold(0.0, f32::max);
        let width = self.width.unwrap_or(longest + 44.0);
        let enabled = ui.is_enabled();
        let (rect, mut response) = ui.allocate_exact_size(egui::vec2(width, height), Sense::click());
        let popup = egui::Id::new(self.id).with("egui_native_look-dropdown");
        let open = egui::Popup::is_id_open(ui.ctx(), popup);
        let arrows = crate::paint::popup_button(ui.painter(), &x.paint(), rect, &response, enabled, open);
        let text = self.items.get(*self.selected).map(String::as_str).unwrap_or("");
        let colour = if enabled { c.fg } else { c.disabled };
        let (galley, _) = truncated(ui, text, font.clone(), colour, rect.width() - 10.0 - arrows);
        let top = crate::galley_top(ui.ctx(), &galley, rect.center().y);
        ui.painter().galley(egui::pos2(rect.left() + 10.0, top), galley, colour);
        let row_height = (height - 4.0).max(20.0);
        let selected = &mut *self.selected;
        let look = x.look;
        egui::Popup::menu(&response).id(popup).width(width).show(|ui| {
            ui.set_min_width(width - 8.0);
            for (i, item) in self.items.iter().enumerate() {
                let (row, hit) = ui.allocate_exact_size(egui::vec2(ui.available_width(), row_height), Sense::click());
                // macOS and the plain look light the row under the pointer in the accent;
                // WinUI and libadwaita in a tint.
                let accent_row = matches!(look, Look::Mac | Look::Plain);
                let hot = hit.hovered();
                if hot {
                    ui.painter().rect_filled(row, CornerRadius::same(4), if accent_row { c.accent } else { c.button_hover });
                } else if i == *selected && look == Look::Windows {
                    ui.painter().rect_filled(row, CornerRadius::same(4), c.row_hover);
                    // WinUI marks the chosen item with an accent bar at its start.
                    let bar = Rect::from_center_size(egui::pos2(row.left() + 2.0, row.center().y), egui::vec2(3.0, row.height() * 0.5));
                    ui.painter().rect_filled(bar, CornerRadius::same(2), c.accent);
                }
                let colour = if hot && accent_row { c.on_accent } else { c.fg };
                if i == *selected && look != Look::Windows {
                    super::basic::check_mark(ui.painter(), egui::pos2(row.left() + 12.0, row.center().y), colour);
                }
                crate::text(ui.painter(), egui::pos2(row.left() + 26.0, row.center().y), egui::Align2::LEFT_CENTER, item, font.clone(), colour);
                if hit.clicked() {
                    *selected = i;
                    ui.close();
                }
            }
        });
        if *self.selected != before {
            response.mark_changed();
        }
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, enabled, text));
        response
    }
}

/// A slider over `range`: a thin rail filled with the accent up to a knob, white with a
/// shadow on macOS and in libadwaita, WinUI's accent dot in a ring on Windows.
pub struct Slider<'a> {
    value: &'a mut f64,
    range: std::ops::RangeInclusive<f64>,
    width: Option<f32>,
}

impl<'a> Slider<'a> {
    pub fn new(value: &'a mut f64, range: std::ops::RangeInclusive<f64>) -> Self {
        Slider { value, range, width: None }
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl Widget for Slider<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        let c = &x.c;
        let height = ui.spacing().interact_size.y;
        let width = self.width.unwrap_or_else(|| ui.spacing().slider_width.max(120.0));
        let (rect, mut response) = ui.allocate_exact_size(egui::vec2(width, height), Sense::click_and_drag());
        let knob_r = match x.look {
            Look::Windows | Look::Gtk | Look::Material => 10.0,
            Look::Ios => 14.0,
            _ => 8.0,
        };
        let rail = Rect::from_min_max(egui::pos2(rect.left() + knob_r, rect.center().y - 2.0), egui::pos2(rect.right() - knob_r, rect.center().y + 2.0));
        let (lo, hi) = (*self.range.start(), *self.range.end());
        if let Some(pos) = response.interact_pointer_pos() {
            let t = ((pos.x - rail.left()) / rail.width()).clamp(0.0, 1.0) as f64;
            let new = lo + t * (hi - lo);
            if new != *self.value {
                *self.value = new;
                response.mark_changed();
            }
        }
        if response.has_focus() {
            let step = (hi - lo) / 100.0;
            let (left, right) = ui.input(|i| (i.key_pressed(egui::Key::ArrowLeft), i.key_pressed(egui::Key::ArrowRight)));
            let new = (*self.value + step * (f64::from(u8::from(right)) - f64::from(u8::from(left)))).clamp(lo, hi);
            if new != *self.value {
                *self.value = new;
                response.mark_changed();
            }
        }
        let enabled = ui.is_enabled();
        let t = if hi > lo { ((*self.value - lo) / (hi - lo)).clamp(0.0, 1.0) as f32 } else { 0.0 };
        let painter = ui.painter();
        let at = egui::pos2(egui::lerp(rail.left()..=rail.right(), t), rail.center().y);
        let accent = if enabled { c.accent } else { c.disabled };
        painter.rect_filled(rail, CornerRadius::same(2), c.track);
        painter.rect_filled(Rect::from_min_max(rail.min, egui::pos2(at.x, rail.max.y)), CornerRadius::same(2), accent);
        match x.look {
            Look::Windows => {
                // WinUI's thumb: a ring in the button's fill round an accent dot that
                // grows under the pointer.
                painter.circle_filled(at, knob_r, c.button);
                painter.circle_stroke(at, knob_r, Stroke::new(1.0, c.control_border));
                painter.circle_filled(at, if response.hovered() || response.dragged() { 7.0 } else { 5.0 }, accent);
            }
            Look::Plain | Look::Material => {
                painter.circle_filled(at, knob_r, if response.hovered() { c.accent_hover } else { accent });
            }
            Look::Mac | Look::Gtk | Look::Ios => {
                painter.circle_filled(at + egui::vec2(0.0, 1.0), knob_r + 0.5, Color32::from_black_alpha(if enabled { 50 } else { 20 }));
                painter.circle_filled(at, knob_r, if enabled { Color32::WHITE } else { Color32::from_gray(if c.dark { 140 } else { 240 }) });
            }
        }
        if response.has_focus() {
            crate::paint::ring(painter, &x.paint(), Rect::from_center_size(at, egui::Vec2::splat(knob_r * 2.0)), CornerRadius::same(knob_r as u8));
        }
        response.widget_info(|| egui::WidgetInfo::slider(enabled, *self.value, ""));
        response
    }
}

/// A row of tabs: on macOS and in libadwaita a segmented control, on Windows WinUI's
/// pivot with an accent bar under the chosen tab, in the plain look an accent underline.
/// `response.changed()` when the tab changed.
pub struct Tabs<'a> {
    id: egui::Id,
    selected: &'a mut usize,
    labels: Vec<String>,
}

impl<'a> Tabs<'a> {
    pub fn new(id: impl std::hash::Hash + std::fmt::Debug, selected: &'a mut usize, labels: &[&str]) -> Self {
        Tabs { id: egui::Id::new(id), selected, labels: labels.iter().map(|s| s.to_string()).collect() }
    }
}

impl Widget for Tabs<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        if matches!(x.look, Look::Mac | Look::Gtk | Look::Ios) {
            let labels: Vec<&str> = self.labels.iter().map(String::as_str).collect();
            let width = labels.iter().map(|l| ui.fonts_mut(|f| f.layout_no_wrap(l.to_string(), x.m.bold.clone(), x.c.fg).size().x) + 28.0).sum::<f32>();
            return ui.add(Segmented::new(self.id, self.selected, &labels).width(width.min(ui.available_width())));
        }
        let c = &x.c;
        let before = *self.selected;
        let height = ui.spacing().interact_size.y + 4.0;
        let mut whole: Option<Response> = None;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for (i, label) in self.labels.iter().enumerate() {
                let on = i == *self.selected;
                let font = if on { x.m.bold.clone() } else { x.m.body.clone() };
                let width = ui.fonts_mut(|f| f.layout_no_wrap(label.clone(), font.clone(), c.fg).size().x) + 20.0;
                let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), Sense::click());
                if response.hovered() && !on {
                    ui.painter().rect_filled(rect.shrink(2.0), CornerRadius::same(4), c.row_hover);
                }
                // Material colours the chosen tab's label in the primary too.
                let colour = if on && x.look == Look::Material { c.accent } else if on || response.hovered() { c.fg } else { c.dim };
                crate::text(ui.painter(), rect.center(), egui::Align2::CENTER_CENTER, label, font, colour);
                if on {
                    let bar_width = if x.look == Look::Windows { 16.0 } else { width - 12.0 };
                    let bar = Rect::from_center_size(egui::pos2(rect.center().x, rect.bottom() - 1.5), egui::vec2(bar_width, 3.0));
                    ui.painter().rect_filled(bar, CornerRadius::same(2), c.accent);
                }
                if response.clicked() {
                    *self.selected = i;
                }
                whole = Some(match whole.take() {
                    Some(w) => w | response,
                    None => response,
                });
            }
        });
        let mut response = whole.unwrap_or_else(|| ui.allocate_response(egui::Vec2::ZERO, Sense::hover()));
        if *self.selected != before {
            response.mark_changed();
        }
        response
    }
}
