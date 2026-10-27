//! Progress: bars and spinners.

use egui::{Color32, CornerRadius, Rect, Response, Sense, Stroke, Ui, Widget};

use super::Ctx;
use crate::Look;

/// A progress bar: a rounded rail filled with the accent up to `progress` (0 to 1), thin
/// on Windows and in libadwaita, a little thicker on macOS and in the plain look.
///
/// The builder must be added to a UI or shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::ProgressBar;
/// ProgressBar::new(0.5);
/// ```
#[must_use = "add the widget to a UI or call its show method"]
pub struct ProgressBar {
    progress: f32,
    width: Option<f32>,
    label: String,
}

impl ProgressBar {
    /// A bar filled to `progress`, from 0 to 1; beyond those it is held to them, and a
    /// value that is not a number shows an empty bar.
    pub fn new(progress: f32) -> Self {
        // Not a number counts as nothing done yet.
        ProgressBar {
            progress: if progress.is_nan() {
                0.0
            } else {
                progress.clamp(0.0, 1.0)
            },
            width: None,
            label: "Progress".into(),
        }
    }
    /// Names the operation whose progress is displayed; defaults to "Progress".
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
}

impl Widget for ProgressBar {
    fn ui(self, ui: &mut Ui) -> Response {
        super::unfaded(ui, |ui| self.add_contents(ui))
    }
}

impl ProgressBar {
    fn add_contents(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        let width = self.width.unwrap_or_else(|| ui.available_width());
        let thickness = if x.look == Look::Gtk {
            // libadwaita's progress bar is as thick as its scale's trough is wide.
            8.0
        } else if matches!(x.look, Look::Windows | Look::Material | Look::Ios) {
            4.0
        } else if x.look == Look::Mac {
            // SwiftUI's small progress bar is six points, the others eight.
            if super::mac_scale(ui) < 0.95 {
                6.0
            } else {
                8.0
            }
        } else {
            6.0
        };
        let (outer, response) = ui.allocate_exact_size(
            egui::vec2(
                width,
                if x.look == Look::Gtk {
                    thickness
                } else {
                    ui.spacing().interact_size.y.min(16.0)
                },
            ),
            Sense::hover(),
        );
        let rail = Rect::from_center_size(outer.center(), egui::vec2(width, thickness));
        let round = CornerRadius::same((thickness / 2.0) as u8);
        // macOS's progress rail is a step lighter than its other tracks; Material's is the
        // secondary container, the tonal buttons' fill.
        let track = match x.look {
            Look::Mac => {
                let ink = if x.c.dark {
                    Color32::WHITE
                } else {
                    Color32::BLACK
                };
                crate::lerp_color(x.c.bg, ink, if x.c.dark { 0.1 } else { 0.05 })
            }
            Look::Material => x.c.button,
            _ => x.c.track,
        };
        ui.painter().rect_filled(rail, round, track);
        let filled = Rect::from_min_max(
            rail.min,
            egui::pos2(
                egui::lerp(rail.left()..=rail.right(), self.progress),
                rail.max.y,
            ),
        );
        // An inactive macOS window shows progress in a mid grey.
        let fill = if x.look == Look::Mac && !x.c.active {
            crate::colors::rgb(0xaaaaaa)
        } else {
            x.c.accent
        };
        ui.painter().rect_filled(filled, round, fill);
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::ProgressIndicator, ui.is_enabled(), &self.label)
        });
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_numeric_value(f64::from(self.progress) * 100.0);
            node.set_min_numeric_value(0.0);
            node.set_max_numeric_value(100.0);
            node.set_value(format!("{:.0} %", self.progress * 100.0));
        });
        response
    }
}

/// A spinner: an accent arc turning, as WinUI's and libadwaita's are; macOS's spokes
/// fading round the circle.
///
/// The builder must be added to a UI or shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::Spinner;
/// Spinner::new();
/// ```
#[must_use = "add the widget to a UI or call its show method"]
pub struct Spinner {
    size: f32,
    label: String,
}

impl Spinner {
    /// A spinner 16 points across.
    pub fn new() -> Self {
        Spinner {
            size: 16.0,
            label: "Loading".into(),
        }
    }
    /// Names the ongoing operation; defaults to "Loading".
    pub fn accessible_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
    /// How far across it is. Invalid sizes restore 16 points; values above 1,000,000
    /// points are capped.
    pub fn size(mut self, size: f32) -> Self {
        self.size = super::dimension(size).unwrap_or(16.0);
        self
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Spinner::new()
    }
}

impl Widget for Spinner {
    fn ui(self, ui: &mut Ui) -> Response {
        super::unfaded(ui, |ui| self.add_contents(ui))
    }
}

impl Spinner {
    fn add_contents(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        let (rect, response) = ui.allocate_exact_size(egui::Vec2::splat(self.size), Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.ctx().request_repaint();
            let time = ui.input(|i| i.time) as f32;
            let m = rect.center();
            let r = (self.size / 2.0 - 1.5).max(0.0);
            if matches!(x.look, Look::Mac | Look::Ios) {
                // Eight spokes with round ends, the leading one darkest, stepping eight
                // times a second.
                let width = (r * 0.28).max(1.5);
                let lead = ((time * 8.0).floor() as i32).rem_euclid(8) as f32;
                for i in 0..8 {
                    let a = i as f32 / 8.0 * std::f32::consts::TAU;
                    let fade = 1.0 - ((lead - i as f32).rem_euclid(8.0) / 8.0);
                    // iOS's spokes are fainter, only the leading few darkened.
                    let share = match (x.look, x.c.dark) {
                        (Look::Ios, false) => 0.14 + 0.26 * fade * fade,
                        (Look::Ios, true) => 0.18 + 0.35 * fade * fade,
                        _ => 0.12 + 0.6 * fade,
                    };
                    let colour = crate::lerp_color(x.c.bg, x.c.fg, share);
                    let dir = egui::vec2(a.cos(), a.sin());
                    let (inner, outer) = (
                        m + dir * (r * 0.42 + width / 2.0),
                        m + dir * (r - width / 2.0),
                    );
                    ui.painter()
                        .line_segment([inner, outer], Stroke::new(width, colour));
                    ui.painter().circle_filled(inner, width / 2.0, colour);
                    ui.painter().circle_filled(outer, width / 2.0, colour);
                }
            } else {
                let start = time * 4.0;
                let sweep = 1.5 + (time * 2.0).sin() * 0.8;
                let points: Vec<egui::Pos2> = (0..=24)
                    .map(|i| {
                        let a = start + sweep * i as f32 / 24.0;
                        m + egui::vec2(a.cos(), a.sin()) * r
                    })
                    .collect();
                ui.painter()
                    .add(egui::Shape::line(points, Stroke::new(2.0, x.c.accent)));
            }
        }
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::ProgressIndicator, ui.is_enabled(), &self.label)
        });
        ui.ctx()
            .accesskit_node_builder(response.id, |node| node.set_busy());
        response
    }
}

#[cfg(test)]
mod tests {
    use super::ProgressBar;

    #[test]
    fn progress_handles_non_finite_values_and_clamps_to_its_range() {
        for (input, expected) in [
            (f32::NAN, 0.0),
            (f32::NEG_INFINITY, 0.0),
            (-0.1, 0.0),
            (0.5, 0.5),
            (1.1, 1.0),
            (f32::INFINITY, 1.0),
        ] {
            assert_eq!(
                ProgressBar::new(input).progress,
                expected,
                "progress {input}"
            );
        }
    }
}
