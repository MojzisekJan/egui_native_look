//! Entering text and numbers.

use egui::{Color32, Response, Stroke, Ui, Widget};

use super::Ctx;
use crate::{entry_margin, underline, Look};

/// A one-line text entry, as tall as the controls around it, its text centred on its
/// capitals; WinUI's underline on Windows, the look's focus ring elsewhere. A search entry
/// carries a magnifier.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui, password: &mut String) {
/// ui.add(egui_native_look::TextField::new(password)
///     .id_salt("password")
///     .password(true)
///     .accessible_label("Password"));
/// # }
/// ```
///
/// The builder must be added to a UI or shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::TextField;
/// TextField::new(&mut String::new());
/// ```
#[must_use = "add the widget to a UI or call its show method"]
pub struct TextField<'a> {
    text: &'a mut String,
    hint: String,
    width: Option<f32>,
    search: bool,
    select_all: bool,
    password: bool,
    id: Option<egui::IdSalt>,
    label: String,
}

impl<'a> TextField<'a> {
    /// An entry editing `text`.
    pub fn new(text: &'a mut String) -> Self {
        TextField {
            text,
            hint: String::new(),
            width: None,
            search: false,
            select_all: false,
            password: false,
            id: None,
            label: String::new(),
        }
    }
    /// Keeps focus and cursor state attached to this field when sibling fields move.
    /// The salt must distinguish fields within the same parent UI.
    pub fn id_salt(mut self, id: impl egui::AsIdSalt) -> Self {
        self.id = Some(egui::IdSalt::new(id));
        self
    }
    /// Masks the text and prevents copying its contents through the field's copy action.
    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }
    /// Text shown while it is empty.
    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = hint.into();
        self
    }
    /// The field's name for a screen reader. A form row's label is used when unset.
    pub fn accessible_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
    /// A fixed width; by default the width available.
    /// Non-positive and non-finite widths restore the automatic width. Widths above
    /// 1,000,000 points are capped to that value; positive widths below one point use one point.
    pub fn width(mut self, width: f32) -> Self {
        self.width = super::dimension(width);
        self
    }
    /// A search entry, with a magnifier at its start.
    pub fn search(mut self) -> Self {
        self.search = true;
        self
    }
    /// Selects all text when the entry gains focus, before its first text or IME input.
    /// Later typing and composition updates preserve the editor's current selection.
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
        let id = self.id.map_or_else(
            || ui.next_auto_id(),
            |salt| ui.make_persistent_id(egui::IdSalt::new(salt)),
        );
        let focus_id = id.with("select-on-focus");
        let previously_focused = ui
            .data(|data| data.get_temp::<bool>(focus_id))
            .unwrap_or(false);
        if self.select_all && ui.is_enabled() {
            let pointer_focus = ui.input(|input| input.pointer.primary_pressed())
                && ui
                    .ctx()
                    .read_response(id)
                    .is_some_and(|response| response.is_pointer_button_down_on());
            if pointer_focus {
                ui.memory_mut(|memory| memory.request_focus(id));
            }
            if ui.memory(|memory| {
                memory.has_focus(id) && (!previously_focused || !memory.had_focus_last_frame(id))
            }) {
                select_all_text(ui.ctx(), id, self.text);
            }
        }
        let composing = prepare_text_input(ui, id);
        let mut edit = egui::TextEdit::singleline(self.text)
            .interactive(ui.is_enabled())
            .password(self.password)
            .hint_text(self.hint)
            .font(x.m.body.clone())
            .desired_width(width - f32::from(margin.left) - f32::from(margin.right))
            .margin(margin)
            .min_size(super::target_size(ui, egui::vec2(width, height)));
        if let Some(id) = self.id {
            edit = edit.id_salt(id);
        }
        let mut filter = edit.get_event_filter();
        filter.escape = composing;
        edit = edit.event_filter(filter);
        let response = ui.add(edit);
        debug_assert_eq!(response.id, id);
        keep_focused_field_visible(ui, &response);
        if self.select_all {
            let focused = response.has_focus();
            ui.data_mut(|data| data.insert_temp(focus_id, focused));
        } else {
            ui.data_mut(|data| data.remove::<bool>(focus_id));
        }
        let name = super::form::accessible_name(ui, &self.label);
        if !name.is_empty() {
            ui.ctx()
                .accesskit_node_builder(response.id, |node| node.set_label(name.as_str()));
        }
        if self.select_all && response.gained_focus() && !response.changed() && !composing {
            select_all_text(ui.ctx(), response.id, self.text);
        }
        if self.search {
            let m = egui::pos2(response.rect.left() + 14.0, response.rect.center().y);
            let stroke = Stroke::new(1.5, x.c.dim);
            ui.painter()
                .circle_stroke(m - egui::vec2(1.0, 1.0), 4.5, stroke);
            ui.painter()
                .line_segment([m + egui::vec2(2.4, 2.4), m + egui::vec2(5.5, 5.5)], stroke);
        }
        super::disabled_actions(ui, &response);
        underline(ui.painter(), response.rect, response.has_focus());
        if response.has_focus() && matches!(x.look, Look::Mac | Look::Gtk | Look::Ios) {
            crate::paint::ring(ui.painter(), &x.paint(), response.rect, x.radius());
        }
        response
    }
}

