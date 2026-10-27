//! Touch target geometry, gesture cancellation and assistive technology contracts.
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
    mut draw: impl FnMut(&mut egui::Ui) -> R,
) -> (R, egui::FullOutput) {
    let mut value = None;
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(390.0, 900.0),
            )),
            events,
            ..Default::default()
        },
        |ui| value = Some(draw(ui)),
    );
    output.textures_delta.clear();
    (value.unwrap(), output)
}
fn touch(phase: egui::TouchPhase, pos: egui::Pos2) -> egui::Event {
    egui::Event::Touch {
        device_id: egui::TouchDeviceId(1),
        id: egui::TouchId(1),
        phase,
        pos,
        force: None,
    }
}
fn button(pos: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        pressed,
        button: egui::PointerButton::Primary,
        modifiers: egui::Modifiers::NONE,
    }
}
fn prime(ctx: &egui::Context) {
    let pos = egui::pos2(380.0, 880.0);
    frame(
        ctx,
        vec![
            touch(egui::TouchPhase::Start, pos),
            touch(egui::TouchPhase::End, pos),
        ],
        |_| {},
    );
}
fn key(key: egui::Key, shift: bool) -> Vec<egui::Event> {
    vec![egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: if shift {
            egui::Modifiers::SHIFT
        } else {
            egui::Modifiers::NONE
        },
    }]
}

#[derive(Clone, Copy, Debug)]
enum Control {
    Button,
    ButtonIcon,
    Icon,
    Checkbox,
    Radio,
    Switch,
    Menu,
    Row,
    Dropdown,
    Colour,
}
const CONTROLS: [Control; 10] = [
    Control::Button,
    Control::ButtonIcon,
    Control::Icon,
    Control::Checkbox,
    Control::Radio,
    Control::Switch,
    Control::Menu,
    Control::Row,
    Control::Dropdown,
    Control::Colour,
];
#[derive(Default)]
struct Model {
    on: bool,
    selected: usize,
    colour: egui::Color32,
}
fn control(ui: &mut egui::Ui, kind: Control, model: &mut Model) -> egui::Response {
    match kind {
        Control::Button => ui.add(look::Button::new("Go")),
        Control::ButtonIcon => {
            ui.add(look::Button::new("Go").icon("+", egui::FontId::proportional(16.0)))
        }
        Control::Icon => ui.add(look::IconButton::new("+").accessible_label("Add")),
        Control::Checkbox => ui.add(look::Checkbox::new(&mut model.on, "Check")),
        Control::Radio => ui.add(look::Radio::new(false, "Radio")),
        Control::Switch => ui.add(look::Switch::new(&mut model.on).accessible_label("Power")),
        Control::Menu => ui.add(look::MenuItem::new("Menu")),
        Control::Row => ui.add(look::ListRow::new(false).accessible_label("Row")),
        Control::Dropdown => ui.add(look::Dropdown::new(
            "options",
            &mut model.selected,
            &["One", "Two"],
        )),
        Control::Colour => ui.add(look::ColorWell::new(&mut model.colour)),
    }
}

