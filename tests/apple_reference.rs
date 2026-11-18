//! Contracts measured by tools/apple-parity's independent SwiftUI fixture.
use egui_native_look as look;

fn frame<R>(
    ctx: &egui::Context,
    events: Vec<egui::Event>,
    mut draw: impl FnMut(&mut egui::Ui) -> R,
) -> R {
    let mut value = None;
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400., 800.),
            )),
            events,
            ..Default::default()
        },
        |ui| value = Some(draw(ui)),
    );
    output.textures_delta.clear();
    value.unwrap()
}

fn pointer(pos: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        pressed,
        button: egui::PointerButton::Primary,
        modifiers: egui::Modifiers::NONE,
    }
}

#[test]
fn mac_button_tracks_a_pointer_excursion_until_release_like_swiftui() {
    for (touch, return_inside, enabled_on_release, expected) in [
        (false, true, true, 1),
        (false, false, true, 0),
        (false, true, false, 0),
        (true, true, true, 0),
    ] {
        let ctx = egui::Context::default();
        look::install(&ctx, look::Theme::new(look::Look::Mac));
        let mut count = 0;
        let mut enabled = true;
        let mut render = |ui: &mut egui::Ui| {
            let r = ui.add_enabled(enabled, look::Button::new("Save"));
            count += usize::from(r.clicked());
            r
        };
        let rect = frame(&ctx, vec![], &mut render).rect;
        let inside = rect.center();
        let outside = inside + egui::vec2(200., 0.);
        frame(&ctx, vec![egui::Event::PointerMoved(inside)], &mut render);
        let mut events = vec![pointer(inside, true)];
        if touch {
            events.push(egui::Event::Touch {
                device_id: egui::TouchDeviceId(1),
                id: egui::TouchId(1),
                phase: egui::TouchPhase::Start,
                pos: inside,
                force: None,
            });
        }
        frame(&ctx, events, &mut render);
        frame(&ctx, vec![egui::Event::PointerMoved(outside)], &mut render);
        let at = if return_inside { inside } else { outside };
        frame(&ctx, vec![egui::Event::PointerMoved(at)], &mut render);
        drop(render);
        enabled = enabled_on_release;
        frame(&ctx, vec![pointer(at, false)], |ui| {
            count += usize::from(ui.add_enabled(enabled, look::Button::new("Save")).clicked());
        });
        assert_eq!(
            count, expected,
            "touch={touch}, return={return_inside}, enabled={enabled_on_release}"
        );
    }
}

#[test]
fn ios_regular_control_geometry_matches_swiftui_26_5() {
    let ctx = egui::Context::default();
    look::install(&ctx, look::Theme::new(look::Look::Ios));
    let mut on = true;
    let mut choice = 1;
    let (switch, segments) = frame(&ctx, vec![], |ui| {
        (
            ui.add(look::Switch::new(&mut on)).rect.size(),
            ui.add(look::Segmented::new("mode", &mut choice, &["One", "Two", "Three"]).width(216.))
                .rect
                .size(),
        )
    });
    assert_eq!(switch, egui::vec2(61., 28.));
    assert_eq!(segments, egui::vec2(216., 31.));
}

#[test]
fn ios_segment_capsule_keeps_native_height_inside_its_touch_target() {
    let ctx = egui::Context::default();
    look::install(&ctx, look::Theme::new(look::Look::Ios));
    let pos = egui::pos2(390., 790.);
    frame(
        &ctx,
        vec![
            egui::Event::Touch {
                device_id: egui::TouchDeviceId(1),
                id: egui::TouchId(1),
                phase: egui::TouchPhase::Start,
                pos,
                force: None,
            },
            egui::Event::Touch {
                device_id: egui::TouchDeviceId(1),
                id: egui::TouchId(1),
                phase: egui::TouchPhase::End,
                pos,
                force: None,
            },
        ],
        |_| {},
    );
    let mut selected = 1;
    let mut bounds = egui::Rect::NOTHING;
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        bounds = ui
            .add(look::Segmented::new("mode", &mut selected, &["One", "Two", "Three"]).width(216.))
            .rect;
    });
    output.textures_delta.clear();
    assert!(bounds.height() >= 44.);
    assert!(output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Rect(rect) if rect.rect.width() == 216. && rect.rect.height() == 31.)));
    // The native-sized capsule must not leave the rest of its reserved target inert.
    let at = egui::pos2(bounds.left() + 12., bounds.top() + 1.);
    for events in [
        vec![egui::Event::PointerMoved(at)],
        vec![pointer(at, true)],
        vec![pointer(at, false)],
    ] {
        frame(&ctx, events, |ui| {
            ui.add(
                look::Segmented::new("mode", &mut selected, &["One", "Two", "Three"]).width(216.),
            );
        });
    }
    assert_eq!(
        selected, 0,
        "a tap in the reserved padding selects its segment"
    );
}