/// Request scrolling on focus or viewport changes without resisting manual scrolling.
fn keep_focused_field_visible(ui: &Ui, response: &Response) {
    let key = response.id.with("focused-viewport");
    if response.has_focus() && response.enabled() {
        let viewport = ui.ctx().content_rect();
        let previous = ui.data(|data| data.get_temp::<egui::Rect>(key));
        if previous != Some(viewport) {
            ui.scroll_to_rect_animation(
                response.rect.expand(8.0),
                None,
                egui::style::ScrollAnimation::none(),
            );
        }
        ui.data_mut(|data| data.insert_temp(key, viewport));
    } else {
        ui.data_mut(|data| data.remove::<egui::Rect>(key));
    }
}

/// Select in character units before events can replace or compose the first input.
fn select_all_text(ctx: &egui::Context, id: egui::Id, text: &str) {
    let mut state = egui::TextEdit::load_state(ctx, id).unwrap_or_default();
    state
        .cursor
        .set_char_range(Some(egui::text::CCursorRange::select_all_str(text)));
    state.store(ctx, id);
}

#[derive(Clone, Copy, Default)]
struct TextComposition {
    active: bool,
    seen_frame: u64,
}

impl TextComposition {
    fn filter_events(&mut self, events: &mut Vec<egui::Event>) -> bool {
        let mut interrupt = false;
        let protect = self.active
            || events.iter().any(|event| match event {
                egui::Event::Ime(
                    egui::ImeEvent::Preedit { text, .. } | egui::ImeEvent::Commit(text),
                ) => !text.is_empty(),
                _ => false,
            });
        events.retain(|event| {
            match event {
                egui::Event::Ime(egui::ImeEvent::Preedit { text, .. }) => {
                    self.active = !text.is_empty()
                }
                egui::Event::Ime(egui::ImeEvent::Commit(_)) => self.active = false,
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } if protect && modifiers.is_none() => match key {
                    egui::Key::Enter => return false,
                    egui::Key::Escape => {
                        interrupt = true;
                        return false;
                    }
                    _ => {}
                },
                _ => {}
            }
            true
        });
        interrupt
    }
}

/// Composition confirmation belongs to the input method; submission needs a later Enter.
fn prepare_text_input(ui: &mut Ui, id: egui::Id) -> bool {
    let state_id = id.with("text-composition");
    let frame = ui.ctx().cumulative_frame_nr();
    let mut state = ui
        .data(|data| data.get_temp::<TextComposition>(state_id))
        .unwrap_or_default();
    let window_focused = ui.input(|input| {
        input.focused
            && !input
                .events
                .iter()
                .any(|event| matches!(event, egui::Event::WindowFocused(false)))
    });
    if !ui.is_enabled()
        || !window_focused
        || !ui.memory(|memory| memory.has_focus(id))
        || frame.saturating_sub(state.seen_frame) > 1
    {
        state.active = false;
    }
    if !ui.is_enabled() || !window_focused || !ui.memory(|memory| memory.has_focus(id)) {
        ui.data_mut(|data| data.remove::<TextComposition>(state_id));
        return false;
    }
    let composing = state.active
        || ui.input(|input| {
            input.events.iter().any(|event| {
                matches!(
                    event,
                    egui::Event::Ime(egui::ImeEvent::Preedit { .. } | egui::ImeEvent::Commit(_))
                )
            })
        });
    super::window::dialog_text_input(ui, composing);
    let interrupt = ui.input_mut(|input| state.filter_events(&mut input.events));
    if interrupt {
        ui.memory_mut(|memory| memory.interrupt_ime());
    }
    state.seen_frame = frame;
    if state.active {
        ui.data_mut(|data| data.insert_temp(state_id, state));
    } else {
        ui.data_mut(|data| data.remove::<TextComposition>(state_id));
    }
    state.active
}