#[test]
fn touch_targets_reserve_space_and_accept_padding_taps_in_every_look() {
    for chosen in LOOKS {
        for kind in CONTROLS {
            let ctx = context(chosen);
            prime(&ctx);
            let mut model = Model::default();
            let (response, _) = frame(&ctx, vec![], |ui| control(ui, kind, &mut model));
            assert!(
                response.rect.width() >= 48.0 && response.rect.height() >= 48.0,
                "{chosen} {kind:?}: {:?}",
                response.rect
            );
            let point = response.rect.center();
            frame(
                &ctx,
                vec![
                    touch(egui::TouchPhase::Start, point),
                    egui::Event::PointerMoved(point),
                    button(point, true),
                ],
                |ui| control(ui, kind, &mut model),
            );
            let (response, _) = frame(
                &ctx,
                vec![touch(egui::TouchPhase::End, point), button(point, false)],
                |ui| control(ui, kind, &mut model),
            );
            assert!(
                response.clicked() || response.changed(),
                "{chosen} {kind:?}: valid tap rejected"
            );
        }
        let ctx = context(chosen);
        prime(&ctx);
        let draw = |ui: &mut egui::Ui| {
            ui.horizontal(|ui| {
                (
                    ui.add(look::IconButton::new("+")),
                    ui.add(look::IconButton::new("−")),
                )
            })
            .inner
        };
        let ((first, second), _) = frame(&ctx, vec![], draw);
        assert!(first.rect.right() <= second.rect.left());
        let point = egui::pos2(first.rect.left() + 0.5, first.rect.center().y);
        frame(
            &ctx,
            vec![
                touch(egui::TouchPhase::Start, point),
                egui::Event::PointerMoved(point),
                button(point, true),
            ],
            draw,
        );
        let ((first, second), _) = frame(
            &ctx,
            vec![touch(egui::TouchPhase::End, point), button(point, false)],
            draw,
        );
        assert!(first.clicked(), "{chosen}: padding was not clickable");
        assert!(!second.clicked(), "{chosen}: overlapping targets");
    }
}

#[test]
fn cancelled_touch_never_activates_on_a_late_release() {
    for chosen in LOOKS {
        for kind in CONTROLS {
            for cancel in 0..4 {
                let ctx = context(chosen);
                prime(&ctx);
                let mut model = Model::default();
                let (response, _) = frame(&ctx, vec![], |ui| control(ui, kind, &mut model));
                let start = response.rect.center();
                frame(
                    &ctx,
                    vec![
                        touch(egui::TouchPhase::Start, start),
                        egui::Event::PointerMoved(start),
                        button(start, true),
                    ],
                    |ui| control(ui, kind, &mut model),
                );
                let events = match cancel {
                    0 => vec![touch(egui::TouchPhase::Cancel, start)],
                    1 => vec![egui::Event::WindowFocused(false)],
                    2 => {
                        let outside = start + egui::vec2(0.0, 100.0);
                        vec![
                            touch(egui::TouchPhase::Move, outside),
                            egui::Event::PointerMoved(outside),
                        ]
                    }
                    _ => {
                        let outside = start + egui::vec2(0.0, 12.0);
                        vec![
                            touch(egui::TouchPhase::Move, outside),
                            egui::Event::PointerMoved(outside),
                        ]
                    }
                };
                frame(&ctx, events, |ui| control(ui, kind, &mut model));
                frame(
                    &ctx,
                    vec![
                        egui::Event::WindowFocused(true),
                        touch(egui::TouchPhase::Move, start),
                        egui::Event::PointerMoved(start),
                    ],
                    |ui| control(ui, kind, &mut model),
                );
                let (response, output) = frame(
                    &ctx,
                    vec![touch(egui::TouchPhase::End, start), button(start, false)],
                    |ui| control(ui, kind, &mut model),
                );
                assert!(
                    output.platform_output.events.iter().all(|event| !matches!(
                        event,
                        egui::output::OutputEvent::Clicked(_)
                            | egui::output::OutputEvent::ValueChanged(_)
                    )),
                    "{chosen} {kind:?}: cancelled event announced"
                );
                assert!(
                    !response.clicked()
                        && !response.changed()
                        && !model.on
                        && model.selected == 0
                        && !egui::Popup::is_any_open(&ctx),
                    "{chosen} {kind:?}: cancel={cancel}"
                );
            }
        }
    }
}

