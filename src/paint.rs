//! Painting the controls' bodies, for this crate's controls and an app's own. Each takes
//! the look and colours of the theme installed in the painter's context.

use egui::{Color32, CornerRadius, FontId, Rect, Response, Stroke};

use crate::{galley_top, Colors, Look};

/// What painting needs: the look, its colours in the scheme of the moment, its rounding.
#[derive(Clone, Copy)]
pub(crate) struct Paint {
    pub look: Look,
    pub c: Colors,
    pub radius: u8,
}

impl Paint {
    /// The theme installed in `ctx`, in its scheme of the moment.
    pub fn of(ctx: &egui::Context) -> Paint {
        let resolved = crate::resolved(ctx);
        Paint { look: resolved.theme.look, c: resolved.colors(ctx), radius: resolved.theme.metrics.radius }
    }
}

/// A raised macOS control's body in `rect`: on the light scheme a soft shadow under it and
/// a hairline round it, on the dark a lighter top edge, then the fill.
pub fn paint_raised(painter: &egui::Painter, rect: Rect, radius: CornerRadius, fill: Color32, enabled: bool) {
    raised(painter, painter.ctx().theme() == egui::Theme::Dark, rect, radius, fill, enabled);
}

pub(crate) fn raised(painter: &egui::Painter, dark: bool, rect: Rect, radius: CornerRadius, fill: Color32, enabled: bool) {
    if enabled && !dark {
        painter.rect_filled(rect.translate(egui::vec2(0.0, 0.5)).expand(0.5), radius, Color32::from_black_alpha(28));
    }
    painter.rect_filled(rect, radius, fill);
    if dark {
        let top = Rect::from_min_max(rect.min + egui::vec2(radius.nw as f32, 0.0), egui::pos2(rect.max.x - radius.ne as f32, rect.min.y + 1.0));
        painter.rect_filled(top, 0.0, Color32::from_white_alpha(if enabled { 18 } else { 8 }));
    } else {
        painter.rect_stroke(rect, radius, Stroke::new(1.0, Color32::from_black_alpha(if enabled { 30 } else { 16 })), egui::StrokeKind::Inside);
    }
}

/// A WinUI control's body in `rect`: the fill, a hairline edge, and the darker bottom edge
/// that lifts it off the light scheme.
pub fn paint_fluent(painter: &egui::Painter, rect: Rect, radius: CornerRadius, fill: Color32, enabled: bool) {
    fluent(painter, &Paint::of(painter.ctx()).c, rect, radius, fill, enabled);
}

pub(crate) fn fluent(painter: &egui::Painter, c: &Colors, rect: Rect, radius: CornerRadius, fill: Color32, enabled: bool) {
    painter.rect_filled(rect, radius, fill);
    painter.rect_stroke(rect, radius, Stroke::new(1.0, c.control_border), egui::StrokeKind::Inside);
    if enabled && !c.dark {
        let r = f32::from(radius.sw.max(radius.se));
        let y = rect.bottom() - 0.5;
        painter.line_segment([egui::pos2(rect.left() + r, y), egui::pos2(rect.right() - r, y)], Stroke::new(1.0, Color32::from_black_alpha(42)));
    }
}

/// The body of a control in the look: raised on macOS, Fluent on Windows, flat in
/// libadwaita and the plain look (the plain look adds its own edge).
pub(crate) fn paint_body(painter: &egui::Painter, p: &Paint, rect: Rect, radius: CornerRadius, fill: Color32, enabled: bool) {
    match p.look {
        Look::Mac => raised(painter, p.c.dark, rect, radius, fill, enabled),
        Look::Windows => fluent(painter, &p.c, rect, radius, fill, enabled),
        Look::Gtk | Look::Ios | Look::Material | Look::Plain => {
            painter.rect_filled(rect, radius, fill);
        }
    }
}