/// What writes a number, and what reads it back.
type Formatter<'a> = Box<dyn 'a + Fn(f64, std::ops::RangeInclusive<usize>) -> String>;
type Parser<'a> = Box<dyn 'a + Fn(&str) -> Option<f64>>;

/// How a number is written in its field: what [`NumberField`] and the number beside a
/// [`crate::Slider`] take from egui's `DragValue`.
#[derive(Default)]
pub(crate) struct Format<'a> {
    pub prefix: String,
    pub suffix: String,
    pub formatter: Option<Formatter<'a>>,
    pub parser: Option<Parser<'a>>,
    pub decimals: (Option<usize>, Option<usize>),
}

impl<'a> Format<'a> {
    /// `drag` writing its number this way.
    pub fn apply(self, mut drag: egui::DragValue<'a>) -> egui::DragValue<'a> {
        drag = drag.prefix(self.prefix).suffix(self.suffix);
        if let Some(formatter) = self.formatter {
            drag = drag.custom_formatter(formatter);
        }
        if let Some(parser) = self.parser {
            drag = drag.custom_parser(parser);
        }
        if let Some(min) = self.decimals.0 {
            drag = drag.min_decimals(min);
        }
        if let Some(max) = self.decimals.1 {
            drag = drag.max_decimals(max);
        }
        drag
    }
}

/// The builder methods for how a number is written, shared by [`NumberField`] and
/// [`crate::Slider`]; each type keeps its [`Format`] in a field named `format`.
macro_rules! format_methods {
    () => {
        /// Text before the number, `"x: "`.
        pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
            self.format.prefix = prefix.into();
            self
        }
        /// Text after the number, `" mm"`, `"°"`, `" %"`.
        pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
            self.format.suffix = suffix.into();
            self
        }
        /// Writes the number, as egui's `DragValue::custom_formatter`: given the value and
        /// the decimals allowed.
        pub fn custom_formatter(
            mut self,
            formatter: impl 'a + Fn(f64, std::ops::RangeInclusive<usize>) -> String,
        ) -> Self {
            self.format.formatter = Some(Box::new(formatter));
            self
        }
        /// Reads what was typed, as egui's `DragValue::custom_parser`; `None` keeps the
        /// value.
        pub fn custom_parser(mut self, parser: impl 'a + Fn(&str) -> Option<f64>) -> Self {
            self.format.parser = Some(Box::new(parser));
            self
        }
        /// At least this many decimals.
        pub fn min_decimals(mut self, decimals: usize) -> Self {
            self.format.decimals.0 = Some(decimals);
            self
        }
        /// At most this many decimals.
        pub fn max_decimals(mut self, decimals: usize) -> Self {
            self.format.decimals.1 = Some(decimals);
            self
        }
        /// Exactly this many decimals.
        pub fn fixed_decimals(self, decimals: usize) -> Self {
            self.min_decimals(decimals).max_decimals(decimals)
        }
    };
}
pub(crate) use format_methods;

/// A number in an entry of the look, with − and + beside it unless [`NumberField::steppers`] leaves
/// them out, stepping by `step` and staying inside the [`NumberField::range`] if it has
/// one; drag it sideways to scrub, click it to type, which selects the number so typing
/// replaces it. The number keeps a width for five digits, so the row stays still while a
/// longer number is typed. It takes what egui's `DragValue` takes: a prefix and a suffix,
/// a formatter and a parser (for units), decimals, and whether a value already outside
/// the range is pulled in.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui, length: &mut f64, xyz: &mut [f32; 3]) {
/// use egui_native_look as look;
/// ui.add(
///     look::NumberField::new(length)
///         .range(0.0..=10_000.0)
///         .step(10.0)
///         .suffix(" mm")
///         .custom_parser(|text| text.trim().trim_end_matches("mm").trim().parse().ok()),
/// );
/// // Three to a row in a narrow panel: the fields alone.
/// ui.horizontal(|ui| {
///     for (axis, value) in ["x ", "y ", "z "].into_iter().zip(xyz) {
///         ui.add(look::NumberField::new(value).steppers(false).prefix(axis).width(64.0));
///     }
/// });
/// # }
/// ```
///
/// The builder must be added to a UI or shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::NumberField;
/// NumberField::new(&mut 1.0);
/// ```
#[must_use = "add the widget to a UI or call its show method"]
pub struct NumberField<'a, T: egui::emath::Numeric> {
    value: &'a mut T,
    range: std::ops::RangeInclusive<f64>,
    step: f64,
    speed: Option<f64>,
    width: Option<f32>,
    steppers: bool,
    step_labels: [String; 2],
    format: Format<'a>,
    clamp_existing: bool,
    label: String,
}