#[test]
fn icon_glyphs_keep_their_size_when_touch_targets_grow() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        let (_, mouse) = frame(&ctx, vec![], |ui| {
            ui.add(look::IconButton::new("+").selected(true))
        });
        prime(&ctx);
        let (_, touch) = frame(&ctx, vec![], |ui| {
            ui.add(look::IconButton::new("+").selected(true))
        });
        let text_sizes = |output: &egui::FullOutput| {
            output
                .shapes
                .iter()
                .filter_map(|shape| {
                    if let egui::Shape::Text(text) = &shape.shape {
                        Some(text.galley.size())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(
            text_sizes(&mouse),
            text_sizes(&touch),
            "{chosen}: icon grew"
        );
    }
}

#[test]
fn accessibility_exposes_names_selection_values_and_disabled_states() {
    for chosen in LOOKS {
        for enabled in [true, false] {
            let ctx = context(chosen);
            ctx.enable_accesskit();
            let mut colour = egui::Color32::from_rgba_unmultiplied(20, 40, 60, 128);
            let (ids, output) = frame(&ctx, vec![], |ui| {
                ui.add_enabled_ui(enabled, |ui| {
                    let icon = ui.add(
                        look::IconButton::new("+")
                            .tooltip("Tooltip")
                            .accessible_label("Add layer"),
                    );
                    let row = ui.add(look::ListRow::new(true).accessible_label("Ground"));
                    let menu = ui.add(look::MenuItem::new("Save"));
                    let check = ui.add(look::MenuItem::new("Grid").checked(false));
                    let well = ui.add(look::ColorWell::new(&mut colour).label("Ink").alpha(true));
                    let tab = ui.add(look::Tab::new("Map", true));
                    let section = look::Section::new("Details")
                        .open(Some(true))
                        .show(ui, |_| {});
                    [
                        icon.id,
                        row.id,
                        menu.id,
                        check.id,
                        well.id,
                        tab.id,
                        section.header_response.id,
                    ]
                })
                .inner
            });
            let update = output.platform_output.accesskit_update.unwrap();
            let node = |id: egui::Id| {
                &update
                    .nodes
                    .iter()
                    .find(|(node_id, _)| *node_id == id.accesskit_id())
                    .unwrap()
                    .1
            };
            for id in ids {
                assert_eq!(node(id).is_disabled(), !enabled);
            }
            assert_eq!(node(ids[0]).label(), Some("Add layer"));
            assert_eq!(node(ids[1]).role(), egui::Role::ListBoxOption);
            assert_eq!(node(ids[1]).label(), Some("Ground"));
            assert_eq!(node(ids[1]).is_selected(), Some(true));
            assert_eq!(node(ids[2]).role(), egui::Role::MenuItem);
            assert_eq!(node(ids[3]).role(), egui::Role::MenuItemCheckBox);
            assert_eq!(
                node(ids[3]).toggled(),
                Some(egui::accesskit::Toggled::False)
            );
            assert_eq!(node(ids[4]).value(), Some("#14283C80"));
            assert_eq!(node(ids[4]).color_value().unwrap().alpha, 128);
            assert_eq!(node(ids[4]).is_expanded(), Some(false));
            assert_eq!(node(ids[5]).is_selected(), Some(true));
            assert_eq!(node(ids[5]).toggled(), None);
            assert_eq!(node(ids[6]).is_expanded(), Some(true));
        }
    }
}

#[test]
fn keyboard_focus_follows_visible_order_and_skips_disabled_controls() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        let draw = |ui: &mut egui::Ui| {
            let first = ui.add(look::IconButton::new("+").accessible_label("Add"));
            let disabled = ui.add_enabled(false, look::IconButton::new("x"));
            let row = ui.add(look::ListRow::new(false).accessible_label("Ground"));
            let last = ui.add(look::Button::new("Done"));
            ([first.id, row.id, last.id], disabled.id)
        };
        let ((ids, disabled), _) = frame(&ctx, vec![], draw);
        for expected in ids {
            frame(&ctx, key(egui::Key::Tab, false), draw);
            frame(&ctx, vec![], draw);
            assert_eq!(
                ctx.memory(|memory| memory.focused()),
                Some(expected),
                "{chosen}: forward order"
            );
        }
        for expected in [ids[1], ids[0]] {
            frame(&ctx, key(egui::Key::Tab, true), draw);
            frame(&ctx, vec![], draw);
            assert_eq!(
                ctx.memory(|memory| memory.focused()),
                Some(expected),
                "{chosen}: reverse order"
            );
        }
        assert_ne!(ctx.memory(|memory| memory.focused()), Some(disabled));
    }
}

