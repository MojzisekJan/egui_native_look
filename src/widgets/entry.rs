//! Entering text and numbers.

use egui::{Color32, Rect, Response, Sense, Stroke, Ui, Widget};

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
    clear_label: String,
    select_all: bool,
    password: bool,
    context_menu: bool,
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
            clear_label: "Clear".into(),
            select_all: false,
            password: false,
            context_menu: true,
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
    /// A search entry, with a magnifier at its start and, while it holds text, a button
    /// at its end that clears it and keeps the focus in the entry.
    pub fn search(mut self) -> Self {
        self.search = true;
        self
    }
    /// Whether a secondary click (a long press on a touch screen) opens the menu of Cut,
    /// Copy, Paste and Select All; on by default, off for an app's own menu.
    pub fn context_menu(mut self, menu: bool) -> Self {
        self.context_menu = menu;
        self
    }
    /// The clear button's name for a screen reader, for translation; "Clear" by default.
    pub fn clear_label(mut self, label: impl Into<String>) -> Self {
        self.clear_label = label.into();
        self
    }
    /// Selects all text when the entry gains focus from the pointer too, before its first
    /// text or IME input; focus from the keyboard always selects it.
    /// Later typing and composition updates preserve the editor's current selection.
    pub fn select_all_on_focus(mut self, select: bool) -> Self {
        self.select_all = select;
        self
    }
}

impl Widget for TextField<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        super::unfaded(ui, |ui| self.add_contents(ui))
    }
}

impl TextField<'_> {
    fn add_contents(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        // Material 3's filled field: 48 points without a label, its text in the 16-point
        // body and 16 points in.
        let material = x.look == Look::Material;
        let height = match x.look {
            Look::Mac => mac_field_height(ui, self.search),
            Look::Material => ui.spacing().interact_size.y.max(48.0),
            _ => ui.spacing().interact_size.y,
        };
        let font = if material {
            egui::FontId::new(16.0, x.m.body.family.clone())
        } else {
            x.m.body.clone()
        };
        let width = self.width.unwrap_or_else(|| ui.available_width());
        let left = match (self.search, material) {
            (true, _) => 26,
            (false, true) => 16,
            (false, false) => 8,
        };
        // A search entry keeps room for its clear button, so the text does not move when
        // the button appears; on a touch screen the button is finger-wide.
        let clear_width = if super::minimum_target(ui) > 0.0 {
            44.0
        } else {
            26.0
        };
        let right = if self.search {
            clear_width as i8
        } else {
            left.min(16)
        };
        let margin = entry_margin(ui, &font, height, left, right);
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
        if ui.is_enabled() {
            super::window::dialog_field(ui, id);
        }
        let mut edit = egui::TextEdit::singleline(self.text)
            .interactive(ui.is_enabled())
            .text_color(if ui.is_enabled() {
                x.c.fg
            } else {
                x.c.disabled
            })
            .password(self.password)
            .hint_text(egui::RichText::new(self.hint).font(font.clone()))
            .font(font)
            .desired_width(width - f32::from(margin.left) - f32::from(margin.right))
            .margin(margin)
            .min_size(super::target_size(ui, egui::vec2(width, height)));
        if let Some(id) = self.id {
            edit = edit.id_salt(id);
        }
        let mut filter = edit.get_event_filter();
        filter.escape = composing;
        edit = edit.event_filter(filter);
        // macOS draws its search field as a capsule. The visuals are swapped in place, as a
        // scope would change the field's automatic id.
        let capsule = (self.search && x.look == Look::Mac)
            .then(|| egui::CornerRadius::same((height / 2.0) as u8));
        // The entry's own fill and edge, where buttons have none.
        let saved = (
            ui.visuals().widgets.clone(),
            ui.visuals().weak_text_color,
            ui.visuals().selection,
        );
        entry_visuals(ui, &x);
        if let Some(round) = capsule {
            let w = &mut ui.visuals_mut().widgets;
            for state in [&mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] {
                state.corner_radius = round;
            }
        }
        // egui puts the caret under any press, so a secondary click would drop the
        // selection the menu is about to cut or copy; keep it as the systems do.
        let selection = (self.context_menu
            && ui.input(|input| input.pointer.button_pressed(egui::PointerButton::Secondary)))
        .then(|| egui::TextEdit::load_state(ui.ctx(), id))
        .flatten()
        .and_then(|state| state.cursor.char_range())
        .filter(|range| !range.is_empty());
        // egui grows a text edit to the controls' height; a large macOS field stays lower.
        let controls = ui.spacing().interact_size.y;
        ui.spacing_mut().interact_size.y = height.min(controls);
        let mut response = ui.add(edit);
        ui.spacing_mut().interact_size.y = controls;
        let visuals = ui.visuals_mut();
        (visuals.widgets, visuals.weak_text_color, visuals.selection) = saved;
        if let Some(range) = selection {
            let mut state = egui::TextEdit::load_state(ui.ctx(), id).unwrap_or_default();
            state.cursor.set_char_range(Some(range));
            state.store(ui.ctx(), id);
        }
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
        // Focus from the keyboard (Tab, or a dialog opening) selects the whole text, as
        // macOS, WinUI and GTK entries do; a click only places the caret.
        let pointer = ui.input(|input| input.pointer.any_down() || input.pointer.any_released());
        let from_keyboard = response.gained_focus() && !pointer;
        if (self.select_all || from_keyboard)
            && response.gained_focus()
            && !response.changed()
            && !composing
        {
            select_all_text(ui.ctx(), response.id, self.text);
        }
        if self.search {
            let m = egui::pos2(response.rect.left() + 14.0, response.rect.center().y);
            let stroke = Stroke::new(1.5, x.c.dim);
            ui.painter()
                .circle_stroke(m - egui::vec2(1.0, 1.0), 4.5, stroke);
            ui.painter()
                .line_segment([m + egui::vec2(2.4, 2.4), m + egui::vec2(5.5, 5.5)], stroke);
            if !self.text.is_empty() && response.enabled() {
                let slot = Rect::from_min_max(
                    egui::pos2(response.rect.right() - clear_width, response.rect.top()),
                    response.rect.max,
                );
                if clear_button(ui, &x, slot, response.id.with("clear"), &self.clear_label) {
                    self.text.clear();
                    response.mark_changed();
                    ui.memory_mut(|memory| memory.request_focus(response.id));
                }
            }
        }
        if self.context_menu && response.enabled() {
            edit_menu(ui, &x, &response, self.text, self.password);
        }
        super::disabled_actions(ui, &response);
        underline(ui.painter(), response.rect, response.has_focus());
        if response.has_focus() && matches!(x.look, Look::Mac | Look::Gtk | Look::Ios) {
            crate::paint::ring(
                ui.painter(),
                &x.paint(),
                response.rect,
                capsule.unwrap_or_else(|| x.radius()),
            );
        }
        response
    }
}

