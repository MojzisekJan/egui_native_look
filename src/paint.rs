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
        Paint {
            look: resolved.theme.look,
            c: resolved.colors(ctx),
            radius: resolved.theme.metrics.radius,
        }
    }
}

/// A raised control's body in `rect`, as macOS drew its controls before macOS 26 and iOS
/// draws a chosen segment: on the light scheme a soft shadow under it and a hairline round
/// it, on the dark a lighter top edge, then the fill.
pub fn paint_raised(
    painter: &egui::Painter,
    rect: Rect,
    radius: CornerRadius,
    fill: Color32,
    enabled: bool,
) {
    raised(
        painter,
        painter.ctx().theme() == egui::Theme::Dark,
        rect,
        radius,
        fill,
        enabled,
    );
}

pub(crate) fn raised(
    painter: &egui::Painter,
    dark: bool,
    rect: Rect,
    radius: CornerRadius,
    fill: Color32,
    enabled: bool,
) {
    if enabled && !dark {
        painter.rect_filled(
            rect.translate(egui::vec2(0.0, 0.5)).expand(0.5),
            radius,
            Color32::from_black_alpha(28),
        );
    }
    painter.rect_filled(rect, radius, fill);
    if dark {
        let top = Rect::from_min_max(
            rect.min + egui::vec2(radius.nw as f32, 0.0),
            egui::pos2(rect.max.x - radius.ne as f32, rect.min.y + 1.0),
        );
        painter.rect_filled(
            top,
            0.0,
            Color32::from_white_alpha(if enabled { 18 } else { 8 }),
        );
    } else {
        painter.rect_stroke(
            rect,
            radius,
            Stroke::new(
                1.0,
                Color32::from_black_alpha(if enabled { 30 } else { 16 }),
            ),
            egui::StrokeKind::Inside,
        );
    }
}

/// A WinUI control's body in `rect`: the fill, a hairline edge, and the darker bottom edge
/// that lifts it off the light scheme.
pub fn paint_fluent(
    painter: &egui::Painter,
    rect: Rect,
    radius: CornerRadius,
    fill: Color32,
    enabled: bool,
) {
    fluent(
        painter,
        &Paint::of(painter.ctx()).c,
        rect,
        radius,
        fill,
        enabled,
    );
}

pub(crate) fn fluent(
    painter: &egui::Painter,
    c: &Colors,
    rect: Rect,
    radius: CornerRadius,
    fill: Color32,
    enabled: bool,
) {
    painter.rect_filled(rect, radius, fill);
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(1.0, c.control_border),
        egui::StrokeKind::Inside,
    );
    if enabled && !c.dark {
        let r = f32::from(radius.sw.max(radius.se));
        let y = rect.bottom() - 0.5;
        painter.line_segment(
            [
                egui::pos2(rect.left() + r, y),
                egui::pos2(rect.right() - r, y),
            ],
            Stroke::new(1.0, Color32::from_black_alpha(42)),
        );
    }
}

