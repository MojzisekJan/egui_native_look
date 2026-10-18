//! The colours of the controls, resolved for a look and a scheme.

use egui::Color32;

use crate::{lighten, Look, Theme};

pub(crate) fn rgb(c: u32) -> Color32 {
    Color32::from_rgb((c >> 16) as u8, (c >> 8) as u8, c as u8)
}

/// The colours the controls are drawn in, for one look and scheme.
#[derive(Clone, Copy, Debug, PartialEq)]
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
}

/// `(dark, light)` for macOS, Windows, GTK, iOS and Material.
type Shades = [(u32, u32); 5];

impl Colors {
    /// The colours of `theme` in the dark scheme or the light one.
    pub fn resolve(theme: &Theme, dark: bool) -> Colors {
        let p = if dark { theme.schemes.dark } else { theme.schemes.light };
        let look = theme.look;
        let by = |shades: Shades, plain: Color32| -> Color32 {
            let pick = |(d, l): (u32, u32)| rgb(if dark { d } else { l });
            match look {
                Look::Mac => pick(shades[0]),
                Look::Windows => pick(shades[1]),
                Look::Gtk => pick(shades[2]),
                Look::Ios => pick(shades[3]),
                Look::Material => pick(shades[4]),
                Look::Plain => plain,
            }
        };
        let system_accent = theme.accent.map(|[d, l]| rgb(if dark { d } else { l }));
        let accent = match look {
            // WinUI's default blue, Light 2 on the dark scheme and Dark 1 on the light.
            Look::Windows => system_accent.unwrap_or(rgb(if dark { 0x60cdff } else { 0x005fb8 })),
            Look::Gtk => system_accent.unwrap_or(rgb(0x3584e4)),
            // systemBlue; Material 3's baseline primary.
            Look::Ios => system_accent.unwrap_or(rgb(if dark { 0x0a84ff } else { 0x007aff })),
            Look::Material => system_accent.unwrap_or(rgb(if dark { 0xd0bcff } else { 0x6750a4 })),
            Look::Mac => system_accent.unwrap_or(p.accent),
            Look::Plain => p.accent,
        };
        let accent_hover = match look {
            // WinUI shows the accent at 90 % under the pointer.
            Look::Windows => accent.gamma_multiply(0.9).to_opaque(),
            Look::Gtk | Look::Ios => lighten(accent, 0.1),
            Look::Material => accent.gamma_multiply(0.92).to_opaque(),
            Look::Mac if system_accent.is_some() => lighten(accent, 0.15),
            _ => p.accent_hover,
        };
        Colors {
            dark,
            fg: p.fg,
            dim: p.dim,
            disabled: p.disabled,
            bg: p.bg,
            border: p.border,
            grid: p.grid,
            // libadwaita's buttons and entries are the text colour at 10 % (15 % hovered, 30 %
            // pressed) over the window.
            // iOS: the system fills (tertiarySystemFill and friends); Material: the tonal
            // container (secondaryContainer) and the highest surface container.
            button: by([(0x3d3f45, 0xffffff), (0x2f3136, 0xfbfbfb), (0x323336, 0xdadbde), (0x3a3a3c, 0xe5e5ea), (0x4a4458, 0xe8def8)], p.button),
            button_hover: by([(0x46484f, 0xf7f7f8), (0x36383d, 0xf5f5f5), (0x3d3e41, 0xd0d1d4), (0x48484a, 0xdcdce1), (0x564f66, 0xddd2ee)], p.button_hover),
            button_pressed: by([(0x35373c, 0xe4e4e7), (0x2a2c30, 0xf3f3f3), (0x5f6063, 0xb0b1b4), (0x545456, 0xd1d1d6), (0x625a74, 0xd2c6e4)], p.button_pressed),
            button_disabled: by([(0x2c2e33, 0xf1f1f3), (0x2a2c30, 0xf5f5f5), (0x26272a, 0xe3e4e7), (0x232325, 0xefeff4), (0x2b292f, 0xeae6ee)], p.button_disabled),
            entry: by([(0x2a2c31, 0xffffff), (0x2f3136, 0xfbfbfb), (0x323336, 0xdadbde), (0x2c2c2e, 0xffffff), (0x36343b, 0xe6e0e9)], p.entry),
            entry_focused: by([(0x2a2c31, 0xffffff), (0x1d1e21, 0xffffff), (0x323336, 0xdadbde), (0x2c2c2e, 0xffffff), (0x36343b, 0xe6e0e9)], p.entry),
            control_border: by([(0x4a4d54, 0xc9c9ce), (0x3a3c41, 0xe5e5e5), (0x3d3e41, 0xd0d1d4), (0x38383a, 0xd1d1d6), (0x49454f, 0xcac4d0)], p.border),
            control_strong: by([(0x8b9199, 0x5e646c), (0x9a9b9e, 0x868686), (0x5f6063, 0xb0b1b4), (0x636366, 0xc7c7cc), (0x938f99, 0x79747e)], p.dim),
            track: by([(0x3a3c42, 0xe3e3e7), (0x26282c, 0xe1e1e3), (0x3d3e41, 0xd0d1d4), (0x39393d, 0xe9e9ea), (0x36343b, 0xe6e0e9)], p.button),
            raised: if dark { rgb(0x5b5e65) } else { Color32::WHITE },
            accent,
            accent_hover,
            on_accent: match look {
                Look::Windows if dark => Color32::BLACK,
                // Material's onPrimary.
                Look::Material if dark => rgb(0x381e72),
                _ => Color32::WHITE,
            },
            row_hover: p.row_hover,
        }
    }
}

/// The colours of the theme installed in `ctx`, in egui's scheme of the moment.
pub fn colors(ctx: &egui::Context) -> Colors {
    crate::resolved(ctx).colors(ctx)
}
