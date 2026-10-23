#![warn(missing_docs)]
//! egui controls drawn the way macOS, Windows, GNOME, iOS and Android draw theirs.
//!
//! An app keeps its own window, panels and palette, and installs a [`Theme`]: the
//! [`Look`] its controls take, its palette for each scheme ([`Schemes`]) and the sizes of
//! its controls ([`Metrics`]). The theme lives in the [`egui::Context`], so every control
//! drawn in it takes it, and two contexts can have two themes.
//!
//! ```no_run
//! # fn setup(ctx: &egui::Context) {
//! use egui_native_look as look;
//! let mut fonts = egui::FontDefinitions::default();
//! look::install_fonts(&mut fonts, look::Look::system());
//! ctx.set_fonts(fonts);
//! look::install(ctx, look::Theme::system());
//! # }
//! # fn frame(ui: &mut egui::Ui, notifications: &mut bool, retries: &mut f64) {
//! # use egui_native_look as look;
//! // Every frame.
//! look::follow_system(ui.ctx());
//! ui.add(look::Button::new("Save").accent());
//! look::Form::new("settings").show(ui, |form| {
//!     form.row("Notifications", |ui| ui.add(look::Switch::new(notifications)));
//!     form.row("Retries", |ui| ui.add(look::NumberField::new(retries).range(1.0..=64.0)));
//! });
//! look::show_toasts(ui.ctx());
//! # }
//! ```
//!
//! # Controls
//!
//! | Kind | Controls |
//! | --- | --- |
//! | Buttons | [`Button`], [`IconButton`], [`Switch`], [`Checkbox`], [`Radio`] ([`radio_value`]) |
//! | Choices | [`Segmented`], [`Dropdown`], [`PopupButton`] ([`menu_value`]), [`Tabs`], [`Tab`] and [`TabBar`], [`Slider`], [`ColorWell`] |
//! | Entries | [`TextField`], [`NumberField`] |
//! | Layout | [`Form`], [`section`] and [`Section`], [`ListRow`] |
//! | Menus | [`MenuItem`], [`submenu`], [`combo_icon`] for egui's `ComboBox` |
//! | Status | [`ProgressBar`], [`Spinner`], [`toast`] and [`show_toasts`] |
//! | Windows | [`Dialog`], [`ToolWindow`], [`window_frame`], [`TitleBar`] |
//!
//! # The system
//!
//! [`Theme::system`] takes the look of the system it runs on, and [`install`] reads its
//! accent (System Settings on macOS, Personalization on Windows, GNOME's accent) unless
//! the theme sets one. [`follow_system`] keeps the accent and the scheme up to date while
//! the app runs; [`system_scheme`] reads the scheme. The scheme the controls take is
//! egui's: `ctx.set_theme`, or eframe following the system's. [`install_fonts`] adds the
//! system's own font where the look has one.
//!
//! All of this is the default `system` feature; without it the looks keep their own
//! accents, [`follow_system`] does nothing and the system's fonts are not read, for an app
//! that wants the controls alone.
//!
//! On a touch screen, in a browser too, [`Look::touch`] gives finger-sized controls
//! (iOS's on Apple's devices, Material's elsewhere) and [`touch_screen`] says whether
//! there is one. [`Metrics::scaled`] makes a look's controls larger or smaller.
//!
//! # Drawing your own
//!
//! [`colors`] gives the resolved colours for an app's own drawing, and the painters
//! ([`paint_button`], [`paint_popup_button`], [`focus_ring`], [`underline`], ...) draw a
//! control's body in the look. Text is centred on its capitals ([`text`],
//! [`galley_top`], [`entry_margin`]) rather than on egui's line box, which rounds
//! differently at each size.

mod colors;
mod paint;
mod style;
mod system;
mod text;
mod widgets;

pub use colors::{colors, Colors};
pub use paint::{
    focus_ring, paint_button, paint_fluent, paint_popup_button, paint_raised, underline,
};
pub use style::apply_style;
pub use system::{system_accent, system_fonts, system_scheme, touch_screen, SystemFonts};
pub use text::{cap_middle, entry_margin, galley_top, text};
pub use widgets::*;

