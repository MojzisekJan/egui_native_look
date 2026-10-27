//! Keyboard and pointer regressions for the GTK containers and text history.
use egui_native_look as look;

fn context(chosen: look::Look) -> egui::Context {
    let ctx = egui::Context::default();
    look::install(&ctx, look::Theme::new(chosen).accent(Some([0x3584e4; 2])));
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
                egui::vec2(400.0, 300.0),
            )),
            events,
            ..Default::default()
        },
        |ui| result = Some(draw(ui)),
    );
    out.textures_delta.clear();
    result.unwrap()
}
fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}
fn wheel(unit: egui::MouseWheelUnit, y: f32) -> egui::Event {
    egui::Event::MouseWheel {
        unit,
        delta: egui::vec2(0.0, y),
        modifiers: egui::Modifiers::NONE,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Wheel,
    }
}
#[test]
fn gtk_wheel_uses_viewport_size_and_point_input_keeps_distance() {
    for (unit, expected) in [
        (egui::MouseWheelUnit::Line, 90.0_f32.powf(2.0 / 3.0)),
        (egui::MouseWheelUnit::Point, 1.0),
    ] {
        let ctx = context(look::Look::Gtk);
        let draw = |ui: &mut egui::Ui| {
            look::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(90.0)
                .show(ui, |ui| {
                    ui.add_space(800.0);
                })
        };
        let output = frame(&ctx, vec![], draw);
        frame(
            &ctx,
            vec![egui::Event::PointerMoved(output.inner_rect.center())],
            draw,
        );
        frame(&ctx, vec![wheel(unit, -1.0)], draw);
        let mut offset = 0.0;
        for _ in 0..50 {
            offset = frame(&ctx, vec![], draw).state.clamped_offset().y;
        }
        assert!(
            (offset - expected).abs() < 0.1,
            "{unit:?}: {offset} vs {expected}"
        );
    }
}
#[test]
fn nested_gtk_scroll_does_not_bubble_at_either_boundary() {
    for bottom in [false, true] {
        let ctx = context(look::Look::Gtk);
        let draw = |ui: &mut egui::Ui| {
            look::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(200.0)
                .show(ui, |ui| {
                    let inner = look::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .id_salt("inner")
                        .max_height(90.0)
                        .vertical_scroll_offset(if bottom { 710.0 } else { 0.0 })
                        .show(ui, |ui| {
                            ui.add_space(800.0);
                        });
                    ui.add_space(800.0);
                    inner.inner_rect
                })
        };
        let output = frame(&ctx, vec![], draw);
        frame(
            &ctx,
            vec![egui::Event::PointerMoved(output.inner.center())],
            draw,
        );
        frame(
            &ctx,
            vec![wheel(
                egui::MouseWheelUnit::Line,
                if bottom { -1.0 } else { 1.0 },
            )],
            draw,
        );
        for _ in 0..40 {
            assert_eq!(frame(&ctx, vec![], draw).state.clamped_offset().y, 0.0);
        }
    }
}
#[test]
fn other_looks_keep_egui_wheel_distance() {
    for chosen in [
        look::Look::Mac,
        look::Look::Windows,
        look::Look::Ios,
        look::Look::Material,
        look::Look::Plain,
    ] {
        let mut results = Vec::new();
        for wrapped in [false, true] {
            let ctx = context(chosen);
            let draw = |ui: &mut egui::Ui| {
                let content = |ui: &mut egui::Ui| {
                    ui.add_space(800.0);
                };
                if wrapped {
                    look::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .max_height(90.0)
                        .show(ui, content)
                } else {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .max_height(90.0)
                        .show(ui, content)
                }
            };
            let output = frame(&ctx, vec![], draw);
            frame(
                &ctx,
                vec![egui::Event::PointerMoved(output.inner_rect.center())],
                draw,
            );
            frame(&ctx, vec![wheel(egui::MouseWheelUnit::Line, -1.0)], draw);
            let mut offset = 0.0;
            for _ in 0..40 {
                offset = frame(&ctx, vec![], draw).state.clamped_offset().y;
            }
            results.push(offset);
        }
        assert_eq!(results[0], results[1], "{chosen}");
    }
}
#[test]
fn list_keyboard_skips_disabled_rows_and_ctrl_moves_only_focus() {
    let ctx = context(look::Look::Gtk);
    let mut selected = 10;
    let draw = |ui: &mut egui::Ui, selected: &mut usize| {
        let list = look::ListBox::new("list", selected).show(ui, |list| {
            list.item(10, "First");
            list.item_enabled(false, 20, "Disabled");
            list.item(30, "Third");
            list.item(40, "Last");
        });
        (list, ui.add(look::Button::new("After")))
    };
    frame(&ctx, vec![], |ui| draw(ui, &mut selected))
        .0
        .response
        .request_focus();
    frame(&ctx, vec![], |ui| draw(ui, &mut selected));
    let out = frame(
        &ctx,
        vec![key(egui::Key::ArrowDown, egui::Modifiers::NONE)],
        |ui| draw(ui, &mut selected),
    );
    assert_eq!(selected, 30);
    assert!(out.0.response.changed());
    assert_eq!(out.0.activated, None);
    frame(
        &ctx,
        vec![key(egui::Key::End, egui::Modifiers::NONE)],
        |ui| draw(ui, &mut selected),
    );
    assert_eq!(selected, 40);
    frame(
        &ctx,
        vec![key(egui::Key::Home, egui::Modifiers::CTRL)],
        |ui| draw(ui, &mut selected),
    );
    assert_eq!(selected, 40);
    let out = frame(
        &ctx,
        vec![key(egui::Key::Space, egui::Modifiers::NONE)],
        |ui| draw(ui, &mut selected),
    );
    assert_eq!(selected, 10);
    assert_eq!(out.0.activated, Some(10));
    frame(
        &ctx,
        vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
        |ui| draw(ui, &mut selected),
    );
    assert!(frame(&ctx, vec![], |ui| draw(ui, &mut selected))
        .1
        .has_focus());
}
#[test]
fn empty_and_disabled_lists_never_activate_or_change_selection() {
    for empty in [false, true] {
        let ctx = context(look::Look::Gtk);
        let mut selected = 10;
        let draw = |ui: &mut egui::Ui, selected: &mut usize| {
            look::ListBox::new("empty", selected).show(ui, |list| {
                if !empty {
                    list.item_enabled(false, 20, "Disabled");
                }
            })
        };
        frame(&ctx, vec![], |ui| draw(ui, &mut selected))
            .response
            .request_focus();
        let out = frame(
            &ctx,
            vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
            |ui| draw(ui, &mut selected),
        );
        assert_eq!(selected, 10);
        assert_eq!(out.activated, None);
        assert!(!out.response.has_focus());
    }
}
#[test]
fn gtk_undo_restores_replaced_selection_and_redo_restores_caret() {
    let ctx = context(look::Look::Gtk);
    let mut text = String::from("Příliš žluťoučký");
    let original = text.clone();
    let draw =
        |ui: &mut egui::Ui, text: &mut String| ui.add(look::TextField::new(text).id_salt("editor"));
    let r = frame(&ctx, vec![], |ui| draw(ui, &mut text));
    r.request_focus();
    frame(&ctx, vec![], |ui| draw(ui, &mut text));
    frame(
        &ctx,
        vec![key(egui::Key::A, egui::Modifiers::COMMAND)],
        |ui| draw(ui, &mut text),
    );
    frame(&ctx, vec![egui::Event::Text("X".into())], |ui| {
        draw(ui, &mut text)
    });
    assert_eq!(text, "X");
    for _ in 0..2 {
        let r = frame(
            &ctx,
            vec![key(egui::Key::Z, egui::Modifiers::COMMAND)],
            |ui| draw(ui, &mut text),
        );
        assert_eq!(text, original);
        assert!(r.changed());
        let range = egui::TextEdit::load_state(&ctx, r.id)
            .unwrap()
            .cursor
            .char_range()
            .unwrap();
        assert_eq!(range.sorted_cursors()[0].index.0, 0);
        assert_eq!(range.sorted_cursors()[1].index.0, original.chars().count());
        frame(
            &ctx,
            vec![key(
                egui::Key::Z,
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            )],
            |ui| draw(ui, &mut text),
        );
        assert_eq!(text, "X");
        let range = egui::TextEdit::load_state(&ctx, r.id)
            .unwrap()
            .cursor
            .char_range()
            .unwrap();
        assert!(range.is_empty());
        assert_eq!(range.primary.index.0, 1);
    }
}
#[test]
fn gtk_dialog_keeps_actions_visible_with_long_content_and_large_text() {
    for size in [egui::vec2(240.0, 160.0), egui::vec2(320.0, 240.0)] {
        let ctx = context(look::Look::Gtk);
        let mut theme = look::theme(&ctx);
        theme.metrics.body.size = 24.0;
        theme.metrics.bold.size = 28.0;
        look::install(&ctx, theme);
        let mut open = true;
        let mut buttons = Vec::new();
        for _ in 0..4 {
            let mut out = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                    ..Default::default()
                },
                |_| {
                    buttons.clear();
                    look::Dialog::new("long", "Long heading with many words").show(
                        &ctx,
                        &mut open,
                        |ui| {
                            for _ in 0..20 {
                                ui.label("Long content");
                            }
                        },
                        |ui| {
                            buttons.push(ui.add(look::Button::new("Save").accent()).rect);
                            buttons.push(ui.add(look::Button::new("Cancel")).rect);
                        },
                    );
                },
            );
            out.textures_delta.clear();
        }
        assert_eq!(buttons.len(), 2);
        for button in buttons {
            assert!(
                button.top() >= 0.0 && button.bottom() <= size.y,
                "{size:?}: {button:?}"
            );
        }
    }
}