/// The field's menu of edits, on a secondary click or a long press. Cut and Copy ask the
/// integration for the system's own commands on the focused field, so they need a
/// selection and stay off for a password. A click in the menu takes the focus and the
/// selection from the field, so both are kept from when the menu opened and given back
/// before a command.
fn edit_menu(ui: &Ui, x: &Ctx, response: &Response, text: &str, password: bool) {
    let key = response.id.with("edit-menu-selection");
    if response.secondary_clicked() || response.long_touched() {
        ui.memory_mut(|memory| memory.request_focus(response.id));
        let range = egui::TextEdit::load_state(ui.ctx(), response.id)
            .and_then(|state| state.cursor.char_range())
            .filter(|range| !range.is_empty());
        ui.data_mut(|data| data.insert_temp(key, range));
    }
    let range = ui
        .data(|data| data.get_temp::<Option<egui::text::CCursorRange>>(key))
        .flatten();
    let shortcut = |key| {
        crate::shortcut_text(
            x.look,
            &egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, key),
        )
    };
    egui::Popup::context_menu(response)
        .style(super::choice::menu_style(x.look))
        .show(|ui| {
            let copyable = range.is_some() && !password;
            let mut command = None;
            let items = [
                (
                    "Cut",
                    egui::Key::X,
                    copyable,
                    egui::ViewportCommand::RequestCut,
                ),
                (
                    "Copy",
                    egui::Key::C,
                    copyable,
                    egui::ViewportCommand::RequestCopy,
                ),
                (
                    "Paste",
                    egui::Key::V,
                    true,
                    egui::ViewportCommand::RequestPaste,
                ),
            ];
            for (label, key, enabled, request) in items {
                let item = super::MenuItem::new(label).shortcut(shortcut(key));
                if ui.add_enabled(enabled, item).clicked() {
                    command = Some(request);
                }
            }
            ui.separator();
            let all = super::MenuItem::new("Select All").shortcut(shortcut(egui::Key::A));
            let restore = if ui.add(all).clicked() {
                Some(egui::text::CCursorRange::select_all_str(text))
            } else {
                command.is_some().then_some(range).flatten()
            };
            if let Some(range) = restore {
                let mut state =
                    egui::TextEdit::load_state(ui.ctx(), response.id).unwrap_or_default();
                state.cursor.set_char_range(Some(range));
                state.store(ui.ctx(), response.id);
            }
            if restore.is_some() || command.is_some() {
                ui.memory_mut(|memory| memory.request_focus(response.id));
            }
            if let Some(command) = command {
                ui.ctx().send_viewport_cmd(command);
            }
        });
}