use egui::{Color32, FontId};

/// How the controls are drawn, on top of the app's own palette and layout.
/// More looks may come, so a `match` on it needs a `_` arm.
///
/// Keep the returned value to use this configuration.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::Look;
/// Look::system();
/// ```
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Look {
    /// The app's own controls in its palette, the same on every system.
    Plain,
    /// SwiftUI / AppKit as of macOS 26: flat grey buttons, tracks and marks, pill-shaped
    /// switch and slider knobs, the accent on chosen segments and check boxes, pop-up
    /// buttons with up and down chevrons, steppers, sheets, soft focus rings.
    Mac,
    /// WinUI 3 / Fluent 2: darker bottom edges, underlined entries, outlined switches and
    /// check boxes, two-tone focus rings, black text on the dark scheme's light accent.
    Windows,
    /// GTK 4 / libadwaita: flat rounded controls in a tint of the text colour, outlined
    /// marks, a white switch knob inset in its pill.
    Gtk,
    /// iOS (SwiftUI / UIKit): touch-sized rounded controls, the green switch, round check
    /// marks, large white slider knobs.
    Ios,
    /// Material 3 (Android): pill buttons, the outlined switch whose thumb grows when on,
    /// square check boxes with a two-point edge, filled text fields with an underline.
    Material,
}

impl Look {
    /// The look of the system this runs on, by the target: macOS, Windows, GTK on Linux,
    /// iOS, Material on Android; the web gets [`Look::Plain`].
    pub fn system() -> Look {
        if cfg!(target_os = "macos") {
            Look::Mac
        } else if cfg!(target_os = "windows") {
            Look::Windows
        } else if cfg!(target_os = "android") {
            Look::Material
        } else if cfg!(target_os = "ios") {
            Look::Ios
        } else if cfg!(target_os = "linux") {
            Look::Gtk
        } else {
            Look::Plain
        }
    }

    /// The touch look of the system this runs on: iOS on Apple's systems (an iPad's or an
    /// iPhone's browser too), Material elsewhere. On a touch screen every control reserves a
    /// target at least 48 points across, whatever its drawn size (iOS draws 34-point
    /// controls, Material 40). Pair it with [`touch_screen`]:
    ///
    /// ```
    /// use egui_native_look::{touch_screen, Look};
    /// let look = if touch_screen() { Look::touch() } else { Look::system() };
    /// ```
    pub fn touch() -> Look {
        if cfg!(any(target_os = "macos", target_os = "ios")) || system::apple_browser() {
            Look::Ios
        } else {
            Look::Material
        }
    }

    /// `"plain"`, `"mac"`, `"windows"`, `"gtk"`, `"ios"` or `"material"`, for an override
    /// that shows one system's controls on another.
    pub fn from_name(name: &str) -> Option<Look> {
        match name {
            "plain" => Some(Look::Plain),
            "mac" => Some(Look::Mac),
            "windows" => Some(Look::Windows),
            "gtk" => Some(Look::Gtk),
            "ios" => Some(Look::Ios),
            "material" => Some(Look::Material),
            _ => None,
        }
    }

    /// Whether the controls are drawn in a system's manner rather than the app's own.
    #[must_use]
    pub fn native(self) -> bool {
        self != Look::Plain
    }

    /// The look's name as people write it: "macOS", "Windows", "GTK", "Plain".
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Look::Plain => "Plain",
            Look::Mac => "macOS",
            Look::Windows => "Windows",
            Look::Gtk => "GTK",
            Look::Ios => "iOS",
            Look::Material => "Material",
        }
    }
}

