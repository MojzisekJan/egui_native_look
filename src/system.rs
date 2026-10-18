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

/// The system's accent as 0xRRGGBB, for the dark and the light scheme: on macOS the colour
/// of System Settings > Appearance (the same for both), on Windows the shades WinUI takes
/// from Personalization > Colors, on GNOME (46 and later) the accent of Settings >
/// Appearance. `None` elsewhere, or where the system says nothing.
pub fn system_accent(look: Look) -> Option<[u32; 2]> {
    match look {
        #[cfg(target_os = "macos")]
        Look::Mac => mac::accent().map(|rgb| [rgb, rgb]),
        #[cfg(target_os = "windows")]
        Look::Windows => windows::accent(),
        #[cfg(target_os = "linux")]
        Look::Gtk => gnome::accent().map(|rgb| [rgb, rgb]),
        _ => None,
    }
}

/// The system's faces for `look`, read from the system's font folder; `None` where they
/// are not there (another system, an old one).
pub fn system_fonts(look: Look) -> Option<SystemFonts> {
    match look {
        #[cfg(target_os = "macos")]
        Look::Mac => mac::fonts(),
        #[cfg(target_os = "windows")]
        Look::Windows => windows::fonts(),
        #[cfg(target_os = "linux")]
        Look::Gtk => gnome::fonts(),
        #[cfg(target_os = "android")]
        Look::Material => android::fonts(),
        _ => None,
    }
}

/// One variable font at the weight `weight`.
#[cfg_attr(not(any(target_os = "macos", target_os = "windows", target_os = "linux", target_os = "android")), allow(dead_code))]
fn weighted(bytes: &[u8], weight: f32) -> FontData {
    let mut data = FontData::from_owned(bytes.to_vec());
    data.tweak = egui::FontTweak { coords: egui::epaint::text::VariationCoords::new([(b"wght", weight)]), ..Default::default() };
    data
}

#[cfg(target_os = "macos")]
mod mac {
    use egui::FontData;

    /// San Francisco, one variable font for both weights: regular, and the semibold (590)
    /// macOS sets its emphasised labels in; SF Mono.
    pub fn fonts() -> Option<super::SystemFonts> {
        let bytes = std::fs::read("/System/Library/Fonts/SFNS.ttf").ok()?;
        let mono = std::fs::read("/System/Library/Fonts/SFNSMono.ttf").ok().map(FontData::from_owned);
        Some(super::SystemFonts { regular: super::weighted(&bytes, 400.0), semibold: super::weighted(&bytes, 590.0), mono })
    }

    /// The accent colour of System Settings > Appearance, as 0xRRGGBB in sRGB.
    pub fn accent() -> Option<u32> {
        use objc2_app_kit::{NSColor, NSColorSpace};
        let color = NSColor::controlAccentColor().colorUsingColorSpace(&NSColorSpace::sRGBColorSpace())?;
        let channel = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u32;
        Some(channel(color.redComponent()) << 16 | channel(color.greenComponent()) << 8 | channel(color.blueComponent()))
    }
}

#[cfg(target_os = "windows")]
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

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "android")]
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
