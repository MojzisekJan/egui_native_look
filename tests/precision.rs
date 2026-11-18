//! Exact integer editing through public controls, keyboard and accessibility.
use egui_native_look as look;
const LOOKS: [look::Look; 6] = [
    look::Look::Plain,
    look::Look::Mac,
    look::Look::Windows,
    look::Look::Gtk,
    look::Look::Ios,
    look::Look::Material,
];
fn context(chosen: look::Look) -> egui::Context {
    let ctx = egui::Context::default();
    look::install(&ctx, look::Theme::new(chosen).accent(None));
    ctx
}
fn frame(
    ctx: &egui::Context,
    events: Vec<egui::Event>,
    mut add: impl FnMut(&mut egui::Ui) -> egui::Response,
) -> egui::Response {
    let mut response = None;
    let mut out = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(600., 400.),
            )),
            events,
            ..Default::default()
        },
        |ui| response = Some(add(ui)),
    );
    out.textures_delta.clear();
    response.unwrap()
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
fn wide_slider_arrows_change_one_exact_unit_even_with_collapsed_f64_bounds() {
    let low = (1u64 << 53) + 3;
    for chosen in LOOKS {
        for (start, end) in [(low, low + 1), (low + 1, low)] {
            let ctx = context(chosen);
            let mut value = start;
            frame(&ctx, vec![], |ui| {
                ui.add(look::Slider::new(&mut value, start..=end))
            })
            .request_focus();
            frame(&ctx, vec![], |ui| {
                ui.add(look::Slider::new(&mut value, start..=end))
            });
            let key_code = if chosen == look::Look::Gtk {
                if end > start {
                    egui::Key::ArrowUp
                } else {
                    egui::Key::ArrowDown
                }
            } else {
                egui::Key::ArrowRight
            };
            let response = frame(&ctx, vec![key(key_code)], |ui| {
                ui.add(look::Slider::new(&mut value, start..=end))
            });
            assert_eq!(value, end, "{chosen:?}");
            assert!(response.changed());
            assert!(response.enabled());
        }
    }
}
#[test]
fn wide_number_fields_preserve_exact_text_and_step_after_commit() {
    for chosen in LOOKS {
        for initial in [(1u64 << 53) + 1, u64::MAX - 2] {
            let ctx = context(chosen);
            let mut value = initial;
            let draw = |ui: &mut egui::Ui, value: &mut u64| {
                ui.add(look::NumberField::new(value).steppers(false))
            };
            frame(&ctx, vec![], |ui| draw(ui, &mut value)).request_focus();
            frame(&ctx, vec![], |ui| draw(ui, &mut value));
            assert_eq!(value, initial, "idle {chosen:?}");
            let response = frame(&ctx, vec![key(egui::Key::ArrowUp)], |ui| {
                draw(ui, &mut value)
            });
            assert_eq!(value, initial + 1, "step {chosen:?}");
            assert!(response.changed());
            let mut select = key(egui::Key::A);
            if let egui::Event::Key { modifiers, .. } = &mut select {
                *modifiers = egui::Modifiers::COMMAND;
            }
            frame(
                &ctx,
                vec![select, egui::Event::Text(initial.to_string())],
                |ui| draw(ui, &mut value),
            );
            let response = frame(&ctx, vec![key(egui::Key::Enter)], |ui| draw(ui, &mut value));
            assert_eq!(value, initial, "typed {chosen:?}");
            assert!(response.changed());
        }
    }
}
#[test]
fn signed_extremes_step_without_overflow_or_rounding() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        let mut value = i64::MIN;
        frame(&ctx, vec![], |ui| {
            ui.add(look::NumberField::new(&mut value).steppers(false))
        })
        .request_focus();
        frame(&ctx, vec![], |ui| {
            ui.add(look::NumberField::new(&mut value).steppers(false))
        });
        let response = frame(&ctx, vec![key(egui::Key::ArrowUp)], |ui| {
            ui.add(look::NumberField::new(&mut value).steppers(false))
        });
        assert_eq!(value, i64::MIN + 1, "{chosen:?}");
        assert!(response.changed());
    }
}
#[test]
fn wide_slider_accessibility_advertises_exact_value_and_unit_increment() {
    use egui::accesskit::{Action, ActionData, ActionRequest};
    let ctx = context(look::Look::Gtk);
    ctx.enable_accesskit();
    let mut value = (1u64 << 53) + 3;
    let response = frame(&ctx, vec![], |ui| {
        ui.add(look::Slider::new(&mut value, 0..=u64::MAX))
    });
    let event = |action, data| {
        egui::Event::AccessKitActionRequest(ActionRequest {
            action,
            target_tree: egui::accesskit::TreeId::ROOT,
            target_node: response.id.accesskit_id(),
            data,
        })
    };
    let r = frame(&ctx, vec![event(Action::Increment, None)], |ui| {
        ui.add(look::Slider::new(&mut value, 0..=u64::MAX))
    });
    assert_eq!(value, (1u64 << 53) + 4);
    assert!(r.changed());
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        ui.add(look::Slider::new(&mut value, 0..=u64::MAX));
    });
    output.textures_delta.clear();
    let nodes = output.platform_output.accesskit_update.unwrap().nodes;
    let node = &nodes
        .iter()
        .find(|(id, _)| *id == response.id.accesskit_id())
        .unwrap()
        .1;
    assert_eq!(node.value(), Some(value.to_string().as_str()));
    assert!(node.supports_action(Action::Increment));
    assert!(node.supports_action(Action::Decrement));
    let r = frame(
        &ctx,
        vec![event(
            Action::SetValue,
            Some(ActionData::Value(u64::MAX.to_string().into())),
        )],
        |ui| ui.add(look::Slider::new(&mut value, 0..=u64::MAX)),
    );
    assert_eq!(value, u64::MAX);
    assert!(r.changed());
}

