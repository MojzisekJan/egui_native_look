//! Push buttons, switches, check boxes and radio buttons.

use egui::{Color32, CornerRadius, Rect, Response, Sense, Stroke, Ui, Widget, WidgetText};

use super::Ctx;
use crate::{lerp_color, Look};

/// A push button, as tall as the controls around it and as wide as its label (or
/// [`Button::width`]).
///
/// The builder must be added to a UI or shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::Button;
/// Button::new("Save");
/// ```
#[must_use = "add the widget to a UI or call its show method"]
pub struct Button {
    label: String,
    icon: Option<(String, egui::FontId)>,
    accent: bool,
    default_action: bool,
    width: Option<f32>,
}

impl Button {
    /// A button reading `label`.
    pub fn new(label: impl Into<String>) -> Self {
        Button {
            label: label.into(),
            icon: None,
            accent: false,
            default_action: false,
            width: None,
        }
    }
    /// A decorative glyph before the label, using `font` for an icon font or text.
    /// The label remains the button's accessible name.
    ///
    /// ```no_run
    /// # fn demo(ui: &mut egui::Ui) {
    /// use egui_native_look::Button;
    /// ui.add(Button::new("Add item").icon("+", egui::FontId::proportional(16.0)));
    /// # }
    /// ```
    pub fn icon(mut self, glyph: impl Into<String>, font: egui::FontId) -> Self {
        let glyph = glyph.into();
        self.icon = (!glyph.is_empty()).then_some((glyph, font));
        self
    }
    /// Paint in the accent colour for a prominent action or selected state.
    /// Use [`Self::default_action`] separately to activate a dialog action with Enter.
    pub fn accent(mut self) -> Self {
        self.accent = true;
        self
    }
    /// Activate on Enter in a [`crate::Dialog`] when a [`crate::TextField`] or no
    /// control has focus. A focused button retains its own activation. Open popups
    /// and input method confirmation suppress the default action. Appearance is unchanged.
    pub fn default_action(mut self) -> Self {
        self.default_action = true;
        self
    }
    /// In the accent when `on`.
    pub fn selected(mut self, on: bool) -> Self {
        self.accent = on;
        self
    }
    /// A fixed width.
    /// Non-positive and non-finite widths restore the automatic width. Widths above
    /// 1,000,000 points are capped to that value; positive widths below one point use one point.
    pub fn width(mut self, width: f32) -> Self {
        self.width = super::dimension(width);
        self
    }
}

impl Widget for Button {
    fn ui(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        let icon = self.icon.as_ref().map(|(glyph, font)| {
            ui.fonts_mut(|fonts| fonts.layout_no_wrap(glyph.clone(), font.clone(), x.c.fg))
        });
        let height = ui
            .spacing()
            .interact_size
            .y
            .max(icon.as_ref().map_or(0.0, |icon| {
                icon.size().y + 2.0 * ui.spacing().button_padding.y
            }));
        let gap = if icon.is_some() && !self.label.is_empty() {
            6.0
        } else {
            0.0
        };
        let icon_width = icon.as_ref().map_or(0.0, |icon| icon.size().x);
        let text_width = ui.fonts_mut(|f| {
            f.layout_no_wrap(self.label.clone(), x.m.body.clone(), x.c.fg)
                .size()
                .x
        });
        let width = self.width.unwrap_or(
            (text_width + icon_width + gap + 2.0 * ui.spacing().button_padding.x.max(10.0))
                .max(height),
        );
        let enabled = ui.is_enabled();
        let (rect, mut response) = super::allocate_control(
            ui,
            egui::vec2(width, height),
            if enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        super::window::dialog_button(ui, &mut response, self.default_action);
        if ui.is_rect_visible(rect) {
            crate::paint::button(
                ui.painter(),
                &x.paint(),
                rect,
                &response,
                if icon.is_some() { "" } else { &self.label },
                x.m.body.clone(),
                enabled,
                self.accent,
            );
            if let Some(icon) = icon {
                let color = if !enabled {
                    x.c.disabled
                } else if self.accent {
                    x.c.on_accent
                } else if response.is_pointer_button_down_on() && x.look == Look::Windows {
                    x.c.dim
                } else {
                    x.c.fg
                };
                let available = (rect.width() - 8.0).max(0.0);
                let glyph_width = icon_width.min(available);
                let gap = gap.min((available - glyph_width).max(0.0));
                let (label, _) = super::truncated(
                    ui,
                    &self.label,
                    x.m.body.clone(),
                    color,
                    available - glyph_width - gap,
                );
                let left = rect.center().x - (glyph_width + gap + label.size().x) / 2.0;
                let painter = ui
                    .painter()
                    .with_clip_rect(ui.clip_rect().intersect(rect.shrink(4.0)));
                let top = crate::galley_top(ui.ctx(), &icon, rect.center().y);
                painter.galley_with_override_text_color(egui::pos2(left, top), icon, color);
                let top = crate::galley_top(ui.ctx(), &label, rect.center().y);
                painter.galley(egui::pos2(left + glyph_width + gap, top), label, color);
            }
        }
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, &self.label));
        response
    }
}