impl<'a, T: egui::emath::Numeric> NumberField<'a, T> {
    /// A field editing `value`, any number its type holds (0 to 255 for a `u8`) until
    /// [`NumberField::range`] bounds it.
    pub fn new(value: &'a mut T) -> Self {
        NumberField {
            value,
            range: T::MIN.to_f64()..=T::MAX.to_f64(),
            step: 1.0,
            speed: None,
            width: None,
            steppers: true,
            step_labels: ["Decrease".into(), "Increase".into()],
            format: Format::default(),
            clamp_existing: true,
            label: String::new(),
        }
    }
    /// Keeps the value inside `range`: − and + stop at its ends, and dragging and typing
    /// stay in it. Endpoint order is ignored; non-finite endpoints leave the previous
    /// range unchanged. In the GTK look, Up/Down use one step, Page Up/Down ten;
    /// Ctrl+Home/End and Ctrl+Page Down/Up select the numeric minimum/maximum.
    pub fn range<N: egui::emath::Numeric>(mut self, range: std::ops::RangeInclusive<N>) -> Self {
        let (lo, hi) = (range.start().to_f64(), range.end().to_f64());
        if lo.is_finite() && hi.is_finite() {
            self.range = lo..=hi;
        }
        self
    }
    /// What − and + add, and the drag's speed unless [`NumberField::speed`] sets it; 1 by
    /// default. A negative step counts as its size; zero and non-finite steps count as 1.
    /// Integer steps are rounded to a whole number and are at least one.
    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }
    /// How much a point of dragging changes the value. In the GTK look, keyboard
    /// and accessibility steps use [`NumberField::step`] independently of this speed.
    pub fn speed(mut self, speed: f64) -> Self {
        self.speed = Some(speed);
        self
    }
    /// The whole control's width, − and + included, so it lines up with the controls
    /// above and below; the number takes what the buttons leave.
    /// Non-positive and non-finite widths restore the automatic width. Widths above
    /// 1,000,000 points are capped to that value; positive widths below one point use one point.
    pub fn width(mut self, width: f32) -> Self {
        self.width = super::dimension(width);
        self
    }
    /// Whether − and + stand beside the number; by default they do. Without them the
    /// field alone remains, as macOS's number fields are, for several to a row.
    pub fn steppers(mut self, steppers: bool) -> Self {
        self.steppers = steppers;
        self
    }
    /// Names of the decrease and increase buttons for a screen reader, for translation.
    /// The field's accessible name is appended when present.
    pub fn step_labels(mut self, decrease: impl Into<String>, increase: impl Into<String>) -> Self {
        self.step_labels = [decrease.into(), increase.into()];
        self
    }
    format_methods!();
    /// What the number is, for a screen reader, as the field draws no label itself. In a
    /// [`crate::Form`] row the row's label is used when this is not set.
    pub fn accessible_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
    /// Whether a value already outside the range is pulled into it, as egui's
    /// `DragValue::clamp_existing_to_range`; by default it is. NaN is normalized
    /// to the lower bound when enabled; otherwise it is preserved until editing.
    pub fn clamp_existing_to_range(mut self, clamp: bool) -> Self {
        self.clamp_existing = clamp;
        self
    }
}

