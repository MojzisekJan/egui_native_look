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
        let enabled = ui.is_enabled();
        // The control as a whole takes the focus, and the arrows move the choice inside it.
        let (rect, mut response) = ui.allocate_exact_size(egui::vec2(width, height), if enabled { Sense::focusable_noninteractive() } else { Sense::hover() });
        let n = self.items.len().max(1);
        let each = rect.width() / n as f32;
        let id = response.id.with(("segmented", self.id));
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
            let segment = ui.interact(slot, id.with(i), if enabled { Sense::CLICK } else { Sense::hover() });
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
            segment.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::RadioButton, enabled, on, label));
            if segment.clicked() {
                *self.selected = i;
            }
            if on && response.has_focus() {
                crate::paint::ring(ui.painter(), &x.paint(), slot.shrink(2.0), CornerRadius::same(r.saturating_sub(1)));
            }
        }
        if !system {
            ui.painter().rect_stroke(rect, CornerRadius::same(r), Stroke::new(1.0, c.border), egui::StrokeKind::Inside);
        }
        choice_keys(ui, &response, n, self.selected);
        let label = self.items.get(*self.selected).map(|(label, _)| label.clone()).unwrap_or_default();
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::RadioGroup, enabled, &label));
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
        let segment = ui.interact(slot, id.with(i), if enabled { Sense::CLICK } else { Sense::hover() });
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
        segment.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::RadioButton, enabled, on, label));
        if segment.clicked() {
            *selected = i;
        }
        if on && response.has_focus() {
            crate::paint::ring(ui.painter(), &x.paint(), slot, round);
        }
    }
    ui.painter().rect_stroke(rect, CornerRadius::same(pill), Stroke::new(1.0, c.control_strong), egui::StrokeKind::Inside);
    choice_keys(ui, &response, n, selected);
    let label = items.get(*selected).map(|(label, _)| label.clone()).unwrap_or_default();
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::RadioGroup, enabled, &label));
    if *selected != before {
        response.mark_changed();
    }
    response
}

/// While the row of `n` choices `response` stands for has the focus, Left and Right choose
/// the one before or after; the arrows stay in the row.
fn choice_keys(ui: &Ui, response: &Response, n: usize, selected: &mut usize) {
    if !response.has_focus() || n == 0 {
        return;
    }
    let filter = egui::EventFilter { horizontal_arrows: true, ..Default::default() };
    ui.memory_mut(|m| m.set_focus_lock_filter(response.id, filter));
    let (left, right) = ui.input(|input| (input.key_pressed(egui::Key::ArrowLeft), input.key_pressed(egui::Key::ArrowRight)));
    if right {
        *selected = (*selected + 1).min(n - 1);
    } else if left {
        *selected = selected.saturating_sub(1).min(n - 1);
    }
}

/// While the item `focused` of a row of choices has the focus, Left and Right choose the
/// item before or after it and move the focus there; the arrows stay in the row.
fn arrow_keys(ui: &Ui, focused: Option<usize>, ids: &[egui::Id], selected: &mut usize) {
    let Some(i) = focused else { return };
    let filter = egui::EventFilter { horizontal_arrows: true, ..Default::default() };
    ui.memory_mut(|m| m.set_focus_lock_filter(ids[i], filter));
    let (left, right) = ui.input(|input| (input.key_pressed(egui::Key::ArrowLeft), input.key_pressed(egui::Key::ArrowRight)));
    let next = if right { (i + 1).min(ids.len() - 1) } else if left { i.saturating_sub(1) } else { i };
    if next != i {
        *selected = next;
        ui.memory_mut(|m| {
            m.request_focus(ids[next]);
            // The next frame's arrows reach it before it can ask for them itself.
            m.set_focus_lock_filter(ids[next], filter);
        });
    }
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
    /// A pop-up button choosing `selected`, an index into `items`; `id` tells its list
    /// from others'.
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
        let text = self.items.get(*self.selected).cloned().unwrap_or_default();
        let row_height = (height - 4.0).max(20.0);
        let selected = &mut *self.selected;
        let look = x.look;
        let items = &self.items;
        let mut response = PopupButton::new(self.id, text).width(width).show(ui, |ui| {
            for (i, item) in items.iter().enumerate() {
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
        })
        .response;
        if *self.selected != before {
            response.mark_changed();
        }
        response
    }
}

