//! Progress: bars and spinners.

use egui::{CornerRadius, Rect, Response, Sense, Stroke, Ui, Widget};

use super::Ctx;
use crate::Look;

/// A progress bar: a rounded rail filled with the accent up to `progress` (0 to 1), thin
/// on Windows and in libadwaita, a little thicker on macOS and in the plain look.
pub struct ProgressBar {
    progress: f32,
    width: Option<f32>,
}

impl ProgressBar {
    /// A bar filled to `progress`, from 0 to 1; beyond those it is held to them, and a
    /// value that is not a number shows an empty bar.
    pub fn new(progress: f32) -> Self {
        // Not a number counts as nothing done yet.
        ProgressBar { progress: if progress.is_nan() { 0.0 } else { progress.clamp(0.0, 1.0) }, width: None }
    }
    /// A fixed width; by default the width available.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl Widget for ProgressBar {
    fn ui(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        let width = self.width.unwrap_or_else(|| ui.available_width());
        let thickness = if matches!(x.look, Look::Windows | Look::Gtk | Look::Material | Look::Ios) { 4.0 } else { 6.0 };
        let (outer, response) = ui.allocate_exact_size(egui::vec2(width, ui.spacing().interact_size.y.min(16.0)), Sense::hover());
        let rail = Rect::from_center_size(outer.center(), egui::vec2(width, thickness));
        let round = CornerRadius::same((thickness / 2.0) as u8);
        ui.painter().rect_filled(rail, round, x.c.track);
        let filled = Rect::from_min_max(rail.min, egui::pos2(egui::lerp(rail.left()..=rail.right(), self.progress), rail.max.y));
        ui.painter().rect_filled(filled, round, x.c.accent);
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::ProgressIndicator, ui.is_enabled(), format!("{:.0} %", self.progress * 100.0)));
        response
    }
}

/// A spinner: an accent arc turning, as WinUI's and libadwaita's are; macOS's spokes
/// fading round the circle.
pub struct Spinner {
    size: f32,
}

impl Spinner {
    /// A spinner 16 points across.
    pub fn new() -> Self {
        Spinner { size: 16.0 }
    }
    /// How far across it is.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
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
            let r = self.size / 2.0 - 1.5;
            if x.look == Look::Mac {
                for i in 0..12 {
                    let a = i as f32 / 12.0 * std::f32::consts::TAU;
                    let lead = ((time * 12.0).floor() as i32).rem_euclid(12) as f32;
                    let fade = 1.0 - ((lead - i as f32).rem_euclid(12.0) / 12.0);
                    let dir = egui::vec2(a.cos(), a.sin());
                    ui.painter().line_segment([m + dir * r * 0.45, m + dir * r], Stroke::new(1.6, x.c.dim.gamma_multiply(0.25 + 0.75 * fade)));
                }
            } else {
                let start = time * 4.0;
                let sweep = 1.5 + (time * 2.0).sin() * 0.8;
                let points: Vec<egui::Pos2> = (0..=24).map(|i| {
                    let a = start + sweep * i as f32 / 24.0;
                    m + egui::vec2(a.cos(), a.sin()) * r
                }).collect();
                ui.painter().add(egui::Shape::line(points, Stroke::new(2.0, x.c.accent)));
            }
        }
        response
    }
}