impl std::fmt::Display for Look {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// `shortcut` as a menu in `look` prints it: ⌥⇧⌘S on macOS, Ctrl+Shift+S elsewhere.
#[must_use]
pub fn shortcut_text(look: Look, shortcut: &egui::KeyboardShortcut) -> String {
    if look == Look::Mac {
        shortcut.format(&egui::ModifierNames::SYMBOLS, true)
    } else {
        shortcut.format(&egui::ModifierNames::NAMES, false)
    }
}

/// The colours the controls start from, for one scheme. The system looks draw their
/// buttons, entries and tracks in their own shades and use only the text colours, the
/// background and the accent (unless the system offers one); the plain look uses all.
///
/// Keep the returned value to use this configuration.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::Palette;
/// Palette::of(false);
/// ```
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    /// Ordinary text.
    pub fg: Color32,
    /// Secondary text: captions, hints.
    pub dim: Color32,
    /// Disabled text.
    pub disabled: Color32,
    /// The window behind the controls, and egui's panels.
    pub bg: Color32,
    /// Floating windows, dialogs, menus and pop-ups; the window's background unless the
    /// app sets its own.
    pub surface: Color32,
    /// Hairline borders round controls.
    pub border: Color32,
    /// A stronger line: a hovered control's edge in the plain look.
    pub grid: Color32,
    /// A button's fill at rest.
    pub button: Color32,
    /// A button's fill under the pointer.
    pub button_hover: Color32,
    /// A button's fill while pressed.
    pub button_pressed: Color32,
    /// A disabled button's fill.
    pub button_disabled: Color32,
    /// A text entry's fill.
    pub entry: Color32,
    /// The accent: selection, switched-on controls, focus.
    pub accent: Color32,
    /// The accent under the pointer.
    pub accent_hover: Color32,
    /// A list row under the pointer.
    pub row_hover: Color32,
}

/// The app's palette for each scheme.
///
/// Keep the returned value to use this configuration.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::Schemes;
/// Schemes::default();
/// ```
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Schemes {
    /// The dark scheme's palette.
    pub dark: Palette,
    /// The light scheme's palette.
    pub light: Palette,
}

impl Palette {
    /// The default palette of the dark or the light scheme, to change a few colours of:
    /// `Palette { bg, accent, ..Palette::of(true) }`.
    pub fn of(dark: bool) -> Palette {
        let rgb = colors::rgb;
        if dark {
            Palette {
                fg: rgb(0xe4e6ea),
                dim: rgb(0x8b9199),
                disabled: rgb(0x52575e),
                bg: rgb(0x1b1c1f),
                surface: rgb(0x1b1c1f),
                border: rgb(0x2f3237),
                grid: rgb(0x3a3d43),
                button: rgb(0x2c2f34),
                button_hover: rgb(0x373a41),
                button_pressed: rgb(0x25272b),
                button_disabled: rgb(0x222428),
                entry: rgb(0x121315),
                accent: rgb(0x3d8bfd),
                accent_hover: rgb(0x5a9dff),
                row_hover: Color32::from_white_alpha(10),
            }
        } else {
            Palette {
                fg: rgb(0x1d2025),
                dim: rgb(0x5e646c),
                disabled: rgb(0xa3a8af),
                bg: rgb(0xeceef1),
                surface: rgb(0xeceef1),
                border: rgb(0xcdd1d6),
                grid: rgb(0xbfc3c9),
                button: rgb(0xe1e4e8),
                button_hover: rgb(0xd5d9de),
                button_pressed: rgb(0xc9cdd3),
                button_disabled: rgb(0xe6e8eb),
                entry: rgb(0xffffff),
                accent: rgb(0x3d8bfd),
                accent_hover: rgb(0x5a9dff),
                row_hover: Color32::from_black_alpha(12),
            }
        }
    }
}

