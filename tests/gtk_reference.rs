//! Regression cases measured against GTK 4.14 / libadwaita 1.5 on X11.
use egui_native_look as look;

fn context() -> egui::Context {
    let ctx = egui::Context::default();
    look::install(
        &ctx,
        look::Theme::new(look::Look::Gtk).accent(Some([0x3584e4; 2])),
    );
    ctx
}
fn frame<R>(
    ctx: &egui::Context,
    events: Vec<egui::Event>,
    mut draw: impl FnMut(&mut egui::Ui) -> R,
) -> R {
    let mut result = None;
    let mut out = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, 400.0),
            )),
            events,
            ..Default::default()
        },
        |ui| result = Some(draw(ui)),
    );
    out.textures_delta.clear();
    result.unwrap()
}
fn pointer(at: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos: at,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    }
}
fn wheel(delta: f32) -> egui::Event {
    egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Line,
        delta: egui::vec2(0.0, delta),
        modifiers: egui::Modifiers::NONE,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Wheel,
    }
}
#[test]
fn gtk_mouse_can_return_to_a_pressed_button() {
    let ctx = context();
    let draw = |ui: &mut egui::Ui| ui.add(look::Button::new("Save"));
    let r = frame(&ctx, vec![], draw);
    let at = r.rect.center();
    frame(&ctx, vec![egui::Event::PointerMoved(at)], draw);
    frame(&ctx, vec![pointer(at, true)], draw);
    frame(
        &ctx,
        vec![egui::Event::PointerMoved(at + egui::vec2(150.0, 0.0))],
        draw,
    );
    frame(&ctx, vec![egui::Event::PointerMoved(at)], draw);
    assert!(frame(&ctx, vec![pointer(at, false)], draw).clicked());
}
#[test]
fn gtk_numeric_drag_selects_text_without_scrubbing_value() {
    let ctx = context();
    let mut value = 4.0;
    let draw = |ui: &mut egui::Ui, value: &mut f64| {
        ui.add(look::NumberField::new(value).range(0.0..=10.0).width(240.0))
    };
    let r = frame(&ctx, vec![], |ui| draw(ui, &mut value));
    let at = r.rect.left_center() + egui::vec2(12.0, 0.0);
    frame(&ctx, vec![egui::Event::PointerMoved(at)], |ui| {
        draw(ui, &mut value)
    });
    frame(&ctx, vec![pointer(at, true)], |ui| draw(ui, &mut value));
    let end = at + egui::vec2(40.0, 0.0);
    frame(&ctx, vec![egui::Event::PointerMoved(end)], |ui| {
        draw(ui, &mut value)
    });
    frame(&ctx, vec![pointer(end, false)], |ui| draw(ui, &mut value));
    assert_eq!(value, 4.0);
}
#[test]
fn gtk_wheel_changes_only_focused_controls_and_consumes_parent_scroll() {
    for slider in [false, true] {
        let ctx = context();
        let mut value = 40.0;
        let draw = |ui: &mut egui::Ui, value: &mut f64| {
            let r = if slider {
                ui.add(
                    look::Slider::new(value, 0.0..=100.0)
                        .step_by(1.0)
                        .show_value(false),
                )
            } else {
                ui.add(look::NumberField::new(value).range(0.0..=100.0))
            };
            (r, ui.input(|i| i.smooth_scroll_delta.y))
        };
        let r = frame(&ctx, vec![], |ui| draw(ui, &mut value)).0;
        let at = if slider {
            r.rect.center()
        } else {
            r.rect.left_center() + egui::vec2(20.0, 0.0)
        };
        frame(
            &ctx,
            vec![egui::Event::PointerMoved(at), wheel(1.0)],
            |ui| draw(ui, &mut value),
        );
        assert_eq!(value, 40.0);
        ctx.memory_mut(|m| m.request_focus(r.id));
        frame(&ctx, vec![], |ui| draw(ui, &mut value));
        let (r, remaining) = frame(&ctx, vec![wheel(1.0)], |ui| draw(ui, &mut value));
        assert_eq!(
            value,
            if slider { 50.0 } else { 41.0 },
            "slider={slider}, r={r:?}, focus={:?}",
            ctx.memory(|m| m.focused())
        );
        assert!(r.changed());
        assert_eq!(remaining, 0.0);
        for _ in 0..5 {
            assert_eq!(frame(&ctx, vec![], |ui| draw(ui, &mut value)).1, 0.0);
        }
    }
}
#[test]
fn gtk_numeric_custom_formatting_and_parsing_survive_text_editing() {
    let ctx = context();
    let mut value = 4.0;
    let draw = |ui: &mut egui::Ui, value: &mut f64| {
        ui.add(
            look::NumberField::new(value)
                .range(0.0..=10.0)
                .prefix("x:")
                .suffix(" mm")
                .fixed_decimals(2)
                .custom_parser(|text| text.replace(',', ".").parse().ok()),
        )
    };
    let r = frame(&ctx, vec![], |ui| draw(ui, &mut value));
    ctx.memory_mut(|m| m.request_focus(r.id));
    frame(&ctx, vec![], |ui| draw(ui, &mut value));
    frame(&ctx, vec![egui::Event::Text("7,5".into())], |ui| {
        draw(ui, &mut value)
    });
    assert_eq!(value, 4.0);
    frame(
        &ctx,
        vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
        |ui| draw(ui, &mut value),
    );
    assert_eq!(
        value,
        7.5,
        "focus={:?}, id={:?}, textstate={:?}",
        ctx.memory(|m| m.focused()),
        r.id,
        egui::TextEdit::load_state(&ctx, r.id).and_then(|s| s.cursor.char_range())
    );
}
#[test]
fn transparent_color_wells_finish_with_tiny_metrics() {
    for chosen in [
        look::Look::Plain,
        look::Look::Gtk,
        look::Look::Mac,
        look::Look::Windows,
        look::Look::Ios,
        look::Look::Material,
    ] {
        for height in [1.0, 6.0, 8.0, 10.0] {
            let ctx = context();
            let mut theme = look::Theme::new(chosen);
            theme.metrics.control_height = height;
            look::install(&ctx, theme);
            let mut color = egui::Color32::from_white_alpha(128);
            let r = frame(&ctx, vec![], |ui| ui.add(look::ColorWell::new(&mut color)));
            assert!(r.rect.is_finite());
        }
    }
}

