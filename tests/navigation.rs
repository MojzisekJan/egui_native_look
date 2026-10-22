//! Group focus, selection and popup navigation contracts.
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
    look::install(&ctx, look::Theme::new(chosen));
    ctx
}
fn frame<R>(
    ctx: &egui::Context,
    events: Vec<egui::Event>,
    draw: impl FnMut(&mut egui::Ui) -> R,
) -> (R, egui::FullOutput) {
    phone_frame(ctx, egui::vec2(340.0, 600.0), events, draw)
}
fn phone_frame<R>(
    ctx: &egui::Context,
    size: egui::Vec2,
    events: Vec<egui::Event>,
    mut draw: impl FnMut(&mut egui::Ui) -> R,
) -> (R, egui::FullOutput) {
    let mut result = None;
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
            events,
            ..Default::default()
        },
        |ui| result = Some(draw(ui)),
    );
    output.textures_delta.clear();
    (result.unwrap(), output)
}
fn key(key: egui::Key, modifiers: egui::Modifiers) -> Vec<egui::Event> {
    [false, true]
        .into_iter()
        .map(|pressed| egui::Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers,
        })
        .collect()
}
fn group(ui: &mut egui::Ui, selected: &mut usize) -> egui::Response {
    ui.add(
        look::Segmented::new("units", selected, &["One", "Two", "Three"])
            .accessible_label("Units")
            .width(240.0),
    )
}

#[test]
fn windows_ctrl_arrows_move_focus_without_selecting_until_space() {
    let ctx = context(look::Look::Windows);
    let mut selected = 1;
    let (response, _) = frame(&ctx, vec![], |ui| group(ui, &mut selected));
    ctx.memory_mut(|memory| memory.request_focus(response.id));
    frame(&ctx, vec![], |ui| group(ui, &mut selected));
    ctx.enable_accesskit();
    let (response, output) = frame(
        &ctx,
        key(egui::Key::ArrowRight, egui::Modifiers::CTRL),
        |ui| group(ui, &mut selected),
    );
    assert!(!response.changed());
    assert_eq!(selected, 1);
    let nodes = output.platform_output.accesskit_update.unwrap().nodes;
    let parent = &nodes
        .iter()
        .find(|(id, _)| *id == response.id.accesskit_id())
        .unwrap()
        .1;
    let (third_id, third) = nodes
        .iter()
        .find(|(_, node)| node.label() == Some("Three"))
        .unwrap();
    assert_eq!(parent.active_descendant(), Some(*third_id));
    assert_eq!(third.toggled(), Some(egui::accesskit::Toggled::False));
    assert_eq!(third.position_in_set(), Some(3));
    assert_eq!(third.size_of_set(), Some(3));
    let (response, _) = frame(&ctx, key(egui::Key::Space, egui::Modifiers::NONE), |ui| {
        group(ui, &mut selected)
    });
    assert!(response.changed());
    assert_eq!(selected, 2);
    frame(
        &ctx,
        key(egui::Key::ArrowLeft, egui::Modifiers::CTRL),
        |ui| group(ui, &mut selected),
    );
    assert_eq!(selected, 2);
    frame(
        &ctx,
        key(egui::Key::ArrowLeft, egui::Modifiers::NONE),
        |ui| group(ui, &mut selected),
    );
    assert_eq!(selected, 0, "selection must follow the focused option");
}

#[test]
fn windows_vertical_keys_stay_in_group_even_immediately_after_tab() {
    let ctx = context(look::Look::Windows);
    let mut selected = 0;
    let draw = |ui: &mut egui::Ui, selected: &mut usize| {
        let response = group(ui, selected);
        ui.add(look::Button::new("After"));
        response
    };
    frame(&ctx, vec![], |ui| draw(ui, &mut selected));
    let (response, _) = frame(&ctx, key(egui::Key::Tab, egui::Modifiers::NONE), |ui| {
        draw(ui, &mut selected)
    });
    assert!(response.has_focus());
    frame(
        &ctx,
        key(egui::Key::ArrowDown, egui::Modifiers::NONE),
        |ui| draw(ui, &mut selected),
    );
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(response.id));
    assert_eq!(selected, 1);
    for _ in 0..4 {
        frame(
            &ctx,
            key(egui::Key::ArrowDown, egui::Modifiers::NONE),
            |ui| draw(ui, &mut selected),
        );
    }
    assert_eq!(selected, 2, "group must not wrap");
    frame(&ctx, key(egui::Key::ArrowUp, egui::Modifiers::CTRL), |ui| {
        draw(ui, &mut selected)
    });
    assert_eq!(selected, 2);
    frame(&ctx, key(egui::Key::Space, egui::Modifiers::NONE), |ui| {
        draw(ui, &mut selected)
    });
    assert_eq!(selected, 1);
}

