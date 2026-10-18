//! Forms: rows of a label and a control, laid out the way the look lays out its settings.

use egui::{Align, Color32, CornerRadius, Layout, Rect, Sense, Stroke, Ui, UiBuilder};

use super::Ctx;
use crate::Look;

/// Rows of a label and its control, aligned the look's way: on macOS and in the plain look
/// the labels right-aligned in a column before the controls; on Windows and in Material
/// left-aligned; in libadwaita and on iOS a boxed list, one row each, the label at the
/// start and the control at the end.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui, radius: &mut f64, faces: &mut u32, snap: &mut bool) {
/// use egui_native_look as look;
/// look::Form::new("fillet").show(ui, |form| {
///     let width = form.control_width();
///     form.row("Radius", |ui| ui.add(look::NumberField::new(radius).range(0.0..=1024.0).width(width)));
///     form.row("Faces", |ui| ui.add(look::NumberField::new(faces).range(1.0..=64.0).width(width)));
///     form.row("Snap", |ui| ui.add(look::Switch::new(snap)));
/// });
/// # }
/// ```
pub struct Form {
    id: egui::Id,
}

impl Form {
    /// A form; `id` keeps its label column's width from frame to frame.
    pub fn new(id: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Form { id: egui::Id::new(id) }
    }

    /// Lays out the rows `add` adds, the width available.
    pub fn show<R>(self, ui: &mut Ui, add: impl FnOnce(&mut FormUi<'_>) -> R) -> R {
        let x = Ctx::of(ui);
        let boxed = matches!(x.look, Look::Gtk | Look::Ios);
        let id = ui.make_persistent_id(self.id);
        // The label column is as wide as the widest label, measured last frame; a wider
        // label this frame lays the form out again.
        let label_width = ui.ctx().data(|d| d.get_temp::<f32>(id)).unwrap_or(0.0);
        let background = boxed.then(|| ui.painter().add(egui::Shape::Noop));
        let (left, top) = (ui.cursor().left(), ui.cursor().top());
        let width = ui.available_width();
        let spacing = ui.spacing().item_spacing.y;
        if boxed {
            ui.spacing_mut().item_spacing.y = 0.0;
        }
        let mut form = FormUi { ui, x, id, boxed, label_width, widest: 0.0, rows: 0 };
        let inner = add(&mut form);
        let FormUi { ui, x, widest, .. } = form;
        if boxed {
            ui.spacing_mut().item_spacing.y = spacing;
        }
        if let Some(background) = background {
            let rect = Rect::from_min_max(egui::pos2(left, top), egui::pos2(left + width, ui.cursor().top()));
            ui.painter().set(background, card(&x, rect));
            ui.add_space(spacing);
        }
        if !boxed && widest != label_width {
            ui.ctx().data_mut(|d| d.insert_temp(id, widest));
            ui.ctx().request_discard("egui_native_look::Form label column");
        }
        inner
    }
}

/// The boxed list's card behind `rect`.
fn card(x: &Ctx, rect: Rect) -> egui::Shape {
    let c = &x.c;
    let (fill, round) = match x.look {
        Look::Ios => (if c.dark { crate::lerp_color(c.bg, Color32::WHITE, 0.1) } else { Color32::WHITE }, 10),
        _ => (if c.dark { crate::lerp_color(c.bg, Color32::WHITE, 0.07) } else { Color32::WHITE }, 12),
    };
    let stroke = if x.look == Look::Gtk { Stroke::new(1.0, c.border) } else { Stroke::NONE };
    egui::epaint::RectShape::new(rect, CornerRadius::same(round), fill, stroke, egui::StrokeKind::Inside).into()
}

/// The line between two rows of a boxed list.
fn separator(x: &Ctx) -> Color32 {
    if x.look == Look::Ios { x.c.fg.gamma_multiply(0.14) } else { x.c.border }
}

/// The rows of a [`Form`] being laid out.
pub struct FormUi<'a> {
    ui: &'a mut Ui,
    x: Ctx,
    id: egui::Id,
    boxed: bool,
    label_width: f32,
    widest: f32,
    rows: usize,
}

