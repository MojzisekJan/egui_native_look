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
///
/// The builder must be added to a UI or shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::Form;
/// Form::new("settings");
/// ```
#[must_use = "add the widget to a UI or call its show method"]
pub struct Form {
    id: egui::IdSalt,
}

impl Form {
    /// A form; `id` keeps its label column's width from frame to frame.
    pub fn new(id: impl egui::AsIdSalt) -> Self {
        Form {
            id: egui::IdSalt::new(id),
        }
    }

    /// Lays out the rows `add` adds, the width available.
    pub fn show<R>(self, ui: &mut Ui, add: impl FnOnce(&mut FormUi<'_>) -> R) -> R {
        let x = Ctx::of(ui);
        let boxed = matches!(x.look, Look::Gtk | Look::Ios);
        let id = ui.make_persistent_id(self.id);
        // The label column is as wide as the widest label, measured last frame; a wider
        // label this frame lays the form out again. Reserve the maximum column before
        // the first measurement, and clamp cached widths after a viewport resize.
        let label_width = ui
            .ctx()
            .data(|d| d.get_temp::<f32>(id))
            .unwrap_or((ui.available_width() * 0.4).max(60.0));
        let label_width = label_width.min((ui.available_width() * 0.4).max(60.0));
        let background = boxed.then(|| ui.painter().add(egui::Shape::Noop));
        let (left, top) = (ui.cursor().left(), ui.cursor().top());
        let width = ui.available_width();
        let spacing = ui.spacing().item_spacing.y;
        if boxed {
            ui.spacing_mut().item_spacing.y = 0.0;
        }
        let mut form = FormUi {
            ui,
            x,
            id,
            boxed,
            label_width,
            widest: 0.0,
            rows: 0,
        };
        let inner = add(&mut form);
        let FormUi { ui, x, widest, .. } = form;
        if boxed {
            ui.spacing_mut().item_spacing.y = spacing;
        }
        if let Some(background) = background {
            let rect = Rect::from_min_max(
                egui::pos2(left, top),
                egui::pos2(left + width, ui.cursor().top()),
            );
            ui.painter().set(background, card(&x, rect));
            ui.add_space(spacing);
        }
        if !boxed && widest != label_width {
            ui.ctx().data_mut(|d| d.insert_temp(id, widest));
            ui.ctx()
                .request_discard("egui_native_look::Form label column");
        }
        inner
    }
}

/// The boxed list's card behind `rect`.
fn card(x: &Ctx, rect: Rect) -> egui::Shape {
    let c = &x.c;
    let (fill, round) = match x.look {
        // iOS's cards take its secondary background, which stands out from the window's
        // white or black; on a window of the app's own colour they keep white.
        Look::Ios => (
            if c.surface != c.bg {
                c.surface
            } else if c.dark {
                crate::lerp_color(c.bg, Color32::WHITE, 0.1)
            } else {
                Color32::WHITE
            },
            10,
        ),
        _ => (
            if c.dark {
                crate::lerp_color(c.bg, Color32::WHITE, 0.07)
            } else {
                Color32::WHITE
            },
            12,
        ),
    };
    let stroke = if x.look == Look::Gtk {
        Stroke::new(1.0, c.border)
    } else {
        Stroke::NONE
    };
    egui::epaint::RectShape::new(
        rect,
        CornerRadius::same(round),
        fill,
        stroke,
        egui::StrokeKind::Inside,
    )
    .into()
}

/// The line between two rows of a boxed list.
fn separator(x: &Ctx) -> Color32 {
    if x.look == Look::Ios {
        x.c.fg.gamma_multiply(0.14)
    } else {
        x.c.border
    }
}

/// What a label shows under the pointer: the whole label when it was cut short, and its tip.
fn hover_text(label: &str, cut: bool, tip: Option<&str>) -> Option<String> {
    match (cut, tip) {
        (true, Some(tip)) => Some(format!("{label}\n\n{tip}")),
        (true, None) => Some(label.to_string()),
        (false, tip) => tip.map(str::to_string),
    }
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
    /// The gap between the label column and the controls: eight points in macOS's
    /// two-column form, as SwiftUI sets it.
    fn gap(&self) -> f32 {
        if self.x.look == Look::Mac {
            8.0
        } else {
            12.0
        }
    }
    /// How far a boxed row's label and control sit from the card's edges.
    const INSET: f32 = 14.0;

    /// The width a control takes to fill its row: what the label column leaves, or in a
    /// boxed list a third of the row, the rest being the label's.
    pub fn control_width(&self) -> f32 {
        let width = self.ui.available_width();
        if self.boxed {
            (width / 3.0).max(120.0)
        } else {
            (width - self.label_width - self.gap()).max(80.0)
        }
    }

    /// A row of `label` and the control `control` adds.
    pub fn row<R>(&mut self, label: &str, control: impl FnOnce(&mut Ui) -> R) -> R {
        self.row_with(label, None, control)
    }

    /// A row whose label shows `tip` under the pointer.
    pub fn row_with_tip<R>(
        &mut self,
        label: &str,
        tip: &str,
        control: impl FnOnce(&mut Ui) -> R,
    ) -> R {
        self.row_with(label, Some(tip), control)
    }

    /// A row of nothing but `contents`, under the controls in the label column's layout
    /// (a check box with its own label, say), or a row of its own in a boxed list.
    pub fn full_row<R>(&mut self, contents: impl FnOnce(&mut Ui) -> R) -> R {
        if self.boxed {
            return self.row_with("", None, contents);
        }
        let indent = self.label_width + self.gap();
        self.rows += 1;
        self.ui
            .horizontal(|ui| {
                ui.add_space(indent - ui.spacing().item_spacing.x);
                contents(ui)
            })
            .inner
    }

    fn row_with<R>(
        &mut self,
        label: &str,
        tip: Option<&str>,
        control: impl FnOnce(&mut Ui) -> R,
    ) -> R {
        // The row's label names its control for a screen reader, as a visible label would.
        let ctx = self.ui.ctx().clone();
        let outer = ctx
            .data(|d| d.get_temp::<Option<String>>(row_label_id()))
            .flatten();
        ctx.data_mut(|d| d.insert_temp(row_label_id(), Some(label.to_owned())));
        let inner = self.lay_out_row(label, tip, control);
        ctx.data_mut(|d| d.insert_temp(row_label_id(), outer));
        inner
    }

    fn lay_out_row<R>(
        &mut self,
        label: &str,
        tip: Option<&str>,
        control: impl FnOnce(&mut Ui) -> R,
    ) -> R {
        let x = &self.x;
        let font = x.m.body.clone();
        let text_width = self.ui.fonts_mut(|f| {
            f.layout_no_wrap(label.to_string(), font.clone(), x.c.fg)
                .size()
                .x
        });
        let first = self.rows == 0;
        self.rows += 1;
        if self.boxed {
            // libadwaita's rows: a 50-point header in 2 points of padding.
            let row: f32 = match x.look {
                Look::Ios => 44.0,
                Look::Gtk => 54.0,
                _ => 50.0,
            };
            let height = row.max(self.ui.spacing().interact_size.y + 12.0);
            let (rect, response) = self.ui.allocate_exact_size(
                egui::vec2(self.ui.available_width(), height),
                Sense::hover(),
            );
            if !first {
                let y = rect.top() + 0.5;
                self.ui.painter().hline(
                    rect.left()
                        + if x.look == Look::Ios {
                            Self::INSET
                        } else {
                            0.0
                        }..=rect.right(),
                    y,
                    Stroke::new(1.0, separator(x)),
                );
            }
            // The control ends at the row's end: it is laid out left to right from where
            // last frame's width puts it, and laid out again if that width changed.
            let id = self.id.with(("egui_native_look::Form row", self.rows));
            let used = self.ui.ctx().data(|d| d.get_temp::<f32>(id)).unwrap_or(0.0);
            // A label too long for what the control leaves is cut short, whole under the
            // pointer.
            let room = (rect.width() - 2.0 * Self::INSET - self.gap() - used).max(40.0);
            let (galley, cut) = super::truncated(self.ui, label, font, x.c.fg, room);
            let shown_width = galley.size().x;
            let top = crate::galley_top(self.ui.ctx(), &galley, rect.center().y);
            self.ui
                .painter()
                .galley(egui::pos2(rect.left() + Self::INSET, top), galley, x.c.fg);
            if let Some(hover) = hover_text(label, cut, tip) {
                let label_rect =
                    Rect::from_min_size(rect.min, egui::vec2(Self::INSET + shown_width, height));
                self.ui
                    .interact(label_rect, response.id.with("tip"), Sense::hover())
                    .on_hover_text(hover);
            }
            let text_width = shown_width;
            let end = rect.right() - Self::INSET;
            // Unmeasured, it has the whole room after the label, which a control that fills
            // its row keeps.
            let after_label = rect.left() + Self::INSET + text_width + self.gap();
            let start = if used > 0.0 {
                (end - used).max(after_label)
            } else {
                after_label
            };
            let room = Rect::from_min_max(
                egui::pos2(start, rect.top()),
                egui::pos2(end, rect.bottom()),
            );
            // Aligned to the end, which a control drawing less than its width (iOS's
            // picker) keeps its content against.
            let mut child = self.ui.new_child(
                UiBuilder::new()
                    .max_rect(room)
                    .layout(Layout::left_to_right(Align::Center).with_main_align(Align::Max)),
            );
            let outer = in_boxed_row(child.ctx());
            child.data_mut(|d| d.insert_temp(boxed_row_id(), true));
            let inner = control(&mut child);
            child.data_mut(|d| d.insert_temp(boxed_row_id(), outer));
            let width = child.min_rect().width();
            if (width - used).abs() > 0.5 {
                self.ui.ctx().data_mut(|d| d.insert_temp(id, width));
                self.ui
                    .ctx()
                    .request_discard("egui_native_look::Form control width");
            }
            return inner;
        }
        // The label column takes at most two fifths of the form; a longer label is cut short,
        // whole under the pointer.
        let most = (self.ui.available_width() * 0.4).max(60.0);
        self.widest = self.widest.max(text_width.min(most));
        let label_width = self.label_width.max(text_width).min(most);
        let right = matches!(x.look, Look::Mac | Look::Plain);
        let colour = x.c.fg;
        let gap = self.gap();
        self.ui
            .horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(label_width, ui.spacing().interact_size.y),
                    Sense::hover(),
                );
                let (galley, cut) = super::truncated(ui, label, font, colour, label_width);
                let left = if right {
                    rect.right() - galley.size().x
                } else {
                    rect.left()
                };
                let top = crate::galley_top(ui.ctx(), &galley, rect.center().y);
                ui.painter().galley(egui::pos2(left, top), galley, colour);
                if let Some(hover) = hover_text(label, cut, tip) {
                    response.on_hover_text(hover);
                }
                control(ui)
            })
            .inner
    }
}