#[test]
fn scroll_area_with_wheel_disabled_leaves_input_to_parent() {
    let ctx = context(look::Look::Gtk);
    let draw = |ui: &mut egui::Ui| {
        look::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(200.0)
            .show(ui, |ui| {
                let inner = look::ScrollArea::vertical()
                    .id_salt("inner")
                    .auto_shrink([false, false])
                    .max_height(90.0)
                    .scroll_source(egui::scroll_area::ScrollSource {
                        mouse_wheel: false,
                        ..Default::default()
                    })
                    .show(ui, |ui| ui.add_space(800.0));
                ui.add_space(800.0);
                inner.inner_rect
            })
    };
    let out = frame(&ctx, vec![], draw);
    frame(
        &ctx,
        vec![egui::Event::PointerMoved(out.inner.center())],
        draw,
    );
    frame(&ctx, vec![wheel(egui::MouseWheelUnit::Line, -1.0)], draw);
    let mut y = 0.0;
    for _ in 0..40 {
        y = frame(&ctx, vec![], draw).state.clamped_offset().y;
    }
    assert!((y - 200.0_f32.powf(2.0 / 3.0)).abs() < 0.1);
}

#[test]
fn gtk_scrollbar_can_be_dragged_without_changing_wheel_policy() {
    let ctx = context(look::Look::Gtk);
    ctx.style_mut_of(ctx.theme(), |s| {
        s.spacing.scroll.floating = false;
        s.spacing.scroll.bar_width = 12.0;
    });
    let draw = |ui: &mut egui::Ui| {
        look::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(160.0)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .show(ui, |ui| ui.add_space(1000.0))
    };
    let out = frame(&ctx, vec![], draw);
    let at = egui::pos2(out.inner_rect.right() + 9.0, out.inner_rect.top() + 15.0);
    let pointer = |pos, pressed| egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    };
    frame(&ctx, vec![egui::Event::PointerMoved(at)], draw);
    frame(&ctx, vec![pointer(at, true)], draw);
    let end = at + egui::vec2(0.0, 90.0);
    frame(&ctx, vec![egui::Event::PointerMoved(end)], draw);
    let out = frame(&ctx, vec![pointer(end, false)], draw);
    assert!(out.state.clamped_offset().y > 300.0);
}