/// A search entry's clear button in `slot`, named `label`: a grey disc with a cross on
/// macOS and iOS, a cross elsewhere. Returns whether it was clicked.
fn clear_button(ui: &mut Ui, x: &Ctx, slot: Rect, id: egui::Id, label: &str) -> bool {
    let c = &x.c;
    let response = super::click_response(ui, ui.interact(slot, id, Sense::click()));
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
    let m = egui::pos2(slot.right() - 13.0, slot.center().y);
    let colour = if response.hovered() { c.fg } else { c.dim };
    let r = 3.0;
    let cross = |painter: &egui::Painter, colour: Color32| {
        let stroke = Stroke::new(1.4, colour);
        painter.line_segment([m + egui::vec2(-r, -r), m + egui::vec2(r, r)], stroke);
        painter.line_segment([m + egui::vec2(r, -r), m + egui::vec2(-r, r)], stroke);
    };
    if matches!(x.look, Look::Mac | Look::Ios) {
        ui.painter()
            .circle_filled(m, 7.0, colour.gamma_multiply(0.7));
        cross(ui.painter(), c.entry);
    } else {
        cross(ui.painter(), colour);
    }
    response.clicked()
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

/// A number in an entry of the look, with − and + beside it (on macOS a stepper after it,
/// except on a touch screen) unless [`NumberField::steppers`] leaves them out, stepping
/// by `step` and staying inside the [`NumberField::range`] if it has one; drag it
/// sideways to scrub, click it to type, which selects the number so typing
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
    /// Whether − and + (macOS's stepper) stand beside the number; by default they do.
    /// Without them the field alone remains, for several to a row.
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
        super::unfaded(ui, |ui| self.add_contents(ui))
    }
}