#[test]
fn disabled_ios_slider_preserves_its_tint_and_fades_its_thumb() {
    let ctx = egui::Context::default();
    look::install(
        &ctx,
        look::Theme::new(look::Look::Ios).accent(Some([0x0091ff, 0x0088ff])),
    );
    ctx.set_theme(egui::Theme::Dark);
    let mut value = 0.4;
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        ui.add_enabled(
            false,
            look::Slider::new(&mut value, 0.0..=1.0)
                .show_value(false)
                .width(216.),
        );
    });
    output.textures_delta.clear();
    for color in [
        egui::Color32::from_rgb(0, 73, 128),
        egui::Color32::from_gray(128),
    ] {
        assert!(
            output
                .shapes
                .iter()
                .any(|shape| matches!(&shape.shape, egui::Shape::Rect(rect) if rect.fill == color)),
            "missing measured disabled colour {color:?}"
        );
    }
    assert_eq!(value, 0.4);
}

#[cfg(all(target_os = "macos", feature = "system"))]
#[test]
fn ios_preview_on_mac_uses_the_system_font_instead_of_the_fallback() {
    let mut fonts = egui::FontDefinitions::default();
    assert!(look::install_fonts(&mut fonts, look::Look::Ios));
}

#[test]
fn normal_window_default_action_filters_shortcuts_and_chooses_first() {
    for (modifiers, enabled, expected) in [
        (egui::Modifiers::NONE, true, true),
        (egui::Modifiers::SHIFT, true, true),
        (egui::Modifiers::COMMAND, true, false),
        (egui::Modifiers::ALT, true, false),
        (egui::Modifiers::NONE, false, false),
    ] {
        let ctx = egui::Context::default();
        look::install(&ctx, look::Theme::new(look::Look::Mac));
        let mut draw = |ui: &mut egui::Ui| {
            look::window_actions(
                ui,
                |_| {},
                |ui| {
                    let first = ui
                        .add_enabled(enabled, look::Button::new("Save").default_action())
                        .clicked();
                    let second = ui
                        .add_enabled(enabled, look::Button::new("Other").default_action())
                        .clicked();
                    (first, second)
                },
            )
            .1
        };
        frame(&ctx, vec![], &mut draw);
        let result = frame(
            &ctx,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            }],
            &mut draw,
        );
        assert_eq!(result, (expected, false));
    }
}

#[test]
fn ios_control_size_keeps_fields_switches_and_sliders_independent() {
    for (metrics, button_height, segment_height) in [
        (look::Metrics::small(look::Look::Ios), 28., 31.),
        (look::Metrics::of(look::Look::Ios), 103. / 3., 31.),
        (look::Metrics::large(look::Look::Ios), 151. / 3., 48.),
    ] {
        let ctx = egui::Context::default();
        ctx.set_pixels_per_point(3.0);
        let mut theme = look::Theme::new(look::Look::Ios);
        theme.metrics = metrics;
        look::install(&ctx, theme);
        frame(&ctx, vec![], |ui| {
            let mut text = String::from("Hello");
            let mut on = true;
            let mut selected = 0;
            let mut value = 0.5;
            assert!(
                (ui.add(look::Button::new("Save")).rect.height() - button_height).abs() <= 0.34
            );
            assert!(
                (ui.add(look::TextField::new(&mut text)).rect.height() - 34.0).abs() <= 1.0 / 3.0
            );
            assert_eq!(ui.add(look::Switch::new(&mut on)).rect.height(), 28.);
            assert_eq!(
                ui.add(look::Segmented::new("mode", &mut selected, &["One", "Two"]))
                    .rect
                    .height(),
                segment_height
            );
            assert_eq!(
                ui.add(look::Slider::new(&mut value, 0.0..=1.0).show_value(false))
                    .rect
                    .height(),
                31.
            );
        });
    }
}

