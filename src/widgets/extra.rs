//! Toolbar buttons, colour wells and toasts.

use egui::{Color32, CornerRadius, Rect, Response, Sense, Stroke, Ui, Widget};

use super::Ctx;
use crate::Look;

/// A square button holding an icon glyph, for a toolbar: flat at rest, lit under the
/// pointer; when selected, tinted with the accent (Material's filled icon button in the
/// accent itself). Round in Material.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui, grid: &mut bool) {
/// use egui_native_look as look;
/// if ui.add(look::IconButton::new("#").selected(*grid).tooltip("Grid")).clicked() {
///     *grid = !*grid;
/// }
/// # }
/// ```
///
/// The builder must be added to a UI or shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::IconButton;
/// IconButton::new("+");
/// ```
#[must_use = "add the widget to a UI or call its show method"]
pub struct IconButton {
    glyph: String,
    font: Option<egui::FontId>,
    selected: Option<bool>,
    tooltip: String,
    label: String,
}

impl IconButton {
    /// A button showing `glyph`, an icon font's or any text.
    pub fn new(glyph: impl Into<String>) -> Self {
        IconButton {
            glyph: glyph.into(),
            font: None,
            selected: None,
            tooltip: String::new(),
            label: String::new(),
        }
    }
    /// The glyph's font, an icon font's; by default the body text's, a little larger.
    pub fn font(mut self, font: egui::FontId) -> Self {
        self.font = Some(font);
        self
    }
    /// Shown as on, a toggle that is set or the tool in use.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }
    /// The action's name for assistive technology, independent of its tooltip.
    pub fn accessible_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
    /// What it does, shown under the pointer. Also supplies its accessible name
    /// unless [`Self::accessible_label`] overrides it.
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = tooltip.into();
        self
    }
}

impl Widget for IconButton {
    fn ui(self, ui: &mut Ui) -> Response {
        super::unfaded(ui, |ui| self.add_contents(ui))
    }
}

impl IconButton {
    fn add_contents(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        let c = &x.c;
        let side = ui.spacing().interact_size.y;
        let font = self
            .font
            .clone()
            .unwrap_or_else(|| egui::FontId::new(x.m.body.size + 3.0, x.m.body.family.clone()));
        // macOS's borderless buttons are as wide as their glyph and a little room.
        let width = if x.look == Look::Mac {
            let glyph = ui.fonts_mut(|f| {
                f.layout_no_wrap(self.glyph.clone(), font.clone(), c.fg)
                    .size()
                    .x
            });
            (glyph + 6.0).max(18.0)
        } else {
            side
        };
        let enabled = ui.is_enabled();
        let (rect, response) = super::allocate_control(
            ui,
            egui::vec2(width, side),
            if enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        let round = if x.look == Look::Material {
            CornerRadius::same((side / 2.0) as u8)
        } else {
            x.radius()
        };
        let down = response.is_pointer_button_down_on();
        let hovered = enabled && response.hovered();
        let tint = |alpha: f32| c.accent.gamma_multiply(alpha);
        let (fill, colour) = match (self.selected.unwrap_or(false), x.look) {
            (true, Look::Material) => (
                Some(if hovered { c.accent_hover } else { c.accent }),
                c.on_accent,
            ),
            // macOS's borderless buttons show only their glyph: in the accent when on,
            // darker while pressed.
            (true, Look::Mac) => (None, c.accent),
            (false, Look::Mac) => (None, if down { c.fg } else { c.dim }),
            (true, _) => (Some(tint(if hovered { 0.32 } else { 0.24 })), c.accent),
            (false, _) if down => (Some(c.button_pressed), c.fg),
            (false, _) if hovered => (Some(c.button_hover), c.fg),
            (false, _) => (None, c.dim),
        };
        if ui.is_rect_visible(rect) {
            if let Some(fill) = fill {
                ui.painter().rect_filled(rect, round, fill);
            }
            let colour = if enabled { colour } else { c.disabled };
            crate::text(
                ui.painter(),
                rect.center(),
                egui::Align2::CENTER_CENTER,
                &self.glyph,
                font,
                colour,
            );
            if response.has_focus() {
                crate::paint::ring(ui.painter(), &x.paint(), rect, round);
            }
        }
        let selected = self.selected;
        let label = if !self.label.is_empty() {
            self.label
        } else if self.tooltip.is_empty() {
            self.glyph.clone()
        } else {
            self.tooltip.clone()
        };
        response.widget_info(|| match selected {
            Some(selected) => {
                egui::WidgetInfo::selected(egui::Role::Button, enabled, selected, &label)
            }
            None => egui::WidgetInfo::labeled(egui::Role::Button, enabled, &label),
        });
        if self.tooltip.is_empty() {
            response
        } else {
            response.on_hover_text(self.tooltip)
        }
    }
}

/// A colour well: a button showing the colour, which opens egui's colour picker under it.
/// The colour has changed when `response.changed()`.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui, tint: &mut egui::Color32) {
/// ui.add(egui_native_look::ColorWell::new(tint).alpha(true));
/// # }
/// ```
///
/// The builder must be added to a UI or shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::ColorWell;
/// ColorWell::new(&mut egui::Color32::WHITE);
/// ```
#[must_use = "add the widget to a UI or call its show method"]
pub struct ColorWell<'a> {
    color: &'a mut Color32,
    alpha: bool,
    label: String,
}