#[test]
fn wide_slider_pointer_positions_use_offsets_and_preserve_exact_endpoints() {
    for chosen in LOOKS {
        for start in [i64::MIN, i64::MAX - 4] {
            for reversed in [false, true] {
                let (a, b) = if reversed {
                    (start + 4, start)
                } else {
                    (start, start + 4)
                };
                let ctx = context(chosen);
                let mut value = a;
                let draw = |ui: &mut egui::Ui, value: &mut i64| {
                    ui.add(look::Slider::new(value, a..=b).width(180.0))
                };
                let response = frame(&ctx, vec![], |ui| draw(ui, &mut value));
                let at = response.rect.center();
                let press = |pressed| egui::Event::PointerButton {
                    pos: at,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                };
                frame(&ctx, vec![egui::Event::PointerMoved(at)], |ui| {
                    draw(ui, &mut value)
                });
                let response = frame(&ctx, vec![press(true)], |ui| draw(ui, &mut value));
                assert_eq!(value, start + 2, "midpoint {chosen:?}, reversed={reversed}");
                assert!(response.changed());
                frame(&ctx, vec![press(false)], |ui| draw(ui, &mut value));
            }
        }
    }
}

#[test]
fn wide_slider_value_editor_accepts_exact_text() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        ctx.enable_accesskit();
        let mut value = (1u64 << 53) + 1;
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(600., 400.),
                )),
                ..Default::default()
            },
            |ui| {
                ui.add(look::Slider::new(&mut value, 0..=u64::MAX).show_value(true));
            },
        );
        output.textures_delta.clear();
        let node = output
            .platform_output
            .accesskit_update
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, n)| n.role() == egui::Role::SpinButton)
            .expect("value editor");
        assert_eq!(node.1.value(), Some(value.to_string().as_str()));
        let rect = node.1.bounds().unwrap();
        let at = egui::pos2(
            ((rect.x0 + rect.x1) * 0.5) as f32,
            ((rect.y0 + rect.y1) * 0.5) as f32,
        );
        let press = |pressed| egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        let draw = |ui: &mut egui::Ui, value: &mut u64| {
            ui.add(look::Slider::new(value, 0..=u64::MAX).show_value(true))
        };
        frame(
            &ctx,
            vec![egui::Event::PointerMoved(at), press(true)],
            |ui| draw(ui, &mut value),
        );
        frame(&ctx, vec![press(false)], |ui| draw(ui, &mut value));
        let mut select = key(egui::Key::A);
        if let egui::Event::Key { modifiers, .. } = &mut select {
            *modifiers = egui::Modifiers::COMMAND;
        }
        frame(
            &ctx,
            vec![select, egui::Event::Text(u64::MAX.to_string())],
            |ui| draw(ui, &mut value),
        );
        let response = frame(&ctx, vec![key(egui::Key::Enter)], |ui| draw(ui, &mut value));
        assert_eq!(value, u64::MAX, "{chosen:?}");
        assert!(response.changed());
    }
}