/// The ring round a focused control: on macOS a soft band of the accent outside it, on
/// Windows WinUI's two-tone ring, in libadwaita a half-tone accent outline just inside the
/// edge, else a thin accent line.
pub fn focus_ring(painter: &egui::Painter, rect: Rect, radius: CornerRadius) {
    ring(painter, &Paint::of(painter.ctx()), rect, radius);
}

pub(crate) fn ring(painter: &egui::Painter, p: &Paint, rect: Rect, radius: CornerRadius) {
    let (look, c) = (p.look, p.c);
    let grown = CornerRadius { nw: radius.nw + 2, ne: radius.ne + 2, sw: radius.sw + 2, se: radius.se + 2 };
    match look {
        Look::Gtk => {
            painter.rect_stroke(rect, radius, Stroke::new(2.0, c.accent.gamma_multiply(0.5)), egui::StrokeKind::Inside);
        }
        Look::Material => {
            painter.rect_stroke(rect.expand(2.0), grown, Stroke::new(3.0, c.accent), egui::StrokeKind::Outside);
        }
        Look::Ios => {
            painter.rect_stroke(rect.expand(0.5), grown, Stroke::new(3.0, c.accent.gamma_multiply(0.5)), egui::StrokeKind::Outside);
        }
        Look::Windows => {
            painter.rect_stroke(rect.expand(1.0), grown, Stroke::new(2.0, c.fg), egui::StrokeKind::Outside);
            painter.rect_stroke(rect, radius, Stroke::new(1.0, c.bg), egui::StrokeKind::Outside);
        }
        Look::Mac => {
            painter.rect_stroke(rect.expand(0.5), grown, Stroke::new(3.0, c.accent.gamma_multiply(0.5)), egui::StrokeKind::Outside);
        }
        Look::Plain => {
            painter.rect_stroke(rect.expand(1.0), radius, Stroke::new(1.0, c.accent_hover), egui::StrokeKind::Outside);
        }
    }
}