#[test]
fn gtk_numeric_wheel_does_not_scroll_its_parent() {
    let ctx = context();
    let mut value = 40.0;
    let draw = |ui: &mut egui::Ui, value: &mut f64| {
        let output = egui::ScrollArea::vertical()
            .max_height(120.0)
            .show(ui, |ui| {
                let response = ui.add(look::NumberField::new(value).range(0.0..=100.0));
                ui.add_space(500.0);
                response
            });
        (output.inner, output.state.clamped_offset().y)
    };
    let (r, _) = frame(&ctx, vec![], |ui| draw(ui, &mut value));
    let at = r.rect.left_center() + egui::vec2(20.0, 0.0);
    ctx.memory_mut(|memory| memory.request_focus(r.id));
    frame(&ctx, vec![egui::Event::PointerMoved(at)], |ui| {
        draw(ui, &mut value)
    });
    let (_, offset) = frame(&ctx, vec![wheel(-1.0)], |ui| draw(ui, &mut value));
    assert_eq!(value, 39.0);
    assert_eq!(offset, 0.0);
    for _ in 0..10 {
        assert_eq!(frame(&ctx, vec![], |ui| draw(ui, &mut value)).1, 0.0);
    }
    ctx.memory_mut(|memory| memory.surrender_focus(r.id));
    let (_, offset) = frame(&ctx, vec![wheel(-1.0)], |ui| draw(ui, &mut value));
    assert_eq!(value, 39.0);
    assert!(offset > 0.0);
}

fn key_event(key: egui::Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }
}

#[test]
fn gtk_numbers_commit_on_enter_keep_focus_and_reject_invalid_text() {
    for (text, expected) in [
        ("-", 4.0),
        ("", 4.0),
        ("abc", 4.0),
        ("NaN", 4.0),
        ("inf", 4.0),
        ("-3.25", -3.25),
        ("12.50", 10.0),
        ("1e2", 10.0),
    ] {
        let ctx = context();
        let mut value = 4.0;
        let draw = |ui: &mut egui::Ui, value: &mut f64| {
            ui.add(
                look::NumberField::new(value)
                    .range(-10.0..=10.0)
                    .fixed_decimals(2),
            )
        };
        let r = frame(&ctx, vec![], |ui| draw(ui, &mut value));
        ctx.memory_mut(|memory| memory.request_focus(r.id));
        frame(&ctx, vec![], |ui| draw(ui, &mut value));
        let event = if text.is_empty() {
            key_event(egui::Key::Backspace)
        } else {
            egui::Event::Text(text.into())
        };
        let r = frame(&ctx, vec![event], |ui| draw(ui, &mut value));
        assert_eq!(value, 4.0, "{text}: editing changed the model");
        assert!(!r.changed());
        let r = frame(&ctx, vec![key_event(egui::Key::Enter)], |ui| {
            draw(ui, &mut value)
        });
        assert_eq!(value, expected, "{text}");
        assert_eq!(r.changed(), expected != 4.0, "{text}");
        assert!(r.has_focus(), "{text}: Enter lost focus");
    }
}