impl Palette {
    /// The palette `look`'s system draws its windows with, in the dark or the light scheme:
    /// its window, surface and text colours, its separators and its default accent. The
    /// plain look takes [`Palette::of`].
    pub fn for_look(look: Look, dark: bool) -> Palette {
        let rgb = colors::rgb;
        let base = Palette::of(dark);
        // (dark, light) pairs.
        let pick = |(d, l): (u32, u32)| rgb(if dark { d } else { l });
        let with =
            |fg, dim, disabled, bg, surface, border, accent: (u32, u32), hover: Color32| Palette {
                fg: pick(fg),
                dim: pick(dim),
                disabled: pick(disabled),
                bg: pick(bg),
                surface: pick(surface),
                border: pick(border),
                grid: pick(border),
                accent: pick(accent),
                accent_hover: pick(accent),
                row_hover: hover,
                ..base
            };
        let hover = if dark {
            Color32::from_white_alpha(15)
        } else {
            Color32::from_black_alpha(10)
        };
        match look {
            // SwiftUI's window, labels and separators on macOS 26.
            Look::Mac => with(
                (0xe0e0e0, 0x262626),
                (0x969696, 0x808080),
                (0x5b5b5b, 0xc6c6c6),
                (0x1e1e1e, 0xffffff),
                (0x1e1e1e, 0xffffff),
                (0x353535, 0xe6e6e6),
                (0x007aff, 0x007aff),
                hover,
            ),
            // WinUI's base and layer fills and its text and card stroke colours.
            Look::Windows => with(
                (0xffffff, 0x1b1b1b),
                (0xcccccc, 0x5c5c5c),
                (0x717171, 0x9b9b9b),
                (0x202020, 0xf3f3f3),
                (0x2c2c2c, 0xfbfbfb),
                (0x2a2a2a, 0xe5e5e5),
                (0x60cdff, 0x005fb8),
                hover,
            ),
            // libadwaita's window, popover and text colours, its dimmed and disabled text.
            Look::Gtk => with(
                (0xffffff, 0x323237),
                (0x9c9c9d, 0x8c8c8f),
                (0x909192, 0x969699),
                (0x222226, 0xfafafb),
                (0x36363a, 0xffffff),
                (0x434347, 0xdcdcde),
                (0x3584e4, 0x3584e4),
                hover,
            ),
            // iOS's system and secondary system backgrounds, labels and separators.
            Look::Ios => with(
                (0xffffff, 0x000000),
                (0x8d8d93, 0x8a8a8e),
                (0x58585e, 0xbcbcc0),
                (0x000000, 0xffffff),
                (0x1c1c1e, 0xf2f2f7),
                (0x38383a, 0xc6c6c8),
                (0x0091ff, 0x0088ff),
                hover,
            ),
            // Material 3's baseline surface, its containers, text and outline variant.
            Look::Material => with(
                (0xe6e0e9, 0x1d1b20),
                (0xcac4d0, 0x49454f),
                (0x67636d, 0xa9a5ad),
                (0x141218, 0xfef7ff),
                (0x211f26, 0xf3edf7),
                (0x49454f, 0xcac4d0),
                (0xd0bcff, 0x6750a4),
                hover,
            ),
            Look::Plain => base,
        }
    }
}

impl Schemes {
    /// Both schemes of [`Palette::for_look`].
    pub fn for_look(look: Look) -> Schemes {
        Schemes {
            dark: Palette::for_look(look, true),
            light: Palette::for_look(look, false),
        }
    }
}

impl Default for Schemes {
    fn default() -> Self {
        Schemes {
            dark: Palette::of(true),
            light: Palette::of(false),
        }
    }
}

/// The sizes the controls take.
///
/// Keep the returned value to use this configuration.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::Metrics;
/// Metrics::of(egui_native_look::Look::Plain);
/// ```
#[must_use]
#[derive(Clone, Debug, PartialEq)]
pub struct Metrics {
    /// The height of a button, an entry, a pop-up button, a segmented choice.
    /// Applying the theme replaces non-positive or non-finite sizes with the look's
    /// default, and bounds positive heights to 1..=1,000,000 points.
    pub control_height: f32,
    /// The rounding of an ordinary control.
    pub radius: u8,
    /// The face labels are set in. Applied font sizes use the look's default when
    /// non-positive or non-finite, and otherwise are bounded to 1..=256 points.
    pub body: FontId,
    /// The face a chosen segment and a heading take: the body's family until
    /// [`Metrics::system_bold`] switches it to the system's semibold. Its size follows
    /// the same validation as [`Metrics::body`].
    pub bold: FontId,
}

