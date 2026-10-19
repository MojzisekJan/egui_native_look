//! What the system offers: its text faces, which an app may not ship, and its accent.

use egui::FontData;

use crate::Look;

/// The system's faces for a look: San Francisco on macOS, Segoe UI Variable (or Segoe
/// UI) on Windows, each with its monospaced face where there is one.
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
    let (now, focused) = ctx.input(|i| (i.time, i.events.iter().any(|e| matches!(e, egui::Event::WindowFocused(true)))));
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
            apply(ctx, Found { look, accent: system_accent(look), scheme: system_scheme() });
        } else {
            // Reading takes a process on GNOME, so it is done off the UI thread, and what it
            // finds is applied here, on a later frame.
            let (send, receive) = std::sync::mpsc::channel();
            state.pending = Some(std::sync::Arc::new(std::sync::Mutex::new(receive)));
            let repaint = ctx.clone();
            std::thread::spawn(move || {
                let _ = send.send(Found { look, accent: system_accent(look), scheme: system_scheme() });
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
        .is_some_and(|agent| ["iPhone", "iPad", "Macintosh"].iter().any(|name| agent.contains(name)));
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

/// The system's faces for `look`, read from the system's font folder; `None` where they
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

/// One variable font at the weight `weight`.
#[cfg_attr(not(all(feature = "system", any(target_os = "macos", target_os = "windows", target_os = "linux", target_os = "android"))), allow(dead_code))]
fn weighted(bytes: &[u8], weight: f32) -> FontData {
    let mut data = FontData::from_owned(bytes.to_vec());
    data.tweak = egui::FontTweak { coords: egui::epaint::text::VariationCoords::new([(b"wght", weight)]), ..Default::default() };
    data
}

#[cfg(all(target_os = "macos", feature = "system"))]
mod mac {
    use egui::FontData;

    /// San Francisco, one variable font for both weights: regular, and the semibold (590)
    /// macOS sets its emphasised labels in; SF Mono.
    pub fn fonts() -> Option<super::SystemFonts> {
        let bytes = std::fs::read("/System/Library/Fonts/SFNS.ttf").ok()?;
        let mono = std::fs::read("/System/Library/Fonts/SFNSMono.ttf").ok().map(FontData::from_owned);
        Some(super::SystemFonts { regular: super::weighted(&bytes, 400.0), semibold: super::weighted(&bytes, 590.0), mono })
    }

    /// Dark when System Settings > Appearance is dark (or automatic, and dark now): the
    /// application's appearance on the main thread, `defaults` elsewhere.
    pub fn scheme() -> Option<egui::Theme> {
        if let Some(main) = objc2::MainThreadMarker::new() {
            let name = objc2_app_kit::NSApplication::sharedApplication(main).effectiveAppearance().name().to_string();
            return Some(if name.contains("Dark") { egui::Theme::Dark } else { egui::Theme::Light });
        }
        let out = std::process::Command::new("defaults").args(["read", "-g", "AppleInterfaceStyle"]).output().ok()?;
        // Light has no value at all, and `defaults` fails for it.
        Some(if String::from_utf8_lossy(&out.stdout).trim() == "Dark" { egui::Theme::Dark } else { egui::Theme::Light })
    }

    /// The accent colour of System Settings > Appearance, as 0xRRGGBB in sRGB.
    pub fn accent() -> Option<u32> {
        use objc2_app_kit::{NSColor, NSColorSpace};
        let color = NSColor::controlAccentColor().colorUsingColorSpace(&NSColorSpace::sRGBColorSpace())?;
        let channel = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u32;
        Some(channel(color.redComponent()) << 16 | channel(color.greenComponent()) << 8 | channel(color.blueComponent()))
    }
}

#[cfg(all(target_os = "windows", feature = "system"))]
mod windows {
    use egui::FontData;

    fn fonts_dir() -> std::path::PathBuf {
        std::env::var_os("WINDIR").map(std::path::PathBuf::from).unwrap_or_else(|| "C:\\Windows".into()).join("Fonts")
    }

    /// Segoe UI Variable (Windows 11), one variable font for both weights, else Segoe UI
    /// and its semibold; Cascadia Mono, else Consolas.
    pub fn fonts() -> Option<super::SystemFonts> {
        let dir = fonts_dir();
        let (regular, semibold) = if let Ok(bytes) = std::fs::read(dir.join("SegUIVar.ttf")) {
            (super::weighted(&bytes, 400.0), super::weighted(&bytes, 600.0))
        } else {
            let regular = std::fs::read(dir.join("segoeui.ttf")).ok()?;
            let semibold = std::fs::read(dir.join("seguisb.ttf")).unwrap_or_else(|_| regular.clone());
            (FontData::from_owned(regular), FontData::from_owned(semibold))
        };
        let mono = ["CascadiaMono.ttf", "consola.ttf"].iter().find_map(|name| std::fs::read(dir.join(name)).ok()).map(FontData::from_owned);
        Some(super::SystemFonts { regular, semibold, mono })
    }

    /// The apps' mode of Personalization > Colors.
    pub fn scheme() -> Option<egui::Theme> {
        use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
        let wide = |s: &str| s.encode_utf16().chain([0]).collect::<Vec<u16>>();
        let key = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
        let name = wide("AppsUseLightTheme");
        let mut light = 0u32;
        let mut size = 4u32;
        // SAFETY: the strings are NUL-terminated and the value is a four-byte DWORD.
        let status = unsafe { RegGetValueW(HKEY_CURRENT_USER, key.as_ptr(), name.as_ptr(), RRF_RT_REG_DWORD, std::ptr::null_mut(), (&mut light as *mut u32).cast(), &mut size) };
        (status == 0).then_some(if light == 0 { egui::Theme::Dark } else { egui::Theme::Light })
    }

    /// The accent's light shade (for the dark scheme) and dark shade (for the light), as
    /// 0xRRGGBB, from the palette Windows keeps beside the chosen accent.
    pub fn accent() -> Option<[u32; 2]> {
        use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_BINARY};
        let wide = |s: &str| s.encode_utf16().chain([0]).collect::<Vec<u16>>();
        let key = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Accent");
        let name = wide("AccentPalette");
        let mut palette = [0u8; 32];
        let mut size = palette.len() as u32;
        // SAFETY: the strings are NUL-terminated and the buffer is `size` bytes long.
        let status = unsafe { RegGetValueW(HKEY_CURRENT_USER, key.as_ptr(), name.as_ptr(), RRF_RT_REG_BINARY, std::ptr::null_mut(), palette.as_mut_ptr().cast(), &mut size) };
        if status != 0 || size < 20 {
            return None;
        }
        // Eight RGBA entries: Light 3, Light 2, Light 1, the accent, Dark 1, Dark 2, Dark 3.
        let rgb = |i: usize| u32::from(palette[i * 4]) << 16 | u32::from(palette[i * 4 + 1]) << 8 | u32::from(palette[i * 4 + 2]);
        Some([rgb(1), rgb(4)])
    }
}

#[cfg(all(target_os = "linux", feature = "system"))]
mod gnome {
    use egui::FontData;

    /// Adwaita Sans (GNOME 48), else Cantarell, one variable font for both weights where it
    /// is one; Adwaita Mono, else DejaVu Sans Mono. The folders of the common distributions.
    pub fn fonts() -> Option<super::SystemFonts> {
        let first = |paths: &[&str]| paths.iter().find_map(|p| std::fs::read(p).ok());
        let text = first(&[
            "/usr/share/fonts/adwaita-sans-fonts/AdwaitaSans-Regular.ttf",
            "/usr/share/fonts/truetype/adwaita/AdwaitaSans-Regular.ttf",
            "/usr/share/fonts/Adwaita/AdwaitaSans-Regular.ttf",
            "/usr/share/fonts/cantarell/Cantarell-VF.otf",
            "/usr/share/fonts/opentype/cantarell/Cantarell-VF.otf",
            "/usr/share/fonts/abattis-cantarell-vf-fonts/Cantarell-VF.otf",
        ])?;
        let mono = first(&[
            "/usr/share/fonts/adwaita-mono-fonts/AdwaitaMono-Regular.ttf",
            "/usr/share/fonts/truetype/adwaita/AdwaitaMono-Regular.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
            "/usr/share/fonts/dejavu-sans-mono-fonts/DejaVuSansMono.ttf",
        ])
        .map(FontData::from_owned);
        // Both faces are variable: semibold is weight 600, as libadwaita's headings take.
        Some(super::SystemFonts { regular: super::weighted(&text, 400.0), semibold: super::weighted(&text, 600.0), mono })
    }

    /// GNOME's style (`org.gnome.desktop.interface color-scheme`): dark when it prefers
    /// dark.
    pub fn scheme() -> Option<egui::Theme> {
        let out = std::process::Command::new("gsettings").args(["get", "org.gnome.desktop.interface", "color-scheme"]).output().ok()?;
        out.status.success().then(|| if String::from_utf8_lossy(&out.stdout).contains("dark") { egui::Theme::Dark } else { egui::Theme::Light })
    }

    /// GNOME's accent (`org.gnome.desktop.interface accent-color`) as libadwaita draws it.
    pub fn accent() -> Option<u32> {
        let out = std::process::Command::new("gsettings").args(["get", "org.gnome.desktop.interface", "accent-color"]).output().ok()?;
        let name = String::from_utf8_lossy(&out.stdout).trim().trim_matches('\'').to_string();
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
        let mono = std::fs::read("/system/fonts/DroidSansMono.ttf").ok().map(FontData::from_owned);
        Some(super::SystemFonts { regular: super::weighted(&regular, 400.0), semibold, mono })
    }
}