impl<T: egui::emath::Numeric> Widget for NumberField<'_, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        let before = *self.value;
        let (lo, hi) = (*self.range.start(), *self.range.end());
        let NumberField {
            value,
            range: _,
            step,
            speed,
            width,
            steppers,
            step_labels,
            format,
            clamp_existing,
            label,
        } = self;
        let name = super::form::accessible_name(ui, &label);
        let (lo, hi) = (lo.min(hi), lo.max(hi));
        if clamp_existing && value.to_f64().is_nan() {
            *value = T::from_f64(lo);
        }
        let step = if step.is_finite() && step != 0.0 {
            step.abs()
        } else {
            1.0
        };
        let step = if T::INTEGRAL {
            step.round().max(1.0)
        } else {
            step
        };
        let step_names = step_labels.map(|label| {
            if name.is_empty() {
                label
            } else {
                format!("{label} {name}")
            }
        });
        let x = Ctx::of(ui);
        let inner = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let side = ui.spacing().interact_size.y.max(super::minimum_target(ui));
            let minus =
                steppers.then(|| step_button(ui, value.to_f64() > lo, false, &step_names[0]));
            if minus.as_ref().is_some_and(Response::clicked) {
                *value = T::from_f64(stepped_number(value.to_f64(), step, -1.0, lo, hi));
            }
            let buttons = if steppers { 2.0 * side + 8.0 } else { 0.0 };
            let number_width = width.map_or(64.0, |w| (w - buttons).max(40.0));
            let size = egui::vec2(number_width, side);
            let mut shown = if value.to_f64().is_nan() {
                T::from_f64(lo)
            } else {
                *value
            };
            let original = value.to_f64();
            let number = ui.scope(|ui| {
                entry_visuals(ui, &x);
                let layout = egui::Layout::centered_and_justified(ui.layout().main_dir());
                ui.allocate_ui_with_layout(size, layout, |ui| {
                    let id = ui.next_auto_id();
                    super::ignore_disabled_numeric_actions(ui, id);
                    let handled = x.look == Look::Gtk
                        && ui.is_enabled()
                        && number_input(ui, id, &mut shown, step, (lo, hi));
                    let keyboard_changed =
                        handled && super::number_changed(original, shown.to_f64());
                    let drag = egui::DragValue::new(&mut shown)
                        .speed(speed.filter(|speed| speed.is_finite()).unwrap_or(step))
                        .range(lo..=hi)
                        .clamp_existing_to_range(clamp_existing);
                    let mut response = ui.add(format.apply(drag));
                    if keyboard_changed {
                        response.mark_changed();
                    }
                    response
                })
                .inner
            });
            let number = number.inner;
            super::disabled_actions(ui, &number);
            if number.changed() {
                *value = shown;
            }
            if !name.is_empty() {
                ui.ctx()
                    .accesskit_node_builder(number.id, |node| node.set_label(name.as_str()));
            }
            underline(ui.painter(), number.rect, number.has_focus());
            if number.has_focus() && matches!(x.look, Look::Mac | Look::Gtk | Look::Ios) {
                crate::paint::ring(ui.painter(), &x.paint(), number.rect, x.radius());
            }
            let plus = steppers.then(|| step_button(ui, value.to_f64() < hi, true, &step_names[1]));
            if plus.as_ref().is_some_and(Response::clicked) {
                *value = T::from_f64(stepped_number(value.to_f64(), step, 1.0, lo, hi));
            }
            if x.look == Look::Gtk {
                ui.ctx().accesskit_node_builder(number.id, |node| {
                    node.set_numeric_value_step(step);
                    node.set_numeric_value(value.to_f64());
                });
            }
            [minus, plus]
                .into_iter()
                .flatten()
                .fold(number, |all, button| all | button)
        });
        let mut response = inner.inner;
        if super::number_changed(before.to_f64(), value.to_f64()) {
            response.mark_changed();
        }
        response
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum NumberKey {
    Start,
    End,
    Step(f64),
    Set(f64),
}

