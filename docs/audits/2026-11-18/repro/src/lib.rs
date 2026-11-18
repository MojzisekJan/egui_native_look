#[cfg(test)]
mod tests {
    use egui_native_look as look;
    fn frame(ctx: &egui::Context, events: Vec<egui::Event>, mut add: impl FnMut(&mut egui::Ui)) {
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(640., 480.),
                )),
                events,
                ..Default::default()
            },
            |ui| add(ui),
        );
        out.textures_delta.clear();
    }
    fn key(key: egui::Key) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }
    }
    #[test]
    fn numeric_ime_arrow_must_not_change_model() {
        let ctx = egui::Context::default();
        look::install(&ctx, look::Theme::new(look::Look::Gtk).accent(None));
        let mut value = 5.;
        let mut id = None;
        frame(&ctx, vec![], |ui| {
            let r = ui.add(look::NumberField::new(&mut value));
            id = Some(r.id);
        });
        ctx.memory_mut(|m| m.request_focus(id.unwrap()));
        frame(&ctx, vec![], |ui| {
            ui.add(look::NumberField::new(&mut value));
        });
        frame(
            &ctx,
            vec![
                egui::Event::Ime(egui::ImeEvent::Preedit {
                    text: "に".into(),
                    active_range_chars: None,
                }),
                key(egui::Key::ArrowUp),
            ],
            |ui| {
                ui.add(look::NumberField::new(&mut value));
            },
        );
        assert_eq!(
            value, 5.,
            "IME candidate navigation changed the numeric model"
        );
    }
    #[test]
    fn wide_integer_must_not_change_without_input() {
        let mut failures = Vec::new();
        for chosen in [
            look::Look::Gtk,
            look::Look::Mac,
            look::Look::Plain,
            look::Look::Windows,
            look::Look::Ios,
            look::Look::Material,
        ] {
            let ctx = egui::Context::default();
            look::install(&ctx, look::Theme::new(chosen).accent(None));
            let initial = 9_007_199_254_740_993u64;
            let mut value = initial;
            let mut changed = false;
            frame(&ctx, vec![], |ui| {
                changed = ui
                    .add(
                        look::Slider::new(&mut value, 0..=u64::MAX)
                            .clamping(egui::SliderClamping::Always),
                    )
                    .changed();
            });
            if value != initial {
                failures.push(format!(
                    "{chosen:?}: {initial} -> {value}, changed={changed}"
                ));
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("; "));
    }
    #[test]
    fn maximum_corner_radius_does_not_panic() {
        let ctx = egui::Context::default();
        frame(&ctx, vec![], |ui| {
            look::focus_ring(
                ui.painter(),
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(100., 40.)),
                egui::CornerRadius::same(255),
            );
        });
    }
}