/// An on/off switch. The value has changed when `response.changed()`.
/// In the GTK look, dragging the thumb previews its position and commits the
/// nearest state on release. Dragging from the empty half does not change it.
///
/// The builder must be added to a UI or shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::Switch;
/// Switch::new(&mut false);
/// ```
#[must_use = "add the widget to a UI or call its show method"]
pub struct Switch<'a> {
    on: &'a mut bool,
    label: String,
}

impl<'a> Switch<'a> {
    /// A switch showing and flipping `on`.
    pub fn new(on: &'a mut bool) -> Self {
        Switch {
            on,
            label: String::new(),
        }
    }
    /// What it switches, for a screen reader, as a switch draws no label itself. In a
    /// [`crate::Form`] row the row's label is used when this is not set.
    pub fn accessible_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
}

/// The width a [`Switch`] takes in `look`, for a row that leaves room for one at its end.
#[must_use]
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
        let (outer, mut response) =
            super::allocate_touch(ui, egui::vec2(size.x + 2.0, height), Sense::click());
        response = super::click_response(ui, response);
        let rect = Rect::from_center_size(outer.center(), size);
        let before = *self.on;
        let preview = if x.look == Look::Gtk {
            switch_drag(ui, &response, rect, self.on)
        } else {
            if response.clicked() {
                *self.on = !*self.on;
            }
            None
        };
        if *self.on != before {
            response.mark_changed();
        }
        let on = *self.on;
        let enabled = ui.is_enabled();
        let name = super::form::accessible_name(ui, &self.label);
        response.widget_info(|| egui::WidgetInfo::selected(egui::Role::Switch, enabled, on, &name));
        if !ui.is_rect_visible(rect) {
            return response;
        }
        let t = preview.unwrap_or_else(|| ui.ctx().animate_bool_responsive(response.id, on));
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
                    painter.rect_stroke(
                        rect,
                        round,
                        Stroke::new(2.0, c.control_strong.gamma_multiply(1.0 - t)),
                        egui::StrokeKind::Inside,
                    );
                }
                let r = egui::lerp(8.0..=12.0, t) + if hovered { 1.0 } else { 0.0 };
                let x = egui::lerp((rect.left() + 16.0)..=(rect.right() - 16.0), t);
                painter.circle_filled(
                    egui::pos2(x, rect.center().y),
                    r,
                    lerp_color(c.control_strong, c.on_accent, t),
                );
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
                    let edge = if c.dark {
                        Color32::from_white_alpha(14)
                    } else {
                        Color32::from_black_alpha(14)
                    };
                    painter.rect_stroke(
                        rect,
                        round,
                        Stroke::new(1.0, edge.gamma_multiply(1.0 - t)),
                        egui::StrokeKind::Inside,
                    );
                }
                knob(painter, rect, t, 1.5, enabled, c.dark, 0.5);
            }
            Look::Windows => {
                // An outlined pill with a small knob in the strong stroke's colour when
                // off, the accent with a knob in the accent's text colour when on; the
                // knob grows under the pointer.
                let strong = if enabled {
                    c.control_strong
                } else {
                    c.disabled
                };
                if t > 0.0 {
                    let fill = if hovered { c.accent_hover } else { c.accent };
                    painter.rect_filled(
                        rect,
                        round,
                        if enabled { fill } else { c.disabled }.gamma_multiply(t),
                    );
                }
                if t < 1.0 {
                    painter.rect_filled(rect, round, c.track.gamma_multiply(1.0 - t));
                    painter.rect_stroke(
                        rect,
                        round,
                        Stroke::new(1.0, strong.gamma_multiply(1.0 - t)),
                        egui::StrokeKind::Inside,
                    );
                }
                let r = if hovered { 7.0 } else { 6.0 };
                let x = egui::lerp((rect.left() + 9.0)..=(rect.right() - 9.0), t);
                painter.circle_filled(
                    egui::pos2(x, rect.center().y),
                    r,
                    lerp_color(strong, c.on_accent, t),
                );
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
                    painter.rect_stroke(
                        rect,
                        round,
                        Stroke::new(1.0, c.grid),
                        egui::StrokeKind::Inside,
                    );
                }
                let r = size.y / 2.0 - 3.0;
                let x = egui::lerp((rect.left() + 3.0 + r)..=(rect.right() - 3.0 - r), t);
                let fill = lerp_color(c.dim, Color32::WHITE, t);
                painter.circle_filled(
                    egui::pos2(x, rect.center().y),
                    r,
                    if enabled { fill } else { c.disabled },
                );
            }
        }
        if response.has_focus() {
            crate::paint::ring(painter, &x.paint(), rect, round);
        }
        response
    }
}

