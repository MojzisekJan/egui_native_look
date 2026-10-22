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
mod status;
mod window;

pub use basic::check_mark;
pub use basic::{radio_value, switch_width, Button, Checkbox, Radio, Switch};
pub use choice::{
    combo_icon, menu_value, Dropdown, PopupButton, Segmented, Slider, Tab, TabBar, TabBarUi, Tabs,
};
pub use containers::{picked_text, section, submenu, ListRow, MenuItem, Section};
pub use entry::{NumberField, TextField};
pub use extra::{show_toasts, toast, ColorWell, IconButton};
pub use form::{Form, FormUi};
pub use status::{ProgressBar, Spinner};
pub use window::{window_frame, Dialog, TitleBar, ToolWindow, WindowAction};

use crate::{paint, Colors, Look, Metrics};

/// What a control needs to know to draw itself: the look, its colours and its sizes.
pub(crate) struct Ctx {
    pub look: Look,
    pub c: Colors,
    pub m: Metrics,
}

impl Ctx {
    pub fn of(ui: &egui::Ui) -> Ctx {
        let resolved = crate::resolved(ui.ctx());
        Ctx {
            look: resolved.theme.look,
            c: resolved.colors(ui.ctx()),
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
    (rect, click_response(ui, response))
}

#[derive(Clone, Copy)]
struct ClickPress {
    time: Option<f64>,
    frame: u64,
    cancelled: bool,
    touch: bool,
}

/// Pointer cancellation is latched until release; keyboard and accessibility clicks stay independent.
pub(crate) fn click_response(ui: &egui::Ui, mut response: egui::Response) -> egui::Response {
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
        });
    }
    let position = pointer_position(ui, &response);
    if let Some(press) = state.as_mut() {
        press.cancelled |= cancelled
            || !response.enabled()
            || frame.saturating_sub(press.frame) > 1
            || (down && press.time != time)
            || position.is_some_and(|position| !response.interact_rect.contains(position))
            || (press.touch && dragging);
        press.frame = frame;
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