#[test]
fn tab_reentry_and_external_model_edits_reset_temporary_choice_focus() {
    let ctx = context(look::Look::Windows);
    let mut selected = 0;
    let draw = |ui: &mut egui::Ui, selected: &mut usize| {
        let response = group(ui, selected);
        let after = ui.add(look::Button::new("After"));
        (response, after.id)
    };
    frame(&ctx, vec![], |ui| draw(ui, &mut selected));
    let ((response, after), _) = frame(&ctx, key(egui::Key::Tab, egui::Modifiers::NONE), |ui| {
        draw(ui, &mut selected)
    });
    frame(
        &ctx,
        key(egui::Key::ArrowRight, egui::Modifiers::CTRL),
        |ui| draw(ui, &mut selected),
    );
    frame(&ctx, key(egui::Key::Tab, egui::Modifiers::NONE), |ui| {
        draw(ui, &mut selected)
    });
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(after));
    frame(&ctx, key(egui::Key::Tab, egui::Modifiers::SHIFT), |ui| {
        draw(ui, &mut selected)
    });
    frame(&ctx, vec![], |ui| draw(ui, &mut selected));
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(response.id));
    frame(&ctx, key(egui::Key::Space, egui::Modifiers::NONE), |ui| {
        draw(ui, &mut selected)
    });
    assert_eq!(selected, 0, "uncommitted focus survived leaving the group");
    selected = 2;
    frame(&ctx, vec![], |ui| draw(ui, &mut selected));
    frame(
        &ctx,
        key(egui::Key::ArrowLeft, egui::Modifiers::CTRL),
        |ui| draw(ui, &mut selected),
    );
    frame(&ctx, key(egui::Key::Space, egui::Modifiers::NONE), |ui| {
        draw(ui, &mut selected)
    });
    assert_eq!(selected, 1);
}

#[test]
fn invalid_choice_stays_unselected_until_space_selects_first_option() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        let mut selected = usize::MAX;
        frame(&ctx, vec![], |ui| group(ui, &mut selected));
        frame(&ctx, key(egui::Key::Tab, egui::Modifiers::NONE), |ui| {
            group(ui, &mut selected)
        });
        assert_eq!(selected, usize::MAX);
        frame(&ctx, key(egui::Key::Space, egui::Modifiers::NONE), |ui| {
            group(ui, &mut selected)
        });
        assert_eq!(selected, 0, "{chosen}");
    }
}

#[test]
fn application_modifiers_are_not_consumed_by_choice_groups() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        let mut selected = 1;
        let (response, _) = frame(&ctx, vec![], |ui| group(ui, &mut selected));
        ctx.memory_mut(|memory| memory.request_focus(response.id));
        frame(&ctx, vec![], |ui| group(ui, &mut selected));
        for modifiers in [
            egui::Modifiers::ALT,
            egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
        ] {
            let (available, _) = frame(&ctx, key(egui::Key::ArrowRight, modifiers), |ui| {
                group(ui, &mut selected);
                ui.input_mut(|input| input.consume_key(modifiers, egui::Key::ArrowRight))
            });
            assert!(available, "{chosen}: app shortcut consumed");
            assert_eq!(selected, 1);
        }
        if chosen != look::Look::Windows {
            let (available, _) = frame(
                &ctx,
                key(egui::Key::ArrowRight, egui::Modifiers::CTRL),
                |ui| {
                    group(ui, &mut selected);
                    ui.input_mut(|input| {
                        input.consume_key(egui::Modifiers::CTRL, egui::Key::ArrowRight)
                    })
                },
            );
            assert!(
                available,
                "{chosen}: Ctrl navigation changed outside Windows"
            );
        }
    }
}

