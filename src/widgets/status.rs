//! Progress: bars and spinners.

use egui::{CornerRadius, Rect, Response, Sense, Stroke, Ui, Widget};

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
        let x = Ctx::of(ui);
        let width = self.width.unwrap_or_else(|| ui.available_width());
        let thickness = if matches!(
            x.look,
            Look::Windows | Look::Gtk | Look::Material | Look::Ios
        ) {
            4.0
        } else {
            6.0
        };
        let (outer, response) = ui.allocate_exact_size(
            egui::vec2(width, ui.spacing().interact_size.y.min(16.0)),
            Sense::hover(),
        );
        let rail = Rect::from_center_size(outer.center(), egui::vec2(width, thickness));
        let round = CornerRadius::same((thickness / 2.0) as u8);
        ui.painter().rect_filled(rail, round, x.c.track);
        let filled = Rect::from_min_max(
            rail.min,
            egui::pos2(
                egui::lerp(rail.left()..=rail.right(), self.progress),
                rail.max.y,
            ),
        );
        ui.painter().rect_filled(filled, round, x.c.accent);
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
        let x = Ctx::of(ui);
        let (rect, response) = ui.allocate_exact_size(egui::Vec2::splat(self.size), Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.ctx().request_repaint();
            let time = ui.input(|i| i.time) as f32;
            let m = rect.center();
            let r = (self.size / 2.0 - 1.5).max(0.0);
            if x.look == Look::Mac {
                for i in 0..12 {
                    let a = i as f32 / 12.0 * std::f32::consts::TAU;
                    let lead = ((time * 12.0).floor() as i32).rem_euclid(12) as f32;
                    let fade = 1.0 - ((lead - i as f32).rem_euclid(12.0) / 12.0);
                    let dir = egui::vec2(a.cos(), a.sin());
                    ui.painter().line_segment(
                        [m + dir * r * 0.45, m + dir * r],
                        Stroke::new(1.6, x.c.dim.gamma_multiply(0.25 + 0.75 * fade)),
                    );
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
