//! egui's own widgets (text edits, drag values, selectable rows, combo boxes) in the look.

use egui::{CornerRadius, Stroke};

use crate::{Colors, Look, Theme};

/// Sets `style` for `theme` in the dark scheme or the light one: egui's own widgets take
/// the look's fills, edges, rounding and accent, so they sit beside this crate's controls.
/// [`crate::install`] calls it for both of egui's schemes. What it does not set stays as
/// the style had it: the app's shadows, spacing (bar the controls' height), heading and
/// other text styles, so an app may set them before or after. The body and button text
/// styles take the theme's [`crate::Metrics::body`]. Popup shadows, disabled opacity,
/// caret stroke, menu margins and scrollbar metrics reset before applying the look.
/// [`crate::install`] additionally restores app overrides when a look stops owning them.
pub fn apply_style(style: &mut egui::Style, theme: &Theme, dark: bool) {
    // Set every look-owned property on every call, so the result does not depend on
    // which look was applied before. Context installation preserves app overrides.
    let defaults = egui::Style::default();
    style.visuals.disabled_alpha = defaults.visuals.disabled_alpha;
    style.visuals.text_cursor.stroke = defaults.visuals.text_cursor.stroke;
    style.visuals.popup_shadow = defaults.visuals.popup_shadow;
    style.spacing.menu_margin = defaults.spacing.menu_margin;
    style.spacing.scroll = defaults.spacing.scroll;
    let metrics = theme.metrics.clone().sanitized(theme.look);
    let c = Colors::resolve(theme, dark);
    let look = theme.look;
    let visuals = &mut style.visuals;
    visuals.dark_mode = dark;
    visuals.panel_fill = c.bg;
    visuals.window_fill = c.surface;
    visuals.extreme_bg_color = c.entry;
    visuals.weak_text_color = Some(c.dim);
    // The caret: the accent on Apple's systems and in Material, the text colour in WinUI
    // and GTK.
    visuals.text_cursor.stroke = match look {
        Look::Mac | Look::Ios | Look::Material => Stroke::new(2.0, c.accent),
        Look::Windows | Look::Gtk => Stroke::new(1.0, c.fg),
        Look::Plain => visuals.text_cursor.stroke,
    };
    // Apple's disabled text is the tertiary label, about a third of the label's ink; egui's
    // own disabled widgets fade to it. This crate's controls carry their own disabled colours.
    if matches!(look, Look::Mac | Look::Ios) {
        visuals.disabled_alpha = 0.32;
    }
    // Material 3 shows disabled content at 38 % of the text colour.
    if look == Look::Material {
        visuals.disabled_alpha = 0.38;
    }
    if look == Look::Gtk {
        visuals.disabled_alpha = 0.5;
    }
    visuals.hyperlink_color = c.accent;
    visuals.selection.bg_fill = c.accent.gamma_multiply(if dark { 0.55 } else { 0.35 });
    visuals.selection.stroke = if look == Look::Mac {
        Stroke::new(2.0, c.accent.gamma_multiply(0.7))
    } else {
        Stroke::new(1.0, c.accent)
    };
    let rounding = CornerRadius::same(metrics.radius);
    visuals.window_corner_radius = CornerRadius::same(match look {
        Look::Mac => 10,
        Look::Windows | Look::Gtk => 8,
        Look::Ios => 14,
        Look::Material => 28,
        Look::Plain => 6,
    });
    visuals.menu_corner_radius = CornerRadius::same(match look {
        Look::Mac => 5,
        Look::Material => 4,
        // libadwaita's popovers: its menus' radius and six more.
        Look::Gtk => 12,
        Look::Plain => 6,
        _ => 8,
    });
    // Material's surfaces stand out by their tone and shadow, without an edge.
    visuals.window_stroke = if look == Look::Material {
        Stroke::NONE
    } else {
        Stroke::new(1.0, c.border)
    };
    if look == Look::Mac {
        // egui frames its tooltips (and any pop-up without a style of its own) with these:
        // macOS's tool tip, a light grey box with a hairline edge. This crate's menus
        // restore macOS's menu with `mac_menu_style`; its windows draw their own frames.
        visuals.window_fill = crate::colors::rgb(if dark { 0x2b2b2b } else { 0xe7e7e7 });
        visuals.popup_shadow = egui::Shadow {
            offset: [0, 2],
            blur: 6,
            spread: 0,
            color: egui::Color32::from_black_alpha(if dark { 70 } else { 30 }),
        };
        visuals.window_stroke = Stroke::new(
            1.0,
            egui::Color32::from_black_alpha(if dark { 230 } else { 56 }),
        );
    }
    let edge = match look {
        // macOS 26 and libadwaita draw no edge round their controls; WinUI keeps its own
        // still under the pointer.
        Look::Mac | Look::Gtk | Look::Ios | Look::Material => Stroke::NONE,
        Look::Windows => Stroke::new(1.0, c.control_border),
        Look::Plain => Stroke::new(1.0, c.border),
    };
    let w = &mut visuals.widgets;
    w.noninteractive.bg_stroke = Stroke::new(1.0, c.border);
    w.noninteractive.fg_stroke = Stroke::new(1.0, c.fg);
    w.noninteractive.corner_radius = rounding;
    for (state, fill) in [
        (&mut w.inactive, c.button),
        (&mut w.hovered, c.button_hover),
        (&mut w.active, c.button_pressed),
        (&mut w.open, c.button_hover),
    ] {
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
    style.spacing.interact_size.y = metrics.control_height;
    match look {
        Look::Mac => style.spacing.menu_margin = egui::Margin::symmetric(8, 4),
        // Material's menus run their items edge to edge, eight points from top and bottom.
        Look::Material => style.spacing.menu_margin = egui::Margin::symmetric(0, 8),
        // WinUI's flyouts: items four points in, the list two points from top and bottom.
        Look::Windows => style.spacing.menu_margin = egui::Margin::same(4),
        // libadwaita's leave six points round them, on a light shadow.
        Look::Gtk => {
            style.spacing.menu_margin = egui::Margin::same(6);
            style.visuals.popup_shadow = egui::Shadow {
                offset: [0, 1],
                blur: 5,
                spread: 1,
                color: egui::Color32::from_black_alpha(23),
            };
        }
        _ => {}
    }
    if look == Look::Gtk {
        style.spacing.scroll = egui::style::ScrollStyle {
            floating: true,
            bar_width: 8.0,
            floating_width: 3.0,
            floating_allocated_width: 0.0,
            handle_min_length: 40.0,
            bar_inner_margin: 3.0,
            bar_outer_margin: 3.0,
            foreground_color: true,
            dormant_background_opacity: 0.0,
            active_background_opacity: 0.0,
            interact_background_opacity: 0.1,
            dormant_handle_opacity: 0.0,
            active_handle_opacity: 0.2,
            interact_handle_opacity: 0.4,
            ..style.spacing.scroll
        };
    }
    if look == Look::Mac {
        // macOS's overlay scrollers: a six-point rounded handle three points in from the
        // edge, in the text colour at about half strength, hidden while the pointer is
        // elsewhere and eleven points wide with a faint track under the pointer.
        let handle = if dark { 0.7 } else { 0.53 };
        style.spacing.scroll = egui::style::ScrollStyle {
            floating: true,
            bar_width: 11.0,
            floating_width: 6.0,
            floating_allocated_width: 0.0,
            handle_min_length: 20.0,
            bar_inner_margin: 3.0,
            bar_outer_margin: 3.0,
            foreground_color: true,
            dormant_background_opacity: 0.0,
            active_background_opacity: 0.0,
            interact_background_opacity: 0.1,
            dormant_handle_opacity: 0.0,
            active_handle_opacity: handle,
            interact_handle_opacity: handle,
            ..style.spacing.scroll
        };
    }
    style
        .text_styles
        .insert(egui::TextStyle::Button, metrics.body.clone());
    style
        .text_styles
        .insert(egui::TextStyle::Body, metrics.body.clone());
}

/// macOS's menu, for a pop-up menu in the macOS look, over the tooltip the style frames
/// pop-ups as: near-white or mid-grey, an edge, twelve-point corners, and egui's own menu
/// adjustments.
pub(crate) fn mac_menu_style() -> egui::style::StyleModifier {
    egui::style::StyleModifier::new(|style: &mut egui::Style| {
        egui::containers::menu::menu_style(style);
        let dark = style.visuals.dark_mode;
        style.visuals.window_fill = crate::mac_menu_fill(dark);
        style.visuals.window_stroke = Stroke::new(
            1.0,
            egui::Color32::from_black_alpha(if dark { 220 } else { 40 }),
        );
        style.visuals.menu_corner_radius = CornerRadius::same(12);
        style.visuals.popup_shadow = egui::Shadow {
            offset: [0, 6],
            blur: 20,
            spread: 0,
            color: egui::Color32::from_black_alpha(if dark { 110 } else { 45 }),
        };
        style.spacing.menu_margin = egui::Margin::same(6);
    })
}