#[test]
fn tabs_expose_tab_roles_and_update_selection_on_keyboard_frame() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        let mut selected = 0;
        let draw = |ui: &mut egui::Ui, selected: &mut usize| {
            ui.add(look::Tabs::new("pages", selected, &["Map", "Layers"]))
        };
        frame(&ctx, vec![], |ui| draw(ui, &mut selected));
        frame(&ctx, key(egui::Key::Tab, false), |ui| {
            draw(ui, &mut selected)
        });
        ctx.enable_accesskit();
        // GTK's view switcher moves the focus with End and chooses with Space.
        let choose = if chosen == look::Look::Gtk {
            frame(&ctx, key(egui::Key::End, false), |ui| {
                draw(ui, &mut selected)
            });
            egui::Key::Space
        } else {
            egui::Key::End
        };
        let (_, output) = frame(&ctx, key(choose, false), |ui| draw(ui, &mut selected));
        assert_eq!(selected, 1, "{chosen}");
        let nodes = output.platform_output.accesskit_update.unwrap().nodes;
        assert!(
            nodes
                .iter()
                .any(|(_, node)| node.role() == egui::Role::TabList),
            "{chosen}: no tab list"
        );
        for (label, selected) in [("Map", false), ("Layers", true)] {
            let node = &nodes
                .iter()
                .find(|(_, node)| node.label() == Some(label))
                .unwrap()
                .1;
            assert_eq!(node.role(), egui::Role::Tab, "{chosen}: {label}");
            assert_eq!(
                node.is_selected(),
                Some(selected),
                "{chosen}: stale {label}"
            );
            assert_eq!(node.toggled(), None);
        }
    }
}

#[test]
fn dropdown_name_is_separate_from_value_and_icon_toggle_state_is_explicit() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        ctx.enable_accesskit();
        let mut selected = 1;
        let (ids, output) = frame(&ctx, vec![], |ui| {
            let dropdown = ui.add(
                look::Dropdown::new("unit", &mut selected, &["Metres", "Feet"])
                    .accessible_label("Units"),
            );
            let action = ui.add(look::IconButton::new("+").accessible_label("Add"));
            let toggle = ui.add(
                look::IconButton::new("#")
                    .selected(false)
                    .accessible_label("Grid"),
            );
            [dropdown.id, action.id, toggle.id]
        });
        let nodes = output.platform_output.accesskit_update.unwrap().nodes;
        let node = |id: egui::Id| {
            &nodes
                .iter()
                .find(|(node_id, _)| *node_id == id.accesskit_id())
                .unwrap()
                .1
        };
        assert_eq!(node(ids[0]).label(), Some("Units"));
        assert_eq!(node(ids[0]).value(), Some("Feet"));
        assert_eq!(node(ids[1]).toggled(), None);
        assert_eq!(
            node(ids[2]).toggled(),
            Some(egui::accesskit::Toggled::False)
        );
    }
}

#[test]
fn disabled_controls_do_not_advertise_click_or_focus_actions() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        ctx.enable_accesskit();
        let mut model = Model::default();
        let (ids, output) = frame(&ctx, vec![], |ui| {
            CONTROLS.map(|kind| {
                ui.add_enabled_ui(false, |ui| control(ui, kind, &mut model))
                    .inner
                    .id
            })
        });
        let nodes = output.platform_output.accesskit_update.unwrap().nodes;
        for (id, kind) in ids.into_iter().zip(CONTROLS) {
            let node = &nodes
                .iter()
                .find(|(node_id, _)| *node_id == id.accesskit_id())
                .unwrap()
                .1;
            assert!(node.is_disabled(), "{chosen} {kind:?}");
            assert!(
                !node.supports_action(egui::accesskit::Action::Click),
                "{chosen} {kind:?}: disabled click advertised"
            );
            assert!(
                !node.supports_action(egui::accesskit::Action::Focus),
                "{chosen} {kind:?}: disabled focus advertised"
            );
        }
    }
}

