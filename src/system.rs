//! What the system offers: its text faces, which an app may not ship, and its accent.

use egui::FontData;

use crate::Look;

/// The system's faces for a look: San Francisco on macOS, Segoe UI Variable (or Segoe
/// UI) on Windows, each with its monospaced face where there is one.
#[non_exhaustive]
pub struct SystemFonts {
    /// Body text.
    pub regular: FontData,
    /// Emphasised labels, headings: semibold.
    pub semibold: FontData,
    /// Code and numbers in columns.
    pub mono: Option<FontData>,
}

/// What one read of the system found, for the look it was read for.
struct Found {
    look: Look,
    accent: Option<[u32; 2]>,
    scheme: Option<egui::Theme>,
}

/// When the system was last asked, per context, and the read under way, if any. A read
/// off the UI thread sends what it found back; one that died drops its sender, which ends
/// the wait as well.
#[derive(Clone, Default)]
struct Follow {
    last: Option<f64>,
    pending: Option<std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Found>>>>,
}

/// See [`crate::follow_system`].
pub(crate) fn follow(ctx: &egui::Context) {
    /// How often a cheap read (no process) is repeated while frames are drawn anyway.
    const EVERY: f64 = 2.0;
    if cfg!(any(target_arch = "wasm32", not(feature = "system"))) {
        return;
    }
    let id = egui::Id::unique("egui_native_look::Follow");
    let (now, focused) = ctx.input(|i| {
        (
            i.time,
            i.events
                .iter()
                .any(|e| matches!(e, egui::Event::WindowFocused(true))),
        )
    });
    let mut state = ctx.data(|d| d.get_temp::<Follow>(id)).unwrap_or_default();
    if let Some(pending) = state.pending.clone() {
        let received = pending.lock().map(|r| r.try_recv());
        match received {
            Ok(Ok(found)) => {
                apply(ctx, found);
                state.pending = None;
            }
            Ok(Err(std::sync::mpsc::TryRecvError::Empty)) => {}
            _ => state.pending = None,
        }
    }
    let cheap = cheap_read();
    let due = match state.last {
        None => true,
        Some(last) => focused || (cheap && now - last >= EVERY),
    };
    if due && state.pending.is_none() {
        state.last = Some(now);
        let look = crate::look(ctx);
        if cheap {
            apply(
                ctx,
                Found {
                    look,
                    accent: system_accent(look),
                    scheme: system_scheme(),
                },
            );
        } else {
            // Reading takes a process on GNOME, so it is done off the UI thread, and what it
            // finds is applied here, on a later frame.
            let (send, receive) = std::sync::mpsc::channel();
            state.pending = Some(std::sync::Arc::new(std::sync::Mutex::new(receive)));
            let repaint = ctx.clone();
            std::thread::spawn(move || {
                let _ = send.send(Found {
                    look,
                    accent: system_accent(look),
                    scheme: system_scheme(),
                });
                repaint.request_repaint();
            });
        }
    }
    ctx.data_mut(|d| d.insert_temp(id, state));
}

/// Whether the system can be read on the UI thread without a process: on Windows from the
/// registry, on macOS from the application's appearance on the main thread.
fn cheap_read() -> bool {
    #[cfg(all(target_os = "macos", feature = "system"))]
    let cheap = objc2::MainThreadMarker::new().is_some();
    #[cfg(not(all(target_os = "macos", feature = "system")))]
    let cheap = cfg!(target_os = "windows");
    cheap
}

/// Takes what a read found, unless the theme's look changed while it was read.
fn apply(ctx: &egui::Context, found: Found) {
    if crate::look(ctx) != found.look {
        return;
    }
    crate::take_system_accent(ctx, found.accent);
    if let Some(scheme) = found.scheme {
        if ctx.options(|o| o.fallback_theme) != scheme {
            ctx.options_mut(|o| o.fallback_theme = scheme);
            ctx.request_repaint();
        }
    }
}