#[test]
fn list_accessibility_click_and_focus_preserve_single_keyboard_stop() {
    let ctx = context(look::Look::Gtk);
    ctx.enable_accesskit();
    let mut selected = 10;
    let draw = |ui: &mut egui::Ui, selected: &mut usize| {
        look::ListBox::new("accessible", selected)
            .accessible_label("Files")
            .show(ui, |list| {
                let a = list.item(10, "Documents");
                let b = list.item(30, "Pictures");
                let c = list.item_enabled(false, 50, "Disabled");
                [a.id, b.id, c.id]
            })
    };
    let out = frame(&ctx, vec![], |ui| draw(ui, &mut selected));
    let event = |action, target: egui::Id| {
        egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action,
            target_tree: egui::accesskit::TreeId::ROOT,
            target_node: target.accesskit_id(),
            data: None,
        })
    };
    let clicked = frame(
        &ctx,
        vec![event(egui::accesskit::Action::Click, out.inner[1])],
        |ui| draw(ui, &mut selected),
    );
    assert_eq!(selected, 30);
    assert_eq!(clicked.activated, Some(30));
    frame(
        &ctx,
        vec![event(egui::accesskit::Action::Focus, out.inner[0])],
        |ui| draw(ui, &mut selected),
    );
    let activated = frame(
        &ctx,
        vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
        |ui| draw(ui, &mut selected),
    );
    assert_eq!(activated.activated, Some(10));
    assert!(activated.response.has_focus());
    let disabled = frame(
        &ctx,
        vec![event(egui::accesskit::Action::Click, out.inner[2])],
        |ui| draw(ui, &mut selected),
    );
    assert_eq!(disabled.activated, None);
    assert_eq!(selected, 10);
}