/// Apply input before building the editor so its text buffer observes the updated value.
fn number_input<T: egui::emath::Numeric>(
    ui: &mut Ui,
    id: egui::Id,
    value: &mut T,
    step: f64,
    bounds: (f64, f64),
) -> bool {
    let focused = ui.memory(|memory| memory.has_focus(id));
    let commands = ui.input_mut(|input| {
        let mut commands = Vec::new();
        if focused {
            input.events.retain(|event| {
                if let egui::Event::Key {
                    key,
                    modifiers,
                    pressed: true,
                    ..
                } = event
                {
                    if let Some(command) = number_key(*key, *modifiers) {
                        commands.push(command);
                        return false;
                    }
                }
                true
            });
        }
        input.consume_accesskit_action_requests(id, |request| {
            let command = match request.action {
                egui::accesskit::Action::Increment => NumberKey::Step(1.0),
                egui::accesskit::Action::Decrement => NumberKey::Step(-1.0),
                egui::accesskit::Action::SetValue => {
                    if let Some(egui::accesskit::ActionData::NumericValue(value)) = request.data {
                        if !value.is_nan() {
                            commands.push(NumberKey::Set(value));
                        }
                    }
                    return true;
                }
                _ => return false,
            };
            commands.push(command);
            true
        });
        commands
    });
    let handled = !commands.is_empty();
    let (lo, hi) = bounds;
    for command in commands {
        *value = T::from_f64(match command {
            NumberKey::Start => lo,
            NumberKey::End => hi,
            NumberKey::Set(value) => value.clamp(lo, hi),
            NumberKey::Step(count) => stepped_number(value.to_f64(), step, count, lo, hi),
        });
    }
    handled
}

/// Numeric shortcuts leave caret navigation and application modifiers untouched.
fn number_key(key: egui::Key, modifiers: egui::Modifiers) -> Option<NumberKey> {
    use egui::Key;
    if modifiers.alt
        || modifiers.shift
        || modifiers.mac_cmd
        || (modifiers.command && !modifiers.ctrl)
    {
        return None;
    }
    match (key, modifiers.ctrl) {
        (Key::ArrowUp, false) => Some(NumberKey::Step(1.0)),
        (Key::ArrowDown, false) => Some(NumberKey::Step(-1.0)),
        (Key::PageUp, false) => Some(NumberKey::Step(10.0)),
        (Key::PageDown, false) => Some(NumberKey::Step(-10.0)),
        (Key::Home | Key::PageDown, true) => Some(NumberKey::Start),
        (Key::End | Key::PageUp, true) => Some(NumberKey::End),
        _ => None,
    }
}

/// Saturate keyboard and button changes at finite bounds, including overflowing increments.
fn stepped_number(current: f64, step: f64, count: f64, lo: f64, hi: f64) -> f64 {
    let current = if current.is_nan() { lo } else { current };
    let next = current + step * count;
    if next.is_nan() {
        if count > 0.0 {
            hi
        } else {
            lo
        }
    } else {
        next.clamp(lo, hi)
    }
}

/// egui's widgets in `ui` drawn as an entry of the look rather than a button: the entry's
/// fill, and the look's thin edge (none in libadwaita, iOS and Material, which fill theirs).
fn entry_visuals(ui: &mut Ui, x: &Ctx) {
    let c = &x.c;
    let edge = match x.look {
        Look::Gtk | Look::Ios | Look::Material => Stroke::NONE,
        Look::Mac | Look::Windows => Stroke::new(1.0, c.control_border),
        Look::Plain => Stroke::new(1.0, c.border),
    };
    let w = &mut ui.visuals_mut().widgets;
    for (state, fill) in [
        (&mut w.inactive, c.entry),
        (&mut w.hovered, c.entry),
        (&mut w.active, c.entry_focused),
    ] {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.bg_stroke = edge;
    }
    if x.look == Look::Plain {
        w.hovered.bg_stroke = Stroke::new(1.0, c.grid);
    }
}

/// A − or + button: the sign drawn as lines on the button's centre, where a font's minus
/// and plus glyphs sit a pixel or two low. The transparent glyph sets its intrinsic
/// size; the accessible label names its action.
fn step_button(ui: &mut Ui, enabled: bool, plus: bool, label: &str) -> Response {
    let side = ui.spacing().interact_size.y.max(super::minimum_target(ui));
    let sign = egui::RichText::new(if plus { "+" } else { "−" }).color(Color32::TRANSPARENT);
    // Exactly square: without padding the hidden glyph cannot widen the button.
    let response = ui
        .add_enabled_ui(enabled, |ui| {
            ui.spacing_mut().button_padding = egui::Vec2::ZERO;
            ui.add(egui::Button::new(sign).min_size(egui::vec2(side, side)))
        })
        .inner;
    let response = super::click_response(ui, response);
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_label(label));
    let colour = if enabled {
        ui.style().interact(&response).fg_stroke.color
    } else {
        ui.visuals().weak_text_color()
    };
    let c = response.rect.center();
    let half = 4.0;
    let stroke = Stroke::new(1.5, colour);
    ui.painter().line_segment(
        [c - egui::vec2(half, 0.0), c + egui::vec2(half, 0.0)],
        stroke,
    );
    if plus {
        ui.painter().line_segment(
            [c - egui::vec2(0.0, half), c + egui::vec2(0.0, half)],
            stroke,
        );
    }
    response
}

