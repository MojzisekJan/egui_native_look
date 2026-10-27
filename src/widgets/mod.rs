//! The controls, each an [`egui::Widget`] built like egui's own: `ui.add(Switch::new(&mut on))`.
//! They take the height of `ui.spacing().interact_size.y`, which [`crate::install`] sets to
//! the theme's control height. Touch targets reserve at least 48 points per axis
//! while the painted controls retain their visual dimensions.

mod basic;
mod choice;
mod containers;
mod entry;
mod extra;
mod form;
mod list;
mod scroll;
mod status;
mod window;

pub use basic::check_mark;
pub use basic::{
    radio_value, switch_width, Button, Checkbox, Radio, RadioGroup, RadioGroupUi, Switch,
};
pub use choice::{
    combo_icon, menu_value, Dropdown, PopupButton, Segmented, Slider, Tab, TabBar, TabBarUi, Tabs,
};
pub use containers::{picked_text, section, submenu, ListRow, MenuItem, Section};
pub use entry::{NumberField, TextField};
pub use extra::{show_toasts, toast, ColorWell, IconButton};
pub use form::{Form, FormUi};
pub use list::{ListBox, ListBoxOutput, ListBoxUi};
pub use scroll::ScrollArea;
pub use status::{ProgressBar, Spinner};
pub use window::{window_frame, Dialog, TitleBar, ToolWindow, WindowAction};

use crate::{paint, Colors, Look, Metrics};

/// How large the macOS look's controls are drawn against its regular 24-point size, so a
/// switch, a mark or a knob follows [`crate::Metrics::small`] and [`crate::Metrics::large`].
pub(crate) fn mac_scale(ui: &egui::Ui) -> f32 {
    (ui.spacing().interact_size.y / 24.0).clamp(0.5, 3.0)
}

/// Where a menu item's label starts, after the column for its tick or icon.
pub(crate) fn menu_label_inset(look: Look) -> f32 {
    if look == Look::Mac {
        24.0
    } else {
        28.0
    }
}

/// What a control needs to know to draw itself: the look, its colours and its sizes.
pub(crate) struct Ctx {
    pub look: Look,
    pub c: Colors,
    pub m: Metrics,
}

impl Ctx {
    pub fn of(ui: &egui::Ui) -> Ctx {
        let resolved = crate::resolved(ui.ctx());
        let mut c = resolved.colors(ui.ctx());
        // GTK fades the entire disabled widget once, including its foreground.
        if resolved.theme.look == Look::Gtk && !ui.is_enabled() {
            c.disabled = c.fg;
        }
        Ctx {
            look: resolved.theme.look,
            c,
            m: resolved.theme.metrics.clone(),
        }
    }

    /// The painting view of this: the look, the colours, the rounding.
    pub fn paint(&self) -> paint::Paint {
        paint::Paint {
            look: self.look,
            c: self.c,
            radius: self.m.radius,
        }
    }

    pub fn radius(&self) -> egui::CornerRadius {
        egui::CornerRadius::same(self.m.radius)
    }
}

/// `text` laid out on one line no wider than `width`, ending in an ellipsis when it had to
/// be cut; and whether it was.
pub(crate) fn truncated(
    ui: &egui::Ui,
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
    width: f32,
) -> (std::sync::Arc<egui::Galley>, bool) {
    let mut job = egui::text::LayoutJob::simple(text.to_string(), font, color, width.max(0.0));
    job.wrap.max_rows = 1;
    job.wrap.overflow_character = Some('…');
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    let cut = galley.elided;
    (galley, cut)
}

/// Reserve separate touch slots rather than overlapping neighboring controls.
pub(crate) fn minimum_target(ui: &egui::Ui) -> f32 {
    let key = egui::Id::unique("egui_native_look::TouchTargets");
    if crate::touch_screen() || ui.data(|data| data.get_temp::<bool>(key)).unwrap_or(false) {
        return 48.0;
    }
    // A newly discovered touch device must not move targets during its first gesture.
    let ready = ui.input(|input| {
        input.has_touch_screen()
            && !input.any_touches()
            && !input.pointer.any_down()
            && !input.pointer.any_pressed()
            && !input.pointer.any_released()
    });
    if ready {
        ui.data_mut(|data| data.insert_temp(key, true));
    }
    if ready {
        48.0
    } else {
        0.0
    }
}

