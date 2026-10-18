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
    select_all: bool,
}

impl<'a> TextField<'a> {
    /// An entry editing `text`.
    pub fn new(text: &'a mut String) -> Self {
        TextField { text, hint: String::new(), width: None, search: false, select_all: false }
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
    /// Selects all the text when the entry comes into focus, so typing replaces it.
    pub fn select_all_on_focus(mut self, select: bool) -> Self {
        self.select_all = select;
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
        if self.select_all && response.gained_focus() {
            if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), response.id) {
                let end = egui::text::CCursor::new(self.text.chars().count());
                state.cursor.set_char_range(Some(egui::text::CCursorRange::two(egui::text::CCursor::default(), end)));
                state.store(ui.ctx(), response.id);
            }
        }
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

/// What writes a [`NumberField`]'s number, and what reads it back.
type Formatter<'a> = Box<dyn 'a + Fn(f64, std::ops::RangeInclusive<usize>) -> String>;
type Parser<'a> = Box<dyn 'a + Fn(&str) -> Option<f64>>;

/// A number with − and + beside it, stepping by `step` and staying inside `range`; drag
/// it sideways to scrub, click it to type, which selects the number so typing replaces
/// it. The number keeps a width for five digits, so the row stays still while a longer
/// number is typed. It takes what egui's `DragValue` takes: a prefix and a suffix, a
/// formatter and a parser (for units), decimals, and whether a value already outside the
/// range is pulled in.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui, length: &mut f64) {
/// use egui_native_look as look;
/// ui.add(
///     look::NumberField::new(length, 0.0..=10_000.0)
///         .step(10.0)
///         .suffix(" mm")
///         .custom_parser(|text| text.trim().trim_end_matches("mm").trim().parse().ok()),
/// );
/// # }
/// ```
pub struct NumberField<'a, T: egui::emath::Numeric> {
    value: &'a mut T,
    range: std::ops::RangeInclusive<f64>,
    step: f64,
    speed: Option<f64>,
    width: Option<f32>,
    prefix: String,
    suffix: String,
    formatter: Option<Formatter<'a>>,
    parser: Option<Parser<'a>>,
    decimals: (Option<usize>, Option<usize>),
    clamp_existing: bool,
}

impl<'a, T: egui::emath::Numeric> NumberField<'a, T> {
    /// A field editing `value`, kept inside `range`.
    pub fn new(value: &'a mut T, range: std::ops::RangeInclusive<f64>) -> Self {
        NumberField {
            value,
            range,
            step: 1.0,
            speed: None,
            width: None,
            prefix: String::new(),
            suffix: String::new(),
            formatter: None,
            parser: None,
            decimals: (None, None),
            clamp_existing: true,
        }
    }
    /// What − and + add, and the drag's speed unless [`NumberField::speed`] sets it; 1 by
    /// default.
    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }
    /// How much a point of dragging changes the value.
    pub fn speed(mut self, speed: f64) -> Self {
        self.speed = Some(speed);
        self
    }
    /// The whole control's width, − and + included, so it lines up with the controls
    /// above and below; the number takes what the buttons leave.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
    /// Text before the number, `"x: "`.
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }
    /// Text after the number, `" mm"`.
    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = suffix.into();
        self
    }
    /// Writes the number, as egui's `DragValue::custom_formatter`: given the value and
    /// the decimals allowed.
    pub fn custom_formatter(mut self, formatter: impl 'a + Fn(f64, std::ops::RangeInclusive<usize>) -> String) -> Self {
        self.formatter = Some(Box::new(formatter));
        self
    }
    /// Reads what was typed, as egui's `DragValue::custom_parser`; `None` keeps the value.
    pub fn custom_parser(mut self, parser: impl 'a + Fn(&str) -> Option<f64>) -> Self {
        self.parser = Some(Box::new(parser));
        self
    }
    /// At least this many decimals.
    pub fn min_decimals(mut self, decimals: usize) -> Self {
        self.decimals.0 = Some(decimals);
        self
    }
    /// At most this many decimals.
    pub fn max_decimals(mut self, decimals: usize) -> Self {
        self.decimals.1 = Some(decimals);
        self
    }
    /// Exactly this many decimals.
    pub fn fixed_decimals(self, decimals: usize) -> Self {
        self.min_decimals(decimals).max_decimals(decimals)
    }
    /// Whether a value already outside the range is pulled into it, as egui's
    /// `DragValue::clamp_existing_to_range`; by default it is.
    pub fn clamp_existing_to_range(mut self, clamp: bool) -> Self {
        self.clamp_existing = clamp;
        self
    }
}

impl<T: egui::emath::Numeric> Widget for NumberField<'_, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        let before = *self.value;
        let (lo, hi) = (*self.range.start(), *self.range.end());
        let NumberField { value, range, step, speed, width, prefix, suffix, formatter, parser, decimals, clamp_existing } = self;
        let inner = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let minus = step_button(ui, value.to_f64() > lo, false);
            if minus.clicked() {
                *value = T::from_f64((value.to_f64() - step).max(lo));
            }
            let side = ui.spacing().interact_size.y;
            let number_width = width.map_or(64.0, |w| (w - 2.0 * side - 8.0).max(40.0));
            let size = egui::vec2(number_width, side);
            let mut drag = egui::DragValue::new(&mut *value)
                .speed(speed.unwrap_or(step))
                .range(range)
                .clamp_existing_to_range(clamp_existing)
                .prefix(prefix)
                .suffix(suffix);
            if let Some(formatter) = formatter {
                drag = drag.custom_formatter(formatter);
            }
            if let Some(parser) = parser {
                drag = drag.custom_parser(parser);
            }
            if let Some(min) = decimals.0 {
                drag = drag.min_decimals(min);
            }
            if let Some(max) = decimals.1 {
                drag = drag.max_decimals(max);
            }
            let number = ui.add_sized(size, drag);
            let plus = step_button(ui, value.to_f64() < hi, true);
            if plus.clicked() {
                *value = T::from_f64((value.to_f64() + step).min(hi));
            }
            minus | number | plus
        });
        let mut response = inner.inner;
        if *value != before {
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