/// A pop-up button whose list is the app's own: `show` draws the button with `text` on
/// it and, while it is open, whatever the closure adds under it, which can be
/// [`menu_value`] rows, egui's `selectable_value`, icons or columns. A click on a row closes
/// it, as a menu's does.
///
/// ```no_run
/// # #[derive(PartialEq, Clone, Copy)] enum Units { Metres, Feet }
/// # fn demo(ui: &mut egui::Ui, units: &mut Units) {
/// use egui_native_look as look;
/// let text = if *units == Units::Metres { "Metres" } else { "Feet" };
/// look::PopupButton::new("units", text).width(140.0).show(ui, |ui| {
///     look::menu_value(ui, units, Units::Metres, "Metres");
///     look::menu_value(ui, units, Units::Feet, "Feet");
/// });
/// # }
/// ```
pub struct PopupButton {
    id: egui::Id,
    text: String,
    width: Option<f32>,
}

impl PopupButton {
    /// A button reading `text`, the choice it stands for; `id` tells its list from
    /// others'.
    pub fn new(id: impl std::hash::Hash + std::fmt::Debug, text: impl Into<String>) -> Self {
        PopupButton { id: egui::Id::new(id), text: text.into(), width: None }
    }
    /// The button's width; by default its text's and the arrows'.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Draws the button and, while open, the list `add` fills. The inner value is what
    /// `add` returned, while open.
    pub fn show<R>(self, ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> egui::InnerResponse<Option<R>> {
        let x = Ctx::of(ui);
        let c = x.c;
        let height = ui.spacing().interact_size.y;
        let font = x.m.body.clone();
        let width = self.width.unwrap_or_else(|| ui.fonts_mut(|f| f.layout_no_wrap(self.text.clone(), font.clone(), c.fg).size().x) + 44.0);
        let enabled = ui.is_enabled();
        let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), if enabled { Sense::click() } else { Sense::hover() });
        // The button's own id, unique to where it is laid out, tells the list apart from
        // another button's built with the same `id` (the rows of a loop).
        let popup = response.id.with((self.id, "egui_native_look-dropdown"));
        let open = egui::Popup::is_id_open(ui.ctx(), popup);
        let arrows = crate::paint::popup_button(ui.painter(), &x.paint(), rect, &response, enabled, open);
        let colour = if enabled { c.fg } else { c.disabled };
        let (galley, _) = truncated(ui, &self.text, font, colour, rect.width() - 10.0 - arrows);
        let top = crate::galley_top(ui.ctx(), &galley, rect.center().y);
        ui.painter().galley(egui::pos2(rect.left() + 10.0, top), galley, colour);
        let inner = egui::Popup::menu(&response).id(popup).width(width).show(|ui| {
            ui.set_min_width(width - 8.0);
            add(ui)
        });
        let text = self.text;
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, enabled, &text));
        egui::InnerResponse::new(inner.map(|i| i.inner), response)
    }
}

/// A row of a [`PopupButton`]'s list choosing `value`: a [`super::MenuItem`] ticked while
/// `*current == value`, which sets it when clicked.
pub fn menu_value<V: PartialEq>(ui: &mut Ui, current: &mut V, value: V, label: impl Into<String>) -> Response {
    let mut response = ui.add(super::MenuItem::new(label).checked(*current == value));
    if response.clicked() && *current != value {
        *current = value;
        response.mark_changed();
    }
    response
}

/// Draws a pop-up button's arrows in the look, for egui's own combo box:
/// `egui::ComboBox::from_id_salt("units").icon(egui_native_look::combo_icon)`. egui
/// passes the room it leaves for an icon at the button's end.
pub fn combo_icon(ui: &Ui, rect: Rect, visuals: &egui::style::WidgetVisuals, open: bool) {
    let x = Ctx::of(ui);
    let c = &x.c;
    let m = rect.center();
    if x.look == Look::Mac {
        let side = rect.height().clamp(12.0, 16.0);
        let square = Rect::from_center_size(m, egui::vec2(side, side));
        ui.painter().rect_filled(square, CornerRadius::same(4), c.accent);
        let m = square.center();
        let stroke = Stroke::new(1.4, Color32::WHITE);
        ui.painter().add(egui::Shape::line(vec![m + egui::vec2(-2.8, -1.5), m + egui::vec2(0.0, -4.2), m + egui::vec2(2.8, -1.5)], stroke));
        ui.painter().add(egui::Shape::line(vec![m + egui::vec2(-2.8, 1.5), m + egui::vec2(0.0, 4.2), m + egui::vec2(2.8, 1.5)], stroke));
    } else {
        let flip = if open { -1.0 } else { 1.0 };
        let chevron = vec![m + egui::vec2(-4.0, -2.0 * flip), m + egui::vec2(0.0, 2.0 * flip), m + egui::vec2(4.0, -2.0 * flip)];
        let colour = if ui.is_enabled() { visuals.fg_stroke.color.gamma_multiply(0.8) } else { c.disabled };
        ui.painter().add(egui::Shape::line(chevron, Stroke::new(1.5, colour)));
    }
}