#[test]
fn scrolling_over_touch_buttons_keeps_parent_scroll_and_cancels_tap() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        prime(&ctx);
        let draw = |ui: &mut egui::Ui| {
            let area = egui::ScrollArea::vertical()
                .max_height(140.0)
                .show(ui, |ui| {
                    ui.add_space(30.0);
                    let response = ui.add(look::IconButton::new("+").accessible_label("Add"));
                    ui.add_space(400.0);
                    response
                });
            (area.inner, area.state.clamped_offset().y)
        };
        let ((response, _), _) = frame(&ctx, vec![], draw);
        frame(&ctx, vec![], draw);
        let start = response.rect.center();
        frame(
            &ctx,
            vec![
                touch(egui::TouchPhase::Start, start),
                egui::Event::PointerMoved(start),
                button(start, true),
            ],
            draw,
        );
        let mut end = start;
        let mut scrolled = 0.0;
        for distance in [12.0, 24.0, 40.0] {
            end = start - egui::vec2(0.0, distance);
            let ((response, offset), _) = frame(
                &ctx,
                vec![
                    touch(egui::TouchPhase::Move, end),
                    egui::Event::PointerMoved(end),
                ],
                draw,
            );
            assert!(!response.clicked());
            scrolled = offset;
        }
        let ((response, _), _) = frame(
            &ctx,
            vec![touch(egui::TouchPhase::End, end), button(end, false)],
            draw,
        );
        assert!(scrolled > 0.0, "{chosen}: button swallowed scrolling");
        assert!(!response.clicked());
    }
}

#[test]
fn text_number_slider_section_and_tabs_keep_touch_sized_targets() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        prime(&ctx);
        let mut text = String::new();
        let mut number = 5.0;
        let mut slider = 0.5;
        let mut selected = 0;
        let (rects, _) = frame(&ctx, vec![], |ui| {
            let text = ui.add(look::TextField::new(&mut text));
            let number = ui.add(look::NumberField::new(&mut number).width(200.0));
            let slider = ui.add(look::Slider::new(&mut slider, 0.0..=1.0));
            let section = look::Section::new("Settings").show(ui, |_| {});
            let tabs = ui.add(look::Tabs::new("short-tabs", &mut selected, &["X", "Y"]));
            [
                text.rect,
                number.rect,
                slider.rect,
                section.header_response.rect,
                tabs.rect,
            ]
        });
        for rect in rects {
            assert!(rect.height() >= 48.0, "{chosen}: {rect:?}");
        }
    }
}

#[test]
fn disabled_colour_picker_closes_without_changing_value() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        let mut colour = egui::Color32::RED;
        let draw = |ui: &mut egui::Ui, colour: &mut egui::Color32, enabled| {
            ui.add_enabled(enabled, look::ColorWell::new(colour))
        };
        let (response, _) = frame(&ctx, vec![], |ui| draw(ui, &mut colour, true));
        let point = response.rect.center();
        frame(
            &ctx,
            vec![egui::Event::PointerMoved(point), button(point, true)],
            |ui| draw(ui, &mut colour, true),
        );
        frame(&ctx, vec![button(point, false)], |ui| {
            draw(ui, &mut colour, true)
        });
        assert!(egui::Popup::is_any_open(&ctx));
        frame(&ctx, vec![], |ui| draw(ui, &mut colour, false));
        assert!(!egui::Popup::is_any_open(&ctx), "{chosen}");
        assert_eq!(colour, egui::Color32::RED);
    }
}

#[test]
fn touch_cancel_does_not_disable_later_keyboard_or_accessibility_activation() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        prime(&ctx);
        let draw = |ui: &mut egui::Ui| ui.add(look::IconButton::new("+").accessible_label("Add"));
        let (response, _) = frame(&ctx, vec![], draw);
        let point = response.rect.center();
        frame(
            &ctx,
            vec![
                touch(egui::TouchPhase::Start, point),
                egui::Event::PointerMoved(point),
                button(point, true),
            ],
            draw,
        );
        frame(&ctx, vec![touch(egui::TouchPhase::Cancel, point)], draw);
        frame(&ctx, vec![button(point, false)], draw);
        ctx.memory_mut(|memory| memory.request_focus(response.id));
        let (keyboard, _) = frame(&ctx, key(egui::Key::Space, false), draw);
        assert!(
            keyboard.clicked(),
            "{chosen}: keyboard blocked after cancel"
        );
        let request = egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Click,
            target_tree: egui::accesskit::TreeId::ROOT,
            target_node: response.id.accesskit_id(),
            data: None,
        });
        let (accessible, _) = frame(&ctx, vec![request], draw);
        assert!(
            accessible.clicked(),
            "{chosen}: accessibility click blocked after cancel"
        );
    }
}