fn boxed_row_id() -> egui::Id {
    egui::Id::unique("egui_native_look::Form boxed row")
}

/// Whether the control being added ends a boxed list's row, where libadwaita draws its
/// spin and combo rows without a field.
pub(super) fn in_boxed_row(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(boxed_row_id()))
        .unwrap_or(false)
}

fn row_label_id() -> egui::Id {
    egui::Id::unique("egui_native_look::Form row label")
}

/// The label of the [`Form`] row being laid out, for a control that draws no label of
/// its own; `own` wins when it is set.
pub(crate) fn accessible_name(ui: &Ui, own: &str) -> String {
    if !own.is_empty() {
        return own.to_owned();
    }
    ui.ctx()
        .data(|d| d.get_temp::<Option<String>>(row_label_id()))
        .flatten()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::hover_text;

    #[test]
    fn truncated_labels_remain_available_with_their_help_text() {
        assert_eq!(
            hover_text("Long label", true, Some("Help")),
            Some("Long label\n\nHelp".into())
        );
        assert_eq!(
            hover_text("Long label", true, None),
            Some("Long label".into())
        );
        assert_eq!(
            hover_text("Label", false, Some("Help")),
            Some("Help".into())
        );
        assert_eq!(hover_text("Label", false, None), None);
    }
}