impl<T: egui::emath::Numeric> NumberField<'_, T> {
    fn add_contents(self, ui: &mut Ui) -> Response {
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
        // macOS puts a narrow stepper after the field; it is too small for a finger, so
        // touch screens keep − and + on either side.
        let stepper = steppers && x.look == Look::Mac && super::minimum_target(ui) == 0.0;
        // iOS puts a capsule of − and + after the field, as its stepper is.
        let capsule = steppers && x.look == Look::Ios;
        // GTK's spin button joins − and + to the field's end; in a boxed list's row (a
        // spin row) it has no field, its number before two round buttons.
        let linked = steppers && x.look == Look::Gtk;
        let spin_row = linked && super::form::in_boxed_row(ui.ctx());
        // The row is as tall as its tallest part from the start, so a macOS field taller than
        // the controls stays centred with them.
        let row = egui::vec2(
            ui.available_width(),
            if x.look == Look::Mac && super::minimum_target(ui) == 0.0 {
                mac_field_height(ui, false).max(ui.spacing().interact_size.y)
            } else {
                ui.spacing().interact_size.y
            },
        );
        let inner = ui.allocate_ui_with_layout(
            row,
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing.x = if linked { 0.0 } else { 4.0 };
                let side = ui.spacing().interact_size.y.max(super::minimum_target(ui));
                let minus = (steppers && !stepper && !capsule && !linked)
                    .then(|| step_button(ui, value.to_f64() > lo, false, &step_names[0]));
                if let Some(minus) = &minus {
                    let count = f64::from(u8::from(minus.clicked())) + held(ui, minus);
                    if count > 0.0 {
                        *value = T::from_f64(stepped_number(value.to_f64(), step, -count, lo, hi));
                    }
                }
                let buttons = match (steppers, stepper) {
                    (false, _) => 0.0,
                    (true, true) => mac_stepper_width(ui) + 4.0,
                    (true, false) if capsule => IOS_STEPPER.x + 8.0,
                    (true, false) if spin_row => GTK_SPIN_ROW,
                    (true, false) if linked => 2.0 * GTK_SPIN_BUTTON,
                    (true, false) => 2.0 * side + 8.0,
                };
                let number_width = if spin_row {
                    40.0
                } else {
                    width.map_or(64.0, |w| (w - buttons).max(40.0))
                };
                // A macOS number field is a text field, as tall as one, with a stepper or without.
                let mac_field = x.look == Look::Mac && super::minimum_target(ui) == 0.0;
                let size = if mac_field {
                    egui::vec2(number_width, mac_field_height(ui, false))
                } else {
                    egui::vec2(number_width, side)
                };
                let mut shown = if value.to_f64().is_nan() {
                    T::from_f64(lo)
                } else {
                    *value
                };
                let original = value.to_f64();
                let number = ui.scope(|ui| {
                    entry_visuals(ui, &x);
                    if linked {
                        let w = &mut ui.visuals_mut().widgets;
                        for state in [
                            &mut w.noninteractive,
                            &mut w.inactive,
                            &mut w.hovered,
                            &mut w.active,
                        ] {
                            state.corner_radius.ne = 0;
                            state.corner_radius.se = 0;
                            if spin_row {
                                state.bg_fill = Color32::TRANSPARENT;
                                state.weak_bg_fill = Color32::TRANSPARENT;
                                state.bg_stroke = Stroke::NONE;
                            }
                        }
                    }
                    if !ui.is_enabled() {
                        // A disabled DragValue draws its number in the non-interactive colour.
                        ui.visuals_mut().widgets.noninteractive.fg_stroke.color = x.c.disabled;
                    }
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
                let (minus, plus) = if stepper || capsule || linked {
                    let enabled = [value.to_f64() > lo, value.to_f64() < hi];
                    let (down, up) = if capsule {
                        ui.add_space(4.0);
                        ios_stepper(ui, &x, enabled, &step_names)
                    } else if spin_row {
                        gtk_spin_row_buttons(ui, &x, enabled, &step_names)
                    } else if linked {
                        gtk_spin_buttons(ui, &x, enabled, &step_names)
                    } else {
                        mac_stepper(ui, &x, enabled, &step_names)
                    };
                    let count = f64::from(u8::from(down.clicked())) + held(ui, &down);
                    if count > 0.0 {
                        *value = T::from_f64(stepped_number(value.to_f64(), step, -count, lo, hi));
                    }
                    (Some(down), Some(up))
                } else {
                    let plus = steppers
                        .then(|| step_button(ui, value.to_f64() < hi, true, &step_names[1]));
                    (minus, plus)
                };
                if let Some(plus) = &plus {
                    let count = f64::from(u8::from(plus.clicked())) + held(ui, plus);
                    if count > 0.0 {
                        *value = T::from_f64(stepped_number(value.to_f64(), step, count, lo, hi));
                    }
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
            },
        );
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
/// fill, and the look's thin edge (none in libadwaita and Material, which fill theirs; a
/// hairline on iOS).
fn entry_visuals(ui: &mut Ui, x: &Ctx) {
    let c = &x.c;
    let edge = match x.look {
        _ if c.high_contrast => Stroke::new(1.0, c.control_border),
        Look::Gtk | Look::Material => Stroke::NONE,
        // iOS's rounded-border field: a hairline.
        Look::Ios => Stroke::new(0.5, c.control_border),
        Look::Mac | Look::Windows => Stroke::new(1.0, c.control_border),
        Look::Plain => Stroke::new(1.0, c.border),
    };
    let w = &mut ui.visuals_mut().widgets;
    // A disabled entry is drawn with the non-interactive style.
    for (state, fill) in [
        (&mut w.noninteractive, c.entry),
        (&mut w.inactive, c.entry),
        (&mut w.hovered, c.entry),
        (&mut w.active, c.entry_focused),
    ] {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.bg_stroke = edge;
        match x.look {
            Look::Ios => state.corner_radius = egui::CornerRadius::same(5),
            // Material 3's filled field: rounded above its indicator line only.
            Look::Material => {
                state.corner_radius = egui::CornerRadius {
                    nw: 4,
                    ne: 4,
                    sw: 0,
                    se: 0,
                }
            }
            _ => {}
        }
    }
    if x.look == Look::Plain {
        w.hovered.bg_stroke = Stroke::new(1.0, c.grid);
    }
    // Where the look draws its own focus ring or, in Material, only thickens its indicator,
    // egui's focused outline is left out and selected text keeps the text colour.
    if matches!(x.look, Look::Mac | Look::Gtk | Look::Ios | Look::Material) {
        ui.visuals_mut().selection.stroke = Stroke::new(0.0, c.fg);
    }
    // iOS's placeholder is its tertiary label, fainter than the secondary.
    if x.look == Look::Ios {
        ui.visuals_mut().weak_text_color = Some(c.disabled);
    }
}

/// The steps a held − or + adds this frame, as a count of steps.
fn held(ui: &Ui, button: &Response) -> f64 {
    f64::from(super::held_repeats(ui, button))
}

/// The height of a macOS text field beside controls `interact_size.y` tall, as SwiftUI
/// sizes them: a point above and below a regular 24-point button, 23 points for small
/// controls and no taller than 26 for large ones; a search field always two points above
/// the controls.
fn mac_field_height(ui: &Ui, search: bool) -> f32 {
    let controls = ui.spacing().interact_size.y;
    if search {
        controls + 2.0
    } else {
        (26.0 + (controls - 24.0).min(0.0) * 0.75).round()
    }
}

/// The size of iOS's stepper capsule.
const IOS_STEPPER: egui::Vec2 = egui::vec2(94.0, 32.0);

/// iOS's stepper: a capsule of − and +, each half its own button, divided by a short
/// line; named by `names` (decrease, increase) and enabled by `enabled`. On a touch screen
/// each half reserves a finger-tall target. Returns the − and + halves.
fn ios_stepper(
    ui: &mut Ui,
    x: &Ctx,
    enabled: [bool; 2],
    names: &[String; 2],
) -> (Response, Response) {
    let c = &x.c;
    let id = ui.next_auto_id();
    let height = IOS_STEPPER.y.max(super::minimum_target(ui));
    let (rect, _) = ui.allocate_exact_size(egui::vec2(IOS_STEPPER.x, height), Sense::hover());
    let body = Rect::from_center_size(rect.center(), IOS_STEPPER);
    let middle = body.center().x;
    let fill = crate::ink(c, 0.07, 0.066);
    let round = egui::CornerRadius::same((body.height() / 2.0) as u8);
    ui.painter().rect_filled(body, round, fill);
    let [down, up] = [0, 1].map(|i| {
        let half = if i == 0 {
            Rect::from_min_max(rect.min, egui::pos2(middle, rect.max.y))
        } else {
            Rect::from_min_max(egui::pos2(middle, rect.min.y), rect.max)
        };
        let on = enabled[i] && ui.is_enabled();
        let mut response = ui.interact(
            half,
            id.with(i),
            if on { Sense::click() } else { Sense::hover() },
        );
        response.flags.set(egui::response::Flags::ENABLED, on);
        let response = super::click_response(ui, response);
        if response.is_pointer_button_down_on() {
            let visible = half.intersect(body);
            let corners = if i == 0 {
                egui::CornerRadius {
                    nw: round.nw,
                    sw: round.sw,
                    ..Default::default()
                }
            } else {
                egui::CornerRadius {
                    ne: round.ne,
                    se: round.se,
                    ..Default::default()
                }
            };
            ui.painter().rect_filled(visible, corners, c.button_pressed);
        }
        let m = egui::pos2(half.center().x, body.center().y);
        let colour = if on { c.fg } else { c.disabled };
        let stroke = Stroke::new(2.0, colour);
        ui.painter()
            .line_segment([m - egui::vec2(6.5, 0.0), m + egui::vec2(6.5, 0.0)], stroke);
        if i == 1 {
            ui.painter()
                .line_segment([m - egui::vec2(0.0, 6.5), m + egui::vec2(0.0, 6.5)], stroke);
        }
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, on, names[i].as_str()));
        if response.has_focus() {
            crate::paint::ring(ui.painter(), &x.paint(), body, round);
        }
        response
    });
    let line = crate::ink(c, 0.33, 0.27);
    ui.painter().line_segment(
        [
            egui::pos2(middle, body.center().y - 9.0),
            egui::pos2(middle, body.center().y + 9.0),
        ],
        Stroke::new(0.5, line),
    );
    (down, up)
}

/// The width of each of GTK's spin buttons: 32 points and the line before it.
const GTK_SPIN_BUTTON: f32 = 33.0;

/// GTK's spin buttons: − and + joined to the end of the field before them, each after a
/// faint line, sharing the field's fill and rounded only at the end. Named by `names`
/// (decrease, increase) and enabled by `enabled`; returns both.
fn gtk_spin_buttons(
    ui: &mut Ui,
    x: &Ctx,
    enabled: [bool; 2],
    names: &[String; 2],
) -> (Response, Response) {
    let c = &x.c;
    let id = ui.next_auto_id();
    let height = ui.spacing().interact_size.y.max(super::minimum_target(ui));
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(2.0 * GTK_SPIN_BUTTON, height), Sense::hover());
    let body = Rect::from_center_size(
        rect.center(),
        egui::vec2(rect.width(), ui.spacing().interact_size.y),
    );
    let r = x.m.radius;
    let end = egui::CornerRadius {
        ne: r,
        se: r,
        ..Default::default()
    };
    ui.painter().rect_filled(body, end, c.entry);
    let [down, up] = [0, 1].map(|i| {
        let left = body.left() + GTK_SPIN_BUTTON * i as f32;
        let half = Rect::from_min_max(
            egui::pos2(left, rect.top()),
            egui::pos2(left + GTK_SPIN_BUTTON, rect.bottom()),
        );
        let on = enabled[i] && ui.is_enabled();
        let mut response = ui.interact(
            half,
            id.with(i),
            if on { Sense::click() } else { Sense::hover() },
        );
        response.flags.set(egui::response::Flags::ENABLED, on);
        let response = super::click_response(ui, response);
        let face = Rect::from_min_max(
            egui::pos2(half.left() + 1.0, body.top()),
            egui::pos2(half.right(), body.bottom()),
        );
        let corners = if i == 1 {
            end
        } else {
            egui::CornerRadius::ZERO
        };
        if on && response.is_pointer_button_down_on() {
            ui.painter().rect_filled(face, corners, c.button_pressed);
        } else if on && response.hovered() {
            ui.painter().rect_filled(face, corners, c.button);
        }
        ui.painter().vline(
            half.left() + 0.5,
            body.y_range(),
            Stroke::new(1.0, c.button),
        );
        let m = egui::pos2(face.center().x, body.center().y);
        let stroke = Stroke::new(2.0, if on { c.fg } else { c.disabled });
        ui.painter()
            .line_segment([m - egui::vec2(6.0, 0.0), m + egui::vec2(6.0, 0.0)], stroke);
        if i == 1 {
            ui.painter()
                .line_segment([m - egui::vec2(0.0, 6.0), m + egui::vec2(0.0, 6.0)], stroke);
        }
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, on, names[i].as_str()));
        if response.has_focus() {
            crate::paint::ring(ui.painter(), &x.paint(), face, corners);
        }
        response
    });
    (down, up)
}