#[test]
fn popup_home_and_end_skip_disabled_rows_and_do_not_activate_them() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        let mut selected = None;
        let mut first = egui::Id::NULL;
        let mut last = egui::Id::NULL;
        let mut draw = |ui: &mut egui::Ui| {
            look::PopupButton::new("actions", "Actions").show(ui, |ui| {
                ui.add_enabled(false, look::MenuItem::new("Unavailable first"));
                let response = ui.add(look::MenuItem::new("First"));
                first = response.id;
                if response.clicked() {
                    selected = Some(0);
                }
                ui.separator();
                ui.add_enabled(false, look::MenuItem::new("Unavailable middle"));
                let response = ui.add(look::MenuItem::new("Last"));
                last = response.id;
                if response.clicked() {
                    selected = Some(1);
                }
                ui.add_enabled(false, look::MenuItem::new("Unavailable last"));
            });
            (first, last, selected)
        };
        frame(&ctx, vec![], &mut draw);
        frame(&ctx, key(egui::Key::Tab, egui::Modifiers::NONE), &mut draw);
        frame(
            &ctx,
            key(egui::Key::Enter, egui::Modifiers::NONE),
            &mut draw,
        );
        let ((first, last, _), _) = frame(&ctx, vec![], &mut draw);
        assert_eq!(
            ctx.memory(|memory| memory.focused()),
            Some(first),
            "{chosen}: initial focus"
        );
        let ((_, _, selected), _) =
            frame(&ctx, key(egui::Key::End, egui::Modifiers::NONE), &mut draw);
        assert_eq!(ctx.memory(|memory| memory.focused()), Some(last));
        assert_eq!(selected, None);
        frame(&ctx, key(egui::Key::Home, egui::Modifiers::NONE), &mut draw);
        assert_eq!(ctx.memory(|memory| memory.focused()), Some(first));
        frame(&ctx, key(egui::Key::End, egui::Modifiers::NONE), &mut draw);
        let ((_, _, selected), _) = frame(
            &ctx,
            key(egui::Key::Enter, egui::Modifiers::NONE),
            &mut draw,
        );
        assert_eq!(selected, Some(1));
        assert!(!egui::Popup::is_any_open(&ctx));
    }
}

struct NestedMenu {
    launcher: egui::Id,
    branch: egui::Id,
    first: egui::Id,
    last: egui::Id,
    visible: bool,
    activated: bool,
}
impl Default for NestedMenu {
    fn default() -> Self {
        Self {
            launcher: egui::Id::NULL,
            branch: egui::Id::NULL,
            first: egui::Id::NULL,
            last: egui::Id::NULL,
            visible: false,
            activated: false,
        }
    }
}
impl NestedMenu {
    fn draw(&mut self, ui: &mut egui::Ui) {
        self.visible = false;
        let popup = look::PopupButton::new("nested-actions", "Actions").show(ui, |ui| {
            let submenu = look::submenu(ui, "Export", |ui| {
                self.visible = true;
                ui.add_enabled(false, look::MenuItem::new("Unavailable"));
                self.first = ui.add(look::MenuItem::new("First format")).id;
                let last = ui.add(look::MenuItem::new("Last format"));
                self.last = last.id;
                self.activated |= last.clicked();
                ui.add_enabled(false, look::MenuItem::new("Unavailable last"));
            });
            self.branch = submenu.response.id;
            ui.add(look::MenuItem::new("Other action"));
        });
        self.launcher = popup.response.id;
    }
}