#[test]
fn gtk_number_focus_transfer_commits_and_skips_step_buttons() {
    let ctx = context();
    let mut value = 4.0;
    let draw = |ui: &mut egui::Ui, value: &mut f64| {
        let number = ui.add(look::NumberField::new(value).range(-10.0..=10.0));
        ui.add_enabled(false, look::Button::new("Unavailable"));
        let next = ui.add(look::Button::new("Next"));
        (number, next)
    };
    let (number, next) = frame(&ctx, vec![], |ui| draw(ui, &mut value));
    ctx.memory_mut(|memory| memory.request_focus(number.id));
    frame(&ctx, vec![], |ui| draw(ui, &mut value));
    frame(&ctx, vec![egui::Event::Text("7.5".into())], |ui| {
        draw(ui, &mut value)
    });
    assert_eq!(value, 4.0);
    frame(&ctx, vec![key_event(egui::Key::Tab)], |ui| {
        draw(ui, &mut value)
    });
    frame(&ctx, vec![], |ui| draw(ui, &mut value));
    assert_eq!(value, 7.5);
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(next.id));
    let mut back = key_event(egui::Key::Tab);
    if let egui::Event::Key { modifiers, .. } = &mut back {
        modifiers.shift = true;
    }
    frame(&ctx, vec![back], |ui| draw(ui, &mut value));
    frame(&ctx, vec![], |ui| draw(ui, &mut value));
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(number.id));
}

#[test]
fn gtk_number_pointer_step_commits_the_draft_and_keeps_the_editor_focused() {
    let ctx = context();
    let mut value = 4.0;
    let draw = |ui: &mut egui::Ui, value: &mut f64| {
        ui.add(
            look::NumberField::new(value)
                .range(-10.0..=10.0)
                .width(240.0),
        )
    };
    let r = frame(&ctx, vec![], |ui| draw(ui, &mut value));
    ctx.memory_mut(|memory| memory.request_focus(r.id));
    frame(&ctx, vec![], |ui| draw(ui, &mut value));
    frame(&ctx, vec![egui::Event::Text("6".into())], |ui| {
        draw(ui, &mut value)
    });
    let at = r.rect.right_center() - egui::vec2(12.0, 0.0);
    frame(&ctx, vec![egui::Event::PointerMoved(at)], |ui| {
        draw(ui, &mut value)
    });
    frame(&ctx, vec![pointer(at, true)], |ui| draw(ui, &mut value));
    frame(&ctx, vec![pointer(at, false)], |ui| draw(ui, &mut value));
    assert_eq!(value, 7.0);
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(r.id));
}