/// Paints a push button in `rect` with `label` in `font`. `accent` is a toggle that is
/// switched on, or the default action of a dialog.
pub fn paint_button(painter: &egui::Painter, rect: Rect, response: &Response, label: &str, font: FontId, enabled: bool, accent: bool) {
    button(painter, &Paint::of(painter.ctx()), rect, response, label, font, enabled, accent);
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn button(painter: &egui::Painter, p: &Paint, rect: Rect, response: &Response, label: &str, font: FontId, enabled: bool, accent: bool) {
    let (look, c, r) = (p.look, p.c, p.radius);
    let radius = if look == Look::Material { CornerRadius::same((rect.height() / 2.0) as u8) } else { CornerRadius::same(r) };
    let down = response.is_pointer_button_down_on();
    let hovered = response.hovered();
    let fill = match (enabled, accent) {
        (false, _) => c.button_disabled,
        (true, true) if down => c.accent.gamma_multiply(if look == Look::Windows { 0.8 } else { 0.85 }).to_opaque(),
        // macOS keeps its default button's fill under the pointer.
        (true, true) if hovered && look != Look::Mac => c.accent_hover,
        (true, true) => c.accent,
        (true, false) if down => c.button_pressed,
        (true, false) if hovered => c.button_hover,
        (true, false) => c.button,
    };
    if accent && look == Look::Windows {
        painter.rect_filled(rect, radius, fill);
        if enabled {
            let y = rect.bottom() - 0.5;
            let r = f32::from(radius.sw);
            painter.line_segment([egui::pos2(rect.left() + r, y), egui::pos2(rect.right() - r, y)], Stroke::new(1.0, Color32::from_black_alpha(60)));
        }
    } else {
        paint_body(painter, p, rect, radius, fill, enabled);
        if look == Look::Plain && enabled && !accent {
            painter.rect_stroke(rect, radius, Stroke::new(1.0, if hovered { c.grid } else { c.border }), egui::StrokeKind::Inside);
        }
    }
    if response.has_focus() {
        ring(painter, p, rect, radius);
    }
    // WinUI dims a pressed button's text rather than darkening its fill much.
    let color = if !enabled { c.disabled } else if accent { c.on_accent } else if down && look == Look::Windows { c.dim } else { c.fg };
    if !label.is_empty() {
        let galley = painter.layout(label.to_string(), font, color, rect.width() - 8.0);
        let top = galley_top(painter.ctx(), &galley, rect.center().y);
        painter.galley(egui::pos2(rect.center().x - galley.size().x / 2.0, top), galley, color);
    }
}

/// Paints a pop-up button's body and arrows in `rect`: on macOS the up and down chevrons
/// white in a square of the accent, elsewhere a chevron down. `open` while its list
/// shows. Returns the width the arrows take at the right end, for the label to keep clear
/// of.
pub fn paint_popup_button(painter: &egui::Painter, rect: Rect, response: &Response, enabled: bool, open: bool) -> f32 {
    popup_button(painter, &Paint::of(painter.ctx()), rect, response, enabled, open)
}

pub(crate) fn popup_button(painter: &egui::Painter, p: &Paint, rect: Rect, response: &Response, enabled: bool, open: bool) -> f32 {
    let (look, c, r) = (p.look, p.c, p.radius);
    let radius = CornerRadius::same(r);
    let fill = if !enabled { c.button_disabled } else if response.hovered() || open { c.button_hover } else { c.button };
    paint_body(painter, p, rect, radius, fill, enabled);
    if look == Look::Mac {
        let side = (rect.height() - 6.0).clamp(12.0, 16.0);
        let square = Rect::from_center_size(egui::pos2(rect.max.x - 4.0 - side / 2.0, rect.center().y), egui::vec2(side, side));
        painter.rect_filled(square, CornerRadius::same(4), if enabled { c.accent } else { c.accent.gamma_multiply(0.45) });
        let m = square.center();
        let stroke = Stroke::new(1.4, Color32::WHITE);
        painter.add(egui::Shape::line(vec![m + egui::vec2(-2.8, -1.5), m + egui::vec2(0.0, -4.2), m + egui::vec2(2.8, -1.5)], stroke));
        painter.add(egui::Shape::line(vec![m + egui::vec2(-2.8, 1.5), m + egui::vec2(0.0, 4.2), m + egui::vec2(2.8, 1.5)], stroke));
    } else {
        if look == Look::Plain {
            painter.rect_stroke(rect, radius, Stroke::new(1.0, if open { c.accent } else { c.border }), egui::StrokeKind::Inside);
        }
        let m = egui::pos2(rect.max.x - 13.0, rect.center().y);
        let chevron = vec![m + egui::vec2(-4.0, -2.0), m + egui::vec2(0.0, 2.0), m + egui::vec2(4.0, -2.0)];
        painter.add(egui::Shape::line(chevron, Stroke::new(1.5, if enabled { c.dim } else { c.disabled })));
    }
    if response.has_focus() && look.native() {
        ring(painter, p, rect, radius);
    }
    24.0
}

/// The underline along a text entry's bottom edge in WinUI and Material: the strong
/// stroke, or two points of the accent while it is typed in. Nothing in the other looks.
pub fn underline(painter: &egui::Painter, rect: Rect, focused: bool) {
    let p = Paint::of(painter.ctx());
    let (look, c, r) = (p.look, p.c, p.radius);
    if !matches!(look, Look::Windows | Look::Material) {
        return;
    }
    let (width, colour) = if focused { (2.0, c.accent) } else { (1.0, c.control_strong) };
    let y = rect.bottom() - width / 2.0;
    let r = f32::from(r);
    painter.line_segment([egui::pos2(rect.left() + r * 0.5, y), egui::pos2(rect.right() - r * 0.5, y)], Stroke::new(width, colour));
}