/// The scheme the system is set to, dark or light: System Settings > Appearance on
/// macOS, the apps' mode on Windows, GNOME's style. `None` where it cannot be read
/// (another system, the web, where the browser tells eframe, or without the `system`
/// feature). On GNOME, and on macOS off the main thread, it takes a process, so call it
/// off the UI thread or seldom.
pub fn system_scheme() -> Option<egui::Theme> {
    #[cfg(all(target_os = "macos", feature = "system"))]
    return mac::scheme();
    #[cfg(all(target_os = "windows", feature = "system"))]
    return windows::scheme();
    #[cfg(all(target_os = "linux", feature = "system"))]
    return gnome::scheme();
    #[allow(unreachable_code)]
    None
}

/// Whether the screen takes touch: in a browser, whether it reports touch points (a
/// tablet, a phone, a touch laptop; not without the `system` feature); on iOS and Android
/// always; elsewhere not.
pub fn touch_screen() -> bool {
    #[cfg(all(target_arch = "wasm32", feature = "system"))]
    let touch = web_sys::window().is_some_and(|w| w.navigator().max_touch_points() > 0);
    #[cfg(not(all(target_arch = "wasm32", feature = "system")))]
    let touch = cfg!(any(target_os = "ios", target_os = "android"));
    touch
}

/// Whether this runs in a browser on an Apple system: an iPhone, an iPad (which calls
/// itself a Mac) or a Mac.
pub(crate) fn apple_browser() -> bool {
    #[cfg(all(target_arch = "wasm32", feature = "system"))]
    let apple = web_sys::window()
        .and_then(|w| w.navigator().user_agent().ok())
        .is_some_and(|agent| {
            ["iPhone", "iPad", "Macintosh"]
                .iter()
                .any(|name| agent.contains(name))
        });
    #[cfg(not(all(target_arch = "wasm32", feature = "system")))]
    let apple = false;
    apple
}

/// The system's accent as 0xRRGGBB, for the dark and the light scheme: on macOS the colour
/// of System Settings > Appearance (the same for both), on Windows the shades WinUI takes
/// from Personalization > Colors, on GNOME (46 and later) the accent of Settings >
/// Appearance. `None` elsewhere, where the system says nothing, or without the `system`
/// feature.
pub fn system_accent(look: Look) -> Option<[u32; 2]> {
    match look {
        #[cfg(all(target_os = "macos", feature = "system"))]
        Look::Mac => mac::accent().map(|rgb| [rgb, rgb]),
        #[cfg(all(target_os = "windows", feature = "system"))]
        Look::Windows => windows::accent(),
        #[cfg(all(target_os = "linux", feature = "system"))]
        Look::Gtk => gnome::accent().map(|rgb| [rgb, rgb]),
        _ => None,
    }
}

/// The system's faces for `look`, resolved through desktop settings and installed fonts;
/// `None` where they
/// are not there (another system, an old one) or without the `system` feature.
pub fn system_fonts(look: Look) -> Option<SystemFonts> {
    match look {
        #[cfg(all(target_os = "macos", feature = "system"))]
        Look::Mac => mac::fonts(),
        #[cfg(all(target_os = "windows", feature = "system"))]
        Look::Windows => windows::fonts(),
        #[cfg(all(target_os = "linux", feature = "system"))]
        Look::Gtk => gnome::fonts(),
        #[cfg(all(target_os = "android", feature = "system"))]
        Look::Material => android::fonts(),
        _ => None,
    }
}

pub(crate) fn font_size(look: Look) -> Option<f32> {
    #[cfg(all(target_os = "linux", feature = "system"))]
    if look == Look::Gtk {
        return gnome::font_size();
    }
    let _ = look;
    None
}

pub(crate) fn schemes(look: Look) -> Option<crate::Schemes> {
    #[cfg(all(target_os = "linux", feature = "system"))]
    if look == Look::Gtk {
        return gnome::schemes();
    }
    let _ = look;
    None
}