pub(crate) fn target_size(ui: &egui::Ui, size: egui::Vec2) -> egui::Vec2 {
    size.max(egui::Vec2::splat(minimum_target(ui)))
}

/// The visual rectangle keeps its size inside the larger interaction slot.
pub(crate) fn allocate_touch(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    sense: egui::Sense,
) -> (egui::Rect, egui::Response) {
    let (slot, response) = ui.allocate_exact_size(target_size(ui, size), sense);
    (egui::Rect::from_center_size(slot.center(), size), response)
}

pub(crate) fn allocate_control(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    sense: egui::Sense,
) -> (egui::Rect, egui::Response) {
    let (rect, response) = allocate_touch(ui, size, sense);
    (rect, click_response_with_reentry(ui, response, true))
}

#[derive(Clone, Copy)]
struct ClickPress {
    time: Option<f64>,
    frame: u64,
    cancelled: bool,
    touch: bool,
    left: bool,
}

/// Pointer cancellation is latched until release; keyboard and accessibility clicks stay independent.
pub(crate) fn click_response(ui: &egui::Ui, response: egui::Response) -> egui::Response {
    click_response_with_reentry(ui, response, false)
}

fn click_response_with_reentry(
    ui: &egui::Ui,
    mut response: egui::Response,
    desktop_button: bool,
) -> egui::Response {
    use egui::response::Flags;
    let key = response.id.with("click-press");
    let frame = ui.ctx().cumulative_frame_nr();
    let (pressed, down, released, time, touch, cancelled, dragging) = ui.input(|input| {
        (
            input.pointer.primary_pressed(),
            input.pointer.primary_down(),
            input.pointer.primary_released(),
            input.pointer.press_start_time(),
            input.any_touches(),
            pointer_cancelled(input),
            input.pointer.is_decidedly_dragging()
                || input
                    .multi_touch()
                    .is_some_and(|touch| touch.num_touches > 1),
        )
    });
    let mut state = ui.data(|data| data.get_temp::<ClickPress>(key));
    if pressed
        && (response.is_pointer_button_down_on()
            || response.clicked_by(egui::PointerButton::Primary))
    {
        state = Some(ClickPress {
            time,
            frame,
            cancelled: false,
            touch,
            left: false,
        });
    }
    let position = pointer_position(ui, &response);
    if let Some(press) = state.as_mut() {
        press.left |= position.is_some_and(|at| !response.interact_rect.contains(at));
        press.cancelled |= cancelled
            || !response.enabled()
            || frame.saturating_sub(press.frame) > 1
            || (down && press.time != time)
            || ((!desktop_button || press.touch || crate::look(ui.ctx()) != Look::Gtk)
                && position.is_some_and(|position| !response.interact_rect.contains(position)))
            || (press.touch && dragging);
        press.frame = frame;
        if desktop_button
            && press.left
            && crate::look(ui.ctx()) == Look::Gtk
            && !press.touch
            && !press.cancelled
            && released
            && position.is_some_and(|at| response.interact_rect.contains(at))
            && response.enabled()
            && response.contains_pointer()
        {
            // egui stops classifying a long pointer excursion as a click. A desktop
            // button still activates when its original press is released back inside.
            response.flags.insert(Flags::FAKE_PRIMARY_CLICKED);
        }
        if press.cancelled {
            response
                .flags
                .remove(Flags::CLICKED | Flags::IS_POINTER_BUTTON_DOWN_ON | Flags::LONG_TOUCHED);
        }
    }
    if cancelled || !response.enabled() {
        response
            .flags
            .remove(Flags::CLICKED | Flags::IS_POINTER_BUTTON_DOWN_ON | Flags::LONG_TOUCHED);
    }
    ui.data_mut(|data| {
        if !down || released {
            data.remove::<ClickPress>(key);
        } else if let Some(state) = state {
            data.insert_temp(key, state);
        }
    });
    if !response.enabled() {
        response.sense = egui::Sense::hover();
        disabled_actions(ui, &response);
    }
    if response.gained_focus() && !down && !pressed {
        ui.scroll_to_rect_animation(response.rect, None, egui::style::ScrollAnimation::none());
    }
    response
}