impl<'a> ColorWell<'a> {
    /// A well showing and picking `color`, opaque unless [`ColorWell::alpha`].
    pub fn new(color: &'a mut Color32) -> Self {
        ColorWell {
            color,
            alpha: false,
            label: "Colour".into(),
        }
    }
    /// What colour it picks, for a screen reader: "Colour" by default, for an app to name
    /// or translate.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
    /// Lets the picker change the colour's opacity too.
    pub fn alpha(mut self, alpha: bool) -> Self {
        self.alpha = alpha;
        self
    }
}

impl Widget for ColorWell<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        super::unfaded(ui, |ui| self.add_contents(ui))
    }
}

impl ColorWell<'_> {
    fn add_contents(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        let c = &x.c;
        let height = ui.spacing().interact_size.y;
        // macOS's colour well is a capsule twice as wide as it is tall.
        let width = if x.look == Look::Mac {
            height * 2.0
        } else {
            (height * 1.75).round()
        };
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
        let popup = response.id.with("egui_native_look-color-well");
        if !enabled {
            egui::Popup::close_id(ui.ctx(), popup);
        }
        let open = egui::Popup::is_id_open(ui.ctx(), popup);
        if ui.is_rect_visible(rect) {
            let fill = if !enabled {
                c.button_disabled
            } else if response.hovered() || open {
                c.button_hover
            } else {
                c.button
            };
            let round = if matches!(x.look, Look::Material | Look::Mac) {
                CornerRadius::same((height / 2.0) as u8)
            } else {
                x.radius()
            };
            crate::paint::paint_body(ui.painter(), &x.paint(), rect, round, fill, enabled);
            if x.look == Look::Plain {
                ui.painter().rect_stroke(
                    rect,
                    round,
                    Stroke::new(1.0, c.border),
                    egui::StrokeKind::Inside,
                );
            }
            let swatch = rect.shrink(match x.look {
                Look::Material => height * 0.25,
                Look::Mac => 3.0,
                _ => 4.0,
            });
            let inner = CornerRadius::same(round.nw.saturating_sub(3).max(2));
            if self.color.a() < 255 {
                // A checkerboard behind a colour that is not opaque, its left half.
                let left =
                    Rect::from_min_max(swatch.min, egui::pos2(swatch.center().x, swatch.max.y));
                ui.painter().rect_filled(swatch, inner, Color32::WHITE);
                let half = left.height() / 2.0;
                let mut at = left.left();
                let mut row = 0;
                while at < left.right() {
                    let cell = Rect::from_min_size(
                        egui::pos2(at, left.top() + if row % 2 == 0 { 0.0 } else { half }),
                        egui::vec2(half.min(left.right() - at), half),
                    );
                    ui.painter().rect_filled(cell, 0.0, Color32::from_gray(200));
                    at += half;
                    row += 1;
                }
                ui.painter().rect_filled(swatch, inner, *self.color);
                let right =
                    Rect::from_min_max(egui::pos2(swatch.center().x, swatch.min.y), swatch.max);
                ui.painter().rect_filled(
                    right,
                    CornerRadius {
                        nw: 0,
                        sw: 0,
                        ..inner
                    },
                    self.color.to_opaque(),
                );
            } else {
                ui.painter().rect_filled(swatch, inner, *self.color);
            }
            ui.painter().rect_stroke(
                swatch,
                inner,
                Stroke::new(1.0, Color32::from_black_alpha(40)),
                egui::StrokeKind::Inside,
            );
            if response.has_focus() {
                crate::paint::ring(ui.painter(), &x.paint(), rect, round);
            }
        }
        let alpha = if self.alpha {
            egui::color_picker::Alpha::OnlyBlend
        } else {
            egui::color_picker::Alpha::Opaque
        };
        let mut changed = false;
        egui::Popup::from_toggle_button_response(&response)
            .id(popup)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                changed = egui::color_picker::color_picker_color32(ui, self.color, alpha);
            });
        if changed {
            response.mark_changed();
        }
        let label = self.label;
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::ColorWell, enabled, &label));
        let [red, green, blue, alpha] = self.color.to_srgba_unmultiplied();
        let value = if self.alpha {
            format!("#{red:02X}{green:02X}{blue:02X}{alpha:02X}")
        } else {
            format!("#{red:02X}{green:02X}{blue:02X}")
        };
        let expanded = egui::Popup::is_id_open(ui.ctx(), popup);
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_value(value);
            node.set_color_value(egui::accesskit::Color {
                red,
                green,
                blue,
                alpha,
            });
            node.set_expanded(expanded);
        });
        response
    }
}