#[test]
fn gtk_long_dialog_headings_wrap_and_responses_use_full_stacked_width() {
    let title = "Opravdu chcete uložit všechny neuložené změny v tomto dokumentu?";
    for width in [240.0, 320.0, 480.0] {
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let ctx = context();
            ctx.set_pixels_per_point(scale);
            let mut open = true;
            let mut buttons = Vec::new();
            let mut output = egui::FullOutput::default();
            for long in [true, false, true] {
                for _ in 0..4 {
                    buttons.clear();
                    output = ctx.run_ui(egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width, 800.0))),
                        ..Default::default()
                    }, |ui| {
                        look::Dialog::new("wrapped-title", title).show(ui.ctx(), &mut open, |ui| {
                            ui.label("Před zavřením dokumentu můžete uložit změny nebo pokračovat v úpravách.");
                        }, |ui| {
                            buttons.clear();
                            for text in if long { ["Uložit všechny provedené změny", "Pokračovat v úpravách dokumentu"] } else { ["Save", "Cancel"] } {
                                buttons.push(ui.add(look::Button::new(text)).rect);
                            }
                        });
                    });
                    output.textures_delta.clear();
                }
                assert_eq!(buttons.len(), 2);
                if long {
                    assert!((buttons[0].left() - buttons[1].left()).abs() < 1.0);
                    assert!(
                        buttons[0].width() >= width.min(300.0) - 34.0,
                        "{width}/{scale}: {buttons:?}"
                    );
                    assert!(buttons[1].top() >= buttons[0].bottom() - 1.0);
                } else {
                    assert!((buttons[0].top() - buttons[1].top()).abs() < 1.0);
                }
                for button in &buttons {
                    assert!(
                        ctx.content_rect().contains_rect(*button),
                        "{width}/{scale}: {button:?}"
                    );
                }
            }
            fn heading(shape: &egui::epaint::Shape, title: &str) -> Option<egui::Rect> {
                match shape {
                    egui::epaint::Shape::Text(text)
                        if text.galley.job.text == title
                            && text.galley.rows.len() > 1
                            && !text.galley.elided =>
                    {
                        Some(text.galley.rect.translate(text.pos.to_vec2()))
                    }
                    egui::epaint::Shape::Vec(shapes) => {
                        shapes.iter().find_map(|shape| heading(shape, title))
                    }
                    _ => None,
                }
            }
            let bounds = output
                .shapes
                .iter()
                .find_map(|shape| heading(&shape.shape, title))
                .expect("heading was truncated");
            assert!(
                ctx.content_rect().contains_rect(bounds),
                "{width}/{scale}: heading {bounds:?}"
            );
            let body_top = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::epaint::Shape::Text(text)
                        if text.galley.job.text.starts_with("Před zavřením") =>
                    {
                        Some(text.pos.y)
                    }
                    _ => None,
                })
                .expect("dialog body");
            assert!(
                bounds.bottom() <= body_top,
                "{width}/{scale}: heading overlaps body"
            );
        }
    }
}

#[test]
fn gtk_number_external_updates_replace_a_draft_and_paste_commits_once() {
    let ctx = context();
    let mut value = 4.0;
    let draw = |ui: &mut egui::Ui, value: &mut f64| {
        ui.add(look::NumberField::new(value).range(-10.0..=10.0))
    };
    let r = frame(&ctx, vec![], |ui| draw(ui, &mut value));
    ctx.memory_mut(|memory| memory.request_focus(r.id));
    frame(&ctx, vec![], |ui| draw(ui, &mut value));
    frame(&ctx, vec![egui::Event::Text("2".into())], |ui| {
        draw(ui, &mut value)
    });
    value = 8.0;
    frame(&ctx, vec![], |ui| draw(ui, &mut value));
    frame(&ctx, vec![key_event(egui::Key::Enter)], |ui| {
        draw(ui, &mut value)
    });
    assert_eq!(value, 8.0);
    let mut select_all = key_event(egui::Key::A);
    if let egui::Event::Key { modifiers, .. } = &mut select_all {
        modifiers.ctrl = true;
        modifiers.command = true;
    }
    frame(&ctx, vec![select_all], |ui| draw(ui, &mut value));
    frame(&ctx, vec![egui::Event::Paste("-7".into())], |ui| {
        draw(ui, &mut value)
    });
    assert_eq!(value, 8.0);
    let r = frame(&ctx, vec![key_event(egui::Key::Enter)], |ui| {
        draw(ui, &mut value)
    });
    assert_eq!(value, -7.0);
    assert!(r.changed());
    assert!(!frame(&ctx, vec![], |ui| draw(ui, &mut value)).changed());
}