#[test]
fn window_default_action_waits_for_text_composition() {
    for composing in [false, true] {
        let ctx = egui::Context::default();
        look::install(&ctx, look::Theme::new(look::Look::Mac));
        let mut text = String::from("Name");
        let mut draw = |ui: &mut egui::Ui| {
            look::window_actions(
                ui,
                |ui| ui.add(look::TextField::new(&mut text)).id,
                |ui| ui.add(look::Button::new("Save").default_action()).clicked(),
            )
        };
        let (id, _) = frame(&ctx, vec![], &mut draw);
        ctx.memory_mut(|m| m.request_focus(id));
        frame(&ctx, vec![], &mut draw);
        if composing {
            frame(
                &ctx,
                vec![egui::Event::Ime(egui::ImeEvent::Preedit {
                    text: "a".into(),
                    active_range_chars: None,
                })],
                &mut draw,
            );
        }
        let (_, clicked) = frame(
            &ctx,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            &mut draw,
        );
        assert_eq!(clicked, !composing);
    }
}

#[test]
fn nested_window_actions_consume_enter_only_once() {
    let ctx = egui::Context::default();
    look::install(&ctx, look::Theme::new(look::Look::Mac));
    let mut draw = |ui: &mut egui::Ui| {
        look::window_actions(
            ui,
            |ui| {
                look::window_actions(
                    ui,
                    |_| {},
                    |ui| {
                        ui.add(look::Button::new("Inner").default_action())
                            .clicked()
                    },
                )
                .1
            },
            |ui| {
                ui.add(look::Button::new("Outer").default_action())
                    .clicked()
            },
        )
    };
    frame(&ctx, vec![], &mut draw);
    let clicks = frame(
        &ctx,
        vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
        &mut draw,
    );
    assert_eq!(clicks, (true, false));
}

#[test]
fn window_default_ignores_repeat_and_window_focus_loss() {
    for events in [
        vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: true,
            modifiers: egui::Modifiers::NONE,
        }],
        vec![
            egui::Event::WindowFocused(false),
            egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    ] {
        let ctx = egui::Context::default();
        look::install(&ctx, look::Theme::new(look::Look::Mac));
        let mut draw = |ui: &mut egui::Ui| {
            look::window_actions(
                ui,
                |_| {},
                |ui| ui.add(look::Button::new("Save").default_action()).clicked(),
            )
            .1
        };
        frame(&ctx, vec![], &mut draw);
        if matches!(events.first(), Some(egui::Event::Key { repeat: true, .. })) {
            // egui derives repeat from held-key state, not the incoming flag.
            frame(
                &ctx,
                vec![egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }],
                &mut draw,
            );
        }
        let description = format!("{events:?}");
        assert!(!frame(&ctx, events, &mut draw), "{description}");
    }
}

#[test]
fn mac_focused_switch_offers_return_to_window_default_but_keeps_space() {
    let ctx = egui::Context::default();
    look::install(&ctx, look::Theme::new(look::Look::Mac));
    let mut on = true;
    let mut draw = |ui: &mut egui::Ui| {
        look::window_actions(
            ui,
            |ui| ui.add(look::Switch::new(&mut on)).id,
            |ui| ui.add(look::Button::new("Save").default_action()).clicked(),
        )
    };
    let (id, _) = frame(&ctx, vec![], &mut draw);
    ctx.memory_mut(|m| m.request_focus(id));
    let key = |key| egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    };
    assert!(frame(&ctx, vec![key(egui::Key::Enter)], &mut draw).1);
    drop(draw);
    assert!(on, "Return must not toggle the switch");
    let mut draw = |ui: &mut egui::Ui| {
        look::window_actions(
            ui,
            |ui| ui.add(look::Switch::new(&mut on)).id,
            |ui| ui.add(look::Button::new("Save").default_action()).clicked(),
        )
    };
    assert!(!frame(&ctx, vec![key(egui::Key::Space)], &mut draw).1);
    drop(draw);
    assert!(!on, "Space still toggles the switch");
}