/// One variable font at the weight `weight`.
#[cfg_attr(
    not(all(
        feature = "system",
        any(
            target_os = "macos",
            target_os = "windows",
            target_os = "linux",
            target_os = "android"
        )
    )),
    allow(dead_code)
)]
fn weighted(bytes: &[u8], weight: f32) -> FontData {
    let mut data = FontData::from_owned(bytes.to_vec());
    data.tweak = egui::FontTweak {
        coords: egui::epaint::text::VariationCoords::new([(b"wght", weight)]),
        ..Default::default()
    };
    data
}

#[cfg(all(target_os = "macos", feature = "system"))]
mod mac {
    use egui::FontData;

    /// San Francisco, one variable font for both weights: regular, and the semibold (590)
    /// macOS sets its emphasised labels in; SF Mono.
    pub fn fonts() -> Option<super::SystemFonts> {
        let bytes = std::fs::read("/System/Library/Fonts/SFNS.ttf").ok()?;
        let mono = std::fs::read("/System/Library/Fonts/SFNSMono.ttf")
            .ok()
            .map(FontData::from_owned);
        // CoreText sets the optical size to the point size, which for interface text
        // (13 points) is the font's smallest, 17: the wider "Text" design rather than the
        // "Display" one the font defaults to.
        let text = |weight: f32| {
            let mut data = super::weighted(&bytes, weight);
            data.tweak.coords =
                egui::epaint::text::VariationCoords::new([(b"wght", weight), (b"opsz", 17.0)]);
            data
        };
        Some(super::SystemFonts {
            regular: text(400.0),
            semibold: text(590.0),
            mono,
        })
    }

    /// Dark when System Settings > Appearance is dark (or automatic, and dark now): the
    /// application's appearance on the main thread, `defaults` elsewhere.
    pub fn scheme() -> Option<egui::Theme> {
        if let Some(main) = objc2::MainThreadMarker::new() {
            let name = objc2_app_kit::NSApplication::sharedApplication(main)
                .effectiveAppearance()
                .name()
                .to_string();
            return Some(if name.contains("Dark") {
                egui::Theme::Dark
            } else {
                egui::Theme::Light
            });
        }
        let out = std::process::Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
            .ok()?;
        // Light has no value at all, and `defaults` fails for it.
        Some(if String::from_utf8_lossy(&out.stdout).trim() == "Dark" {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        })
    }

    /// The accent colour of System Settings > Appearance, as 0xRRGGBB in sRGB.
    pub fn accent() -> Option<u32> {
        use objc2_app_kit::{NSColor, NSColorSpace};
        let color =
            NSColor::controlAccentColor().colorUsingColorSpace(&NSColorSpace::sRGBColorSpace())?;
        let channel = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u32;
        Some(
            channel(color.redComponent()) << 16
                | channel(color.greenComponent()) << 8
                | channel(color.blueComponent()),
        )
    }
}

#[cfg(all(target_os = "windows", feature = "system"))]
mod windows {
    use egui::FontData;