#[test]
fn first_touch_keeps_existing_geometry_until_the_pointer_sequence_finishes() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        let mut checked = false;
        let draw = |ui: &mut egui::Ui, checked: &mut bool| {
            ui.add(look::Button::new("First"));
            ui.add(look::Checkbox::new(checked, "Second"))
        };
        let (before, _) = frame(&ctx, vec![], |ui| draw(ui, &mut checked));
        let point = before.rect.center();
        let (pressed, _) = frame(
            &ctx,
            vec![
                touch(egui::TouchPhase::Start, point),
                egui::Event::PointerMoved(point),
                button(point, true),
            ],
            |ui| draw(ui, &mut checked),
        );
        assert_eq!(
            before.rect, pressed.rect,
            "{chosen}: target moved under finger"
        );
        let (released, _) = frame(
            &ctx,
            vec![touch(egui::TouchPhase::End, point), button(point, false)],
            |ui| draw(ui, &mut checked),
        );
        assert_eq!(before.rect, released.rect);
        assert!(checked && released.changed(), "{chosen}: first touch lost");
        let (after, _) = frame(&ctx, vec![], |ui| draw(ui, &mut checked));
        assert_eq!(before.id, after.id);
        assert!(after.rect.height() >= 48.0);
    }
}

#[test]
fn disabled_editors_remove_actions_that_they_cannot_apply() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        ctx.enable_accesskit();
        let mut text = String::new();
        let mut number = 5.0;
        let mut slider = 0.5;
        let (ids, output) = frame(&ctx, vec![], |ui| {
            ui.add_enabled_ui(false, |ui| {
                [
                    ui.add(look::TextField::new(&mut text)).id,
                    ui.add(look::NumberField::new(&mut number)).id,
                    ui.add(look::Slider::new(&mut slider, 0.0..=1.0)).id,
                ]
            })
            .inner
        });
        let nodes = output.platform_output.accesskit_update.unwrap().nodes;
        for id in ids {
            let node = &nodes
                .iter()
                .find(|(node_id, _)| *node_id == id.accesskit_id())
                .unwrap()
                .1;
            assert!(node.is_disabled());
            for action in [
                egui::accesskit::Action::Click,
                egui::accesskit::Action::Focus,
                egui::accesskit::Action::SetValue,
            ] {
                assert!(
                    !node.supports_action(action),
                    "{chosen}: disabled editor advertises {action:?}"
                );
            }
        }
    }
}

#[test]
fn progress_and_busy_indicators_report_values_without_fake_completion() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        ctx.enable_accesskit();
        let mut selected = 1;
        let (ids, output) = frame(&ctx, vec![], |ui| {
            [
                ui.add(look::ProgressBar::new(0.25).accessible_label("Export"))
                    .id,
                ui.add(look::Spinner::new().accessible_label("Loading map"))
                    .id,
                ui.add(
                    look::Segmented::new("axis", &mut selected, &["X", "Y"])
                        .accessible_label("Axis"),
                )
                .id,
            ]
        });
        let nodes = output.platform_output.accesskit_update.unwrap().nodes;
        let node = |id: egui::Id| {
            &nodes
                .iter()
                .find(|(node_id, _)| *node_id == id.accesskit_id())
                .unwrap()
                .1
        };
        assert_eq!(node(ids[0]).label(), Some("Export"));
        assert_eq!(node(ids[0]).numeric_value(), Some(25.0));
        assert_eq!(node(ids[0]).min_numeric_value(), Some(0.0));
        assert_eq!(node(ids[0]).max_numeric_value(), Some(100.0));
        assert_eq!(node(ids[1]).role(), egui::Role::ProgressIndicator);
        assert!(node(ids[1]).is_busy());
        assert_eq!(node(ids[1]).numeric_value(), None);
        assert_eq!(node(ids[2]).label(), Some("Axis"));
        assert_eq!(node(ids[2]).value(), Some("Y"));
    }
}