#[test]
fn nested_menu_home_end_left_and_activation_keep_the_correct_focus() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        let mut menu = NestedMenu::default();
        frame(&ctx, vec![], |ui| menu.draw(ui));
        ctx.memory_mut(|memory| memory.request_focus(menu.launcher));
        frame(&ctx, key(egui::Key::Enter, egui::Modifiers::NONE), |ui| {
            menu.draw(ui)
        });
        frame(&ctx, vec![], |ui| menu.draw(ui));
        assert_eq!(
            ctx.memory(|memory| memory.focused()),
            Some(menu.branch),
            "{chosen}: root focus"
        );
        frame(
            &ctx,
            key(egui::Key::ArrowRight, egui::Modifiers::NONE),
            |ui| menu.draw(ui),
        );
        frame(&ctx, vec![], |ui| menu.draw(ui));
        assert!(menu.visible, "{chosen}: submenu not opened");
        assert_eq!(ctx.memory(|memory| memory.focused()), Some(menu.first));
        frame(&ctx, key(egui::Key::End, egui::Modifiers::NONE), |ui| {
            menu.draw(ui)
        });
        assert_eq!(ctx.memory(|memory| memory.focused()), Some(menu.last));
        assert!(!menu.activated);
        frame(&ctx, key(egui::Key::Home, egui::Modifiers::NONE), |ui| {
            menu.draw(ui)
        });
        assert_eq!(ctx.memory(|memory| memory.focused()), Some(menu.first));
        frame(
            &ctx,
            key(egui::Key::ArrowLeft, egui::Modifiers::NONE),
            |ui| menu.draw(ui),
        );
        frame(&ctx, vec![], |ui| menu.draw(ui));
        assert!(!menu.visible, "{chosen}: Left did not close submenu");
        assert_eq!(ctx.memory(|memory| memory.focused()), Some(menu.branch));
        assert!(egui::Popup::is_any_open(&ctx));
        frame(
            &ctx,
            key(egui::Key::ArrowRight, egui::Modifiers::NONE),
            |ui| menu.draw(ui),
        );
        frame(&ctx, vec![], |ui| menu.draw(ui));
        frame(&ctx, key(egui::Key::End, egui::Modifiers::NONE), |ui| {
            menu.draw(ui)
        });
        frame(&ctx, key(egui::Key::Enter, egui::Modifiers::NONE), |ui| {
            menu.draw(ui)
        });
        assert!(menu.activated);
        assert!(
            !egui::Popup::is_any_open(&ctx),
            "{chosen}: activation did not close chain"
        );
        frame(&ctx, vec![], |ui| menu.draw(ui));
        assert_eq!(ctx.memory(|memory| memory.focused()), Some(menu.launcher));
    }
}

#[test]
fn escape_from_nested_menu_returns_to_launcher_without_activation() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        let mut menu = NestedMenu::default();
        frame(&ctx, vec![], |ui| menu.draw(ui));
        ctx.memory_mut(|memory| memory.request_focus(menu.launcher));
        frame(&ctx, key(egui::Key::Enter, egui::Modifiers::NONE), |ui| {
            menu.draw(ui)
        });
        frame(&ctx, vec![], |ui| menu.draw(ui));
        frame(
            &ctx,
            key(egui::Key::ArrowRight, egui::Modifiers::NONE),
            |ui| menu.draw(ui),
        );
        frame(&ctx, vec![], |ui| menu.draw(ui));
        assert!(menu.visible);
        frame(&ctx, key(egui::Key::Escape, egui::Modifiers::NONE), |ui| {
            menu.draw(ui)
        });
        frame(&ctx, vec![], |ui| menu.draw(ui));
        assert!(!egui::Popup::is_any_open(&ctx), "{chosen}");
        assert!(!menu.visible);
        assert!(!menu.activated);
        assert_eq!(ctx.memory(|memory| memory.focused()), Some(menu.launcher));
    }
}

fn popup_bounds(ctx: &egui::Context) -> Vec<egui::Rect> {
    ctx.memory(|memory| {
        memory
            .layer_ids()
            .filter(|layer| layer.order == egui::Order::Foreground)
            .filter_map(|layer| memory.area_rect(layer.id))
            .collect()
    })
}