/// A slider over `range`: a thin rail filled with the accent up to a knob, white with a
/// shadow on macOS and in libadwaita, WinUI's accent dot in a ring on Windows. Any of
/// egui's numbers, linear or logarithmic, in steps, with a label and the number beside
/// it.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui, exposure: &mut f32, samples: &mut u32) {
/// use egui_native_look as look;
/// ui.add(look::Slider::new(exposure, 0.01..=100.0).logarithmic(true).text("Exposure"));
/// ui.add(look::Slider::new(samples, 1..=64).show_value(true));
/// ui.add(look::Slider::new(exposure, 0.0..=1.0).show_value(true).custom_formatter(|v, _| format!("{:.0} %", v * 100.0)));
/// # }
/// ```
pub struct Slider<'a, T: egui::emath::Numeric> {
    value: &'a mut T,
    range: std::ops::RangeInclusive<f64>,
    width: Option<f32>,
    text: String,
    logarithmic: bool,
    step: Option<f64>,
    clamping: egui::SliderClamping,
    show_value: bool,
    label: String,
    format: super::entry::Format<'a>,
}

impl<'a, T: egui::emath::Numeric> Slider<'a, T> {
    /// A slider moving `value` across `range`.
    pub fn new(value: &'a mut T, range: std::ops::RangeInclusive<T>) -> Self {
        let range = range.start().to_f64()..=range.end().to_f64();
        Slider { value, range, width: None, text: String::new(), logarithmic: false, step: None, clamping: egui::SliderClamping::default(), show_value: false, label: String::new(), format: super::entry::Format::default() }
    }
    /// The rail's width; by default egui's `slider_width`.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
    /// A label after it.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }
    /// What it sets, for a screen reader, when no [`Slider::text`] says so: the label of
    /// its row in a form, say.
    pub fn accessible_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
    /// Spreads the range logarithmically, for a range of positive numbers spanning
    /// orders of magnitude.
    pub fn logarithmic(mut self, logarithmic: bool) -> Self {
        self.logarithmic = logarithmic;
        self
    }
    /// Moves in steps of `step`, and the arrow keys by one step.
    pub fn step_by(mut self, step: f64) -> Self {
        self.step = (step > 0.0).then_some(step);
        self
    }
    /// When a value outside the range is pulled into it, as egui's slider does: by
    /// default only when the slider changes it.
    pub fn clamping(mut self, clamping: egui::SliderClamping) -> Self {
        self.clamping = clamping;
        self
    }
    /// The number after the rail, in a field to drag or type in, written as the
    /// methods below say: `"1.5×"`, `"90°"`, `"75 %"`.
    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }
    super::entry::format_methods!();

    fn log(&self) -> bool {
        self.logarithmic && *self.range.start() > 0.0 && self.range.end() > self.range.start()
    }

    /// Where `value` sits along the rail, from 0 to 1.
    fn fraction(&self, value: f64) -> f64 {
        let (lo, hi) = (*self.range.start(), *self.range.end());
        if hi == lo || !(hi - lo).is_finite() {
            0.0
        } else if self.log() {
            ((value.max(lo) / lo).ln() / (hi / lo).ln()).clamp(0.0, 1.0)
        } else {
            ((value - lo) / (hi - lo)).clamp(0.0, 1.0)
        }
    }

    /// The value at `t` along the rail, in a step and a whole number where it must be.
    fn value_at(&self, t: f64) -> f64 {
        let (lo, hi) = (*self.range.start(), *self.range.end());
        let mut value = if self.log() { lo * (hi / lo).powf(t) } else { lo + t * (hi - lo) };
        if let Some(step) = self.step {
            value = lo + ((value - lo) / step).round() * step;
        }
        if T::INTEGRAL {
            value = value.round();
        }
        value.clamp(lo.min(hi), hi.max(lo))
    }

    fn set(&mut self, value: f64, response: &mut Response) {
        if value != self.value.to_f64() {
            *self.value = T::from_f64(value);
            response.mark_changed();
        }
    }
}