#[test]
fn window_close_buttons_reserve_touch_targets_without_larger_glyphs() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        prime(&ctx);
        let mut open = true;
        let mut update = None;
        for _ in 0..3 {
            ctx.enable_accesskit();
            let (_, output) = frame(&ctx, vec![], |_ui| {
                look::ToolWindow::new("touch-window", "Tools")
                    .resizable(false)
                    .show(&ctx, &mut open, |_| {});
            });
            update = output.platform_output.accesskit_update;
        }
        let nodes = update.unwrap().nodes;
        let node = &nodes
            .iter()
            .find(|(_, node)| node.role() == egui::Role::Button && node.label() == Some("Close"))
            .unwrap()
            .1;
        let bounds = node.bounds().unwrap();
        assert!(
            bounds.width() >= 48.0 && bounds.height() >= 48.0,
            "{chosen}: {bounds:?}"
        );
        let point = egui::pos2(
            bounds.x0 as f32 + 1.0,
            ((bounds.y0 + bounds.y1) * 0.5) as f32,
        );
        let show = |ui: &mut egui::Ui, open: &mut bool| {
            look::ToolWindow::new("touch-window", "Tools")
                .resizable(false)
                .show(ui.ctx(), open, |_| {});
        };
        frame(
            &ctx,
            vec![
                touch(egui::TouchPhase::Start, point),
                egui::Event::PointerMoved(point),
                button(point, true),
            ],
            |ui| show(ui, &mut open),
        );
        frame(&ctx, vec![touch(egui::TouchPhase::Cancel, point)], |ui| {
            show(ui, &mut open)
        });
        frame(&ctx, vec![button(point, false)], |ui| show(ui, &mut open));
        assert!(open, "{chosen}: cancelled close activated");
        frame(
            &ctx,
            vec![
                touch(egui::TouchPhase::Start, point),
                egui::Event::PointerMoved(point),
                button(point, true),
            ],
            |ui| show(ui, &mut open),
        );
        frame(
            &ctx,
            vec![touch(egui::TouchPhase::End, point), button(point, false)],
            |ui| show(ui, &mut open),
        );
        assert!(!open, "{chosen}: close padding did not activate");
    }
}

#[test]
fn hidden_controls_and_second_fingers_do_not_resume_cancelled_taps() {
    for chosen in LOOKS {
        for second_finger in [false, true] {
            let ctx = context(chosen);
            prime(&ctx);
            let draw =
                |ui: &mut egui::Ui| ui.add(look::IconButton::new("+").accessible_label("Add"));
            let (response, _) = frame(&ctx, vec![], draw);
            let point = response.rect.center();
            frame(
                &ctx,
                vec![
                    touch(egui::TouchPhase::Start, point),
                    egui::Event::PointerMoved(point),
                    button(point, true),
                ],
                draw,
            );
            if second_finger {
                let mut start = touch(egui::TouchPhase::Start, point + egui::vec2(10.0, 0.0));
                if let egui::Event::Touch { id, .. } = &mut start {
                    *id = egui::TouchId(2);
                }
                frame(&ctx, vec![start], draw);
                let mut end = touch(egui::TouchPhase::End, point + egui::vec2(10.0, 0.0));
                if let egui::Event::Touch { id, .. } = &mut end {
                    *id = egui::TouchId(2);
                }
                frame(&ctx, vec![end], draw);
            } else {
                frame(&ctx, vec![touch(egui::TouchPhase::Cancel, point)], |_| {});
                frame(&ctx, vec![], draw);
            }
            let (response, _) = frame(
                &ctx,
                vec![touch(egui::TouchPhase::End, point), button(point, false)],
                draw,
            );
            assert!(
                !response.clicked(),
                "{chosen}, second_finger={second_finger}"
            );
        }
    }
}