impl Metrics {
    /// The system's own sizes for `look`: macOS's regular controls are 24 points tall in
    /// 13-point text, WinUI's 32 in 14, libadwaita's 34 in 14.67 (11 pt), iOS 26's 34 in
    /// 17, Material's 40 in 14; the plain look's 28 in 13.
    pub fn of(look: Look) -> Metrics {
        let (height, radius, size) = match look {
            Look::Mac => (24.0, 6, 13.0),
            Look::Windows => (32.0, 4, 14.0),
            Look::Gtk => (34.0, 9, 14.67),
            Look::Ios => (34.0, 8, 17.0),
            Look::Material => (40.0, 8, 14.0),
            Look::Plain => (28.0, 4, 13.0),
        };
        Metrics {
            control_height: height,
            radius,
            body: FontId::proportional(size),
            bold: FontId::proportional(size),
        }
    }

    /// Smaller controls, as SwiftUI's `.small` control size draws them on macOS: 20 points
    /// tall in 11-point text, the switches, marks and knobs in proportion. The other looks
    /// take five sixths of their size.
    pub fn small(look: Look) -> Metrics {
        if look == Look::Mac {
            Metrics {
                control_height: 20.0,
                radius: 5,
                body: FontId::proportional(11.0),
                bold: FontId::proportional(11.0),
            }
        } else {
            Self::of(look).scaled(5.0 / 6.0)
        }
    }

    /// Larger controls, as SwiftUI's `.large` control size draws them on macOS: 28 points
    /// tall in the regular 13-point text. The other looks grow by a sixth and keep their
    /// text.
    pub fn large(look: Look) -> Metrics {
        let mut metrics = Self::of(look);
        let height = if look == Look::Mac {
            28.0
        } else {
            (metrics.control_height * 7.0 / 6.0).round()
        };
        metrics.radius =
            (f32::from(metrics.radius) * height / metrics.control_height).round() as u8;
        metrics.control_height = height;
        metrics
    }

    pub(crate) fn sanitized(mut self, look: Look) -> Self {
        let defaults = Self::of(look);
        self.control_height =
            widgets::dimension(self.control_height).unwrap_or(defaults.control_height);
        self.body.size = widgets::dimension(self.body.size)
            .unwrap_or(defaults.body.size)
            .min(256.0);
        self.bold.size = widgets::dimension(self.bold.size)
            .unwrap_or(defaults.bold.size)
            .min(256.0);
        self
    }

    /// With the bold face in the system's semibold, once [`install_fonts`] has put it in
    /// egui's fonts.
    pub fn system_bold(mut self) -> Metrics {
        self.bold = FontId::new(self.bold.size, bold_family());
        self
    }

    /// Everything `factor` times the size: the controls' height, their rounding and their
    /// text. To scale the whole interface instead (the gaps, the marks and the app's own
    /// widgets with it), set egui's zoom: `ctx.set_zoom_factor`. The factor is held between
    /// 0.25 and 8; one that is not a number leaves the sizes as they are.
    pub fn scaled(mut self, factor: f32) -> Metrics {
        let factor = if factor.is_nan() {
            1.0
        } else {
            factor.clamp(0.25, 8.0)
        };
        self.control_height = (self.control_height * factor).round().max(1.0);
        self.radius = (f32::from(self.radius) * factor).round().clamp(0.0, 255.0) as u8;
        self.body.size *= factor;
        self.bold.size *= factor;
        self
    }
}

/// The family [`install_fonts`] registers the system's semibold face under; a body face
/// stands in where there is none.
pub fn bold_family() -> egui::FontFamily {
    egui::FontFamily::Name("egui_native_look-bold".into())
}

/// The whole of a look: which, the app's palette, the controls' sizes, and the system's
/// accent once [`install`] has read it.
///
/// Keep the returned value to use this configuration.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::Theme;
/// Theme::system();
/// ```
#[must_use]
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    /// How the controls are drawn.
    pub look: Look,
    /// The app's palette for each scheme.
    pub schemes: Schemes,
    /// The controls' sizes and fonts.
    pub metrics: Metrics,
    /// The system's accent for the dark and the light scheme, as 0xRRGGBB; `None` keeps
    /// the palette's. [`install`] fills it in unless it was set.
    pub accent: Option<[u32; 2]>,
    /// Whether the system asks for more contrast: controls take an edge, troughs and
    /// separators darken and secondary text comes closer to the text colour, as
    /// libadwaita, macOS and iOS draw them then. Off by default.
    pub high_contrast: bool,
}