    fn fonts_dir() -> std::path::PathBuf {
        std::env::var_os("WINDIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| "C:\\Windows".into())
            .join("Fonts")
    }

    /// Segoe UI Variable (Windows 11), one variable font for both weights, else Segoe UI
    /// and its semibold; Cascadia Mono, else Consolas.
    pub fn fonts() -> Option<super::SystemFonts> {
        let dir = fonts_dir();
        let (regular, semibold) = if let Ok(bytes) = std::fs::read(dir.join("SegUIVar.ttf")) {
            (
                super::weighted(&bytes, 400.0),
                super::weighted(&bytes, 600.0),
            )
        } else {
            let regular = std::fs::read(dir.join("segoeui.ttf")).ok()?;
            let semibold =
                std::fs::read(dir.join("seguisb.ttf")).unwrap_or_else(|_| regular.clone());
            (
                FontData::from_owned(regular),
                FontData::from_owned(semibold),
            )
        };
        let mono = ["CascadiaMono.ttf", "consola.ttf"]
            .iter()
            .find_map(|name| std::fs::read(dir.join(name)).ok())
            .map(FontData::from_owned);
        Some(super::SystemFonts {
            regular,
            semibold,
            mono,
        })
    }

    /// The apps' mode of Personalization > Colors.
    pub fn scheme() -> Option<egui::Theme> {
        use windows_sys::Win32::System::Registry::{
            RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD,
        };
        let wide = |s: &str| s.encode_utf16().chain([0]).collect::<Vec<u16>>();
        let key = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
        let name = wide("AppsUseLightTheme");
        let mut light = 0u32;
        let mut size = 4u32;
        // SAFETY: the strings are NUL-terminated and the value is a four-byte DWORD.
        let status = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                name.as_ptr(),
                RRF_RT_REG_DWORD,
                std::ptr::null_mut(),
                (&mut light as *mut u32).cast(),
                &mut size,
            )
        };
        (status == 0).then_some(if light == 0 {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        })
    }

    /// The accent's light shade (for the dark scheme) and dark shade (for the light), as
    /// 0xRRGGBB, from the palette Windows keeps beside the chosen accent.
    pub fn accent() -> Option<[u32; 2]> {
        use windows_sys::Win32::System::Registry::{
            RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_BINARY,
        };
        let wide = |s: &str| s.encode_utf16().chain([0]).collect::<Vec<u16>>();
        let key = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Accent");
        let name = wide("AccentPalette");
        let mut palette = [0u8; 32];
        let mut size = palette.len() as u32;
        // SAFETY: the strings are NUL-terminated and the buffer is `size` bytes long.
        let status = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                name.as_ptr(),
                RRF_RT_REG_BINARY,
                std::ptr::null_mut(),
                palette.as_mut_ptr().cast(),
                &mut size,
            )
        };
        if status != 0 || size < 20 {
            return None;
        }
        // Eight RGBA entries: Light 3, Light 2, Light 1, the accent, Dark 1, Dark 2, Dark 3.
        let rgb = |i: usize| {
            u32::from(palette[i * 4]) << 16
                | u32::from(palette[i * 4 + 1]) << 8
                | u32::from(palette[i * 4 + 2])
        };
        Some([rgb(1), rgb(4)])
    }
}

#[cfg(all(target_os = "linux", feature = "system"))]
mod gnome {
    use egui::FontData;

    /// Read the desktop font settings through Fontconfig, falling back to installed
    /// Adwaita or Cantarell fonts when the settings are unavailable.
    pub fn fonts() -> Option<super::SystemFonts> {
        if let Some((family, _)) = desktop_setting("font-name").and_then(|s| parse_font(&s)) {
            if let (Some(regular), Some(semibold)) = (
                matched_font(&family, "regular", 400.0),
                matched_font(&family, "bold", 700.0),
            ) {
                let mono = desktop_setting("monospace-font-name")
                    .and_then(|s| parse_font(&s))
                    .and_then(|(family, _)| matched_font(&family, "regular", 400.0));
                return Some(super::SystemFonts {
                    regular,
                    semibold,
                    mono,
                });
            }
        }
        let first = |paths: &[&str]| paths.iter().find_map(|p| std::fs::read(p).ok());
        let (text, bold_path) = [
            (
                "/usr/share/fonts/adwaita-sans-fonts/AdwaitaSans-Regular.ttf",
                None,
            ),
            (
                "/usr/share/fonts/truetype/adwaita/AdwaitaSans-Regular.ttf",
                None,
            ),
            ("/usr/share/fonts/Adwaita/AdwaitaSans-Regular.ttf", None),
            ("/usr/share/fonts/cantarell/Cantarell-VF.otf", None),
            ("/usr/share/fonts/opentype/cantarell/Cantarell-VF.otf", None),
            (
                "/usr/share/fonts/opentype/cantarell/Cantarell-Regular.otf",
                Some("/usr/share/fonts/opentype/cantarell/Cantarell-Bold.otf"),
            ),
            (
                "/usr/share/fonts/abattis-cantarell-vf-fonts/Cantarell-VF.otf",
                None,
            ),
        ]
        .into_iter()
        .find_map(|(path, bold)| std::fs::read(path).ok().map(|text| (text, bold)))?;
        let mono = first(&[
            "/usr/share/fonts/adwaita-mono-fonts/AdwaitaMono-Regular.ttf",
            "/usr/share/fonts/truetype/adwaita/AdwaitaMono-Regular.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
            "/usr/share/fonts/dejavu-sans-mono-fonts/DejaVuSansMono.ttf",
        ])
        .map(FontData::from_owned);
        // Static faces need a separate bold file; variable faces accept a weight axis.
        let bold = bold_path.and_then(|path| std::fs::read(path).ok());
        Some(super::SystemFonts {
            regular: super::weighted(&text, 400.0),
            semibold: bold
                .map(FontData::from_owned)
                .unwrap_or_else(|| super::weighted(&text, 700.0)),
            mono,
        })
    }

