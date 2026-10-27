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
    destructive: bool,
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
            destructive: false,
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
        self.destructive = false;
        self
    }
    /// Mark a destructive action, such as deleting an item, with a red fill.
    /// This changes appearance only; activation and confirmation remain the app's responsibility.
    pub fn destructive(mut self) -> Self {
        self.accent = true;
        self.destructive = true;
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
        self.destructive = false;
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
        super::unfaded(ui, |ui| self.add_contents(ui))
    }
}

impl Button {
    fn add_contents(self, ui: &mut Ui) -> Response {
        let mut x = Ctx::of(ui);
        if self.destructive {
            x.c.accent = crate::colors::rgb(if x.c.dark { 0xc01c28 } else { 0xe01b24 });
            x.c.on_accent = Color32::WHITE;
            x.c.accent_hover = lerp_color(x.c.accent, Color32::WHITE, 0.1);
        }
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
        let icon_width = icon.as_ref().map_or(0.0, |icon| {
            if x.look == Look::Gtk {
                icon.size().x.max(16.0)
            } else {
                icon.size().x
            }
        });
        // Material sets its button labels in the medium weight, libadwaita in bold.
        let font = if matches!(x.look, Look::Material | Look::Gtk) {
            x.m.bold.clone()
        } else {
            x.m.body.clone()
        };
        let text_width = ui.fonts_mut(|f| {
            f.layout_no_wrap(self.label.clone(), font.clone(), x.c.fg)
                .size()
                .x
        });
        // macOS pads its push buttons a little wider than WinUI and GTK, Material 3 its
        // capsules wider still.
        // A Material dialog's actions are text buttons: the label in the primary, without a
        // container, twelve points in.
        let text_button = x.look == Look::Material && super::window::in_dialog_actions(ui.ctx());
        let padding = match x.look {
            Look::Mac => (12.0 * super::mac_scale(ui)).round(),
            // WinUI's ButtonPadding: 11 either side.
            Look::Windows => 11.0,
            Look::Material if text_button => 12.0,
            Look::Material => 24.0,
            // libadwaita's text buttons.
            Look::Gtk if icon.is_some() => 9.0,
            Look::Gtk => 17.0,
            _ => 10.0,
        };
        // Material 3's buttons are at least 64 points wide.
        let least = if x.look == Look::Material {
            height.max(64.0)
        } else {
            height
        };
        // A libadwaita alert's responses: 44 points high, sharing the row equally, as a
        // WinUI dialog's buttons do.
        let in_actions = super::window::in_dialog_actions(ui.ctx());
        let gtk_response = x.look == Look::Gtk && in_actions;
        let response_width = (in_actions
            && matches!(x.look, Look::Gtk | Look::Windows | Look::Ios))
        .then(|| {
            super::window::shared_action_width(
                ui,
                text_width + icon_width + gap + 2.0 * ui.spacing().button_padding.x.max(padding),
            )
        })
        .flatten();
        // iOS 26's alert buttons: 48-point capsules, every one alike.
        let ios_alert = x.look == Look::Ios && in_actions;
        let height = if gtk_response {
            height.max(44.0)
        } else if ios_alert {
            height.max(48.0)
        } else {
            height
        };
        let width = self.width.or(response_width).unwrap_or(
            (text_width + icon_width + gap + 2.0 * ui.spacing().button_padding.x.max(padding))
                .max(least),
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
        if text_button && icon.is_none() && ui.is_rect_visible(rect) {
            let round = egui::CornerRadius::same((rect.height() / 2.0) as u8);
            let layer = if !enabled {
                0.0
            } else if response.is_pointer_button_down_on() {
                0.1
            } else if response.hovered() {
                0.08
            } else {
                0.0
            };
            if layer > 0.0 {
                ui.painter()
                    .rect_filled(rect, round, x.c.accent.gamma_multiply(layer));
            }
            let colour = if enabled { x.c.accent } else { x.c.disabled };
            let (galley, _) =
                super::truncated(ui, &self.label, font.clone(), colour, rect.width() - 8.0);
            let top = crate::galley_top(ui.ctx(), &galley, rect.center().y);
            ui.painter().galley(
                egui::pos2(rect.center().x - galley.size().x / 2.0, top),
                galley,
                colour,
            );
            if response.has_focus() {
                crate::paint::ring(ui.painter(), &x.paint(), rect, round);
            }
        } else if gtk_response && icon.is_none() && ui.is_rect_visible(rect) {
            let colour = if !enabled {
                x.c.disabled
            } else if self.destructive {
                crate::colors::rgb(if x.c.dark { 0xff7b63 } else { 0xc01c28 })
            } else if self.accent {
                x.c.accent
            } else {
                x.c.fg
            };
            let left_edge = rect.left() <= ui.max_rect().left() + 1.0;
            let right_edge = rect.right() >= ui.max_rect().right() - 1.0;
            let bottom =
                !ui.layout().main_dir().is_vertical() || super::window::last_dialog_action(ui);
            let corners = CornerRadius {
                sw: if left_edge && bottom { 12 } else { 0 },
                se: if right_edge && bottom { 12 } else { 0 },
                ..CornerRadius::ZERO
            };
            if enabled && (response.hovered() || response.is_pointer_button_down_on()) {
                ui.painter().rect_filled(rect, corners, x.c.row_hover);
            }
            ui.painter()
                .hline(rect.x_range(), rect.top(), Stroke::new(1.0, x.c.border));
            if !left_edge {
                ui.painter()
                    .vline(rect.left(), rect.y_range(), Stroke::new(1.0, x.c.border));
            }
            let (galley, _) =
                super::truncated(ui, &self.label, font.clone(), colour, rect.width() - 16.0);
            let top = crate::galley_top(ui.ctx(), &galley, rect.center().y);
            ui.painter().galley(
                egui::pos2(rect.center().x - galley.size().x / 2.0, top),
                galley,
                colour,
            );
            if response.has_focus() {
                crate::paint::ring(ui.painter(), &x.paint(), rect.shrink(1.0), corners);
            }
        } else if ios_alert && icon.is_none() && ui.is_rect_visible(rect) {
            // Grey capsules in the text colour, the default one too: black at an eighth or
            // white at a twelfth over the alert, darker pressed.
            let round = egui::CornerRadius::same((rect.height() / 2.0) as u8);
            let share = match (x.c.dark, response.is_pointer_button_down_on() && enabled) {
                (false, false) => 0.126,
                (false, true) => 0.2,
                (true, false) => 0.084,
                (true, true) => 0.16,
            };
            let ink = if x.c.dark {
                Color32::WHITE
            } else {
                Color32::BLACK
            };
            ui.painter()
                .rect_filled(rect, round, ink.gamma_multiply(share));
            let colour = if enabled { x.c.fg } else { x.c.disabled };
            let (galley, _) =
                super::truncated(ui, &self.label, font.clone(), colour, rect.width() - 16.0);
            let top = crate::galley_top(ui.ctx(), &galley, rect.center().y);
            ui.painter().galley(
                egui::pos2(rect.center().x - galley.size().x / 2.0, top),
                galley,
                colour,
            );
            if response.has_focus() {
                crate::paint::ring(ui.painter(), &x.paint(), rect, round);
            }
        } else if ui.is_rect_visible(rect) {
            let mut paint = x.paint();
            if gtk_response {
                // libadwaita rounds an alert's responses by 12 points.
                paint.radius = 12;
            }
            crate::paint::button(
                ui.painter(),
                &paint,
                rect,
                &response,
                if icon.is_some() { "" } else { &self.label },
                font.clone(),
                enabled,
                self.accent,
            );
            if let Some(icon) = icon {
                let color = if !enabled && x.look != Look::Gtk {
                    x.c.disabled
                } else if self.accent {
                    x.c.on_accent
                } else if x.look == Look::Ios {
                    x.c.accent
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
                    font.clone(),
                    color,
                    available - glyph_width - gap,
                );
                let left = rect.center().x - (glyph_width + gap + label.size().x) / 2.0;
                let painter = ui
                    .painter()
                    .with_clip_rect(ui.clip_rect().intersect(rect.shrink(4.0)));
                let top = crate::galley_top(ui.ctx(), &icon, rect.center().y);
                let icon_left = left + (glyph_width - icon.size().x).max(0.0) * 0.5;
                painter.galley_with_override_text_color(egui::pos2(icon_left, top), icon, color);
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

/// The width a [`Switch`] takes in `look` at the look's regular size, for a row that leaves
/// room for one at its end. macOS's switch sits flush; the others keep a point either side.
/// macOS's small and large controls ([`crate::Metrics::small`]) scale it in proportion.
#[must_use]
pub fn switch_width(look: Look) -> f32 {
    switch_size(look).x
        + if matches!(look, Look::Mac | Look::Gtk) {
            0.0
        } else {
            2.0
        }
}

/// The size of a switch's pill in `look`.
pub(crate) fn switch_size(look: Look) -> egui::Vec2 {
    match look {
        Look::Mac => egui::vec2(54.0, 24.0),
        // WinUI's toggle switch: 40 by 20.
        Look::Windows => egui::vec2(40.0, 20.0),
        // libadwaita: two 20-point sliders in 3 points of padding.
        Look::Gtk => egui::vec2(46.0, 26.0),
        Look::Ios => egui::vec2(63.0, 28.0),
        Look::Material => egui::vec2(52.0, 32.0),
        Look::Plain => egui::vec2(34.0, 18.0),
    }
}

impl Widget for Switch<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        super::unfaded(ui, |ui| self.add_contents(ui))
    }
}

impl Switch<'_> {
    fn add_contents(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        let c = &x.c;
        let size = if x.look == Look::Mac {
            (switch_size(x.look) * super::mac_scale(ui)).round()
        } else {
            switch_size(x.look)
        };
        let height = if x.look == Look::Gtk {
            size.y
        } else {
            ui.spacing().interact_size.y.max(size.y)
        };
        // macOS's switch starts flush with its row; the others keep a point either side.
        let margin = if matches!(x.look, Look::Mac | Look::Gtk) {
            0.0
        } else {
            2.0
        };
        let (outer, mut response) =
            super::allocate_touch(ui, egui::vec2(size.x + margin, height), Sense::click());
        response = super::click_response(ui, response);
        let rect = Rect::from_center_size(outer.center(), size);
        let before = *self.on;
        if x.look == Look::Gtk && super::form::take_row_activation(ui) {
            *self.on = !*self.on;
            response.request_focus();
        }
        // Every system's switch lets its thumb be dragged; the plain look's only toggles.
        let preview = if x.look.native() {
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
        let t = preview.unwrap_or_else(|| {
            if x.look == Look::Gtk {
                ui.ctx().animate_bool_with_time_and_easing(
                    response.id,
                    on,
                    ui.style().animation_time.min(0.1),
                    egui::emath::easing::cubic_out,
                )
            } else {
                ui.ctx().animate_bool_responsive(response.id, on)
            }
        });
        let painter = ui.painter();
        let round = CornerRadius::same((size.y / 2.0) as u8);
        let hovered = enabled && response.hovered();
        match x.look {
            Look::Gtk => {
                // libadwaita's switch: the trough colour, the accent when on, a 20-point
                // slider three points in with a soft shadow, white when on or hovered and a
                // fifth of the view behind it otherwise.
                let pressed = enabled && response.is_pointer_button_down_on();
                // Twice as dark with more contrast, as libadwaita's high-contrast troughs.
                let more = if c.high_contrast { 2.0 } else { 1.0 };
                let off = c.fg.gamma_multiply(
                    more * match (pressed, hovered) {
                        (true, _) => 0.25,
                        (false, true) => 0.2,
                        (false, false) => 0.15,
                    },
                );
                let on_fill = if hovered {
                    lerp_color(c.accent, c.on_accent, 0.1)
                } else {
                    c.accent
                };
                painter.rect_filled(rect, round, lerp_color(off, on_fill, t));
                let r = 10.0;
                let centre = egui::pos2(
                    egui::lerp((rect.left() + 3.0 + r)..=(rect.right() - 3.0 - r), t),
                    rect.center().y,
                );
                let slider = Rect::from_center_size(centre, egui::vec2(2.0 * r, 2.0 * r));
                if enabled {
                    painter.add(
                        egui::Shadow {
                            offset: [0, 2],
                            blur: 4,
                            spread: 0,
                            color: Color32::from_rgba_unmultiplied(0, 0, 6, 51),
                        }
                        .as_shape(slider, CornerRadius::same(10)),
                    );
                }
                let view = if c.dark { c.surface } else { Color32::WHITE };
                let rest = lerp_color(view, Color32::WHITE, 0.8);
                let fill = if hovered || pressed {
                    Color32::WHITE
                } else {
                    lerp_color(rest, Color32::WHITE, t)
                };
                painter.circle_filled(centre, r, fill);
            }
            Look::Ios => {
                // iOS 26's switch: a grey track, systemGreen when on, and a white pill knob
                // inset by two points.
                let green = crate::colors::rgb(if c.dark { 0x30d158 } else { 0x34c759 });
                let off = crate::ink(c, 0.275, 0.23);
                let mut fill = lerp_color(off, green, t);
                if !enabled {
                    fill = fill.gamma_multiply(0.5);
                }
                painter.rect_filled(rect, round, fill);
                pill_knob(painter, rect, t, c.dark, Color32::WHITE);
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
                // Pressed, the thumb grows to 28 points; under the pointer a 40-point state
                // layer rings it and the thumb darkens.
                let pressed = enabled && response.is_pointer_button_down_on();
                let r = if pressed {
                    14.0
                } else {
                    egui::lerp(8.0..=12.0, t)
                };
                let x = egui::lerp((rect.left() + 16.0)..=(rect.right() - 16.0), t);
                let centre = egui::pos2(x, rect.center().y);
                if hovered || pressed {
                    let layer = lerp_color(c.fg, c.accent, t);
                    painter.circle_filled(
                        centre,
                        20.0,
                        layer.gamma_multiply(if pressed { 0.1 } else { 0.08 }),
                    );
                }
                let (off_thumb, on_thumb) = if hovered || pressed {
                    (c.dim, lerp_color(c.on_accent, c.accent, 0.12))
                } else {
                    (c.control_strong, c.on_accent)
                };
                painter.circle_filled(centre, r, lerp_color(off_thumb, on_thumb, t));
            }
            Look::Mac => {
                // A grey track, the accent when on, and a wide pill-shaped knob inset by
                // two points. A disabled switch fades its track into the window.
                let (off, on_fill) = if enabled && response.is_pointer_button_down_on() {
                    (c.button_pressed, crate::mac_pressed_accent(c))
                } else if enabled {
                    (c.track, c.accent)
                } else {
                    (
                        lerp_color(c.bg, c.track, 0.5),
                        crate::mac_disabled_accent(c),
                    )
                };
                painter.rect_filled(rect, round, lerp_color(off, on_fill, t));
                let knob = if c.dark {
                    lerp_color(
                        crate::colors::rgb(0xe5e5e5),
                        lerp_color(Color32::WHITE, c.accent, 0.13),
                        t,
                    )
                } else {
                    Color32::WHITE
                };
                pill_knob(painter, rect, t, c.dark, knob);
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
                    // ControlAltFillColorSecondary: a faint shade inside the stroke.
                    let off = Color32::from_black_alpha(if c.dark { 25 } else { 6 });
                    painter.rect_filled(rect, round, off.gamma_multiply(1.0 - t));
                    painter.rect_stroke(
                        rect,
                        round,
                        Stroke::new(1.0, strong.gamma_multiply(1.0 - t)),
                        egui::StrokeKind::Inside,
                    );
                }
                let r = if hovered { 7.0 } else { 6.0 };
                let x = egui::lerp((rect.left() + 10.0)..=(rect.right() - 10.0), t);
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

/// The pill-shaped knob of macOS's and iOS 26's switches, two points inside the track, in
/// `fill`, with a soft shadow.
fn pill_knob(painter: &egui::Painter, rect: Rect, t: f32, dark: bool, fill: Color32) {
    let size = egui::vec2(rect.width() * 16.0 / 27.0, rect.height() - 4.0);
    let x = egui::lerp(
        (rect.left() + 2.0 + size.x / 2.0)..=(rect.right() - 2.0 - size.x / 2.0),
        t,
    );
    let pill = Rect::from_center_size(egui::pos2(x, rect.center().y), size);
    let round = CornerRadius::same((size.y / 2.0) as u8);
    painter.add(mac_shadow(dark).as_shape(pill, round));
    painter.rect_filled(pill, round, fill);
}

/// The soft shadow under macOS's switch and slider knobs.
pub(crate) fn mac_shadow(dark: bool) -> egui::Shadow {
    egui::Shadow {
        offset: [0, 2],
        blur: 4,
        spread: 0,
        color: Color32::from_black_alpha(if dark { 90 } else { 40 }),
    }
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
        super::unfaded(ui, |ui| self.add_contents(ui))
    }
}

impl Checkbox<'_> {
    fn add_contents(self, ui: &mut Ui) -> Response {
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
        let (mut response, mark, mut info) = mark_row(
            ui,
            &x,
            self.text,
            egui::Role::CheckBox,
            *self.checked,
            Sense::click(),
        );
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
            Look::Gtk => CornerRadius::same(6),
            _ => CornerRadius::same(4),
        };
        let enabled = ui.is_enabled();
        let c = &x.c;
        if (*self.checked || self.indeterminate) && !enabled && x.look == Look::Mac {
            // macOS greys out a disabled mark rather than fading its accent.
            painter.rect_filled(mark, round, mac_disabled_mark(c));
            if self.indeterminate {
                let m = mark.center();
                painter.line_segment(
                    [m - egui::vec2(3.5, 0.0), m + egui::vec2(3.5, 0.0)],
                    Stroke::new(1.8, c.disabled),
                );
            } else {
                check_mark(painter, mark.center(), c.disabled);
            }
        } else if *self.checked || self.indeterminate {
            painter.rect_filled(
                mark,
                round,
                if enabled || x.look == Look::Gtk {
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
    /// Whether Tab stops on it; a [`RadioGroup`] stops only on its chosen option.
    focusable: bool,
    /// Drawn by this crate in every look, as a group's options are.
    grouped: bool,
}

impl Radio {
    /// A radio button, chosen when `selected`, with `text` after it; see also
    /// [`radio_value`].
    pub fn new(selected: bool, text: impl Into<WidgetText>) -> Self {
        Radio {
            selected,
            text: text.into(),
            focusable: true,
            grouped: false,
        }
    }
}

impl Widget for Radio {
    fn ui(self, ui: &mut Ui) -> Response {
        super::unfaded(ui, |ui| self.add_contents(ui))
    }
}

impl Radio {
    fn add_contents(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        // egui's own radio is always a Tab stop, so a group's options draw their own.
        if !x.look.native() && !self.grouped {
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
        let sense = if self.focusable {
            Sense::click()
        } else {
            Sense::CLICK
        };
        let (response, mark, info) = mark_row(
            ui,
            &x,
            self.text,
            egui::Role::RadioButton,
            self.selected,
            sense,
        );
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
        } else if self.selected && !enabled && x.look == Look::Mac {
            painter.circle_filled(mark.center(), mark.width() / 2.0, mac_disabled_mark(c));
            painter.circle_filled(mark.center(), 3.0, c.disabled);
        } else if self.selected && x.look == Look::Ios {
            // iOS marks a chosen option with a tick in an accent circle.
            painter.circle_filled(
                mark.center(),
                mark.width() / 2.0,
                if enabled || x.look == Look::Gtk {
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
                if enabled || x.look == Look::Gtk {
                    c.accent
                } else {
                    c.accent.gamma_multiply(0.5)
                },
            );
            // WinUI's dot is 12 points wide, 14 under the pointer and 10 pressed; the others
            // keep a small one.
            let dot = if x.look == Look::Windows {
                if response.is_pointer_button_down_on() {
                    5.0
                } else if response.hovered() {
                    7.0
                } else {
                    6.0
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

/// Radio buttons choosing one value for `current`, as one control for the keyboard: Tab
/// stops only on the chosen option (the first enabled one while none is chosen), the
/// arrows choose and focus the previous or next enabled option, wrapping round at the
/// ends except on Windows, and Space chooses the focused one. Lay the options out with
/// the `Ui` the closure gets, in a row or a column.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui, mode: &mut u8) {
/// use egui_native_look as look;
/// look::RadioGroup::new("mode", mode).accessible_label("Mode").show(ui, |group| {
///     group.option(0, "Automatic");
///     group.option(1, "Manual");
/// });
/// # }
/// ```
///
/// The builder must be shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::RadioGroup;
/// RadioGroup::new("mode", &mut 0);
/// ```
#[must_use = "call its show method"]
pub struct RadioGroup<'a, T> {
    id: egui::IdSalt,
    current: &'a mut T,
    label: String,
}

impl<'a, T: PartialEq + Clone> RadioGroup<'a, T> {
    /// A group setting `current`; `id` tells it from other groups in the same `Ui`.
    pub fn new(id: impl egui::AsIdSalt, current: &'a mut T) -> Self {
        RadioGroup {
            id: egui::IdSalt::new(id),
            current,
            label: String::new(),
        }
    }
    /// The group's name for a screen reader.
    pub fn accessible_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Lays out the options `add` adds. The response is the group's, changed when the
    /// choice changed.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add: impl FnOnce(&mut RadioGroupUi<'_, T>) -> R,
    ) -> egui::InnerResponse<R> {
        let x = Ctx::of(ui);
        let id = ui.make_persistent_id(self.id);
        // Whether an option matched the model last frame; while none does, the first
        // enabled option is the group's Tab stop.
        let matched = ui.data(|data| data.get_temp::<bool>(id)).unwrap_or(true);
        let current = self.current;
        let scope = ui.scope(|ui| {
            let mut group = RadioGroupUi {
                ui,
                current: &mut *current,
                matched,
                options: Vec::new(),
            };
            let inner = add(&mut group);
            (inner, group.options)
        });
        let (inner, options) = scope.inner;
        let mut response = scope.response;
        ui.data_mut(|data| {
            data.insert_temp(id, options.iter().any(|option| option.value == *current))
        });
        let focused = ui.memory(|memory| memory.focused());
        if let Some(at) = options.iter().position(|option| Some(option.id) == focused) {
            let wrap = x.look != Look::Windows;
            let (step, space) = ui.input_mut(|input| {
                let mut take = |key| input.consume_key(egui::Modifiers::NONE, key);
                let forward = take(egui::Key::ArrowDown) | take(egui::Key::ArrowRight);
                let back = take(egui::Key::ArrowUp) | take(egui::Key::ArrowLeft);
                (i32::from(forward) - i32::from(back), take(egui::Key::Space))
            });
            let target = if step != 0 {
                next_enabled(&options, at, step, wrap)
            } else if space {
                Some(at)
            } else {
                None
            };
            if let Some(target) = target {
                let option = &options[target];
                if *current != option.value {
                    *current = option.value.clone();
                    response.mark_changed();
                }
                ui.memory_mut(|memory| memory.request_focus(option.id));
            }
        }
        if options.iter().any(|option| option.changed) {
            response.mark_changed();
        }
        let name = super::form::accessible_name(ui, &self.label);
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(egui::Role::RadioGroup);
            if !name.is_empty() {
                node.set_label(name.as_str());
            }
        });
        egui::InnerResponse::new(inner, response)
    }
}

/// The option after (`step` 1) or before (-1) `at` that is enabled, wrapping round when
/// `wrap`; none at an end without wrapping.
fn next_enabled<T>(options: &[GroupOption<T>], at: usize, step: i32, wrap: bool) -> Option<usize> {
    let n = options.len() as i32;
    let mut i = at as i32;
    for _ in 1..n {
        i += step;
        if !(0..n).contains(&i) {
            if !wrap {
                return None;
            }
            i = i.rem_euclid(n);
        }
        if options[i as usize].enabled {
            return Some(i as usize);
        }
    }
    None
}

struct GroupOption<T> {
    id: egui::Id,
    value: T,
    enabled: bool,
    changed: bool,
}

/// The options of a [`RadioGroup`] being laid out.
pub struct RadioGroupUi<'a, T> {
    ui: &'a mut Ui,
    current: &'a mut T,
    matched: bool,
    options: Vec<GroupOption<T>>,
}

impl<T: PartialEq + Clone> RadioGroupUi<'_, T> {
    /// An option choosing `value`, reading `text`.
    pub fn option(&mut self, value: T, text: impl Into<WidgetText>) -> Response {
        self.option_enabled(true, value, text)
    }

    /// An option that can be disabled; the arrows pass over a disabled one.
    pub fn option_enabled(
        &mut self,
        enabled: bool,
        value: T,
        text: impl Into<WidgetText>,
    ) -> Response {
        let selected = *self.current == value;
        let enabled = enabled && self.ui.is_enabled();
        let first_enabled = enabled && !self.options.iter().any(|option| option.enabled);
        let focusable = selected || (!self.matched && first_enabled);
        let mut response = self.ui.add_enabled(
            enabled,
            Radio {
                selected,
                text: text.into(),
                focusable,
                grouped: true,
            },
        );
        let changed = response.clicked() && !selected;
        if changed {
            *self.current = value.clone();
            response.mark_changed();
        }
        if response.has_focus() {
            // The group moves the focus with the arrows itself.
            let filter = egui::EventFilter {
                horizontal_arrows: true,
                vertical_arrows: true,
                ..Default::default()
            };
            self.ui
                .memory_mut(|memory| memory.set_focus_lock_filter(response.id, filter));
        }
        self.options.push(GroupOption {
            id: response.id,
            value,
            enabled,
            changed,
        });
        response
    }

    /// The `Ui` the options go in, for anything else between them.
    pub fn ui(&mut self) -> &mut Ui {
        self.ui
    }
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

/// The row of a check box or radio button: the mark (14 points, 20 on Windows), then the
/// label, all of it one click target. The label wraps as the `Ui` wraps its text, the mark
/// beside its first line. Returns the response and the mark's square.
fn mark_row(
    ui: &mut Ui,
    x: &Ctx,
    text: WidgetText,
    kind: egui::Role,
    selected: bool,
    sense: Sense,
) -> (Response, Rect, egui::WidgetInfo) {
    let (side, gap) = match x.look {
        // 14, 16 and 18 points for small, regular and large controls.
        Look::Mac => ((16.0 + (super::mac_scale(ui) - 1.0) * 12.0).round(), 5.0),
        Look::Windows => (20.0, 8.0),
        Look::Ios => (22.0, 10.0),
        Look::Material => (18.0, 12.0),
        // libadwaita: a 14-point icon in 3 points of padding, 4 points from its label.
        Look::Gtk => (20.0, 4.0),
        _ => (14.0, 6.0),
    };
    // libadwaita pads the whole check button by 3 points.
    let pad = if x.look == Look::Gtk {
        if text.text().is_empty() {
            3.0
        } else {
            4.0
        }
    } else {
        0.0
    };
    let galley = text.into_galley(
        ui,
        None,
        ui.available_width() - side - gap - 2.0 * pad,
        egui::FontSelection::FontId(x.m.body.clone()),
    );
    let width = side + gap + galley.size().x + 2.0 * pad;
    let line = if x.look == Look::Gtk {
        (side + 2.0 * pad).max(super::minimum_target(ui))
    } else {
        ui.spacing().interact_size.y.max(side)
    };
    let first_row = galley
        .rows
        .first()
        .map_or(galley.size().y, |row| row.height());
    let height = line.max(galley.size().y + line - first_row);
    let (rect, response) = super::allocate_control(ui, egui::vec2(width, height), sense);
    let middle = rect.top() + line / 2.0;
    let mark = Rect::from_center_size(
        egui::pos2(rect.left() + pad + side / 2.0, middle),
        egui::vec2(side, side),
    );
    let enabled = ui.is_enabled();
    // Material lays a 40-point state layer round the mark under the pointer: the primary's
    // when it is on, the text colour's when off.
    if x.look == Look::Material && enabled && response.hovered() {
        let layer = if selected { x.c.accent } else { x.c.fg };
        ui.painter()
            .circle_filled(mark.center(), 20.0, layer.gamma_multiply(0.08));
    }
    let colour = if enabled { x.c.fg } else { x.c.disabled };
    let top = crate::galley_top(ui.ctx(), &galley, middle);
    let info = egui::WidgetInfo::selected(kind, enabled, selected, galley.text());
    ui.painter()
        .galley(egui::pos2(mark.right() + gap, top), galley, colour);
    (response, mark, info)
}

/// A disabled chosen mark on macOS: a grey a step darker than an unchecked one.
fn mac_disabled_mark(c: &crate::Colors) -> Color32 {
    lerp_color(c.bg, c.track, 0.7)
}

/// An unchecked mark: on macOS a grey fill without an edge, fading into the window when
/// disabled; on Windows a faint fill inside the strong stroke; in libadwaita a two-point
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
        Look::Mac => {
            let fill = if enabled {
                c.track
            } else {
                lerp_color(c.bg, c.track, 0.5)
            };
            painter.rect_filled(mark, round, fill);
        }
        Look::Gtk => {
            // libadwaita's trough colour, a step stronger under the pointer.
            let tint = if hovered {
                c.fg.gamma_multiply(0.2)
            } else {
                c.track
            };
            painter.rect_stroke(
                mark,
                round,
                Stroke::new(2.0, tint),
                egui::StrokeKind::Inside,
            );
        }
        Look::Ios | Look::Material => {
            // An outline: two points thick in Material, in the variant text colour; one and a
            // half on iOS in the strong stroke. Material lays a state layer over it under the
            // pointer.
            let width = if x.look == Look::Material { 2.0 } else { 1.5 };
            painter.rect_stroke(
                mark,
                round,
                Stroke::new(
                    width,
                    match (enabled, x.look) {
                        (false, _) => c.disabled,
                        (true, Look::Material) => c.dim,
                        (true, _) => c.control_strong,
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
        // The plain look's own mark, for the options of a radio group.
        Look::Plain => {
            painter.rect_filled(
                mark,
                round,
                if enabled { c.button } else { c.button_disabled },
            );
            painter.rect_stroke(
                mark,
                round,
                Stroke::new(1.0, if hovered { c.grid } else { c.border }),
                egui::StrokeKind::Inside,
            );
        }
    }
}