/// Disabled controls must not advertise actions that they ignore.
pub(crate) fn disabled_actions(ui: &egui::Ui, response: &egui::Response) {
    if !response.enabled() {
        ui.ctx().accesskit_node_builder(response.id, |node| {
            for action in [
                egui::accesskit::Action::Click,
                egui::accesskit::Action::Focus,
                egui::accesskit::Action::Increment,
                egui::accesskit::Action::Decrement,
                egui::accesskit::Action::SetValue,
                egui::accesskit::Action::ReplaceSelectedText,
            ] {
                node.remove_action(action);
            }
        });
    }
}

/// A finite positive layout extent, capped to keep arithmetic in a useful UI range.
pub(crate) fn dimension(value: f32) -> Option<f32> {
    (value.is_finite() && value > 0.0).then_some(value.clamp(1.0, 1_000_000.0))
}

pub(crate) fn number_changed(before: f64, after: f64) -> bool {
    before != after && !(before.is_nan() && after.is_nan())
}

/// Disabled numeric editors must not apply queued accessibility edits.
pub(crate) fn ignore_disabled_numeric_actions(ui: &mut egui::Ui, id: egui::Id) {
    if !ui.is_enabled() {
        ui.input_mut(|input| {
            input.consume_accesskit_action_requests(id, |request| {
                matches!(
                    request.action,
                    egui::accesskit::Action::Increment
                        | egui::accesskit::Action::Decrement
                        | egui::accesskit::Action::SetValue
                )
            })
        });
    }
}

/// Cancellation must stop updates even if a backend later sends a button release.
pub(crate) fn pointer_cancelled(input: &egui::InputState) -> bool {
    !input.focused
        || input.events.iter().any(|event| {
            matches!(
                event,
                egui::Event::WindowFocused(false)
                    | egui::Event::Touch {
                        phase: egui::TouchPhase::Cancel,
                        ..
                    }
            )
        })
}

/// Pointer coordinates in the widget's layer, including frames when a parent owns the drag.
pub(crate) fn pointer_position(ui: &egui::Ui, response: &egui::Response) -> Option<egui::Pos2> {
    ui.input(|input| input.pointer.interact_pos())
        .map(|position| {
            ui.ctx()
                .layer_transform_from_global(response.layer_id)
                .map_or(position, |transform| transform * position)
        })
}

/// How long a step button is held before it repeats, and how often it then repeats.
const REPEAT_DELAY: f64 = 0.5;
const REPEAT_INTERVAL: f64 = 0.1;

/// The steps a held step button adds this frame beyond its click: none for the first
/// half second, then ten a second while the pointer stays pressed on it without moving
/// away, as spin buttons and steppers repeat. egui hands a press longer than a click to
/// dragging, so the hold is followed here from the press that began on the button.
pub(crate) fn held_repeats(ui: &egui::Ui, response: &egui::Response) -> u32 {
    #[derive(Clone, Copy)]
    struct Held {
        start: f64,
        done: u32,
    }
    let key = response.id.with("held-repeats");
    let (start, now, down, at, travel) = ui.input(|input| {
        let pointer = &input.pointer;
        let travel = pointer
            .press_origin()
            .zip(pointer.interact_pos())
            .map_or(0.0, |(origin, at)| origin.distance(at));
        (
            pointer.press_start_time(),
            input.time,
            pointer.primary_down(),
            pointer.interact_pos(),
            travel,
        )
    });
    let mut held = ui.data(|data| data.get_temp::<Held>(key));
    if held.is_none() && response.is_pointer_button_down_on() {
        held = start.map(|start| Held { start, done: 0 });
    }
    let still = down
        && response.enabled()
        && travel <= 6.0
        && at.is_some_and(|at| response.interact_rect.contains(at));
    let Some(mut state) = held.filter(|held| still && start == Some(held.start)) else {
        ui.data_mut(|data| data.remove::<Held>(key));
        return 0;
    };
    let elapsed = now - state.start;
    let total = if elapsed < REPEAT_DELAY {
        0
    } else {
        ((elapsed - REPEAT_DELAY) / REPEAT_INTERVAL) as u32 + 1
    };
    let new = total.saturating_sub(state.done);
    state.done = total;
    ui.data_mut(|data| data.insert_temp(key, state));
    ui.ctx()
        .request_repaint_after(std::time::Duration::from_secs_f64(REPEAT_INTERVAL));
    new
}