#[test]
fn long_phone_menus_fit_and_scroll_keyboard_focus_to_last_item() {
    for nested in [false, true] {
        for chosen in LOOKS {
            for size in [egui::vec2(288.0, 320.0), egui::vec2(390.0, 240.0)] {
                let ctx = context(chosen);
                let mut selected = None;
                let mut last_rect = egui::Rect::NOTHING;
                let mut visible = egui::Rect::NOTHING;
                let mut draw = |ui: &mut egui::Ui| {
                    look::PopupButton::new("long-phone", "Actions").show(ui, |ui| {
                        let mut rows = |ui: &mut egui::Ui| {
                            visible = ui.clip_rect();
                            for i in 0..40 {
                                let row = ui.add(look::MenuItem::new(format!(
                            "Action {i} with a very long descriptive name that must fit on a phone"
                        )));
                                if i == 39 {
                                    last_rect = row.rect;
                                }
                                if row.clicked() {
                                    selected = Some(i);
                                }
                            }
                        };
                        if nested {
                            look::submenu(ui, "Formats", &mut rows);
                        } else {
                            rows(ui);
                        }
                    });
                    (last_rect, visible, selected)
                };
                phone_frame(&ctx, size, vec![], &mut draw);
                phone_frame(
                    &ctx,
                    size,
                    key(egui::Key::Tab, egui::Modifiers::NONE),
                    &mut draw,
                );
                phone_frame(
                    &ctx,
                    size,
                    key(egui::Key::Enter, egui::Modifiers::NONE),
                    &mut draw,
                );
                phone_frame(&ctx, size, vec![], &mut draw);
                if nested {
                    phone_frame(
                        &ctx,
                        size,
                        key(egui::Key::ArrowRight, egui::Modifiers::NONE),
                        &mut draw,
                    );
                }
                for _ in 0..3 {
                    phone_frame(&ctx, size, vec![], &mut draw);
                }
                for rect in popup_bounds(&ctx) {
                    assert!(
                        rect.left() >= -1.0
                            && rect.right() <= size.x + 1.0
                            && rect.top() >= -1.0
                            && rect.bottom() <= size.y + 1.0,
                        "{chosen} {size:?}: {rect:?}"
                    );
                }
                phone_frame(
                    &ctx,
                    size,
                    key(egui::Key::End, egui::Modifiers::NONE),
                    &mut draw,
                );
                let mut geometry = None;
                for _ in 0..4 {
                    geometry = Some(phone_frame(&ctx, size, vec![], &mut draw).0);
                }
                let (last, clip, selected) = geometry.unwrap();
                assert!(
                    clip.contains(last.center()),
                    "{chosen}: last row {last:?} outside {clip:?}"
                );
                assert_eq!(selected, None);
                let ((_, _, selected), _) = phone_frame(
                    &ctx,
                    size,
                    key(egui::Key::Enter, egui::Modifiers::NONE),
                    &mut draw,
                );
                assert_eq!(selected, Some(39));
                assert!(!egui::Popup::is_any_open(&ctx));
            }
        }
    }
}

fn touch_event(phase: egui::TouchPhase, pos: egui::Pos2) -> egui::Event {
    egui::Event::Touch {
        device_id: egui::TouchDeviceId(1),
        id: egui::TouchId(1),
        phase,
        pos,
        force: None,
    }
}
fn pointer_button(pos: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        pressed,
        button: egui::PointerButton::Primary,
        modifiers: egui::Modifiers::NONE,
    }
}

