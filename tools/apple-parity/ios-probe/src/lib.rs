//! Invoked by the native SwiftUI fixture inside the simulator, not on the host Mac.
use egui_native_look as look;

/// # Safety
/// `path` must point to a valid NUL-terminated UTF-8 pathname during this call.
#[no_mangle]
pub unsafe extern "C" fn apple_parity_probe(path: *const std::ffi::c_char) -> bool {
    let run = std::panic::catch_unwind(|| {
        if path.is_null() {
            return false;
        }
        // SAFETY: the caller supplies a valid NUL-terminated pathname for this call.
        let path = unsafe { std::ffi::CStr::from_ptr(path) }.to_str().unwrap();
        let system = look::system_fonts(look::Look::Ios);
        let bytes = system
            .as_ref()
            .map(|fonts| fonts.regular.font.as_ref().as_ref().len());
        let mut fonts = egui::FontDefinitions::default();
        let installed = look::install_fonts(&mut fonts, look::Look::Ios);
        let ctx = egui::Context::default();
        ctx.set_fonts(fonts);
        let mut theme = look::Theme::new(look::Look::Ios);
        theme.metrics = theme.metrics.system_bold();
        look::install(&ctx, theme);
        let mut width = 0.;
        let mut controls = serde_json::json!({});
        for _ in 0..3 {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400., 800.),
                    )),
                    ..Default::default()
                },
                |ui| {
                    width = ui.fonts_mut(|fonts| {
                        fonts
                            .layout_no_wrap(
                                "Save".into(),
                                egui::FontId::proportional(17.),
                                egui::Color32::BLACK,
                            )
                            .size()
                            .x
                    });
                    let mut on = true;
                    let mut selected = 1;
                    for (name, r) in [
                        ("button", ui.add(look::Button::new("Save"))),
                        ("switch", ui.add(look::Switch::new(&mut on))),
                        (
                            "segments",
                            ui.add(
                                look::Segmented::new(
                                    "mode",
                                    &mut selected,
                                    &["One", "Two", "Three"],
                                )
                                .width(216.),
                            ),
                        ),
                    ] {
                        controls[name] = serde_json::json!([r.rect.width(), r.rect.height()]);
                    }
                },
            );
            output.textures_delta.clear();
        }
        std::fs::write(path,serde_json::to_vec_pretty(&serde_json::json!({"target":std::env::consts::OS,"system_fonts":system.is_some(),"installed":installed,"font_bytes":bytes,"save_width":width,"controls":controls})).unwrap()).is_ok()
    });
    run.unwrap_or(false)
}