#[test]
fn list_filtering_keeps_item_identity_and_selection() {
    let ctx = context(look::Look::Gtk);
    let mut selected = 30;
    let draw = |ui: &mut egui::Ui, selected: &mut usize, filtered: bool| {
        look::ListBox::new("filter", selected).show(ui, |list| {
            if !filtered {
                list.item(10, "First");
            }
            let id = list.item(30, "Kept").id;
            list.item(50, "Last");
            id
        })
    };
    let first = frame(&ctx, vec![], |ui| draw(ui, &mut selected, false));
    first.response.request_focus();
    let second = frame(&ctx, vec![], |ui| draw(ui, &mut selected, true));
    assert_eq!(first.inner, second.inner);
    assert_eq!(selected, 30);
    let out = frame(
        &ctx,
        vec![key(egui::Key::ArrowDown, egui::Modifiers::NONE)],
        |ui| draw(ui, &mut selected, true),
    );
    assert_eq!(selected, 50);
    assert!(out.response.changed());
}

#[test]
fn list_uses_current_enabled_state_when_key_arrives_during_update() {
    let ctx = context(look::Look::Gtk);
    let mut selected = 10;
    let draw = |ui: &mut egui::Ui, selected: &mut usize, middle: bool| {
        look::ListBox::new("dynamic", selected).show(ui, |list| {
            list.item(10, "First");
            list.item_enabled(middle, 20, "Middle");
            list.item(30, "Last");
        })
    };
    frame(&ctx, vec![], |ui| draw(ui, &mut selected, true))
        .response
        .request_focus();
    frame(&ctx, vec![], |ui| draw(ui, &mut selected, true));
    let out = frame(
        &ctx,
        vec![key(egui::Key::ArrowDown, egui::Modifiers::NONE)],
        |ui| draw(ui, &mut selected, false),
    );
    assert_eq!(selected, 30);
    assert!(out.response.changed());
    assert_eq!(out.activated, None);
}
#[test]
fn gtk_wheel_uses_resized_viewport_on_first_changed_frame() {
    let ctx = context(look::Look::Gtk);
    let draw = |ui: &mut egui::Ui, height| {
        look::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(height)
            .show(ui, |ui| ui.add_space(1000.0))
    };
    let out = frame(&ctx, vec![], |ui| draw(ui, 180.0));
    frame(
        &ctx,
        vec![egui::Event::PointerMoved(
            out.inner_rect.min + egui::vec2(20.0, 20.0),
        )],
        |ui| draw(ui, 180.0),
    );
    frame(&ctx, vec![wheel(egui::MouseWheelUnit::Line, -1.0)], |ui| {
        draw(ui, 90.0)
    });
    let mut y = 0.0;
    for _ in 0..40 {
        y = frame(&ctx, vec![], |ui| draw(ui, 90.0))
            .state
            .clamped_offset()
            .y;
    }
    assert!((y - 90.0_f32.powf(2.0 / 3.0)).abs() < 0.1);
}