impl Default for Theme {
    fn default() -> Self {
        Self::system()
    }
}

impl Theme {
    /// This system's look, with its palette and sizes.
    pub fn system() -> Theme {
        Theme::new(Look::system())
    }

    /// `look` with its system's palette ([`Schemes::for_look`]) and sizes.
    pub fn new(look: Look) -> Theme {
        Theme {
            look,
            schemes: Schemes::for_look(look),
            metrics: Metrics::of(look),
            accent: None,
            high_contrast: false,
        }
    }

    /// With more contrast, as the system's accessibility setting asks; see
    /// [`Theme::high_contrast`](Theme#structfield.high_contrast).
    pub fn high_contrast(mut self, on: bool) -> Theme {
        self.high_contrast = on;
        self
    }

    /// With the app's palette.
    pub fn schemes(mut self, schemes: Schemes) -> Theme {
        self.schemes = schemes;
        self
    }

    /// With the app's sizes, normalized according to [`Metrics`].
    pub fn metrics(mut self, metrics: Metrics) -> Theme {
        self.metrics = metrics.sanitized(self.look);
        self
    }
}

/// A theme with its colours resolved for both schemes, as the controls read it: resolved
/// once when installed, shared by every control of every frame.
pub(crate) struct Resolved {
    pub theme: Theme,
    pub dark: Colors,
    pub light: Colors,
    /// Whether the accent is the system's, read when installed, which [`follow_system`]
    /// keeps up to date; an app's own accent is left alone.
    pub system_accent: bool,
    /// Whether [`install`] styled egui's widgets, which then follow the accent too.
    pub styled: bool,
}

impl Resolved {
    fn of(theme: Theme, system_accent: bool, styled: bool) -> Resolved {
        Resolved {
            dark: Colors::resolve(&theme, true),
            light: Colors::resolve(&theme, false),
            theme,
            system_accent,
            styled,
        }
    }

    /// The colours for `ctx`'s scheme of the moment.
    pub fn colors(&self, ctx: &egui::Context) -> Colors {
        let colors = if ctx.theme() == egui::Theme::Dark {
            self.dark
        } else {
            self.light
        };
        if ctx.input(|input| input.focused) {
            colors
        } else {
            colors.inactive(self.theme.look)
        }
    }
}

fn resolved_id() -> egui::Id {
    egui::Id::unique("egui_native_look::Resolved")
}

/// The theme installed in `ctx` with its colours; while none is, this system's look with
/// its own accent, read from nothing (the system's accent comes with [`install`] or
/// [`follow_system`]).
pub(crate) fn resolved(ctx: &egui::Context) -> std::sync::Arc<Resolved> {
    static FALLBACK: std::sync::OnceLock<std::sync::Arc<Resolved>> = std::sync::OnceLock::new();
    ctx.data(|d| d.get_temp::<std::sync::Arc<Resolved>>(resolved_id()))
        .unwrap_or_else(|| {
            FALLBACK
                .get_or_init(|| std::sync::Arc::new(Resolved::of(Theme::system(), true, false)))
                .clone()
        })
}

/// Installs `theme` in `ctx`: [`set_theme`], then styles egui's own widgets to match in
/// both of egui's schemes ([`apply_style`]). The app's own changes to egui's style that
/// it does not draw with (shadows, spacing, the heading, its other text styles) are kept;
/// the body and button text styles take the theme's [`Metrics::body`], so an app with a
/// face of its own puts it in the metrics, or calls [`set_theme`] and styles egui itself.
///
/// The theme is kept in `ctx`'s memory, as egui keeps its own state: an app that resets
/// egui's memory installs it again.
pub fn install(ctx: &egui::Context, theme: Theme) {
    let theme = keep(ctx, theme, true);
    restyle(ctx, &theme);
}