/// A card lifted off the window: a step lighter on the dark scheme, white on the light.
fn card_fill(c: &crate::Colors) -> Color32 {
    if c.dark {
        crate::lerp_color(c.bg, Color32::WHITE, 0.08)
    } else {
        Color32::WHITE
    }
}

/// How long a toast stays, and how long it takes to come and go.
const TOAST_SECONDS: f64 = 4.0;
const TOAST_FADE: f64 = 0.2;

/// The toasts asked for, and how many have been.
#[derive(Clone, Default)]
struct Toasts(Vec<Toast>, u64);

#[derive(Clone)]
struct Toast {
    text: String,
    /// When it was asked for, moved on while it is paused under the pointer.
    start: f64,
    viewport: egui::ViewportId,
    /// Its number, which keeps its id while it is shown.
    serial: u64,
    /// Where it was drawn last frame.
    rect: Option<Rect>,
}

fn toasts_id() -> egui::Id {
    egui::Id::unique("egui_native_look::Toasts")
}

/// Shows `text` in a toast at the bottom of the window for a few seconds; see
/// [`show_toasts`]. It shows in the viewport (the window) it was asked for in.
pub fn toast(ctx: &egui::Context, text: impl Into<String>) {
    let now = ctx.input(|i| i.time);
    let viewport = ctx.viewport_id();
    ctx.data_mut(|d| {
        let toasts = d.get_temp_mut_or_default::<Toasts>(toasts_id());
        toasts.1 += 1;
        let serial = toasts.1;
        toasts.0.push(Toast {
            text: text.into(),
            start: now,
            viewport,
            serial,
            rect: None,
        });
    });
    ctx.request_repaint();
}

/// Draws the toasts [`toast`] asked for, once a frame, over everything else: a dark pill
/// in libadwaita, a HUD on macOS and iOS, Material's snackbar, a raised card on Windows and
/// in the plain look. The newest is at the bottom; a long text wraps. Each viewport draws
/// its own toasts. A toast under the pointer waits there, and screen readers announce each
/// one as a status message.
pub fn show_toasts(ctx: &egui::Context) {
    let (now, dt, hover) = ctx.input(|i| (i.time, f64::from(i.unstable_dt), i.pointer.hover_pos()));
    let viewport = ctx.viewport_id();
    let mut all = ctx
        .data(|d| d.get_temp::<Toasts>(toasts_id()))
        .unwrap_or_default();
    for toast in &mut all.0 {
        // Paused once fully shown, so it neither fades nor runs out while being read.
        let shown = now - toast.start >= TOAST_FADE;
        if toast.viewport == viewport
            && shown
            && hover.is_some_and(|at| toast.rect.is_some_and(|rect| rect.contains(at)))
        {
            toast.start = (toast.start + dt).min(now - TOAST_FADE);
        }
    }
    all.0.retain(|toast| now - toast.start < TOAST_SECONDS);
    let mut toasts: Vec<Toast> = all
        .0
        .iter()
        .filter(|toast| toast.viewport == viewport)
        .cloned()
        .collect();
    if toasts.is_empty() {
        ctx.data_mut(|d| d.insert_temp(toasts_id(), all));
        return;
    }
    let resolved = crate::resolved(ctx);
    let look = resolved.theme.look;
    let c = resolved.colors(ctx);
    let font = resolved.theme.metrics.body.clone();
    let screen = ctx.content_rect();
    egui::Area::new(toasts_id().with(viewport))
        .order(egui::Order::Tooltip)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            ui.set_clip_rect(screen);
            draw_toasts(ui, &mut toasts, now, look, &c, &font, screen);
        });
    for toast in toasts {
        if let Some(stored) = all
            .0
            .iter_mut()
            .find(|stored| stored.serial == toast.serial)
        {
            stored.rect = toast.rect;
        }
    }
    ctx.data_mut(|d| d.insert_temp(toasts_id(), all));
    ctx.request_repaint();
}