#[derive(Clone, Copy)]
struct SwitchDrag {
    press_time: Option<f64>,
    seen_frame: u64,
    origin: egui::Pos2,
    initial: f32,
    position: f32,
    eligible: bool,
    dragging: bool,
}

/// Preview a thumb drag without changing the model until the primary button is released.
fn switch_drag(ui: &mut Ui, response: &Response, rect: Rect, on: &mut bool) -> Option<f32> {
    let id = response.id.with("thumb-drag");
    let (pressed, released, down, cancelled, press_time) = ui.input(|i| {
        (
            i.pointer.primary_pressed(),
            i.pointer.primary_released(),
            i.pointer.primary_down(),
            super::pointer_cancelled(i),
            i.pointer.press_start_time(),
        )
    });
    let frame = ui.ctx().cumulative_frame_nr();
    let stale = ui
        .data(|data| data.get_temp::<SwitchDrag>(id))
        .is_some_and(|state| {
            (!down && !released)
                || (down && state.press_time != press_time)
                || frame.saturating_sub(state.seen_frame) > 1
        });
    if stale {
        ui.data_mut(|data| data.remove::<SwitchDrag>(id));
        if ui.ctx().dragged_id() == Some(response.id) {
            ui.ctx().stop_dragging();
        }
    }
    if !response.enabled() || cancelled {
        ui.data_mut(|data| data.remove::<SwitchDrag>(id));
        if ui.ctx().dragged_id() == Some(response.id) {
            ui.ctx().stop_dragging();
        }
        return None;
    }
    if pressed && response.is_pointer_button_down_on() {
        response.request_focus();
        if let Some(origin) = response.interact_pointer_pos() {
            let initial = if *on { 1.0 } else { 0.0 };
            ui.data_mut(|data| {
                data.insert_temp(
                    id,
                    SwitchDrag {
                        press_time,
                        seen_frame: frame,
                        origin,
                        initial,
                        position: initial,
                        eligible: if *on {
                            origin.x >= rect.center().x
                        } else {
                            origin.x <= rect.center().x
                        },
                        dragging: false,
                    },
                )
            });
        }
    }
    let mut drag = ui.data(|data| data.get_temp::<SwitchDrag>(id));
    if let Some(state) = drag.as_mut() {
        let position = super::pointer_position(ui, response);
        if let Some(pos) = position {
            let delta = pos - state.origin;
            let decided = ui.input(|i| i.pointer.is_decidedly_dragging());
            if state.eligible && (decided || state.dragging) {
                if !state.dragging && delta.y.abs() >= delta.x.abs() {
                    state.eligible = false;
                } else {
                    state.dragging = true;
                    state.position =
                        (state.initial + delta.x / (rect.width() * 0.5)).clamp(0.0, 1.0);
                    if down {
                        // Claim horizontal movement only; vertical gestures stay with the scroll area.
                        ui.ctx().set_dragged_id(response.id);
                    }
                }
            }
        }
        if released && state.dragging {
            *on = state.position >= 0.5;
            ui.ctx().animate_bool_with_time(response.id, *on, 0.0);
        } else if response.clicked() {
            *on = !*on;
        }
    } else if response.clicked()
        && (pressed
            || response
                .flags
                .contains(egui::response::Flags::FAKE_PRIMARY_CLICKED))
    {
        *on = !*on;
    }
    let preview = drag
        .filter(|state| down && state.dragging)
        .map(|state| state.position);
    if down {
        if let Some(mut state) = drag {
            state.seen_frame = frame;
            ui.data_mut(|data| data.insert_temp(id, state));
        }
    } else {
        ui.data_mut(|data| data.remove::<SwitchDrag>(id));
    }
    preview
}