impl<T: egui::emath::Numeric> Widget for Slider<'_, T> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let (lo, hi) = (*self.range.start(), *self.range.end());
        if self.clamping == egui::SliderClamping::Always {
            let clamped = self.value.to_f64().clamp(lo.min(hi), hi.max(lo));
            *self.value = T::from_f64(clamped);
        }
        if !self.show_value && self.text.is_empty() {
            return self.rail(ui);
        }
        let inner = ui.horizontal(|ui| {
            let mut response = self.rail(ui);
            if self.show_value {
                let before = self.value.to_f64();
                let speed = self.step.unwrap_or((hi - lo).abs() / 200.0);
                let range = if self.clamping == egui::SliderClamping::Never { f64::NEG_INFINITY..=f64::INFINITY } else { lo..=hi };
                let format = std::mem::take(&mut self.format);
                // At least 56 points wide, wider for a long prefix or suffix.
                let field = ui
                    .scope(|ui| {
                        ui.spacing_mut().interact_size.x = 56.0;
                        ui.add(format.apply(egui::DragValue::new(&mut *self.value).speed(speed).range(range)))
                    })
                    .inner;
                if self.value.to_f64() != before {
                    response.mark_changed();
                }
                response |= field;
            }
            if !self.text.is_empty() {
                ui.label(&self.text);
            }
            response
        });
        inner.inner
    }
}

impl<T: egui::emath::Numeric> Slider<'_, T> {
    /// The rail and its knob.
    fn rail(&mut self, ui: &mut Ui) -> Response {
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
        if let Some(pos) = response.interact_pointer_pos() {
            let t = ((pos.x - rail.left()) / rail.width()).clamp(0.0, 1.0) as f64;
            let value = self.value_at(t);
            self.set(value, &mut response);
        }
        if response.has_focus() {
            // The arrows move the knob, not the focus on to the next control.
            let filter = egui::EventFilter { horizontal_arrows: true, ..Default::default() };
            ui.memory_mut(|m| m.set_focus_lock_filter(response.id, filter));
            let (left, right) = ui.input(|i| (i.key_pressed(egui::Key::ArrowLeft), i.key_pressed(egui::Key::ArrowRight)));
            let steps = f64::from(u8::from(right)) - f64::from(u8::from(left));
            if steps != 0.0 {
                let value = match self.step {
                    Some(step) if !self.log() => self.value.to_f64() + step * steps,
                    _ => self.value_at(self.fraction(self.value.to_f64()) + 0.01 * steps),
                };
                let (lo, hi) = (*self.range.start(), *self.range.end());
                self.set(value.clamp(lo.min(hi), hi.max(lo)), &mut response);
            }
        }
        let enabled = ui.is_enabled();
        let t = self.fraction(self.value.to_f64()) as f32;
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
        let value = self.value.to_f64();
        let text = if self.text.is_empty() { self.label.clone() } else { self.text.clone() };
        response.widget_info(|| egui::WidgetInfo::slider(enabled, value, &text));
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
    /// Tabs reading `labels`, `selected` the index of the one shown.
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
        let before = *self.selected;
        let mut whole: Option<Response> = None;
        let (mut ids, mut focused) = (Vec::new(), None);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for (i, label) in self.labels.iter().enumerate() {
                let response = pivot_tab(ui, &x, label, i == *self.selected);
                if response.clicked() {
                    *self.selected = i;
                }
                if response.has_focus() {
                    focused = Some(i);
                }
                ids.push(response.id);
                whole = Some(match whole.take() {
                    Some(w) => w | response,
                    None => response,
                });
            }
        });
        arrow_keys(ui, focused, &ids, self.selected);
        let mut response = whole.unwrap_or_else(|| ui.allocate_response(egui::Vec2::ZERO, Sense::hover()));
        if *self.selected != before {
            response.mark_changed();
        }
        response
    }
}

/// One tab, for a row the app builds itself (filtered, or in a loop): it draws as one tab
/// of [`Tabs`] and is clicked like a button. In a [`TabBar`] the tabs of macOS,
/// libadwaita and iOS sit in the segmented track; alone, a tab is a segment without it.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui, shown: &mut usize, pages: &[(&str, bool)]) {
/// use egui_native_look as look;
/// look::TabBar::new().show(ui, |bar| {
///     for (i, (name, visible)) in pages.iter().enumerate() {
///         if *visible && bar.tab(name, *shown == i).clicked() {
///             *shown = i;
///         }
///     }
/// });
/// # }
/// ```
pub struct Tab {
    label: String,
    selected: bool,
}

impl Tab {
    /// A tab reading `label`, the one shown when `selected`.
    pub fn new(label: impl Into<String>, selected: bool) -> Self {
        Tab { label: label.into(), selected }
    }
}