fn restyle(ctx: &egui::Context, theme: &Theme) {
    for scheme in [egui::Theme::Dark, egui::Theme::Light] {
        ctx.style_mut_of(scheme, |style| {
            apply_style(style, theme, scheme == egui::Theme::Dark)
        });
    }
}

/// Keeps `theme` in `ctx` for the controls, reading the system's accent unless the theme
/// sets one, and leaves egui's style alone: for an app that styles egui itself, or that
/// changes the theme's sizes later. Returns the theme as kept.
pub fn set_theme(ctx: &egui::Context, theme: Theme) -> Theme {
    keep(ctx, theme, false)
}

fn keep(ctx: &egui::Context, mut theme: Theme, styled: bool) -> Theme {
    theme.metrics = theme.metrics.sanitized(theme.look);
    let system_accent = theme.accent.is_none();
    if system_accent {
        theme.accent = system::system_accent(theme.look);
    }
    ctx.data_mut(|d| {
        d.insert_temp(
            resolved_id(),
            std::sync::Arc::new(Resolved::of(theme.clone(), system_accent, styled)),
        )
    });
    theme
}

/// Keeps the theme in `ctx` up to date with the system: call it once a frame. It reads the
/// system's accent and scheme on the first frame and whenever the window comes to the
/// front, where the system's settings are changed from; on Windows, and on macOS from the
/// main thread, where reading is cheap, also every two seconds of frames drawn anyway. It
/// never asks for frames itself, so an idle app stays asleep. On GNOME reading takes a
/// process, which runs off the UI thread; what it finds is applied on a later frame.
///
/// A new accent (unless the app set its own) re-resolves the colours, restyles egui's
/// widgets if [`install`] styled them, and asks for a repaint. A new scheme becomes egui's
/// fallback, which egui shows while its theme preference is `System` and the backend does
/// not report the system's theme (eframe on the desktop does, and wins). Without the
/// `system` feature it does nothing.
pub fn follow_system(ctx: &egui::Context) {
    system::follow(ctx);
}

/// Replaces the accent of the theme in `ctx` with the system's `accent`, as
/// [`follow_system`] found it.
pub(crate) fn take_system_accent(ctx: &egui::Context, accent: Option<[u32; 2]>) {
    let current = resolved(ctx);
    if !current.system_accent || current.theme.accent == accent {
        return;
    }
    let mut theme = current.theme.clone();
    theme.accent = accent;
    let styled = current.styled;
    ctx.data_mut(|d| {
        d.insert_temp(
            resolved_id(),
            std::sync::Arc::new(Resolved::of(theme.clone(), true, styled)),
        )
    });
    if styled {
        restyle(ctx, &theme);
    }
    ctx.request_repaint();
}

/// The theme installed in `ctx`, or this system's when none was.
pub fn theme(ctx: &egui::Context) -> Theme {
    resolved(ctx).theme.clone()
}

/// The look installed in `ctx`.
pub fn look(ctx: &egui::Context) -> Look {
    resolved(ctx).theme.look
}

/// Puts the system's faces for `look` (San Francisco, Segoe UI Variable, Adwaita Sans or
/// Cantarell) in front of `fonts`' proportional and monospace families, and its semibold
/// behind [`bold_family`]. Returns whether the system had them.
pub fn install_fonts(fonts: &mut egui::FontDefinitions, look: Look) -> bool {
    let system = system_fonts(look);
    let proportional = fonts
        .families
        .get(&egui::FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let mut bold = proportional.clone();
    let found = system.is_some();
    if let Some(system) = system {
        fonts
            .font_data
            .insert("egui_native_look-regular".into(), system.regular.into());
        fonts
            .font_data
            .insert("egui_native_look-semibold".into(), system.semibold.into());
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "egui_native_look-regular".into());
        bold.insert(0, "egui_native_look-semibold".into());
        if let Some(mono) = system.mono {
            fonts
                .font_data
                .insert("egui_native_look-mono".into(), mono.into());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "egui_native_look-mono".into());
        }
    }
    fonts.families.entry(bold_family()).or_insert(bold);
    found
}

