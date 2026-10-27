//! The colours of the controls, resolved for a look and a scheme.

use egui::Color32;

use crate::{lerp_color, lighten, Look, Theme};

pub(crate) fn rgb(c: u32) -> Color32 {
    Color32::from_rgb((c >> 16) as u8, (c >> 8) as u8, c as u8)
}

/// The colours the controls are drawn in, for one look and scheme. Made by
/// [`Colors::resolve`] or [`colors`](crate::colors); more may be added.
#[derive(Clone, Copy, Debug, PartialEq)]
///
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::{Colors, Theme};
/// Colors::resolve(&Theme::default(), false);
/// ```
#[must_use]
#[non_exhaustive]
pub struct Colors {
    /// Whether these are the dark scheme's.
    pub dark: bool,
    /// Text.
    pub fg: Color32,
    /// Secondary text: hints, shortcuts, glyphs at rest.
    pub dim: Color32,
    /// Text and glyphs of a disabled control.
    pub disabled: Color32,
    /// The window's background.
    pub bg: Color32,
    /// Floating windows, dialogs, menus and pop-ups.
    pub surface: Color32,
    /// Hairline borders round cards and the plain look's controls.
    pub border: Color32,
    /// The plain look's hovered edge.
    pub grid: Color32,
    /// A push button's fill.
    pub button: Color32,
    /// A push button's fill under the pointer.
    pub button_hover: Color32,
    /// A push button's fill while pressed.
    pub button_pressed: Color32,
    /// A disabled push button's fill.
    pub button_disabled: Color32,
    /// A text entry's fill.
    pub entry: Color32,
    /// An entry's fill while typed in: WinUI's turns solid.
    pub entry_focused: Color32,
    /// The edge of a button, an entry or a pop-up button.
    pub control_border: Color32,
    /// WinUI's strong stroke, libadwaita's outline: an entry's underline, an empty mark.
    pub control_strong: Color32,
    /// A switched-off switch's track, a segmented control's track, a slider's rail.
    pub track: Color32,
    /// A segmented control's chosen segment.
    pub raised: Color32,
    /// The accent: default buttons, checked marks, selection, focus.
    pub accent: Color32,
    /// The accent under the pointer.
    pub accent_hover: Color32,
    /// Text and glyphs on an accent fill.
    pub on_accent: Color32,
    /// A list row or menu item under the pointer.
    pub row_hover: Color32,
    /// Whether the window is active. macOS draws an inactive window's accent in grey,
    /// which these colours already carry in the macOS look.
    pub active: bool,
    /// Whether the theme asks for more contrast; the controls then draw an edge in
    /// `control_border`.
    pub high_contrast: bool,
}

/// `(dark, light)` for Windows, GTK, iOS and Material.
type Shades = [(u32, u32); 4];