#[test]
fn wide_logarithmic_integer_sliders_keep_exact_keyboard_steps() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        let initial = (1u64 << 53) + 1;
        let mut value = initial;
        let draw = |ui: &mut egui::Ui, value: &mut u64| {
            ui.add(
                look::Slider::new(value, 1..=u64::MAX)
                    .logarithmic(true)
                    .step_by(1.0),
            )
        };
        frame(&ctx, vec![], |ui| draw(ui, &mut value)).request_focus();
        frame(&ctx, vec![], |ui| draw(ui, &mut value));
        let key_code = if chosen == look::Look::Gtk {
            egui::Key::ArrowUp
        } else {
            egui::Key::ArrowRight
        };
        let response = frame(&ctx, vec![key(key_code)], |ui| draw(ui, &mut value));
        assert_eq!(value, initial + 1, "{chosen:?}");
        assert!(response.changed());
    }
}

#[test]
fn unclamped_wide_slider_editor_advertises_its_full_integer_range() {
    let ctx = context(look::Look::Gtk);
    ctx.enable_accesskit();
    let mut value = u64::MAX;
    let mut response = None;
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        response = Some(
            ui.add(
                look::Slider::new(&mut value, ((1u64 << 53) + 3)..=((1u64 << 53) + 23))
                    .clamping(egui::SliderClamping::Never)
                    .show_value(true),
            ),
        );
    });
    output.textures_delta.clear();
    assert_eq!(value, u64::MAX);
    assert!(!response.unwrap().changed());
    let nodes = output.platform_output.accesskit_update.unwrap().nodes;
    let rail = &nodes
        .iter()
        .find(|(_, n)| n.role() == egui::Role::Slider)
        .unwrap()
        .1;
    assert_eq!(rail.value(), Some(((1u64 << 53) + 23).to_string().as_str()));
    let node = &nodes
        .iter()
        .find(|(_, n)| n.role() == egui::Role::SpinButton)
        .unwrap()
        .1;
    assert_eq!(node.min_numeric_value(), Some(0.0));
    assert_eq!(node.max_numeric_value(), Some(u64::MAX as f64));
    assert_eq!(node.value(), Some(u64::MAX.to_string().as_str()));
}

#[test]
fn wide_editors_preserve_other_looks_wheel_policy() {
    for chosen in LOOKS.into_iter().filter(|look| *look != look::Look::Gtk) {
        let ctx = context(chosen);
        let initial = (1u64 << 53) + 1;
        let mut value = initial;
        let draw = |ui: &mut egui::Ui, value: &mut u64| {
            ui.add(look::NumberField::new(value).steppers(false))
        };
        let response = frame(&ctx, vec![], |ui| draw(ui, &mut value));
        response.request_focus();
        frame(
            &ctx,
            vec![egui::Event::PointerMoved(response.rect.center())],
            |ui| draw(ui, &mut value),
        );
        let response = frame(
            &ctx,
            vec![egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Line,
                delta: egui::vec2(0.0, 1.0),
                modifiers: egui::Modifiers::NONE,
                phase: egui::TouchPhase::Move,
                source: egui::MouseWheelSource::Wheel,
            }],
            |ui| draw(ui, &mut value),
        );
        assert_eq!(value, initial, "{chosen:?}");
        assert!(!response.changed());
    }
}
