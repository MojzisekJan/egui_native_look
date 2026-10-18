//! Entering text and numbers.

use egui::{Color32, Response, Stroke, Ui, Widget};

use super::Ctx;
use crate::{entry_margin, underline, Look};

/// A one-line text entry, as tall as the controls around it, its text centred on its
/// capitals; WinUI's underline on Windows, the look's focus ring elsewhere. A search entry
/// carries a magnifier.
pub struct TextField<'a> {
    text: &'a mut String,
    hint: String,
    width: Option<f32>,
    search: bool,
}

impl<'a> TextField<'a> {
    pub fn new(text: &'a mut String) -> Self {
        TextField { text, hint: String::new(), width: None, search: false }
    }
    /// Text shown while it is empty.
    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = hint.into();
        self
    }
    /// A fixed width; by default the width available.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
    /// A search entry, with a magnifier at its start.
    pub fn search(mut self) -> Self {
        self.search = true;
        self
    }
}

impl Widget for TextField<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        let height = ui.spacing().interact_size.y;
        let width = self.width.unwrap_or_else(|| ui.available_width());
        let left = if self.search { 26 } else { 8 };
        let margin = entry_margin(ui, &x.m.body, height, left, 8);
        let response = ui.add(
            egui::TextEdit::singleline(self.text)
                .hint_text(self.hint)
                .font(x.m.body.clone())
                .desired_width(width - f32::from(margin.left) - f32::from(margin.right))
                .margin(margin)
                .min_size(egui::vec2(width, height)),
        );
        if self.search {
            let m = egui::pos2(response.rect.left() + 14.0, response.rect.center().y);
            let stroke = Stroke::new(1.5, x.c.dim);
            ui.painter().circle_stroke(m - egui::vec2(1.0, 1.0), 4.5, stroke);
            ui.painter().line_segment([m + egui::vec2(2.4, 2.4), m + egui::vec2(5.5, 5.5)], stroke);
        }
        underline(ui.painter(), response.rect, response.has_focus());
        if response.has_focus() && matches!(x.look, Look::Mac | Look::Gtk | Look::Ios) {
            crate::paint::ring(ui.painter(), &x.paint(), response.rect, x.radius());
        }
        response
    }
}

/// A number with − and + beside it, stepping by `step` and staying inside `range`; drag
/// it sideways to scrub, click it to type. The number keeps a width for five digits, so
/// the row stays still while a longer number is typed.
pub struct NumberField<'a, T: egui::emath::Numeric> {
    value: &'a mut T,
    range: std::ops::RangeInclusive<f64>,
    step: f64,
}

impl<'a, T: egui::emath::Numeric> NumberField<'a, T> {
    pub fn new(value: &'a mut T, range: std::ops::RangeInclusive<f64>) -> Self {
        NumberField { value, range, step: 1.0 }
    }
    /// What − and + add, and the drag's speed; 1 by default.
    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }
}

impl<T: egui::emath::Numeric> Widget for NumberField<'_, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        let before = *self.value;
        let (lo, hi) = (*self.range.start(), *self.range.end());
        let inner = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let minus = step_button(ui, self.value.to_f64() > lo, false);
            if minus.clicked() {
                *self.value = T::from_f64((self.value.to_f64() - self.step).max(lo));
            }
            let size = egui::vec2(64.0, ui.spacing().interact_size.y);
            let number = ui.add_sized(size, egui::DragValue::new(self.value).speed(self.step).range(self.range.clone()));
            let plus = step_button(ui, self.value.to_f64() < hi, true);
            if plus.clicked() {
                *self.value = T::from_f64((self.value.to_f64() + self.step).min(hi));
            }
            minus | number | plus
        });
        let mut response = inner.inner;
        if *self.value != before {
            response.mark_changed();
        }
        response
    }
}

/// A − or + button: the sign drawn as lines on the button's centre, where a font's minus
/// and plus glyphs sit a pixel or two low. The glyph stays, unseen, as the button's text:
/// its size and its name for a screen reader.
fn step_button(ui: &mut Ui, enabled: bool, plus: bool) -> Response {
    let side = ui.spacing().interact_size.y;
    let sign = egui::RichText::new(if plus { "+" } else { "−" }).color(Color32::TRANSPARENT);
    let response = ui.add_enabled(enabled, egui::Button::new(sign).min_size(egui::vec2(side, side)));
    let colour = if enabled { ui.style().interact(&response).fg_stroke.color } else { ui.visuals().weak_text_color() };
    let c = response.rect.center();
    let half = 4.0;
    let stroke = Stroke::new(1.5, colour);
    ui.painter().line_segment([c - egui::vec2(half, 0.0), c + egui::vec2(half, 0.0)], stroke);
    if plus {
        ui.painter().line_segment([c - egui::vec2(0.0, half), c + egui::vec2(0.0, half)], stroke);
    }
    response
}