impl Colors {
    /// The colours of `theme` in the dark scheme or the light one.
    pub fn resolve(theme: &Theme, dark: bool) -> Colors {
        let p = if dark {
            theme.schemes.dark
        } else {
            theme.schemes.light
        };
        let look = theme.look;
        let by = |shades: Shades, mac: Color32, plain: Color32| -> Color32 {
            let pick = |(d, l): (u32, u32)| rgb(if dark { d } else { l });
            match look {
                Look::Mac => mac,
                Look::Windows => pick(shades[0]),
                Look::Gtk => pick(shades[1]),
                Look::Ios => pick(shades[2]),
                Look::Material => pick(shades[3]),
                Look::Plain => plain,
            }
        };
        // macOS 26 fills its controls with the text colour's ink at a low opacity, so they
        // show on any window: `(dark, light)` opacities of white and of black.
        let ink = |(d, l): (f32, f32)| {
            if dark {
                crate::lerp_color(p.bg, Color32::WHITE, d)
            } else {
                crate::lerp_color(p.bg, Color32::BLACK, l)
            }
        };
        let system_accent = theme.accent.map(|[d, l]| rgb(if dark { d } else { l }));
        let accent = match look {
            // WinUI's default blue, Light 2 on the dark scheme and Dark 1 on the light.
            Look::Windows => system_accent.unwrap_or(rgb(if dark { 0x60cdff } else { 0x005fb8 })),
            Look::Gtk => system_accent.unwrap_or(p.accent),
            // iOS 26's systemBlue; Material 3's baseline primary.
            Look::Ios => system_accent.unwrap_or(rgb(if dark { 0x0091ff } else { 0x0088ff })),
            Look::Material => system_accent.unwrap_or(rgb(if dark { 0xd0bcff } else { 0x6750a4 })),
            Look::Mac => system_accent.unwrap_or(p.accent),
            Look::Plain => p.accent,
        };
        let accent_hover = match look {
            // WinUI shows the accent at 90 % under the pointer.
            Look::Windows => accent.gamma_multiply(0.9).to_opaque(),
            Look::Gtk | Look::Ios => lighten(accent, 0.1),
            // Material lays its 8 % state layer in the primary's text colour.
            Look::Material => lerp_color(accent, on(accent, look), 0.08),
            Look::Mac if system_accent.is_some() => lighten(accent, 0.15),
            _ => p.accent_hover,
        };
        let colors = Colors {
            dark,
            fg: p.fg,
            dim: p.dim,
            disabled: p.disabled,
            bg: p.bg,
            surface: p.surface,
            border: p.border,
            grid: p.grid,
            // libadwaita's buttons and entries are the text colour at 10 % (15 % hovered, 30 %
            // pressed) over the window.
            // iOS: the system fills (tertiarySystemFill and friends); Material: the tonal
            // container (secondaryContainer) and the highest surface container.
            button: by(
                [
                    (0x2f3136, 0xfbfbfb),
                    (0x323336, 0xdadbde),
                    (0x262629, 0xe9e9eb),
                    (0x4a4458, 0xe8def8),
                ],
                ink((0.09, 0.063)),
                p.button,
            ),
            button_hover: by(
                [
                    (0x36383d, 0xf5f5f5),
                    (0x3d3e41, 0xd0d1d4),
                    (0x2f2f33, 0xe1e1e4),
                    (0x564f66, 0xddd2ee),
                ],
                ink((0.11, 0.085)),
                p.button_hover,
            ),
            button_pressed: by(
                [
                    (0x2a2c30, 0xf3f3f3),
                    (0x5f6063, 0xb0b1b4),
                    (0x3a3a3e, 0xd6d6da),
                    (0x625a74, 0xd2c6e4),
                ],
                ink((0.2, 0.15)),
                p.button_pressed,
            ),
            button_disabled: by(
                [
                    (0x2a2c30, 0xf5f5f5),
                    (0x26272a, 0xe3e4e7),
                    (0x262629, 0xe9e9eb),
                    (0x2b292f, 0xeae6ee),
                ],
                ink((0.045, 0.035)),
                p.button_disabled,
            ),
            entry: by(
                [
                    (0x2f3136, 0xfbfbfb),
                    (0x323336, 0xdadbde),
                    (0x000000, 0xffffff),
                    (0x36343b, 0xe6e0e9),
                ],
                rgb(if dark { 0x181818 } else { 0xffffff }),
                p.entry,
            ),
            entry_focused: by(
                [
                    (0x1d1e21, 0xffffff),
                    (0x323336, 0xdadbde),
                    (0x000000, 0xffffff),
                    (0x36343b, 0xe6e0e9),
                ],
                rgb(if dark { 0x181818 } else { 0xffffff }),
                p.entry,
            ),
            control_border: by(
                [
                    (0x3a3c41, 0xe5e5e5),
                    (0x3d3e41, 0xd0d1d4),
                    (0x333333, 0xcccccc),
                    (0x49454f, 0xcac4d0),
                ],
                ink((0.058, 0.067)),
                p.border,
            ),
            control_strong: by(
                [
                    (0x9a9b9e, 0x868686),
                    (0x5f6063, 0xb0b1b4),
                    (0x636366, 0xc7c7cc),
                    (0x938f99, 0x79747e),
                ],
                rgb(if dark { 0x8b9199 } else { 0x5e646c }),
                p.dim,
            ),
            track: by(
                [
                    (0x26282c, 0xe1e1e3),
                    (0x3d3e41, 0xd0d1d4),
                    (0x2b2b2e, 0xe5e5e6),
                    (0x36343b, 0xe6e0e9),
                ],
                ink((0.117, 0.09)),
                p.button,
            ),
            raised: if dark { rgb(0x5b5e65) } else { Color32::WHITE },
            accent,
            accent_hover,
            on_accent: on(accent, look),
            row_hover: p.row_hover,
            active: true,
            high_contrast: false,
        };
        let colors = match look {
            Look::Ios => colors.ios_fills(),
            Look::Gtk => colors.gtk_fills(),
            // Material 3's disabled roles: content at 38 % of the text colour, containers
            // at 12 %.
            Look::Material => Colors {
                disabled: lerp_color(colors.bg, colors.fg, 0.38),
                button_disabled: lerp_color(colors.bg, colors.fg, 0.12),
                ..colors
            },
            _ => colors,
        };
        if theme.high_contrast {
            colors.more_contrast()
        } else {
            colors
        }
    }
}