#[test]
fn mac_segmented_return_activates_default_without_changing_selection() {
    let ctx = egui::Context::default();
    look::install(&ctx, look::Theme::new(look::Look::Mac));
    let mut selected = 1;
    let mut draw = |ui: &mut egui::Ui| {
        look::window_actions(
            ui,
            |ui| {
                ui.add(look::Segmented::new(
                    "mode",
                    &mut selected,
                    &["One", "Two", "Three"],
                ))
                .id
            },
            |ui| ui.add(look::Button::new("Save").default_action()).clicked(),
        )
    };
    let (id, _) = frame(&ctx, vec![], &mut draw);
    ctx.memory_mut(|m| m.request_focus(id));
    assert!(
        frame(
            &ctx,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE
            }],
            &mut draw
        )
        .1
    );
    drop(draw);
    assert_eq!(selected, 1);
}

#[cfg(all(target_os = "macos", feature = "system"))]
#[test]
fn apple_tracking_uses_logical_point_size_at_every_display_scale() {
    let mut widths = Vec::new();
    for scale in [1.0, 3.0] {
        let ctx = egui::Context::default();
        ctx.set_pixels_per_point(scale);
        let mut fonts = egui::FontDefinitions::default();
        assert!(look::install_fonts(&mut fonts, look::Look::Ios));
        ctx.set_fonts(fonts);
        let pair = frame(&ctx, vec![], |ui| {
            ui.fonts_mut(|fonts| {
                [13.0, 17.0].map(|size| {
                    fonts
                        .layout_no_wrap(
                            "Save".into(),
                            egui::FontId::proportional(size),
                            egui::Color32::BLACK,
                        )
                        .size()
                        .x
                })
            })
        });
        assert!(pair[0]/13.0-pair[1]/17.0>0.05,"SF tracking must vary with point size, not remain at the shaper's 12pt fallback: {pair:?}");
        widths.push(pair);
    }
    for i in 0..2 {
        assert!(
            (widths[0][i] - widths[1][i]).abs() < 0.05,
            "Display scaling changed tracking: {widths:?}"
        );
    }
}

#[cfg(all(target_os = "macos", feature = "system"))]
#[test]
fn apple_default_label_weights_match_native_platform_roles() {
    for chosen in [look::Look::Mac, look::Look::Ios] {
        let ctx = egui::Context::default();
        let mut fonts = egui::FontDefinitions::default();
        assert!(look::install_fonts(&mut fonts, chosen));
        ctx.set_fonts(fonts);
        look::install(&ctx, look::Theme::new(chosen));
        let (ordinary, default) = frame(&ctx, vec![], |ui| {
            (
                ui.add(look::Button::new("Continue")).rect.width(),
                ui.add(look::Button::new("Continue").default_action())
                    .rect
                    .width(),
            )
        });
        if chosen == look::Look::Mac {
            assert_eq!(ordinary, default);
        } else {
            assert!(default - ordinary >= 4.0);
        }
    }
}

#[test]
fn apple_progress_and_ios_picker_keep_native_layout_heights() {
    for look in [look::Look::Mac, look::Look::Ios] {
        for (metrics, small) in [
            (look::Metrics::small(look), true),
            (look::Metrics::of(look), false),
            (look::Metrics::large(look), false),
        ] {
            let ctx = egui::Context::default();
            let mut theme = look::Theme::new(look);
            theme.metrics = metrics;
            look::install(&ctx, theme);
            let mut selected = 0;
            frame(&ctx, vec![], |ui| {
                let progress = ui.add(look::ProgressBar::new(0.4).width(216.));
                let expected = if look == look::Look::Ios {
                    4.0
                } else if small {
                    12.0
                } else {
                    20.0
                };
                assert_eq!(progress.rect.height(), expected);
                if look == look::Look::Ios {
                    let picker = ui.add(
                        look::Dropdown::new("choice", &mut selected, &["One", "Two"]).width(160.),
                    );
                    assert!((picker.rect.height() - 103.0 / 3.0).abs() < 0.04);
                }
            });
        }
    }
}