#[test]
fn gtk_scrollable_child_retains_orthogonal_wheel_events() {
    let ctx = context(look::Look::Gtk);
    let draw = |ui: &mut egui::Ui| {
        look::ScrollArea::horizontal()
            .auto_shrink([false, false])
            .max_width(240.0)
            .show(ui, |ui| {
                let inner = look::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_width(150.0)
                    .max_height(90.0)
                    .show(ui, |ui| {
                        ui.add_space(800.0);
                    });
                ui.set_min_width(900.0);
                inner.inner_rect
            })
    };
    let out = frame(&ctx, vec![], draw);
    frame(
        &ctx,
        vec![egui::Event::PointerMoved(out.inner.center())],
        draw,
    );
    let mut event = wheel(egui::MouseWheelUnit::Point, 0.0);
    if let egui::Event::MouseWheel { delta, .. } = &mut event {
        *delta = egui::vec2(-30.0, 0.0);
    }
    frame(&ctx, vec![event], draw);
    for _ in 0..30 {
        assert_eq!(frame(&ctx, vec![], draw).state.clamped_offset().x, 0.0);
    }
}

#[test]
fn list_page_navigation_moves_focus_without_activation_at_fractional_scales() {
    for scale in [1.0, 1.25, 1.5, 2.0] {
        let ctx = context(look::Look::Gtk);
        ctx.set_pixels_per_point(scale);
        let mut selected = 0;
        let draw = |ui: &mut egui::Ui, selected: &mut usize| {
            look::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(160.0)
                .show(ui, |ui| {
                    look::ListBox::new("paged", selected).show(ui, |list| {
                        for i in 0..40 {
                            list.item_enabled(i != 2, i, format!("Item {i}"));
                        }
                    })
                })
        };
        frame(&ctx, vec![], |ui| draw(ui, &mut selected))
            .inner
            .response
            .request_focus();
        frame(&ctx, vec![], |ui| draw(ui, &mut selected));
        let out = frame(
            &ctx,
            vec![key(egui::Key::PageDown, egui::Modifiers::NONE)],
            |ui| draw(ui, &mut selected),
        );
        assert!(selected > 1 && selected < 10, "{scale}: {selected}");
        assert_ne!(selected, 2);
        assert_eq!(out.inner.activated, None);
        let previous = selected;
        frame(
            &ctx,
            vec![key(egui::Key::PageDown, egui::Modifiers::CTRL)],
            |ui| draw(ui, &mut selected),
        );
        assert_eq!(selected, previous);
        let out = frame(
            &ctx,
            vec![key(egui::Key::Space, egui::Modifiers::NONE)],
            |ui| draw(ui, &mut selected),
        );
        assert!(selected > previous);
        assert_eq!(out.inner.activated, Some(selected));
        for _ in 0..20 {
            frame(
                &ctx,
                vec![key(egui::Key::PageUp, egui::Modifiers::NONE)],
                |ui| draw(ui, &mut selected),
            );
        }
        assert_eq!(selected, 0);
    }
}