    fn desktop_setting(key: &str) -> Option<String> {
        let desktop = std::env::var("XDG_CURRENT_DESKTOP")
            .unwrap_or_default()
            .to_ascii_lowercase();
        let schema = if desktop.contains("cinnamon") {
            "org.cinnamon.desktop.interface"
        } else {
            "org.gnome.desktop.interface"
        };
        let output = std::process::Command::new("gsettings")
            .args(["get", schema, key])
            .output()
            .ok()?;
        output.status.success().then(|| {
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .trim_matches('\'')
                .to_owned()
        })
    }

    fn parse_font(description: &str) -> Option<(String, f32)> {
        let (family, size) = description.trim().rsplit_once(' ')?;
        let pixels = size.ends_with("px");
        let size: f32 = size.trim_end_matches("px").parse().ok()?;
        let size = if pixels { size } else { size * 96.0 / 72.0 };
        (!family.trim().is_empty() && size.is_finite() && size > 0.0 && size <= 256.0)
            .then(|| (family.trim().to_owned(), size))
    }

    fn matched_font(family: &str, weight: &str, numeric_weight: f32) -> Option<FontData> {
        let pattern = format!("{family}:weight={weight}");
        let output = std::process::Command::new("fc-match")
            .args(["--format=%{file}\n%{index}", "--", &pattern])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let output = String::from_utf8(output.stdout).ok()?;
        let mut lines = output.lines();
        let bytes = std::fs::read(lines.next()?).ok()?;
        let mut data = super::weighted(&bytes, numeric_weight);
        data.index = lines
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0)
            & 0xffff;
        Some(data)
    }

    pub(super) fn schemes() -> Option<crate::Schemes> {
        let theme = desktop_setting("gtk-theme")?;
        // A theme name is one directory component, not a path supplied to a shell.
        if theme.is_empty() || theme.contains(['/', '\\']) || theme == "." || theme == ".." {
            return None;
        }
        let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
        let mut roots = Vec::new();
        if let Some(data) = std::env::var_os("XDG_DATA_HOME") {
            roots.push(std::path::PathBuf::from(data).join("themes"));
        } else if let Some(home) = &home {
            roots.push(home.join(".local/share/themes"));
        }
        if let Some(home) = &home {
            roots.push(home.join(".themes"));
        }
        for root in std::env::split_paths(
            &std::env::var_os("XDG_DATA_DIRS")
                .unwrap_or_else(|| "/usr/local/share:/usr/share".into()),
        ) {
            roots.push(root.join("themes"));
        }
        roots.into_iter().find_map(|root| {
            let root = root.join(&theme).join("libadwaita-1.5");
            let dark = std::fs::read_to_string(root.join("defaults-dark.css")).ok()?;
            let light = std::fs::read_to_string(root.join("defaults-light.css")).ok()?;
            Some(crate::Schemes {
                dark: palette_from_css(&dark, true),
                light: palette_from_css(&light, false),
            })
        })
    }

    fn palette_from_css(css: &str, dark: bool) -> crate::Palette {
        let defaults = crate::Schemes::for_look(crate::Look::Gtk);
        let mut palette = if dark { defaults.dark } else { defaults.light };
        let mut clean = String::new();
        let mut rest = css;
        while let Some((before, comment)) = rest.split_once("/*") {
            clean.push_str(before);
            rest = comment.split_once("*/").map_or("", |(_, after)| after);
        }
        clean.push_str(rest);
        let definitions: std::collections::HashMap<_, _> = clean
            .split("@define-color")
            .skip(1)
            .filter_map(|part| {
                let declaration = part.split_once(';')?.0.trim();
                let at = declaration.find(char::is_whitespace)?;
                Some((&declaration[..at], declaration[at..].trim()))
            })
            .collect();
        let resolve = |name| css_color(name, &definitions, 0);
        if let Some(color) = resolve("window_bg_color") {
            palette.bg = opaque(color, palette.bg);
        }
        if let Some(color) = resolve("window_fg_color") {
            palette.fg = opaque(color, palette.bg);
        }
        if let Some(color) = resolve("popover_bg_color") {
            palette.surface = opaque(color, palette.bg);
        }
        if let Some(color) = resolve("accent_bg_color") {
            palette.accent = opaque(color, palette.bg);
        }
        palette.dim = crate::lerp_color(palette.bg, palette.fg, 0.55);
        palette.disabled = crate::lerp_color(palette.bg, palette.fg, 0.5);
        palette
    }

    fn opaque([r, g, b, a]: [f32; 4], bg: egui::Color32) -> egui::Color32 {
        egui::Color32::from_rgb(
            (r * a + f32::from(bg.r()) * (1.0 - a)).round() as u8,
            (g * a + f32::from(bg.g()) * (1.0 - a)).round() as u8,
            (b * a + f32::from(bg.b()) * (1.0 - a)).round() as u8,
        )
    }

    fn css_color(
        name: &str,
        definitions: &std::collections::HashMap<&str, &str>,
        depth: usize,
    ) -> Option<[f32; 4]> {
        if depth >= 16 {
            return None;
        }
        let value = definitions.get(name)?.trim();
        if let Some(name) = value.strip_prefix('@') {
            return css_color(name, definitions, depth + 1);
        }
        if value == "white" {
            return Some([255.0, 255.0, 255.0, 1.0]);
        }
        if value == "black" {
            return Some([0.0, 0.0, 0.0, 1.0]);
        }
        if let Some(hex) = value.strip_prefix('#').filter(|s| s.len() == 6) {
            let rgb = u32::from_str_radix(hex, 16).ok()?;
            return Some([
                ((rgb >> 16) & 255) as f32,
                ((rgb >> 8) & 255) as f32,
                (rgb & 255) as f32,
                1.0,
            ]);
        }
        let (values, count) = if let Some(v) = value.strip_prefix("rgba(") {
            (v.strip_suffix(')')?, 4)
        } else {
            (value.strip_prefix("rgb(")?.strip_suffix(')')?, 3)
        };
        let values: Vec<f32> = values
            .split(',')
            .map(|s| s.trim().parse())
            .collect::<Result<_, _>>()
            .ok()?;
        if values.len() != count
            || values.iter().any(|v| !v.is_finite())
            || values[..3].iter().any(|v| !(0.0..=255.0).contains(v))
        {
            return None;
        }
        let a = if count == 4 { values[3] } else { 1.0 };
        (0.0..=1.0)
            .contains(&a)
            .then_some([values[0], values[1], values[2], a])
    }
    pub(super) fn font_size() -> Option<f32> {
        desktop_setting("font-name")
            .and_then(|s| parse_font(&s))
            .map(|(_, size)| size)
    }

    /// GNOME's style (`org.gnome.desktop.interface color-scheme`): dark when it prefers
    /// dark.
    pub fn scheme() -> Option<egui::Theme> {
        let out = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "color-scheme"])
            .output()
            .ok()?;
        out.status.success().then(|| {
            if String::from_utf8_lossy(&out.stdout).contains("dark") {
                egui::Theme::Dark
            } else {
                egui::Theme::Light
            }
        })
    }

    /// GNOME's accent (`org.gnome.desktop.interface accent-color`) as libadwaita draws it.
    pub fn accent() -> Option<u32> {
        let out = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "accent-color"])
            .output()
            .ok()?;
        let name = String::from_utf8_lossy(&out.stdout)
            .trim()
            .trim_matches('\'')
            .to_string();
        Some(match name.as_str() {
            "blue" => 0x3584e4,
            "teal" => 0x2190a4,
            "green" => 0x3a944a,
            "yellow" => 0xc88800,
            "orange" => 0xed5b00,
            "red" => 0xe62d42,
            "pink" => 0xd56199,
            "purple" => 0x9141ac,
            "slate" => 0x6f8396,
            _ => return None,
        })
    }
    #[cfg(test)]
    mod font_tests {
        use super::{palette_from_css, parse_font};
        #[test]
        fn theme_colours_resolve_aliases_and_composite_text() {
            let p=palette_from_css("@define-color base #222226; @define-color window_bg_color @base; @define-color window_fg_color rgba(255,255,255,0.5); @define-color popover_bg_color #383838;",true);
            assert_eq!(p.bg, egui::Color32::from_rgb(34, 34, 38));
            assert_eq!(p.fg, egui::Color32::from_rgb(145, 145, 147));
            assert_eq!(p.surface, egui::Color32::from_gray(56));
        }
        #[test]
        fn unsupported_or_cyclic_theme_colours_keep_defaults() {
            let default = crate::Schemes::for_look(crate::Look::Gtk).dark;
            for css in [
                "@define-color window_bg_color @window_bg_color;",
                "@define-color window_bg_color rgba(0,0,0,NaN);",
                "@define-color window_bg_color #oops;",
                "/* @define-color window_bg_color #000000; */",
            ] {
                assert_eq!(palette_from_css(css, true).bg, default.bg);
            }
        }
        #[test]
        fn desktop_font_description_keeps_family_and_converts_points() {
            assert_eq!(parse_font("Noto Sans 12"), Some(("Noto Sans".into(), 16.0)));
            assert_eq!(parse_font("Ubuntu 15px"), Some(("Ubuntu".into(), 15.0)));
            for invalid in ["", "12", "Sans NaN", "Sans -2", "Sans 9999"] {
                assert!(parse_font(invalid).is_none());
            }
        }
    }
}