#[test]
fn gtk_form_number_edits_text_commits_and_skips_step_buttons() {
    let ctx = context();
    let mut value = 4.0;
    let draw = |ui: &mut egui::Ui, value: &mut f64| {
        let number = look::Form::new("settings").show(ui, |form| {
            form.row("Count", |ui| {
                ui.add(look::NumberField::new(value).range(-10.0..=10.0))
            })
        });
        (number, ui.add(look::Button::new("Next")))
    };
    let (number, _) = frame(&ctx, vec![], |ui| draw(ui, &mut value));
    number.request_focus();
    frame(&ctx, vec![], |ui| draw(ui, &mut value));
    frame(&ctx, vec![egui::Event::Text("7".into())], |ui| {
        draw(ui, &mut value)
    });
    assert_eq!(value, 4.0);
    let (number, _) = frame(&ctx, vec![key_event(egui::Key::Enter)], |ui| {
        draw(ui, &mut value)
    });
    assert_eq!(value, 7.0);
    assert!(number.has_focus());
    frame(&ctx, vec![key_event(egui::Key::Tab)], |ui| {
        draw(ui, &mut value)
    });
    let (_, next) = frame(&ctx, vec![], |ui| draw(ui, &mut value));
    assert!(next.has_focus());
    let at = number.rect.left_center() + egui::vec2(15.0, 0.0);
    frame(
        &ctx,
        vec![egui::Event::PointerMoved(at), pointer(at, true)],
        |ui| draw(ui, &mut value),
    );
    let end = at + egui::vec2(25.0, 0.0);
    frame(&ctx, vec![egui::Event::PointerMoved(end)], |ui| {
        draw(ui, &mut value)
    });
    frame(&ctx, vec![pointer(end, false)], |ui| draw(ui, &mut value));
    assert_eq!(value, 7.0);
}

#[test]
fn gtk_live_theme_and_zoom_changes_preserve_numeric_draft_and_focus() {
    let ctx = context();
    let mut value = 4.0;
    let draw = |ui: &mut egui::Ui, value: &mut f64| {
        look::Form::new("settings").show(ui, |form| {
            form.row("Count", |ui| {
                ui.add(look::NumberField::new(value).range(-10.0..=10.0))
            })
        })
    };
    frame(&ctx, vec![], |ui| draw(ui, &mut value)).request_focus();
    frame(&ctx, vec![], |ui| draw(ui, &mut value));
    frame(&ctx, vec![egui::Event::Text("7".into())], |ui| {
        draw(ui, &mut value)
    });
    for (zoom, dark, accent) in [
        (1.25, true, 0xe64b35),
        (1.5, false, 0x26a269),
        (2.0, true, 0x9141ac),
        (1.0, false, 0x3584e4),
    ] {
        ctx.set_zoom_factor(zoom);
        ctx.set_theme(if dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        });
        look::install(&ctx, look::theme(&ctx).accent(Some([accent; 2])));
        let response = frame(&ctx, vec![], |ui| draw(ui, &mut value));
        assert!(response.has_focus());
        assert_eq!(value, 4.0);
        assert_eq!(
            look::colors(&ctx).accent,
            egui::Color32::from_rgb((accent >> 16) as u8, (accent >> 8) as u8, accent as u8)
        );
    }
    frame(&ctx, vec![key_event(egui::Key::Enter)], |ui| {
        draw(ui, &mut value)
    });
    assert_eq!(value, 7.0);
}

#[test]
fn gtk_numeric_ime_confirmation_does_not_commit_the_model() {
    for boxed in [false, true] {
        let ctx = context();
        let mut value = 4.0;
        let draw = |ui: &mut egui::Ui, value: &mut f64| {
            if boxed {
                look::Form::new("settings").show(ui, |form| {
                    form.row("Count", |ui| ui.add(look::NumberField::new(value)))
                })
            } else {
                ui.add(look::NumberField::new(value))
            }
        };
        frame(&ctx, vec![], |ui| draw(ui, &mut value)).request_focus();
        frame(&ctx, vec![], |ui| draw(ui, &mut value));
        frame(
            &ctx,
            vec![egui::Event::Ime(egui::ImeEvent::Preedit {
                text: "7".into(),
                active_range_chars: Some(0..1),
            })],
            |ui| draw(ui, &mut value),
        );
        let response = frame(
            &ctx,
            vec![
                egui::Event::Ime(egui::ImeEvent::Commit("7".into())),
                key_event(egui::Key::Enter),
            ],
            |ui| draw(ui, &mut value),
        );
        assert_eq!(value, 4.0);
        assert!(response.has_focus());
        frame(&ctx, vec![key_event(egui::Key::Enter)], |ui| {
            draw(ui, &mut value)
        });
        assert_eq!(value, 7.0);
    }
}

