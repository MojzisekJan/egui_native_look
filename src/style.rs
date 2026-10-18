//! egui's own widgets (text edits, drag values, selectable rows, combo boxes) in the look.

use egui::{CornerRadius, Stroke};

use crate::{Colors, Look, Theme};

/// Sets `style` for `theme` in the dark scheme or the light one: egui's own widgets take
/// the look's fills, edges, rounding and accent, so they sit beside this crate's controls.
/// [`crate::install`] calls it for both of egui's schemes. What it does not set stays as
/// the style had it: the app's shadows, spacing (bar the controls' height), heading and
/// other text styles, so an app may set them before or after. The body and button text
/// styles take the theme's [`crate::Metrics::body`].
pub fn apply_style(style: &mut egui::Style, theme: &Theme, dark: bool) {
    let c = Colors::resolve(theme, dark);
    let look = theme.look;
    let visuals = &mut style.visuals;
    visuals.dark_mode = dark;
    visuals.panel_fill = c.bg;
    visuals.window_fill = c.surface;
    visuals.extreme_bg_color = c.entry;
    visuals.weak_text_color = Some(c.dim);
    visuals.hyperlink_color = c.accent;
    visuals.selection.bg_fill = c.accent.gamma_multiply(if dark { 0.55 } else { 0.35 });
    visuals.selection.stroke = if look == Look::Mac { Stroke::new(2.0, c.accent.gamma_multiply(0.7)) } else { Stroke::new(1.0, c.accent) };
    let rounding = CornerRadius::same(theme.metrics.radius);
    visuals.window_corner_radius = CornerRadius::same(match look {
        Look::Mac => 10,
        Look::Windows | Look::Gtk => 8,
        Look::Ios => 14,
        Look::Material => 28,
        Look::Plain => 6,
    });
    visuals.menu_corner_radius = CornerRadius::same(if look.native() { 8 } else { 6 });
    visuals.window_stroke = Stroke::new(1.0, c.border);
    let edge = match look {
        // libadwaita draws no edge round its controls; the other system looks keep theirs
        // still under the pointer.
        Look::Gtk | Look::Ios | Look::Material => Stroke::NONE,
        Look::Mac | Look::Windows => Stroke::new(1.0, c.control_border),
        Look::Plain => Stroke::new(1.0, c.border),
    };
    let w = &mut visuals.widgets;
    w.noninteractive.bg_stroke = Stroke::new(1.0, c.border);
    w.noninteractive.fg_stroke = Stroke::new(1.0, c.fg);
    w.noninteractive.corner_radius = rounding;
    for (state, fill) in [(&mut w.inactive, c.button), (&mut w.hovered, c.button_hover), (&mut w.active, c.button_pressed), (&mut w.open, c.button_hover)] {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.corner_radius = rounding;
        state.expansion = 0.0;
        state.fg_stroke = Stroke::new(1.0, c.fg);
        state.bg_stroke = edge;
    }
    match look {
        Look::Plain => {
            w.hovered.bg_stroke = Stroke::new(1.0, c.grid);
            w.active.bg_stroke = Stroke::new(1.0, c.accent);
        }
        Look::Mac => w.active.bg_stroke = Stroke::new(1.0, c.accent),
        Look::Windows | Look::Gtk | Look::Ios | Look::Material => {}
    }
    style.spacing.interact_size.y = theme.metrics.control_height;
    style.text_styles.insert(egui::TextStyle::Button, theme.metrics.body.clone());
    style.text_styles.insert(egui::TextStyle::Body, theme.metrics.body.clone());
}