#[test]
fn gtk_shift_delete_cuts_selection_and_undo_restores_it() {
    let ctx = context(look::Look::Gtk);
    let mut text = String::from("žluťoučký kůň");
    let draw = |ui: &mut egui::Ui, text: &mut String| {
        ui.add(look::TextField::new(text).id_salt("clipboard"))
    };
    frame(&ctx, vec![], |ui| draw(ui, &mut text)).request_focus();
    frame(
        &ctx,
        vec![key(egui::Key::A, egui::Modifiers::COMMAND)],
        |ui| draw(ui, &mut text),
    );
    frame(
        &ctx,
        vec![key(egui::Key::Delete, egui::Modifiers::SHIFT)],
        |ui| draw(ui, &mut text),
    );
    assert_eq!(text, "");
    frame(
        &ctx,
        vec![key(egui::Key::Z, egui::Modifiers::COMMAND)],
        |ui| draw(ui, &mut text),
    );
    assert_eq!(text, "žluťoučký kůň");
}

#[test]
fn gtk_insert_clipboard_shortcuts_use_backend_commands_and_preserve_passwords() {
    for password in [false, true] {
        let ctx = context(look::Look::Gtk);
        let mut text = String::from("private text");
        let draw = |ui: &mut egui::Ui, text: &mut String| {
            ui.add(
                look::TextField::new(text)
                    .id_salt("clipboard")
                    .password(password),
            )
        };
        frame(&ctx, vec![], |ui| draw(ui, &mut text)).request_focus();
        frame(
            &ctx,
            vec![key(egui::Key::A, egui::Modifiers::COMMAND)],
            |ui| draw(ui, &mut text),
        );
        let copy = ctx.run_ui(
            egui::RawInput {
                events: vec![key(egui::Key::Insert, egui::Modifiers::CTRL)],
                ..Default::default()
            },
            |ui| {
                draw(ui, &mut text);
            },
        );
        let copied = copy.platform_output.commands.iter().any(|command| matches!(command, egui::OutputCommand::CopyText(value) if value == "private text"));
        assert_eq!(copied, !password);
        let paste = ctx.run_ui(
            egui::RawInput {
                events: vec![key(egui::Key::Insert, egui::Modifiers::SHIFT)],
                ..Default::default()
            },
            |ui| {
                draw(ui, &mut text);
            },
        );
        assert!(paste.viewport_output.values().any(|out| out
            .commands
            .iter()
            .any(|cmd| matches!(cmd, egui::ViewportCommand::RequestPaste))));
        assert_eq!(text, "private text");
    }
}

#[test]
fn list_pages_match_native_160_point_viewport_sequence() {
    let ctx = context(look::Look::Gtk);
    let mut selected = 0;
    let draw = |ui: &mut egui::Ui, selected: &mut usize| {
        ui.spacing_mut().interact_size.y = 30.0;
        ui.spacing_mut().item_spacing.y = 0.0;
        look::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(160.0)
            .show(ui, |ui| {
                look::ListBox::new("native-pages", selected).show(ui, |list| {
                    for i in 0..40 {
                        list.item(i, format!("Item {i}"));
                    }
                })
            })
    };
    frame(&ctx, vec![], |ui| draw(ui, &mut selected))
        .inner
        .response
        .request_focus();
    for (keycode, modifiers, expected) in [
        (egui::Key::PageDown, egui::Modifiers::NONE, 4),
        (egui::Key::PageDown, egui::Modifiers::NONE, 8),
        (egui::Key::PageUp, egui::Modifiers::NONE, 3),
        (egui::Key::PageDown, egui::Modifiers::CTRL, 3),
        (egui::Key::Space, egui::Modifiers::NONE, 7),
        (egui::Key::Home, egui::Modifiers::NONE, 0),
        (egui::Key::PageUp, egui::Modifiers::NONE, 0),
        (egui::Key::End, egui::Modifiers::NONE, 39),
        (egui::Key::PageDown, egui::Modifiers::NONE, 39),
    ] {
        for _ in 0..20 {
            frame(&ctx, vec![], |ui| draw(ui, &mut selected));
        }
        frame(&ctx, vec![key(keycode, modifiers)], |ui| {
            draw(ui, &mut selected)
        });
        assert_eq!(selected, expected, "{keycode:?} {modifiers:?}");
    }
}