/// Runs `add` with the opacity `ui` had before egui faded it for being disabled, in the
/// macOS, iOS and Material looks: their controls carry their own disabled colours, as Apple
/// and Material 3 draw them, and would otherwise be faded twice. The other looks keep egui's
/// fading.
pub(crate) fn unfaded<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let before = ui.opacity();
    let restore = !ui.is_enabled()
        && matches!(
            crate::look(ui.ctx()),
            Look::Mac | Look::Ios | Look::Material
        );
    if restore {
        ui.set_opacity((before / ui.visuals().disabled_alpha().max(0.01)).min(1.0));
    }
    let result = add(ui);
    if restore {
        ui.set_opacity(before);
    }
    result
}

/// A focused GTK control owns its wheel gesture, including the smoothed tail.
/// Unfocused controls leave scrolling to their parent.
pub(crate) fn gtk_wheel(ui: &egui::Ui, response: &egui::Response) -> f64 {
    if !response.enabled() || !response.has_focus() {
        return 0.0;
    }
    let key = response.id.with("wheel-capture");
    let captured = ui.data(|data| data.get_temp::<bool>(key)).unwrap_or(false);
    let mut count = 0.0;
    let scrolling = ui.input_mut(|input| {
        if !input.focused || !input.modifiers.is_none() {
            return false;
        }
        if response.hovered() {
            input.events.retain(|event| {
                if let egui::Event::MouseWheel {
                    unit,
                    delta,
                    modifiers,
                    ..
                } = event
                {
                    if modifiers.is_none() && delta.y != 0.0 {
                        count += f64::from(delta.y)
                            / match unit {
                                egui::MouseWheelUnit::Point => 40.0,
                                egui::MouseWheelUnit::Line | egui::MouseWheelUnit::Page => 1.0,
                            };
                        return false;
                    }
                }
                true
            });
        }
        if count != 0.0 || captured {
            input.smooth_scroll_delta.y = 0.0;
        }
        input.is_scrolling()
    });
    ui.data_mut(|data| data.insert_temp(key, scrolling && (captured || count != 0.0)));
    count
}

#[cfg(test)]
mod tests {
    use super::{dimension, number_changed};

    #[test]
    fn dimensions_reject_invalid_values_and_bound_positive_values() {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0, -1.0] {
            assert_eq!(dimension(value), None);
        }
        assert_eq!(dimension(f32::MIN_POSITIVE), Some(1.0));
        assert_eq!(dimension(42.0), Some(42.0));
        assert_eq!(dimension(f32::MAX), Some(1_000_000.0));
    }

    #[test]
    fn numeric_change_detection_treats_unchanged_nan_as_unchanged() {
        assert!(!number_changed(f64::NAN, f64::NAN));
        assert!(!number_changed(-0.0, 0.0));
        assert!(!number_changed(f64::INFINITY, f64::INFINITY));
        assert!(number_changed(f64::NAN, 0.0));
        assert!(number_changed(0.0, f64::NAN));
        assert!(number_changed(f64::INFINITY, f64::NEG_INFINITY));
    }
}