/// The body of a control in the look: Fluent on Windows, flat on macOS, in libadwaita and
/// the plain look (the plain look adds its own edge).
pub(crate) fn paint_body(
    painter: &egui::Painter,
    p: &Paint,
    rect: Rect,
    radius: CornerRadius,
    fill: Color32,
    enabled: bool,
) {
    match p.look {
        Look::Windows => fluent(painter, &p.c, rect, radius, fill, enabled),
        Look::Mac | Look::Gtk | Look::Ios | Look::Material | Look::Plain => {
            painter.rect_filled(rect, radius, fill);
            // With more contrast every control takes an edge, as libadwaita's do.
            if p.c.high_contrast {
                painter.rect_stroke(
                    rect,
                    radius,
                    Stroke::new(1.0, p.c.control_border),
                    egui::StrokeKind::Inside,
                );
            }
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
    let grown = CornerRadius {
        nw: radius.nw + 2,
        ne: radius.ne + 2,
        sw: radius.sw + 2,
        se: radius.se + 2,
    };
    match look {
        Look::Gtk => {
            painter.rect_stroke(
                rect,
                radius,
                Stroke::new(2.0, c.accent.gamma_multiply(0.5)),
                egui::StrokeKind::Inside,
            );
        }
        Look::Material => {
            painter.rect_stroke(
                rect.expand(2.0),
                grown,
                Stroke::new(3.0, c.accent),
                egui::StrokeKind::Outside,
            );
        }
        Look::Ios => {
            painter.rect_stroke(
                rect.expand(0.5),
                grown,
                Stroke::new(3.0, c.accent.gamma_multiply(0.5)),
                egui::StrokeKind::Outside,
            );
        }
        Look::Windows => {
            painter.rect_stroke(
                rect.expand(1.0),
                grown,
                Stroke::new(2.0, c.fg),
                egui::StrokeKind::Outside,
            );
            painter.rect_stroke(
                rect,
                radius,
                Stroke::new(1.0, c.bg),
                egui::StrokeKind::Outside,
            );
        }
        // An inactive macOS window shows no focus.
        Look::Mac if !c.active => {}
        Look::Mac => {
            // Three points outside the control and its edge's point within, about half strength,
            // as macOS's keyboard focus indicator is.
            painter.rect_stroke(
                rect.shrink(2.0),
                radius,
                Stroke::new(
                    4.0,
                    mac_focus_colour(c.accent, c.dark).gamma_multiply(if c.dark {
                        0.55
                    } else {
                        0.5
                    }),
                ),
                egui::StrokeKind::Outside,
            );
        }
        Look::Plain => {
            painter.rect_stroke(
                rect.expand(1.0),
                radius,
                Stroke::new(1.0, c.accent_hover),
                egui::StrokeKind::Outside,
            );
        }
    }
}

/// macOS's keyboard focus colour from its accent: the hue turned toward violet and a
/// little darker on the light scheme, toward cyan on the dark, as measured from AppKit
/// with the blue accent.
fn mac_focus_colour(accent: Color32, dark: bool) -> Color32 {
    let [r, g, b] = [accent.r(), accent.g(), accent.b()].map(|v| f32::from(v) / 255.0);
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let span = max - min;
    let mut h = if span == 0.0 {
        0.0
    } else if max == r {
        ((g - b) / span).rem_euclid(6.0)
    } else if max == g {
        (b - r) / span + 2.0
    } else {
        (r - g) / span + 4.0
    } / 6.0;
    let (mut s, mut v) = (if max == 0.0 { 0.0 } else { span / max }, max);
    if dark {
        h -= 0.019;
        s *= 0.89;
    } else {
        h += 0.02;
        s *= 0.85;
        v *= 0.96;
    }
    let channel = |n: f32| {
        let k = (n + h.rem_euclid(1.0) * 6.0).rem_euclid(6.0);
        let value = v - v * s * k.min(4.0 - k).clamp(0.0, 1.0);
        (value * 255.0).round() as u8
    };
    Color32::from_rgb(channel(5.0), channel(3.0), channel(1.0))
}

/// Paints a push button in `rect` with `label` in `font`. `accent` is a toggle that is
/// switched on, or the default action of a dialog.
pub fn paint_button(
    painter: &egui::Painter,
    rect: Rect,
    response: &Response,
    label: &str,
    font: FontId,
    enabled: bool,
    accent: bool,
) {
    button(
        painter,
        &Paint::of(painter.ctx()),
        rect,
        response,
        label,
        font,
        enabled,
        accent,
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn button(
    painter: &egui::Painter,
    p: &Paint,
    rect: Rect,
    response: &Response,
    label: &str,
    font: FontId,
    enabled: bool,
    accent: bool,
) {
    let (look, c, r) = (p.look, p.c, p.radius);
    // An inactive macOS window shows its default button as an ordinary one.
    let accent = accent && (look != Look::Mac || c.active);
    // Material and iOS 26 draw their buttons as capsules.
    let radius = if matches!(look, Look::Material | Look::Ios) {
        CornerRadius::same((rect.height() / 2.0) as u8)
    } else {
        CornerRadius::same(r)
    };
    let down = response.is_pointer_button_down_on();
    let hovered = response.hovered();
    let mut fill = match (enabled, accent) {
        (false, true) if look == Look::Gtk => c.accent,
        (false, _) => c.button_disabled,
        (true, true) if down && look == Look::Mac => crate::mac_pressed_accent(&c),
        // Material's pressed state layer: 10 % of the primary's text colour.
        (true, true) if down && look == Look::Material => {
            crate::lerp_color(c.accent, c.on_accent, 0.1)
        }
        (true, true) if down && look == Look::Gtk => {
            crate::lerp_color(c.accent, Color32::BLACK, 0.2)
        }
        (true, true) if down => c
            .accent
            .gamma_multiply(if look == Look::Windows { 0.8 } else { 0.85 })
            .to_opaque(),
        // macOS keeps its default button's fill under the pointer.
        (true, true) if hovered && look != Look::Mac => c.accent_hover,
        (true, true) => c.accent,
        (true, false) if down => c.button_pressed,
        (true, false) if hovered => c.button_hover,
        (true, false) => c.button,
    };
    if look == Look::Gtk && enabled {
        let fills = if accent {
            [
                c.accent,
                c.accent_hover,
                crate::lerp_color(c.accent, Color32::BLACK, 0.2),
            ]
        } else {
            [c.button, c.button_hover, c.button_pressed]
        };
        fill = gtk_state_fill(painter, response, fills);
    }
    if accent && look == Look::Windows {
        painter.rect_filled(rect, radius, fill);
        if enabled {
            let y = rect.bottom() - 0.5;
            let r = f32::from(radius.sw);
            painter.line_segment(
                [
                    egui::pos2(rect.left() + r, y),
                    egui::pos2(rect.right() - r, y),
                ],
                Stroke::new(1.0, Color32::from_black_alpha(60)),
            );
        }
    } else {
        paint_body(painter, p, rect, radius, fill, enabled);
        if look == Look::Plain && enabled && !accent {
            painter.rect_stroke(
                rect,
                radius,
                Stroke::new(1.0, if hovered { c.grid } else { c.border }),
                egui::StrokeKind::Inside,
            );
        }
    }
    if response.has_focus() {
        ring(painter, p, rect, radius);
    }
    // WinUI dims a pressed button's text rather than darkening its fill much.
    // iOS sets an ordinary (bordered) button's text in the accent.
    let color = if !enabled && look != Look::Gtk {
        c.disabled
    } else if accent {
        c.on_accent
    } else if look == Look::Ios {
        c.accent
    } else if down && look == Look::Windows {
        c.dim
    } else {
        c.fg
    };
    if !label.is_empty() {
        let galley = painter.layout(label.to_string(), font, color, rect.width() - 8.0);
        let top = galley_top(painter.ctx(), &galley, rect.center().y);
        painter.galley(
            egui::pos2(rect.center().x - galley.size().x / 2.0, top),
            galley,
            color,
        );
    }
}

/// Paints a pop-up button's body and arrows in `rect`: on macOS small up and down
/// chevrons, on iOS the same in the accent without a body, elsewhere a chevron down. `open` while its list
/// shows. Returns the width the arrows take at the right end, for the label to keep clear
/// of.
pub fn paint_popup_button(
    painter: &egui::Painter,
    rect: Rect,
    response: &Response,
    enabled: bool,
    open: bool,
) -> f32 {
    popup_button(
        painter,
        &Paint::of(painter.ctx()),
        rect,
        response,
        enabled,
        open,
    )
}

pub(crate) fn popup_button(
    painter: &egui::Painter,
    p: &Paint,
    rect: Rect,
    response: &Response,
    enabled: bool,
    open: bool,
) -> f32 {
    let (look, c, r) = (p.look, p.c, p.radius);
    let radius = CornerRadius::same(r);
    let fill = if !enabled {
        c.button_disabled
    } else if response.hovered() || open {
        c.button_hover
    } else {
        c.button
    };
    if look == Look::Material {
        // Material 3's exposed dropdown: a filled field rounded above its indicator line,
        // with a small triangle.
        let round = CornerRadius {
            nw: 4,
            ne: 4,
            sw: 0,
            se: 0,
        };
        painter.rect_filled(
            rect,
            round,
            if enabled { c.entry } else { c.button_disabled },
        );
        let active = open || response.has_focus();
        let (width, line) = match (enabled, active) {
            (false, _) => (1.0, c.disabled),
            (true, true) => (2.0, c.accent),
            (true, false) => (1.0, c.dim),
        };
        let y = rect.bottom() - width / 2.0;
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            Stroke::new(width, line),
        );
        let m = egui::pos2(rect.right() - 24.0, rect.center().y);
        let (up, down) = (-2.5, 2.5);
        let tip = if open { up } else { down };
        let base = if open { down } else { up };
        painter.add(egui::Shape::convex_polygon(
            vec![
                m + egui::vec2(-5.0, base),
                m + egui::vec2(5.0, base),
                m + egui::vec2(0.0, tip),
            ],
            if enabled { c.dim } else { c.disabled },
            Stroke::NONE,
        ));
        return 52.0 - 16.0;
    }
    // iOS's menu picker has no body: its value and chevrons in the accent.
    if look != Look::Ios {
        paint_body(painter, p, rect, radius, fill, enabled);
    }
    if matches!(look, Look::Mac | Look::Ios) {
        let m = egui::pos2(rect.max.x - 13.0, rect.center().y);
        let ink = match (enabled, look) {
            (false, _) => c.disabled,
            (true, Look::Ios) => c.accent,
            (true, _) => c.fg,
        };
        let stroke = Stroke::new(1.3, ink);
        for sign in [-1.0, 1.0] {
            painter.add(egui::Shape::line(
                vec![
                    m + egui::vec2(-3.0, sign * 1.5),
                    m + egui::vec2(0.0, sign * 4.5),
                    m + egui::vec2(3.0, sign * 1.5),
                ],
                stroke,
            ));
        }
    } else {
        if look == Look::Plain {
            painter.rect_stroke(
                rect,
                radius,
                Stroke::new(1.0, if open { c.accent } else { c.border }),
                egui::StrokeKind::Inside,
            );
        }
        let m = egui::pos2(rect.max.x - 13.0, rect.center().y);
        let chevron = vec![
            m + egui::vec2(-4.0, -2.0),
            m + egui::vec2(0.0, 2.0),
            m + egui::vec2(4.0, -2.0),
        ];
        // GTK's pan-down arrow is in the text colour, heavier than the others' chevron.
        let (width, ink) = match (enabled, look) {
            (false, _) => (1.5, c.disabled),
            (true, Look::Gtk) => (2.0, c.fg),
            (true, _) => (1.5, c.dim),
        };
        if look == Look::Gtk {
            painter.add(egui::Shape::convex_polygon(chevron, ink, Stroke::NONE));
        } else {
            painter.add(egui::Shape::line(chevron, Stroke::new(width, ink)));
        }
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
    // Material's indicator is the variant text colour, WinUI's the strong stroke.
    let (width, colour) = match (focused, look) {
        (true, _) => (2.0, c.accent),
        (false, Look::Material) => (1.0, c.dim),
        (false, _) => (1.0, c.control_strong),
    };
    let y = rect.bottom() - width / 2.0;
    // Material's filled field is square below, its line full width.
    let r = if look == Look::Material {
        0.0
    } else {
        f32::from(r)
    };
    painter.line_segment(
        [
            egui::pos2(rect.left() + r * 0.5, y),
            egui::pos2(rect.right() - r * 0.5, y),
        ],
        Stroke::new(width, colour),
    );
}

/// State transitions use the toolkit's 200 ms ease-out curve and honor disabled animations.
pub(crate) fn gtk_state_fill(
    painter: &egui::Painter,
    response: &Response,
    fills: [Color32; 3],
) -> Color32 {
    let ctx = painter.ctx();
    let duration = ctx.global_style().animation_time.min(0.2);
    let hover = ctx.animate_bool_with_time_and_easing(
        response.id.with("gtk-hover"),
        response.hovered(),
        duration,
        gtk_easing,
    );
    let press = ctx.animate_bool_with_time_and_easing(
        response.id.with("gtk-press"),
        response.is_pointer_button_down_on(),
        duration,
        gtk_easing,
    );
    crate::lerp_color(
        crate::lerp_color(fills[0], fills[1], hover),
        fills[2],
        press,
    )
}

fn gtk_easing(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t == 0.0 || t == 1.0 {
        return t;
    }
    let bezier = |u: f32, a: f32, b: f32| {
        3.0 * (1.0 - u).powi(2) * u * a + 3.0 * (1.0 - u) * u * u * b + u * u * u
    };
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..12 {
        let mid = (lo + hi) * 0.5;
        if bezier(mid, 0.25, 0.45) < t {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    bezier((lo + hi) * 0.5, 0.46, 0.94)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mac_focus_colour_matches_appkit_for_the_blue_accent() {
        // Measured from AppKit's focus ring over white and over the dark window, with the
        // blue accent.
        let close = |a: Color32, [r, g, b]: [u8; 3]| {
            a.r().abs_diff(r) <= 3 && a.g().abs_diff(g) <= 3 && a.b().abs_diff(b) <= 3
        };
        let blue = Color32::from_rgb(0x00, 0x7a, 0xff);
        assert!(close(mac_focus_colour(blue, false), [37, 111, 245]));
        assert!(close(mac_focus_colour(blue, true), [28, 166, 255]));
    }
}