#[test]
fn gtk_overwrite_handles_navigation_selection_paste_unicode_and_undo() {
    for (initial, events, expected) in [
        (
            "abcdef",
            vec![
                key(egui::Key::Home, egui::Modifiers::NONE),
                key(egui::Key::Insert, egui::Modifiers::NONE),
                egui::Event::Text("X".into()),
                egui::Event::Text("Y".into()),
            ],
            "XYcdef",
        ),
        (
            "abcdef",
            vec![
                key(egui::Key::Home, egui::Modifiers::NONE),
                key(egui::Key::Insert, egui::Modifiers::NONE),
                egui::Event::Text("XY".into()),
            ],
            "XYbcdef",
        ),
        (
            "abcdef",
            vec![
                key(egui::Key::Home, egui::Modifiers::NONE),
                key(egui::Key::Insert, egui::Modifiers::NONE),
                egui::Event::Paste("XY".into()),
                egui::Event::Text("Z".into()),
            ],
            "XYZbcdef",
        ),
        (
            "a\u{301}bc",
            vec![
                key(egui::Key::Home, egui::Modifiers::NONE),
                key(egui::Key::Insert, egui::Modifiers::NONE),
                egui::Event::Text("Ž".into()),
            ],
            "Žbc",
        ),
        (
            "abcdef",
            vec![
                key(egui::Key::A, egui::Modifiers::COMMAND),
                key(egui::Key::Insert, egui::Modifiers::NONE),
                egui::Event::Text("X".into()),
            ],
            "X",
        ),
        (
            "abcdef",
            vec![
                key(egui::Key::End, egui::Modifiers::NONE),
                key(egui::Key::Insert, egui::Modifiers::NONE),
                egui::Event::Text("X".into()),
            ],
            "abcdefX",
        ),
    ] {
        let ctx = context(look::Look::Gtk);
        let mut text = initial.to_owned();
        let draw = |ui: &mut egui::Ui, text: &mut String| {
            ui.add(look::TextField::new(text).id_salt("overwrite"))
        };
        frame(&ctx, vec![], |ui| draw(ui, &mut text)).request_focus();
        frame(&ctx, vec![], |ui| draw(ui, &mut text));
        frame(&ctx, events, |ui| draw(ui, &mut text));
        assert_eq!(text, expected, "initial {initial:?}");
        frame(
            &ctx,
            vec![key(egui::Key::Z, egui::Modifiers::COMMAND)],
            |ui| draw(ui, &mut text),
        );
        assert_eq!(text, initial);
    }
}

#[test]
fn gtk_ime_surrounding_delete_uses_characters_and_keeps_editor_focus() {
    let ctx = context(look::Look::Gtk);
    let mut text = String::from("Až🙂B");
    let draw = |ui: &mut egui::Ui, text: &mut String| {
        ui.add(look::TextField::new(text).id_salt("ime-surrounding"))
    };
    let response = frame(&ctx, vec![], |ui| draw(ui, &mut text));
    response.request_focus();
    frame(&ctx, vec![], |ui| draw(ui, &mut text));
    let mut state = egui::TextEdit::load_state(&ctx, response.id).unwrap();
    state
        .cursor
        .set_char_range(Some(egui::text::CCursorRange::one(
            egui::text::CCursor::new(2),
        )));
    state.store(&ctx, response.id);
    let response = frame(
        &ctx,
        vec![egui::Event::Ime(egui::ImeEvent::DeleteSurrounding {
            before_chars: 1,
            after_chars: 1,
        })],
        |ui| draw(ui, &mut text),
    );
    assert_eq!(text, "AB");
    assert!(response.has_focus());
    assert!(response.changed());
    let range = egui::TextEdit::load_state(&ctx, response.id)
        .unwrap()
        .cursor
        .char_range()
        .unwrap();
    assert_eq!(range.primary.index.0, 1);
}