/// A white switch knob inset by `inset`, with a shadow `drop` points below it.
fn knob(
    painter: &egui::Painter,
    rect: Rect,
    t: f32,
    inset: f32,
    enabled: bool,
    dark: bool,
    drop: f32,
) {
    let r = rect.height() / 2.0 - inset;
    let x = egui::lerp((rect.left() + inset + r)..=(rect.right() - inset - r), t);
    let c = egui::pos2(x, rect.center().y);
    painter.circle_filled(
        c + egui::vec2(0.0, drop),
        r + 0.5,
        Color32::from_black_alpha(if enabled { 48 } else { 20 }),
    );
    painter.circle_filled(
        c,
        r,
        if enabled {
            Color32::WHITE
        } else {
            Color32::from_gray(if dark { 145 } else { 245 })
        },
    );
}

/// A check box with its label: on macOS a small rounded box, on Windows WinUI's larger
/// outlined one, in libadwaita a box outlined in a tint of the text colour, each filled
/// with the accent and ticked when on; egui's own in the plain look. An indeterminate one
/// shows a dash, for a mixed selection.
///
/// The builder must be added to a UI or shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::Checkbox;
/// Checkbox::new(&mut false, "Snap");
/// ```
#[must_use = "add the widget to a UI or call its show method"]
pub struct Checkbox<'a> {
    checked: &'a mut bool,
    text: WidgetText,
    indeterminate: bool,
}