/// Lays out and paints `toasts` in `ui`, newest at the bottom, and keeps where each went.
fn draw_toasts(
    ui: &mut egui::Ui,
    toasts: &mut [Toast],
    now: f64,
    look: Look,
    c: &crate::Colors,
    font: &egui::FontId,
    screen: Rect,
) {
    let ctx = ui.ctx().clone();
    let painter = ui.painter().clone();
    let mut bottom = screen.bottom() - 24.0;
    for toast in toasts.iter_mut().rev() {
        let text = &toast.text;
        let age = now - toast.start;
        let shown =
            ((age / TOAST_FADE).min((TOAST_SECONDS - age) / TOAST_FADE)).clamp(0.0, 1.0) as f32;
        let (fill, colour, height, round, stroke) = match look {
            Look::Gtk => (
                Color32::from_rgb(0x30, 0x30, 0x34),
                Color32::WHITE,
                40.0,
                20,
                Stroke::NONE,
            ),
            Look::Mac | Look::Ios => (
                Color32::from_black_alpha(200),
                Color32::WHITE,
                36.0,
                12,
                Stroke::NONE,
            ),
            // Material's inverse surface and its text.
            Look::Material if c.dark => (
                crate::colors::rgb(0xe6e0e9),
                crate::colors::rgb(0x322f35),
                48.0,
                4,
                Stroke::NONE,
            ),
            Look::Material => (
                crate::colors::rgb(0x322f35),
                crate::colors::rgb(0xf5eff7),
                48.0,
                4,
                Stroke::NONE,
            ),
            Look::Windows => (
                card_fill(c),
                c.fg,
                44.0,
                8,
                Stroke::new(1.0, c.control_border),
            ),
            Look::Plain => (card_fill(c), c.fg, 36.0, 4, Stroke::new(1.0, c.border)),
        };
        let galley = painter.layout(
            text.clone(),
            font.clone(),
            colour,
            (screen.width() - 72.0).max(40.0),
        );
        let width = (galley.size().x + 40.0).min(screen.width() - 32.0);
        let first_row = galley
            .rows
            .first()
            .map_or(galley.size().y, |row| row.height());
        let height: f32 = height + galley.size().y - first_row;
        let lift = (1.0 - shown) * 12.0;
        let rect = Rect::from_min_size(
            egui::pos2(screen.center().x - width / 2.0, bottom - height + lift),
            egui::vec2(width, height),
        );
        let round = CornerRadius::same(round);
        let shadow = egui::Shadow {
            offset: [0, 4],
            blur: 16,
            spread: 0,
            color: Color32::from_black_alpha((60.0 * shown) as u8),
        };
        painter.add(shadow.as_shape(rect, round));
        painter.add(egui::epaint::RectShape::new(
            rect,
            round,
            fill.gamma_multiply(shown),
            stroke,
            egui::StrokeKind::Inside,
        ));
        // The first line's capitals centred where a one-line toast has them.
        let top = crate::galley_top(
            &ctx,
            &galley,
            rect.center().y - (galley.size().y - first_row) / 2.0,
        );
        painter.galley(
            egui::pos2(rect.center().x - galley.size().x / 2.0, top),
            galley,
            colour.gamma_multiply(shown),
        );
        let response = ui.interact(rect, toasts_id().with(toast.serial), Sense::hover());
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Status, true, text));
        ctx.accesskit_node_builder(response.id, |node| {
            node.set_live(egui::accesskit::Live::Polite);
        });
        toast.rect = Some(rect);
        bottom = rect.top() - 8.0;
    }
}