#[test]
fn text_buttons_with_icons_keep_one_accessible_name_and_separate_glyphs() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        ctx.enable_accesskit();
        let ((plain, icon), output) = frame(&ctx, vec![], |ui| {
            let plain = ui.add(look::Button::new("Save"));
            let icon =
                ui.add(look::Button::new("Save").icon("+", egui::FontId::proportional(16.0)));
            (plain, icon)
        });
        // GTK reduces horizontal padding when an icon is present: 16 + 6 - 2*8 = 6.
        assert!(icon.rect.width() >= plain.rect.width() + 6.0, "{chosen}");
        let nodes = output.platform_output.accesskit_update.unwrap().nodes;
        let node = &nodes
            .iter()
            .find(|(id, _)| *id == icon.id.accesskit_id())
            .unwrap()
            .1;
        assert_eq!(node.role(), egui::accesskit::Role::Button);
        assert_eq!(node.label(), Some("Save"));
        let texts: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|shape| {
                if let egui::Shape::Text(text) = &shape.shape {
                    Some(text)
                } else {
                    None
                }
            })
            .collect();
        let glyph = texts.iter().find(|text| text.galley.text() == "+").unwrap();
        let label = texts
            .iter()
            .rfind(|text| text.galley.text() == "Save")
            .unwrap();
        assert!(
            glyph.pos.x + glyph.galley.size().x < label.pos.x,
            "{chosen}: glyph overlaps label"
        );
        assert!(icon.rect.contains(glyph.pos));
        assert!(icon.rect.contains(label.pos));
    }
}

#[test]
fn empty_button_icons_preserve_size_and_narrow_buttons_clip_their_content() {
    for chosen in LOOKS {
        let ctx = context(chosen);
        let ((plain, empty, narrow), output) = frame(&ctx, vec![], |ui| {
            let plain = ui.add(look::Button::new("Save"));
            let empty =
                ui.add(look::Button::new("Save").icon("", egui::FontId::proportional(80.0)));
            let narrow = ui.add(
                look::Button::new("A long action label")
                    .icon("+", egui::FontId::proportional(16.0))
                    .width(48.0),
            );
            (plain, empty, narrow)
        });
        assert_eq!(plain.rect.size(), empty.rect.size(), "{chosen}");
        assert_eq!(narrow.rect.width(), 48.0);
        let glyph = output.shapes.iter().find(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.text() == "+")).unwrap();
        assert!(
            narrow.rect.contains_rect(glyph.clip_rect),
            "{chosen}: icon escapes button clip"
        );
    }
}

#[test]
fn button_icon_colours_follow_the_label_in_accent_and_disabled_states() {
    for chosen in LOOKS {
        for light in [false, true] {
            for enabled in [false, true] {
                for accent in [false, true] {
                    let ctx = context(chosen);
                    ctx.set_theme(if light {
                        egui::Theme::Light
                    } else {
                        egui::Theme::Dark
                    });
                    let (_, output) = frame(&ctx, vec![], |ui| {
                        ui.add_enabled(enabled, look::Button::new("Save").selected(accent));
                        ui.add_enabled(
                            enabled,
                            look::Button::new("Save")
                                .icon("+", egui::FontId::proportional(16.0))
                                .selected(accent),
                        )
                    });
                    let texts: Vec<_> = output
                        .shapes
                        .iter()
                        .filter_map(|shape| {
                            if let egui::Shape::Text(text) = &shape.shape {
                                Some(text)
                            } else {
                                None
                            }
                        })
                        .collect();
                    let reference = texts
                        .iter()
                        .find(|text| text.galley.text() == "Save")
                        .unwrap();
                    let expected = reference
                        .override_text_color
                        .unwrap_or(reference.fallback_color);
                    for label in ["+", "Save"] {
                        let text = texts
                            .iter()
                            .rfind(|text| text.galley.text() == label)
                            .unwrap();
                        assert_eq!(
                            text.override_text_color.unwrap_or(text.fallback_color),
                            expected,
                            "{chosen}: {label}"
                        );
                    }
                }
            }
        }
    }
}