#[test]
fn overwrite_mode_is_local_to_field_and_other_looks_keep_insertion() {
    for chosen in [
        look::Look::Gtk,
        look::Look::Mac,
        look::Look::Windows,
        look::Look::Material,
    ] {
        let ctx = context(chosen);
        let mut first = String::from("abc");
        let mut second = String::from("def");
        let draw = |ui: &mut egui::Ui, first: &mut String, second: &mut String| {
            (
                ui.add(look::TextField::new(first).id_salt("first")),
                ui.add(look::TextField::new(second).id_salt("second")),
            )
        };
        frame(&ctx, vec![], |ui| draw(ui, &mut first, &mut second))
            .0
            .request_focus();
        frame(&ctx, vec![], |ui| draw(ui, &mut first, &mut second));
        frame(
            &ctx,
            vec![
                key(egui::Key::Home, egui::Modifiers::NONE),
                key(egui::Key::Insert, egui::Modifiers::NONE),
                egui::Event::Text("X".into()),
            ],
            |ui| draw(ui, &mut first, &mut second),
        )
        .1
        .request_focus();
        assert_eq!(
            first,
            if chosen == look::Look::Gtk {
                "Xbc"
            } else {
                "Xabc"
            }
        );
        frame(&ctx, vec![], |ui| draw(ui, &mut first, &mut second));
        frame(
            &ctx,
            vec![
                key(egui::Key::Home, egui::Modifiers::NONE),
                egui::Event::Text("Y".into()),
            ],
            |ui| draw(ui, &mut first, &mut second),
        );
        assert_eq!(second, "Ydef");
    }
}

#[test]
fn overwrite_commits_ime_once_without_overwriting_during_preedit() {
    for selected in [false, true] {
        let ctx = context(look::Look::Gtk);
        let mut text = String::from("abc");
        let draw = |ui: &mut egui::Ui, text: &mut String| {
            ui.add(look::TextField::new(text).id_salt("overwrite-ime"))
        };
        frame(&ctx, vec![], |ui| draw(ui, &mut text)).request_focus();
        frame(&ctx, vec![], |ui| draw(ui, &mut text));
        frame(
            &ctx,
            vec![
                key(egui::Key::Home, egui::Modifiers::NONE),
                key(egui::Key::Insert, egui::Modifiers::NONE),
            ],
            |ui| draw(ui, &mut text),
        );
        if selected {
            frame(
                &ctx,
                vec![key(egui::Key::A, egui::Modifiers::COMMAND)],
                |ui| draw(ui, &mut text),
            );
        }
        frame(
            &ctx,
            vec![egui::Event::Ime(egui::ImeEvent::Preedit {
                text: "ni".into(),
                active_range_chars: None,
            })],
            |ui| draw(ui, &mut text),
        );
        assert_eq!(text, if selected { "ni" } else { "niabc" });
        frame(
            &ctx,
            vec![egui::Event::Ime(egui::ImeEvent::Preedit {
                text: "nihon".into(),
                active_range_chars: None,
            })],
            |ui| draw(ui, &mut text),
        );
        frame(
            &ctx,
            vec![egui::Event::Ime(egui::ImeEvent::Commit("日本".into()))],
            |ui| draw(ui, &mut text),
        );
        assert_eq!(text, if selected { "日本" } else { "日本bc" });
        frame(
            &ctx,
            vec![key(egui::Key::Z, egui::Modifiers::COMMAND)],
            |ui| draw(ui, &mut text),
        );
        assert_eq!(text, "abc");
    }
}