#[cfg(test)]
mod tests {
    use super::{number_key, stepped_number, NumberKey};

    fn composition_key(key: egui::Key, modifiers: egui::Modifiers, pressed: bool) -> egui::Event {
        egui::Event::Key {
            key,
            modifiers,
            pressed,
            physical_key: None,
            repeat: false,
        }
    }

    #[test]
    fn composition_confirmation_is_independent_of_commit_event_order() {
        for enter_first in [false, true] {
            let mut state = super::TextComposition {
                active: true,
                seen_frame: 0,
            };
            let enter = composition_key(egui::Key::Enter, egui::Modifiers::NONE, true);
            let commit = egui::Event::Ime(egui::ImeEvent::Commit("日本".into()));
            let mut events = if enter_first {
                vec![enter, commit.clone()]
            } else {
                vec![commit.clone(), enter]
            };
            assert!(!state.filter_events(&mut events));
            assert_eq!(events, vec![commit]);
            assert!(!state.active);
            let mut events = vec![composition_key(
                egui::Key::Enter,
                egui::Modifiers::NONE,
                true,
            )];
            state.filter_events(&mut events);
            assert_eq!(events.len(), 1);
        }
    }

    #[test]
    fn composition_cancellation_waits_for_the_backend_end_event() {
        let mut state = super::TextComposition {
            active: true,
            seen_frame: 0,
        };
        let mut events = vec![composition_key(
            egui::Key::Escape,
            egui::Modifiers::NONE,
            true,
        )];
        assert!(state.filter_events(&mut events));
        assert!(state.active);
        assert!(events.is_empty());
        let mut events = vec![egui::Event::Ime(egui::ImeEvent::Preedit {
            text: String::new(),
            active_range_chars: None,
        })];
        state.filter_events(&mut events);
        assert!(!state.active);
    }

    #[test]
    fn composition_keeps_modified_shortcuts_and_key_releases() {
        let mut state = super::TextComposition {
            active: true,
            seen_frame: 0,
        };
        let original = vec![
            composition_key(egui::Key::Enter, egui::Modifiers::CTRL, true),
            composition_key(egui::Key::Enter, egui::Modifiers::NONE, false),
            composition_key(egui::Key::ArrowLeft, egui::Modifiers::NONE, true),
            egui::Event::Copy,
        ];
        let mut events = original.clone();
        assert!(!state.filter_events(&mut events));
        assert_eq!(events, original);
    }

    #[test]
    fn numeric_shortcuts_leave_caret_and_application_commands_untouched() {
        use egui::{Key, Modifiers};
        for key in [Key::Home, Key::End, Key::ArrowLeft, Key::ArrowRight] {
            assert_eq!(number_key(key, Modifiers::NONE), None);
        }
        for modifiers in [Modifiers::ALT, Modifiers::SHIFT, Modifiers::MAC_CMD] {
            assert_eq!(number_key(Key::PageUp, modifiers), None);
        }
        assert_eq!(number_key(Key::ArrowUp, Modifiers::CTRL), None);
        assert_eq!(
            number_key(Key::PageUp, Modifiers::CTRL),
            Some(NumberKey::End)
        );
        assert_eq!(
            number_key(Key::PageDown, Modifiers::CTRL),
            Some(NumberKey::Start)
        );
    }

    #[test]
    fn numeric_steps_saturate_even_when_the_increment_overflows() {
        let max = f64::MAX;
        for current in [0.0, f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
            assert_eq!(stepped_number(current, max, 10.0, -max, max), max);
            assert_eq!(stepped_number(current, max, -10.0, -max, max), -max);
        }
        assert_eq!(stepped_number(f64::NAN, 3.0, 1.0, 0.0, 10.0), 3.0);
        assert_eq!(stepped_number(9.0, 3.0, 1.0, 0.0, 10.0), 10.0);
    }
}