/// The width of a libadwaita spin row's buttons: two 30-point circles, 2 points of margin
/// round each, six points after the number.
const GTK_SPIN_ROW: f32 = 6.0 + 2.0 * 34.0;

/// A libadwaita spin row's − and +: round buttons in the buttons' fill, without a field.
/// Named by `names` (decrease, increase) and enabled by `enabled`; returns both.
fn gtk_spin_row_buttons(
    ui: &mut Ui,
    x: &Ctx,
    enabled: [bool; 2],
    names: &[String; 2],
) -> (Response, Response) {
    let c = &x.c;
    let id = ui.next_auto_id();
    let height = ui.spacing().interact_size.y.max(super::minimum_target(ui));
    let (rect, _) = ui.allocate_exact_size(egui::vec2(GTK_SPIN_ROW, height), Sense::hover());
    let [down, up] = [0, 1].map(|i| {
        let centre = egui::pos2(rect.left() + 6.0 + 17.0 + 34.0 * i as f32, rect.center().y);
        let hit = Rect::from_center_size(centre, egui::vec2(34.0, height));
        let on = enabled[i] && ui.is_enabled();
        let mut response = ui.interact(
            hit,
            id.with(i),
            if on { Sense::click() } else { Sense::hover() },
        );
        response.flags.set(egui::response::Flags::ENABLED, on);
        let response = super::click_response(ui, response);
        let fill = if on && response.is_pointer_button_down_on() {
            c.button_pressed
        } else if on && response.hovered() {
            c.button_hover
        } else {
            c.button
        };
        ui.painter().circle_filled(centre, 15.0, fill);
        let stroke = Stroke::new(2.0, if on { c.fg } else { c.disabled });
        ui.painter().line_segment(
            [centre - egui::vec2(6.0, 0.0), centre + egui::vec2(6.0, 0.0)],
            stroke,
        );
        if i == 1 {
            ui.painter().line_segment(
                [centre - egui::vec2(0.0, 6.0), centre + egui::vec2(0.0, 6.0)],
                stroke,
            );
        }
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, on, names[i].as_str()));
        if response.has_focus() {
            let round = Rect::from_center_size(centre, egui::vec2(30.0, 30.0));
            crate::paint::ring(
                ui.painter(),
                &x.paint(),
                round,
                egui::CornerRadius::same(15),
            );
        }
        response
    });
    (down, up)
}