/// macOS's menus and pop-overs: near white, or a mid grey on the dark scheme, whatever the
/// window behind them.
pub(crate) fn mac_menu_fill(dark: bool) -> Color32 {
    colors::rgb(if dark { 0x3e3e3e } else { 0xfcfcfc })
}

/// A translucent ink, as Apple's system fills are: white at `dark` opacity on the dark
/// scheme, black at `light` on the light one, so a fill reads the same over a card.
pub(crate) fn ink(c: &Colors, dark: f32, light: f32) -> Color32 {
    let alpha = |share: f32| (share * 255.0).round() as u8;
    if c.dark {
        Color32::from_white_alpha(alpha(dark))
    } else {
        Color32::from_black_alpha(alpha(light))
    }
}

/// macOS's accent under a pressed pointer: darker on the light scheme, lighter on the dark.
pub(crate) fn mac_pressed_accent(c: &Colors) -> Color32 {
    if c.dark {
        lerp_color(c.accent, Color32::WHITE, 0.17)
    } else {
        lerp_color(c.accent, Color32::BLACK, 0.1)
    }
}

/// macOS's accent on a disabled control: faded most of the way from the window.
pub(crate) fn mac_disabled_accent(c: &Colors) -> Color32 {
    lerp_color(c.bg, c.accent, if c.dark { 0.75 } else { 0.68 })
}

/// `c` a fraction `t` of the way to white.
pub(crate) fn lighten(c: Color32, t: f32) -> Color32 {
    let up = |v: u8| (f32::from(v) + (255.0 - f32::from(v)) * t).round() as u8;
    Color32::from_rgb(up(c.r()), up(c.g()), up(c.b()))
}

/// `a` a fraction `t` of the way to `b`, alpha too.
pub(crate) fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    // Premultiplied components lerp as they are, translucent inks included.
    Color32::from_rgba_premultiplied(
        l(a.r(), b.r()),
        l(a.g(), b.g()),
        l(a.b(), b.b()),
        l(a.a(), b.a()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_system_accent_reaches_the_colours_and_the_style() {
        let ctx = egui::Context::default();
        install(&ctx, Theme::new(Look::Mac));
        let before = resolved(&ctx).dark.accent;
        take_system_accent(&ctx, Some([0xff0000, 0xff0000]));
        assert_eq!(theme(&ctx).accent, Some([0xff0000, 0xff0000]));
        let after = resolved(&ctx).dark.accent;
        assert_ne!(after, before);
        assert_eq!(
            ctx.style_of(egui::Theme::Dark).visuals.selection.bg_fill,
            after.gamma_multiply(0.55)
        );
    }

    #[test]
    fn an_accent_the_app_chose_stays() {
        let ctx = egui::Context::default();
        let mut chosen = Theme::new(Look::Gtk);
        chosen.accent = Some([0x00ff00, 0x00ff00]);
        let _ = set_theme(&ctx, chosen);
        take_system_accent(&ctx, Some([0xff0000, 0xff0000]));
        assert_eq!(theme(&ctx).accent, Some([0x00ff00, 0x00ff00]));
    }
    #[test]
    fn theme_configuration_builders_preserve_the_base_configuration() {
        let base = Theme::default();
        assert_eq!(base.look, Look::system());
        let metrics = Metrics::of(Look::Gtk).scaled(1.5);
        let schemes = Schemes {
            dark: Palette {
                accent: Color32::RED,
                ..Palette::of(true)
            },
            light: Palette {
                accent: Color32::BLUE,
                ..Palette::of(false)
            },
        };
        let changed = base.clone().metrics(metrics.clone()).schemes(schemes);
        assert_eq!(changed.look, base.look);
        assert_eq!(changed.accent, base.accent);
        assert_eq!(changed.metrics, metrics);
        assert_eq!(changed.schemes, schemes);
        assert_ne!(base.schemes, changed.schemes);
    }
}