impl FormUi<'_> {
    /// The gap between the label column and the controls.
    const GAP: f32 = 12.0;
    /// How far a boxed row's label and control sit from the card's edges.
    const INSET: f32 = 14.0;

    /// The width a control takes to fill its row: what the label column leaves, or in a
    /// boxed list a third of the row, the rest being the label's.
    pub fn control_width(&self) -> f32 {
        let width = self.ui.available_width();
        if self.boxed {
            (width / 3.0).max(120.0)
        } else {
            (width - self.label_width - Self::GAP).max(80.0)
        }
    }

    /// A row of `label` and the control `control` adds.
    pub fn row<R>(&mut self, label: &str, control: impl FnOnce(&mut Ui) -> R) -> R {
        self.row_with(label, None, control)
    }

    /// A row whose label shows `tip` under the pointer.
    pub fn row_with_tip<R>(&mut self, label: &str, tip: &str, control: impl FnOnce(&mut Ui) -> R) -> R {
        self.row_with(label, Some(tip), control)
    }

    /// A row of nothing but `contents`, under the controls in the label column's layout
    /// (a check box with its own label, say), or a row of its own in a boxed list.
    pub fn full_row<R>(&mut self, contents: impl FnOnce(&mut Ui) -> R) -> R {
        if self.boxed {
            return self.row_with("", None, contents);
        }
        let indent = self.label_width + Self::GAP;
        self.rows += 1;
        self.ui.horizontal(|ui| {
            ui.add_space(indent - ui.spacing().item_spacing.x);
            contents(ui)
        })
        .inner
    }

    fn row_with<R>(&mut self, label: &str, tip: Option<&str>, control: impl FnOnce(&mut Ui) -> R) -> R {
        let x = &self.x;
        let font = x.m.body.clone();
        let text_width = self.ui.fonts_mut(|f| f.layout_no_wrap(label.to_string(), font.clone(), x.c.fg).size().x);
        let first = self.rows == 0;
        self.rows += 1;
        if self.boxed {
            let row: f32 = if x.look == Look::Ios { 44.0 } else { 50.0 };
            let height = row.max(self.ui.spacing().interact_size.y + 12.0);
            let (rect, response) = self.ui.allocate_exact_size(egui::vec2(self.ui.available_width(), height), Sense::hover());
            if !first {
                let y = rect.top() + 0.5;
                self.ui.painter().hline(rect.left() + if x.look == Look::Ios { Self::INSET } else { 0.0 }..=rect.right(), y, Stroke::new(1.0, separator(x)));
            }
            crate::text(self.ui.painter(), egui::pos2(rect.left() + Self::INSET, rect.center().y), egui::Align2::LEFT_CENTER, label, font, x.c.fg);
            if let Some(tip) = tip {
                let label_rect = Rect::from_min_size(rect.min, egui::vec2(Self::INSET + text_width, height));
                self.ui.interact(label_rect, response.id.with("tip"), Sense::hover()).on_hover_text(tip);
            }
            // The control ends at the row's end: it is laid out left to right from where
            // last frame's width puts it, and laid out again if that width changed.
            let id = self.id.with(("egui_native_look::Form row", self.rows));
            let used = self.ui.ctx().data(|d| d.get_temp::<f32>(id)).unwrap_or(0.0);
            let end = rect.right() - Self::INSET;
            // Unmeasured, it has the whole room after the label, which a control that fills
            // its row keeps.
            let after_label = rect.left() + Self::INSET + text_width + Self::GAP;
            let start = if used > 0.0 { (end - used).max(after_label) } else { after_label };
            let room = Rect::from_min_max(egui::pos2(start, rect.top()), egui::pos2(end, rect.bottom()));
            let mut child = self.ui.new_child(UiBuilder::new().max_rect(room).layout(Layout::left_to_right(Align::Center)));
            let inner = control(&mut child);
            let width = child.min_rect().width();
            if (width - used).abs() > 0.5 {
                self.ui.ctx().data_mut(|d| d.insert_temp(id, width));
                self.ui.ctx().request_discard("egui_native_look::Form control width");
            }
            return inner;
        }
        self.widest = self.widest.max(text_width);
        let label_width = self.label_width.max(text_width);
        let right = matches!(x.look, Look::Mac | Look::Plain);
        let colour = x.c.fg;
        self.ui
            .horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = Self::GAP;
                let (rect, response) = ui.allocate_exact_size(egui::vec2(label_width, ui.spacing().interact_size.y), Sense::hover());
                let (at, align) = if right { (rect.right_center(), egui::Align2::RIGHT_CENTER) } else { (rect.left_center(), egui::Align2::LEFT_CENTER) };
                crate::text(ui.painter(), at, align, label, font, colour);
                if let Some(tip) = tip {
                    response.on_hover_text(tip);
                }
                control(ui)
            })
            .inner
    }
}