impl<'a> Checkbox<'a> {
    /// A check box showing and flipping `checked`, with `text` after it.
    pub fn new(checked: &'a mut bool, text: impl Into<WidgetText>) -> Self {
        Checkbox {
            checked,
            text: text.into(),
            indeterminate: false,
        }
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
            let label = self.text.text().to_owned();
            let events_before = ui.ctx().output(|output| output.events.len());
            let mut checked = *self.checked;
            let target = super::minimum_target(ui);
            let response = ui
                .scope(|ui| {
                    ui.spacing_mut().interact_size =
                        ui.spacing().interact_size.max(egui::Vec2::splat(target));
                    ui.add(
                        egui::Checkbox::new(&mut checked, self.text)
                            .indeterminate(self.indeterminate),
                    )
                })
                .inner;
            let mut response = super::click_response(ui, response);
            if response.clicked() {
                *self.checked = checked;
            } else {
                response.flags.remove(egui::response::Flags::CHANGED);
                if checked != *self.checked {
                    ui.ctx()
                        .output_mut(|output| output.events.truncate(events_before));
                    ui.ctx().request_repaint();
                }
            }
            response.widget_info(|| {
                let mut info = egui::WidgetInfo::selected(
                    egui::Role::CheckBox,
                    ui.is_enabled(),
                    *self.checked,
                    &label,
                );
                info.selected = (!self.indeterminate).then_some(*self.checked);
                info
            });
            return response;
        }
        let (mut response, mark, mut info) =
            mark_row(ui, &x, self.text, egui::Role::CheckBox, *self.checked);
        if response.clicked() {
            *self.checked = !*self.checked;
            response.mark_changed();
        }
        info.selected = (!self.indeterminate).then_some(*self.checked);
        response.widget_info(|| info.clone());
        let painter = ui.painter();
        let round = match x.look {
            Look::Ios => CornerRadius::same((mark.width() / 2.0) as u8),
            Look::Material => CornerRadius::same(2),
            _ => CornerRadius::same(4),
        };
        let enabled = ui.is_enabled();
        let c = &x.c;
        if *self.checked || self.indeterminate {
            painter.rect_filled(
                mark,
                round,
                if enabled {
                    c.accent
                } else {
                    c.accent.gamma_multiply(0.5)
                },
            );
            if self.indeterminate {
                let m = mark.center();
                painter.line_segment(
                    [m - egui::vec2(3.5, 0.0), m + egui::vec2(3.5, 0.0)],
                    Stroke::new(1.8, c.on_accent),
                );
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
///
/// The builder must be added to a UI or shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::Radio;
/// Radio::new(false, "Move");
/// ```
#[must_use = "add the widget to a UI or call its show method"]
pub struct Radio {
    selected: bool,
    text: WidgetText,
}

impl Radio {
    /// A radio button, chosen when `selected`, with `text` after it; see also
    /// [`radio_value`].
    pub fn new(selected: bool, text: impl Into<WidgetText>) -> Self {
        Radio {
            selected,
            text: text.into(),
        }
    }
}

impl Widget for Radio {
    fn ui(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        if !x.look.native() {
            let events_before = ui.ctx().output(|output| output.events.len());
            let target = super::minimum_target(ui);
            let response = ui
                .scope(|ui| {
                    ui.spacing_mut().interact_size =
                        ui.spacing().interact_size.max(egui::Vec2::splat(target));
                    ui.radio(self.selected, self.text)
                })
                .inner;
            let raw_click = response.clicked();
            let response = super::click_response(ui, response);
            if raw_click && !response.clicked() {
                ui.ctx()
                    .output_mut(|output| output.events.truncate(events_before));
            }
            return response;
        }
        let (response, mark, info) =
            mark_row(ui, &x, self.text, egui::Role::RadioButton, self.selected);
        response.widget_info(|| info.clone());
        let painter = ui.painter();
        let round = CornerRadius::same((mark.width() / 2.0) as u8);
        let enabled = ui.is_enabled();
        let c = &x.c;
        if self.selected && x.look == Look::Material {
            // Material 3: a ring and a dot, both in the primary.
            let colour = if enabled { c.accent } else { c.disabled };
            painter.circle_stroke(
                mark.center(),
                mark.width() / 2.0 - 1.0,
                Stroke::new(2.0, colour),
            );
            painter.circle_filled(mark.center(), 5.0, colour);
        } else if self.selected && x.look == Look::Ios {
            // iOS marks a chosen option with a tick in an accent circle.
            painter.circle_filled(
                mark.center(),
                mark.width() / 2.0,
                if enabled {
                    c.accent
                } else {
                    c.accent.gamma_multiply(0.5)
                },
            );
            check_mark(painter, mark.center(), c.on_accent);
        } else if self.selected {
            painter.circle_filled(
                mark.center(),
                mark.width() / 2.0,
                if enabled {
                    c.accent
                } else {
                    c.accent.gamma_multiply(0.5)
                },
            );
            // WinUI's dot is wide and grows under the pointer; the others keep a small one.
            let dot = if x.look == Look::Windows {
                if response.hovered() {
                    5.0
                } else {
                    4.5
                }
            } else {
                3.0
            };
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
pub fn radio_value<T: PartialEq>(
    ui: &mut Ui,
    current: &mut T,
    value: T,
    text: impl Into<WidgetText>,
) -> Response {
    let mut response = ui.add(Radio::new(*current == value, text));
    if response.clicked() && *current != value {
        *current = value;
        response.mark_changed();
    }
    response
}

/// A tick centred on `c`.
pub fn check_mark(painter: &egui::Painter, c: egui::Pos2, color: Color32) {
    painter.add(egui::Shape::line(
        vec![
            c + egui::vec2(-4.0, 0.0),
            c + egui::vec2(-1.2, 3.0),
            c + egui::vec2(4.0, -3.5),
        ],
        Stroke::new(1.6, color),
    ));
}

/// The row of a check box or radio button: the mark (14 points, 18 on Windows), then the
/// label, all of it one click target. The label wraps as the `Ui` wraps its text, the mark
/// beside its first line. Returns the response and the mark's square.
fn mark_row(
    ui: &mut Ui,
    x: &Ctx,
    text: WidgetText,
    kind: egui::Role,
    selected: bool,
) -> (Response, Rect, egui::WidgetInfo) {
    let (side, gap) = match x.look {
        Look::Windows => (18.0, 8.0),
        Look::Ios => (22.0, 10.0),
        Look::Material => (18.0, 12.0),
        _ => (14.0, 6.0),
    };
    let galley = text.into_galley(
        ui,
        None,
        ui.available_width() - side - gap,
        egui::FontSelection::FontId(x.m.body.clone()),
    );
    let width = side + gap + galley.size().x;
    let line = ui.spacing().interact_size.y.max(side);
    let first_row = galley
        .rows
        .first()
        .map_or(galley.size().y, |row| row.height());
    let height = line.max(galley.size().y + line - first_row);
    let (rect, response) = super::allocate_control(ui, egui::vec2(width, height), Sense::click());
    let middle = rect.top() + line / 2.0;
    let mark = Rect::from_center_size(
        egui::pos2(rect.left() + side / 2.0, middle),
        egui::vec2(side, side),
    );
    let enabled = ui.is_enabled();
    let colour = if enabled { x.c.fg } else { x.c.disabled };
    let top = crate::galley_top(ui.ctx(), &galley, middle);
    let info = egui::WidgetInfo::selected(kind, enabled, selected, galley.text());
    ui.painter()
        .galley(egui::pos2(mark.right() + gap, top), galley, colour);
    (response, mark, info)
}

/// An unchecked mark: on macOS raised and white on the light scheme, a faint well on the
/// dark; on Windows a faint fill inside the strong stroke; in libadwaita a two-point
/// outline in a tint of the text colour.
fn paint_mark_box(
    painter: &egui::Painter,
    x: &Ctx,
    mark: Rect,
    round: CornerRadius,
    enabled: bool,
    hovered: bool,
) {
    let c = &x.c;
    match x.look {
        Look::Gtk => {
            let tint = if hovered {
                c.button_pressed
            } else {
                c.button_hover
            };
            painter.rect_stroke(
                mark,
                round,
                Stroke::new(2.0, if enabled { tint } else { c.button_disabled }),
                egui::StrokeKind::Inside,
            );
        }
        Look::Ios | Look::Material => {
            // An outline in the strong stroke: two points thick in Material, one and a half
            // on iOS; Material lays a state layer over it under the pointer.
            if hovered && x.look == Look::Material {
                painter.circle_filled(mark.center(), mark.width() * 1.1, c.fg.gamma_multiply(0.08));
            }
            let width = if x.look == Look::Material { 2.0 } else { 1.5 };
            painter.rect_stroke(
                mark,
                round,
                Stroke::new(
                    width,
                    if enabled {
                        c.control_strong
                    } else {
                        c.disabled
                    },
                ),
                egui::StrokeKind::Inside,
            );
        }
        Look::Windows => {
            let fill = if c.dark {
                Color32::from_black_alpha(if hovered { 40 } else { 26 })
            } else {
                Color32::from_black_alpha(if hovered { 14 } else { 6 })
            };
            painter.rect_filled(mark, round, fill);
            painter.rect_stroke(
                mark,
                round,
                Stroke::new(
                    1.0,
                    if enabled {
                        c.control_strong
                    } else {
                        c.disabled
                    },
                ),
                egui::StrokeKind::Inside,
            );
        }
        _ if c.dark => {
            painter.rect_filled(
                mark,
                round,
                Color32::from_white_alpha(if enabled { 22 } else { 10 }),
            );
            painter.rect_stroke(
                mark,
                round,
                Stroke::new(
                    1.0,
                    Color32::from_white_alpha(if enabled { 30 } else { 14 }),
                ),
                egui::StrokeKind::Inside,
            );
        }
        _ => crate::paint::raised(
            painter,
            x.c.dark,
            mark,
            round,
            if enabled {
                Color32::WHITE
            } else {
                c.button_disabled
            },
            enabled,
        ),
    }
}