/// The width of macOS's stepper: 20 points beside regular controls.
fn mac_stepper_width(ui: &Ui) -> f32 {
    (20.0 * super::mac_scale(ui)).round()
}

/// macOS's stepper: a narrow rounded body with an up arrow over a down arrow, each half
/// its own button, named by `names` (decrease, increase) and enabled by `enabled`.
/// Returns the down and up halves.
fn mac_stepper(
    ui: &mut Ui,
    x: &Ctx,
    enabled: [bool; 2],
    names: &[String; 2],
) -> (Response, Response) {
    let c = &x.c;
    let height = ui.spacing().interact_size.y;
    let id = ui.next_auto_id();
    let scale = super::mac_scale(ui);
    let width = mac_stepper_width(ui);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), Sense::hover());
    let body = Rect::from_center_size(rect.center(), egui::vec2(width, height.min(24.0 * scale)));
    let middle = body.center().y;
    let round = egui::CornerRadius::same(5);
    let fill = if ui.is_enabled() {
        c.button
    } else {
        c.button_disabled
    };
    ui.painter().rect_filled(body, round, fill);
    let halves = [
        (
            Rect::from_min_max(egui::pos2(body.left(), middle), body.max),
            -1.0,
            egui::CornerRadius {
                sw: 5,
                se: 5,
                ..Default::default()
            },
        ),
        (
            Rect::from_min_max(body.min, egui::pos2(body.right(), middle)),
            1.0,
            egui::CornerRadius {
                nw: 5,
                ne: 5,
                ..Default::default()
            },
        ),
    ];
    let [down, up] = [0, 1].map(|i| {
        let (half, direction, corners) = halves[i];
        let on = enabled[i] && ui.is_enabled();
        let mut response = ui.interact(
            half,
            id.with(i),
            if on { Sense::click() } else { Sense::hover() },
        );
        response.flags.set(egui::response::Flags::ENABLED, on);
        let response = super::click_response(ui, response);
        if response.is_pointer_button_down_on() {
            ui.painter().rect_filled(half, corners, c.button_pressed);
        }
        // Each chevron sits six points from the middle, ten wide and six tall.
        let m = egui::pos2(half.center().x, middle - direction * 6.0 * scale);
        let colour = if on { c.fg } else { c.disabled };
        let (w, h) = (4.5 * scale, 2.75 * scale);
        ui.painter().add(egui::Shape::line(
            vec![
                m + egui::vec2(-w, direction * h),
                m + egui::vec2(0.0, -direction * h),
                m + egui::vec2(w, direction * h),
            ],
            Stroke::new(1.6, colour),
        ));
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, on, names[i].as_str()));
        if response.has_focus() {
            crate::paint::ring(ui.painter(), &x.paint(), half, corners);
        }
        response
    });
    let ink = if c.dark {
        Color32::WHITE
    } else {
        Color32::BLACK
    };
    ui.painter().line_segment(
        [
            egui::pos2(body.left() + 4.0, middle),
            egui::pos2(body.right() - 4.0, middle),
        ],
        Stroke::new(1.0, ink.gamma_multiply(0.15)),
    );
    (down, up)
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