#[test]
fn phone_menu_touch_scroll_and_outside_tap_do_not_activate_rows() {
    for nested in [false, true] {
        for chosen in LOOKS {
            let ctx = context(chosen);
            let size = egui::vec2(320.0, 420.0);
            let prime = egui::pos2(310.0, 410.0);
            phone_frame(
                &ctx,
                size,
                vec![
                    touch_event(egui::TouchPhase::Start, prime),
                    touch_event(egui::TouchPhase::End, prime),
                ],
                |_| {},
            );
            let mut activated = false;
            let mut tracked = egui::Rect::NOTHING;
            let mut draw = |ui: &mut egui::Ui| {
                look::PopupButton::new("touch-menu", "Actions").show(ui, |ui| {
                    let mut rows = |ui: &mut egui::Ui| {
                        for i in 0..30 {
                            let row = ui.add(look::MenuItem::new(format!("Action {i}")));
                            if i == 4 {
                                tracked = row.rect;
                            }
                            activated |= row.clicked();
                        }
                    };
                    if nested {
                        look::submenu(ui, "Formats", &mut rows);
                    } else {
                        rows(ui);
                    }
                });
                (tracked, activated)
            };
            phone_frame(&ctx, size, vec![], &mut draw);
            phone_frame(
                &ctx,
                size,
                key(egui::Key::Tab, egui::Modifiers::NONE),
                &mut draw,
            );
            phone_frame(
                &ctx,
                size,
                key(egui::Key::Enter, egui::Modifiers::NONE),
                &mut draw,
            );
            phone_frame(&ctx, size, vec![], &mut draw);
            if nested {
                phone_frame(
                    &ctx,
                    size,
                    key(egui::Key::ArrowRight, egui::Modifiers::NONE),
                    &mut draw,
                );
            }
            let mut initial = egui::Rect::NOTHING;
            for _ in 0..3 {
                initial = phone_frame(&ctx, size, vec![], &mut draw).0 .0;
            }
            let start = initial.center();
            phone_frame(
                &ctx,
                size,
                vec![
                    touch_event(egui::TouchPhase::Start, start),
                    egui::Event::PointerMoved(start),
                    pointer_button(start, true),
                ],
                &mut draw,
            );
            let mut end = start;
            for distance in [15.0, 35.0, 60.0, 90.0] {
                end = start - egui::vec2(0.0, distance);
                phone_frame(
                    &ctx,
                    size,
                    vec![
                        touch_event(egui::TouchPhase::Move, end),
                        egui::Event::PointerMoved(end),
                    ],
                    &mut draw,
                );
            }
            let ((moved, activated), _) = phone_frame(
                &ctx,
                size,
                vec![
                    touch_event(egui::TouchPhase::End, end),
                    pointer_button(end, false),
                    egui::Event::PointerGone,
                ],
                &mut draw,
            );
            assert!(
                moved.top() < initial.top() - 20.0,
                "{chosen}: menu did not scroll {initial:?} -> {moved:?}"
            );
            assert!(!activated, "{chosen}: scrolling activated a row");
            assert!(
                egui::Popup::is_any_open(&ctx),
                "{chosen}: scrolling closed menu"
            );
            let outside = egui::pos2(1.0, 1.0);
            phone_frame(
                &ctx,
                size,
                vec![
                    touch_event(egui::TouchPhase::Start, outside),
                    egui::Event::PointerMoved(outside),
                    pointer_button(outside, true),
                ],
                &mut draw,
            );
            let ((_, activated), _) = phone_frame(
                &ctx,
                size,
                vec![
                    touch_event(egui::TouchPhase::End, outside),
                    pointer_button(outside, false),
                    egui::Event::PointerGone,
                ],
                &mut draw,
            );
            assert!(!activated);
            assert!(
                !egui::Popup::is_any_open(&ctx),
                "{chosen}: outside tap did not close menu"
            );
        }
    }
}

#[test]
fn phone_submenus_remain_within_viewport_edges() {
    for chosen in LOOKS {
        for width in [288.0, 320.0, 390.0] {
            let ctx = context(chosen);
            let size = egui::vec2(width, 320.0);
            let mut menu = NestedMenu::default();
            let mut draw = |ui: &mut egui::Ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    menu.draw(ui)
                });
            };
            phone_frame(&ctx, size, vec![], &mut draw);
            phone_frame(
                &ctx,
                size,
                key(egui::Key::Tab, egui::Modifiers::NONE),
                &mut draw,
            );
            phone_frame(
                &ctx,
                size,
                key(egui::Key::Enter, egui::Modifiers::NONE),
                &mut draw,
            );
            phone_frame(&ctx, size, vec![], &mut draw);
            phone_frame(
                &ctx,
                size,
                key(egui::Key::ArrowRight, egui::Modifiers::NONE),
                &mut draw,
            );
            for _ in 0..4 {
                phone_frame(&ctx, size, vec![], &mut draw);
            }
            assert!(menu.visible, "{chosen}: submenu hidden");
            let rectangles = popup_bounds(&ctx);
            assert!(rectangles.len() >= 2);
            for rect in rectangles {
                assert!(
                    rect.left() >= -1.0
                        && rect.right() <= size.x + 1.0
                        && rect.top() >= -1.0
                        && rect.bottom() <= size.y + 1.0,
                    "{chosen} {size:?}: {rect:?}"
                );
            }
        }
    }
}