impl Widget for Tab {
    fn ui(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        if segmented_tabs(x.look) {
            segment_tab(ui, &x, &self.label, self.selected)
        } else {
            pivot_tab(ui, &x, &self.label, self.selected)
        }
    }
}

/// A row of [`Tab`]s the app adds one by one.
pub struct TabBar;

impl TabBar {
    /// An empty row.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        TabBar
    }

    /// Lays out the tabs `add` adds, in the look's track where it has one.
    pub fn show<R>(self, ui: &mut Ui, add: impl FnOnce(&mut TabBarUi<'_>) -> R) -> egui::InnerResponse<R> {
        let x = Ctx::of(ui);
        let segmented = segmented_tabs(x.look);
        let track = ui.painter().add(egui::Shape::Noop);
        let inner = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = if segmented { 0.0 } else { 4.0 };
            let x = Ctx::of(ui);
            add(&mut TabBarUi { ui, x })
        });
        if segmented && inner.response.rect.width() > 0.0 {
            ui.painter().set(track, egui::epaint::RectShape::filled(inner.response.rect, CornerRadius::same(x.m.radius + 1), x.c.track));
        }
        inner
    }
}

/// The tabs of a [`TabBar`] being laid out.
pub struct TabBarUi<'a> {
    ui: &'a mut Ui,
    x: Ctx,
}

impl TabBarUi<'_> {
    /// A tab reading `label`, the one shown when `selected`.
    pub fn tab(&mut self, label: &str, selected: bool) -> Response {
        if segmented_tabs(self.x.look) {
            segment_tab(self.ui, &self.x, label, selected)
        } else {
            pivot_tab(self.ui, &self.x, label, selected)
        }
    }

    /// The row's `Ui`, for anything else in it.
    pub fn ui(&mut self) -> &mut Ui {
        self.ui
    }
}

/// Whether the look's tabs are a segmented control.
fn segmented_tabs(look: Look) -> bool {
    matches!(look, Look::Mac | Look::Gtk | Look::Ios)
}

/// A tab of WinUI's pivot, Material's tabs or the plain look's: its label, with an accent
/// bar under it when chosen.
fn pivot_tab(ui: &mut Ui, x: &Ctx, label: &str, on: bool) -> Response {
    let c = &x.c;
    let height = ui.spacing().interact_size.y + 4.0;
    let font = if on { x.m.bold.clone() } else { x.m.body.clone() };
    let width = ui.fonts_mut(|f| f.layout_no_wrap(label.to_string(), font.clone(), c.fg).size().x) + 20.0;
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
    if response.has_focus() {
        crate::paint::ring(ui.painter(), &x.paint(), rect.shrink(2.0), CornerRadius::same(4));
    }
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, ui.is_enabled(), on, label));
    response
}

/// A segment of a segmented control as a tab: raised out of the track when chosen, as
/// [`Segmented`] draws it.
fn segment_tab(ui: &mut Ui, x: &Ctx, label: &str, on: bool) -> Response {
    let c = &x.c;
    let height = ui.spacing().interact_size.y;
    let font = if on { x.m.bold.clone() } else { x.m.body.clone() };
    // The bold width either way, so the row stays still as the choice moves.
    let width = ui.fonts_mut(|f| f.layout_no_wrap(label.to_string(), x.m.bold.clone(), c.fg).size().x) + 28.0;
    let enabled = ui.is_enabled();
    let (slot, response) = ui.allocate_exact_size(egui::vec2(width, height), if enabled { Sense::click() } else { Sense::hover() });
    let inner = CornerRadius::same(x.m.radius.saturating_sub(1));
    let body = slot.shrink(2.0);
    if on {
        let fill = if enabled { c.raised } else { c.raised.gamma_multiply(0.6) };
        if x.look == Look::Gtk {
            ui.painter().rect_filled(body.translate(egui::vec2(0.0, 1.0)), inner, Color32::from_black_alpha(if c.dark { 50 } else { 20 }));
            ui.painter().rect_filled(body, inner, fill);
        } else {
            crate::paint::raised(ui.painter(), c.dark, body, inner, fill, enabled);
        }
    } else if response.hovered() && enabled {
        ui.painter().rect_filled(body, inner, c.row_hover);
    }
    let colour = if enabled { c.fg } else { c.disabled };
    crate::text(ui.painter(), slot.center(), egui::Align2::CENTER_CENTER, label, font, colour);
    if response.has_focus() {
        crate::paint::ring(ui.painter(), &x.paint(), body, inner);
    }
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, enabled, on, label));
    response
}