/// Text on `accent`: white on a dark accent, and on a light one (WinUI's dark-scheme
/// accent, Material's, a yellow) black, or Material's onPrimary.
fn on(accent: Color32, look: Look) -> Color32 {
    let linear = |v: u8| {
        let v = f32::from(v) / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    let luminance =
        0.2126 * linear(accent.r()) + 0.7152 * linear(accent.g()) + 0.0722 * linear(accent.b());
    // Lower than the even-contrast point (0.18), so the systems' blues keep white text as
    // they do, while a yellow or a pale accent takes dark text.
    if luminance < 0.28 {
        Color32::WHITE
    } else if look == Look::Material {
        rgb(0x381e72)
    } else {
        Color32::BLACK
    }
}

impl Colors {
    /// More contrast, after libadwaita's high-contrast values: separators and control
    /// edges at half the text colour, troughs at 30 %, secondary text at 90 %, and controls
    /// outlined.
    fn more_contrast(mut self) -> Colors {
        let fg = self.fg;
        let mix = |share: f32| {
            Color32::from_rgba_unmultiplied(fg.r(), fg.g(), fg.b(), (share * 255.0).round() as u8)
        };
        self.high_contrast = true;
        self.dim = lerp_color(self.bg, fg, 0.9);
        self.border = lerp_color(self.bg, fg, 0.5);
        self.grid = self.border;
        self.control_border = mix(0.5);
        self.control_strong = lerp_color(self.bg, fg, 0.8);
        self.track = mix(0.3);
        self
    }

    /// libadwaita's fills: the text colour mixed with transparency, so they read the same
    /// over any surface (buttons and entries 10 %, hovered 15 %, pressed 30 %; troughs 15 %).
    fn gtk_fills(mut self) -> Colors {
        let fg = self.fg;
        let mix = |share: f32| {
            Color32::from_rgba_unmultiplied(fg.r(), fg.g(), fg.b(), (share * 255.0).round() as u8)
        };
        self.button = mix(0.1);
        self.button_hover = mix(0.15);
        self.button_pressed = mix(0.3);
        self.button_disabled = self.button;
        self.entry = mix(0.1);
        self.entry_focused = mix(0.1);
        self.track = mix(0.15);
        self
    }

    /// iOS 26's fills, translucent over the window as its system fills are, measured on
    /// white and black: `(dark, light)` opacities of white and black.
    fn ios_fills(mut self) -> Colors {
        let base = self;
        let ink = |(d, l): (f32, f32)| crate::ink(&base, d, l);
        self.button = ink((0.15, 0.086));
        self.button_hover = ink((0.18, 0.11));
        self.button_pressed = ink((0.23, 0.16));
        self.button_disabled = self.button;
        self.track = ink((0.17, 0.1));
        self.entry = self.bg;
        self.entry_focused = self.bg;
        self.control_border = ink((0.2, 0.2));
        // The tertiary label, for disabled text and placeholders.
        self.disabled = lerp_color(self.bg, self.fg, if self.dark { 0.345 } else { 0.27 });
        self
    }

    /// These colours as an inactive window shows them: in the macOS look the accent
    /// turns a grey with the text colour on it; the other looks keep theirs.
    pub(crate) fn inactive(mut self, look: Look) -> Colors {
        self.active = false;
        if look == Look::Mac {
            let ink = if self.dark {
                Color32::WHITE
            } else {
                Color32::BLACK
            };
            self.accent = crate::lerp_color(self.bg, ink, if self.dark { 0.17 } else { 0.12 });
            self.accent_hover = self.accent;
            self.on_accent = self.fg;
        }
        self
    }
}

/// The colours of the theme installed in `ctx`, in egui's scheme of the moment.
pub fn colors(ctx: &egui::Context) -> Colors {
    crate::resolved(ctx).colors(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gtk_palette_accent_is_used_unless_explicitly_overridden() {
        let mut theme = Theme::new(Look::Gtk).accent(None);
        theme.schemes.dark.accent = rgb(0x1f9ede);
        theme.schemes.light.accent = rgb(0x007755);
        for dark in [false, true] {
            let expected = if dark { rgb(0x1f9ede) } else { rgb(0x007755) };
            let colors = Colors::resolve(&theme, dark);
            assert_eq!(colors.accent, expected);
            assert_eq!(colors.accent_hover, lighten(expected, 0.1));
        }
        let theme = theme.accent(Some([0xaa2233, 0x3344bb]));
        assert_eq!(Colors::resolve(&theme, true).accent, rgb(0xaa2233));
        assert_eq!(Colors::resolve(&theme, false).accent, rgb(0x3344bb));
    }
}