#[test]
fn gtk_form_label_focuses_its_editor_without_adding_a_tab_stop() {
    for text_field in [false, true] {
        for enabled in [false, true] {
            let ctx = context();
            let mut number = 4.0;
            let mut text = String::from("Example");
            let mut draw = |ui: &mut egui::Ui| {
                let editor = look::Form::new("settings").show(ui, |form| {
                    form.row("Value", |ui| {
                        if text_field {
                            ui.add_enabled(enabled, look::TextField::new(&mut text).width(120.0))
                        } else {
                            ui.add_enabled(enabled, look::NumberField::new(&mut number))
                        }
                    })
                });
                (editor, ui.add(look::Button::new("Next")))
            };
            let (editor, next) = frame(&ctx, vec![], &mut draw);
            next.request_focus();
            let at = egui::pos2(15.0, editor.rect.center().y);
            frame(&ctx, vec![egui::Event::PointerMoved(at)], &mut draw);
            frame(&ctx, vec![pointer(at, true)], &mut draw);
            let (editor, _) = frame(&ctx, vec![pointer(at, false)], &mut draw);
            assert_eq!(
                editor.has_focus(),
                enabled,
                "text={text_field}, enabled={enabled}"
            );
            if enabled {
                frame(&ctx, vec![key_event(egui::Key::Tab)], &mut draw);
                assert!(frame(&ctx, vec![], &mut draw).1.has_focus());
            }
        }
    }
}

#[test]
fn gtk_compact_controls_match_native_allocation() {
    let ctx = context();
    let mut on = false;
    let (switch, check, progress, icon) = frame(&ctx, vec![], |ui| {
        (
            ui.add(look::Switch::new(&mut on)),
            ui.add(look::Checkbox::new(&mut false, "Enabled")),
            ui.add(look::ProgressBar::new(0.5).width(200.0)),
            ui.add(look::IconButton::new("+")),
        )
    });
    assert_eq!(switch.rect.size(), egui::vec2(46.0, 26.0));
    assert_eq!(check.rect.height(), 28.0);
    assert_eq!(progress.rect.height(), 8.0);
    assert_eq!(icon.rect.size(), egui::vec2(34.0, 34.0));
}

#[test]
fn gtk_switch_row_label_toggles_once_and_respects_disabled_state() {
    for enabled in [false, true] {
        let ctx = context();
        let mut on = false;
        let draw = |ui: &mut egui::Ui, on: &mut bool| {
            look::Form::new("settings").show(ui, |form| {
                form.row("Notifications", |ui| {
                    ui.add_enabled(enabled, look::Switch::new(on))
                })
            })
        };
        let switch = frame(&ctx, vec![], |ui| draw(ui, &mut on));
        let at = egui::pos2(15.0, switch.rect.center().y);
        frame(&ctx, vec![egui::Event::PointerMoved(at)], |ui| {
            draw(ui, &mut on)
        });
        frame(&ctx, vec![pointer(at, true)], |ui| draw(ui, &mut on));
        assert!(!on);
        let response = frame(&ctx, vec![pointer(at, false)], |ui| draw(ui, &mut on));
        assert_eq!(on, enabled);
        assert_eq!(response.changed(), enabled);
        frame(&ctx, vec![], |ui| draw(ui, &mut on));
        assert_eq!(on, enabled);
    }
}

#[test]
fn gtk_invalid_entry_paints_error_background_in_both_schemes() {
    for dark in [false, true] {
        let ctx = context();
        ctx.set_theme(if dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        });
        let mut value = String::from("Invalid");
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.add(look::TextField::new(&mut value).invalid(true));
        });
        output.textures_delta.clear();
        let error = if dark {
            egui::Color32::from_rgb(255, 123, 99)
        } else {
            egui::Color32::from_rgb(192, 28, 40)
        }
        .gamma_multiply(0.1);
        assert!(output
            .shapes
            .iter()
            .any(|shape| matches!(&shape.shape, egui::Shape::Rect(rect) if rect.fill == error)));
    }
}

#[test]
fn gtk_disabled_slider_keeps_its_accent_and_white_thumb_under_opacity() {
    let ctx = context();
    let mut value = 0.5;
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        ui.add_enabled(
            false,
            look::Slider::new(&mut value, 0.0..=1.0).show_value(false),
        );
    });
    output.textures_delta.clear();
    let accent = look::colors(&ctx).accent.gamma_multiply(0.5);
    assert!(output
        .shapes
        .iter()
        .any(|s| matches!(&s.shape, egui::Shape::Rect(r) if r.fill == accent)));
    assert!(output.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Circle(c) if c.fill == egui::Color32::WHITE.gamma_multiply(0.5))));
    assert_eq!(value, 0.5);
}