#[cfg(all(target_os = "android", feature = "system"))]
mod android {
    use egui::FontData;

    /// Roboto from the system's fonts: one variable face for both weights since Android 12
    /// (medium, 500, for labels), else its regular and medium files.
    pub fn fonts() -> Option<super::SystemFonts> {
        let regular = std::fs::read("/system/fonts/Roboto-Regular.ttf").ok()?;
        let semibold = match std::fs::read("/system/fonts/Roboto-Medium.ttf") {
            Ok(medium) => FontData::from_owned(medium),
            Err(_) => super::weighted(&regular, 500.0),
        };
        let mono = std::fs::read("/system/fonts/DroidSansMono.ttf")
            .ok()
            .map(FontData::from_owned);
        Some(super::SystemFonts {
            regular: super::weighted(&regular, 400.0),
            semibold,
            mono,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{follow, Follow};

    #[test]
    fn an_idle_system_follower_does_not_request_more_frames() {
        let ctx = egui::Context::default();
        // Start after the initial system read has completed. Its completion is allowed
        // to wake the UI once; an idle follower must not keep the UI repainting.
        ctx.data_mut(|data| {
            data.insert_temp(
                egui::Id::unique("egui_native_look::Follow"),
                Follow {
                    last: Some(0.0),
                    pending: None,
                },
            )
        });
        for frame in 0..8 {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    time: Some(f64::from(frame) * 0.01),
                    ..Default::default()
                },
                |ui| follow(ui.ctx()),
            );
            output.textures_delta.clear();
            if frame >= 3 {
                let delay = output.viewport_output[&egui::ViewportId::ROOT].repaint_delay;
                assert!(
                    delay > std::time::Duration::from_secs(60),
                    "idle follower requested repaint after {delay:?}"
                );
            }
        }
    }
}
