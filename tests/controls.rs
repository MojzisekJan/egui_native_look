//! Every control draws and takes a click in every look and scheme.

use egui_native_look as look;

fn frame<R>(
    ctx: &egui::Context,
    events: Vec<egui::Event>,
    mut add: impl FnMut(&mut egui::Ui) -> R,
) -> R {
    let mut result = None;
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(340.0, 600.0),
        )),
        events,
        ..Default::default()
    };
    let mut output = ctx.run_ui(input, |ui| result = Some(add(ui)));
    // Nothing renders the font atlas here; egui asks that its updates are taken.
    output.textures_delta.clear();
    result.unwrap()
}

fn click(at: egui::Pos2) -> Vec<Vec<egui::Event>> {
    let press = |pressed| egui::Event::PointerButton {
        pos: at,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    };
    vec![
        vec![egui::Event::PointerMoved(at)],
        vec![press(true)],
        vec![press(false)],
    ]
}

/// The centres of a number field's Decrease and Increase buttons, wherever the look puts
/// them: on either side of the field, or stacked in a stepper after it.
fn step_points(
    ctx: &egui::Context,
    mut add: impl FnMut(&mut egui::Ui),
) -> (egui::Pos2, egui::Pos2) {
    ctx.enable_accesskit();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(340.0, 600.0),
        )),
        ..Default::default()
    };
    let mut output = ctx.run_ui(input, |ui| add(ui));
    output.textures_delta.clear();
    let nodes = output.platform_output.accesskit_update.unwrap().nodes;
    let centre = |label| {
        let bounds = nodes
            .iter()
            .find(|(_, node)| node.role() == egui::Role::Button && node.label() == Some(label))
            .and_then(|(_, node)| node.bounds())
            .unwrap_or_else(|| panic!("no {label} button"));
        egui::pos2(
            ((bounds.x0 + bounds.x1) * 0.5) as f32,
            ((bounds.y0 + bounds.y1) * 0.5) as f32,
        )
    };
    (centre("Decrease"), centre("Increase"))
}

/// A context with `chosen` installed, in the dark scheme or the light one.
fn context(chosen: look::Look, dark: bool) -> egui::Context {
    let ctx = egui::Context::default();
    look::install(&ctx, look::Theme::new(chosen));
    ctx.set_theme(if dark {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    });
    ctx
}

/// The open pop-up's rect: the topmost foreground area.
fn popup_rect(ctx: &egui::Context) -> Option<egui::Rect> {
    let layer = ctx.memory(|m| {
        m.layer_ids()
            .filter(|l| l.order == egui::Order::Foreground)
            .last()
    })?;
    ctx.memory(|m| m.area_rect(layer.id))
}

fn key(key: egui::Key) -> Vec<egui::Event> {
    vec![egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Default::default(),
    }]
}

/// The looks of systems, without the plain look's own controls.
const SYSTEM_LOOKS: [look::Look; 5] = [
    look::Look::Mac,
    look::Look::Windows,
    look::Look::Gtk,
    look::Look::Ios,
    look::Look::Material,
];

const LOOKS: [look::Look; 6] = [
    look::Look::Plain,
    look::Look::Mac,
    look::Look::Windows,
    look::Look::Gtk,
    look::Look::Ios,
    look::Look::Material,
];

#[test]
fn controls_take_clicks_in_every_look() {
    for chosen in LOOKS {
        for dark in [true, false] {
            let ctx = context(chosen, dark);
            let mut checked = false;
            for events in click(egui::pos2(6.0, 10.0)) {
                frame(&ctx, events, |ui| {
                    ui.add(look::Checkbox::new(&mut checked, "Snap"))
                });
            }
            assert!(checked, "{chosen}: the check box did not take the click");
            let mut chosen_value = 0;
            for events in click(egui::pos2(6.0, 10.0)) {
                frame(&ctx, events, |ui| {
                    look::radio_value(ui, &mut chosen_value, 1, "Rotate")
                });
            }
            assert_eq!(
                chosen_value, 1,
                "{chosen}: the radio button did not take the click"
            );
            let mut on = false;
            for events in click(egui::pos2(15.0, 10.0)) {
                frame(&ctx, events, |ui| ui.add(look::Switch::new(&mut on)));
            }
            assert!(on, "{chosen}: the switch did not take the click");
            let mut segment = 0;
            for events in click(egui::pos2(250.0, 10.0)) {
                frame(&ctx, events, |ui| {
                    ui.add(
                        look::Segmented::new("axis", &mut segment, &["X", "Y", "Z"]).width(300.0),
                    )
                });
            }
            assert_eq!(
                segment, 2,
                "{chosen}: the segmented choice did not take the click"
            );
            let mut pressed = false;
            for events in click(egui::pos2(20.0, 10.0)) {
                pressed |= frame(&ctx, events, |ui| {
                    ui.add(look::Button::new("Apply")).clicked()
                });
            }
            assert!(pressed, "{chosen}: the button did not take the click");
            let mut value = 50.0;
            for events in click(egui::pos2(190.0, 10.0)) {
                frame(&ctx, events, |ui| {
                    ui.add(look::Slider::new(&mut value, 0.0..=100.0).width(200.0))
                });
            }
            assert!(
                value > 90.0,
                "{chosen}: the slider did not follow the click ({value})"
            );
        }
    }
}

#[test]
fn a_dropdown_picks_from_its_list() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut shape = 0;
        let add = |ui: &mut egui::Ui, shape: &mut usize| {
            ui.add(
                look::Dropdown::new("shape", shape, &["Cuboid", "Cylinder", "Sphere"]).width(160.0),
            )
        };
        for events in click(egui::pos2(40.0, 10.0)) {
            frame(&ctx, events, |ui| add(ui, &mut shape));
        }
        // The list opens under the button; its third row.
        frame(&ctx, vec![], |ui| add(ui, &mut shape));
        let row = look::theme(&ctx).metrics.control_height.max(20.0);
        let list = popup_rect(&ctx).expect("the dropdown is open");
        let third = egui::pos2(list.center().x, list.bottom() - 4.0 - row / 2.0);
        for events in click(third) {
            frame(&ctx, events, |ui| add(ui, &mut shape));
        }
        assert_eq!(
            shape, 2,
            "{chosen}: the dropdown did not pick from its list"
        );
    }
}

#[test]
fn themes_live_in_their_context() {
    let mac = context(look::Look::Mac, true);
    let gtk = context(look::Look::Gtk, true);
    assert_eq!(look::look(&mac), look::Look::Mac);
    assert_eq!(look::look(&gtk), look::Look::Gtk);
    assert_ne!(look::colors(&mac).button, look::colors(&gtk).button);
    gtk.set_theme(egui::Theme::Light);
    assert!(
        !look::colors(&gtk).dark && look::colors(&mac).dark,
        "the scheme follows each context's egui theme"
    );
}

#[test]
fn looks_have_names() {
    for (name, chosen, label) in [
        ("plain", look::Look::Plain, "Plain"),
        ("mac", look::Look::Mac, "macOS"),
        ("windows", look::Look::Windows, "Windows"),
        ("gtk", look::Look::Gtk, "GTK"),
        ("ios", look::Look::Ios, "iOS"),
        ("material", look::Look::Material, "Material"),
    ] {
        assert_eq!(look::Look::from_name(name), Some(chosen));
        assert_eq!(chosen.label(), label);
    }
    assert_eq!(look::Look::from_name("motif"), None);
}

#[test]
fn shortcuts_read_as_each_system_writes_them() {
    let save_as = egui::KeyboardShortcut::new(
        egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
        egui::Key::S,
    );
    assert_eq!(look::shortcut_text(look::Look::Mac, &save_as), "⇧⌘S");
    assert_eq!(
        look::shortcut_text(look::Look::Windows, &save_as),
        "Ctrl+Shift+S"
    );
}

#[test]
fn an_icon_button_and_a_colour_well_take_clicks() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut pressed = false;
        for events in click(egui::pos2(10.0, 10.0)) {
            pressed |= frame(&ctx, events, |ui| {
                ui.add(look::IconButton::new("#").tooltip("Grid")).clicked()
            });
        }
        assert!(pressed, "{chosen}: the icon button did not take the click");
        let mut tint = egui::Color32::RED;
        let mut opened = false;
        for events in click(egui::pos2(12.0, 10.0)) {
            frame(&ctx, events, |ui| ui.add(look::ColorWell::new(&mut tint)));
            opened |= egui::Popup::is_any_open(&ctx);
        }
        frame(&ctx, vec![], |ui| ui.add(look::ColorWell::new(&mut tint)));
        assert!(
            opened || egui::Popup::is_any_open(&ctx),
            "{chosen}: the colour well did not open its picker"
        );
    }
}

#[test]
fn a_form_lines_its_controls_up() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut rects = Vec::new();
        for _ in 0..3 {
            rects = frame(&ctx, vec![], |ui| {
                look::Form::new("form").show(ui, |form| {
                    let a = form
                        .row("Radius", |ui| ui.add(look::Button::new("A").width(60.0)))
                        .rect;
                    let b = form
                        .row("A much longer label", |ui| {
                            ui.add(look::Button::new("B").width(60.0))
                        })
                        .rect;
                    vec![a, b]
                })
            });
        }
        assert!(
            (rects[0].left() - rects[1].left()).abs() < 0.5,
            "{chosen}: the controls start at different places: {rects:?}"
        );
        assert!(
            rects[1].top() > rects[0].bottom(),
            "{chosen}: the rows overlap"
        );
    }
}

#[test]
fn a_toast_goes_after_a_while() {
    let ctx = context(look::Look::Gtk, true);
    let run = |time: f64| {
        let input = egui::RawInput {
            time: Some(time),
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| look::show_toasts(ui.ctx()));
        output.textures_delta.clear();
        output.shapes.len()
    };
    let before = run(0.0);
    look::toast(&ctx, "Saved");
    let shown = run(1.0);
    assert!(shown > before, "the toast was not drawn");
    run(10.0);
    assert_eq!(run(10.1), before, "the toast stayed");
}

#[test]
fn a_toast_waits_under_the_pointer_and_is_announced() {
    let ctx = context(look::Look::Gtk, true);
    ctx.enable_accesskit();
    let run = |time: f64, events: Vec<egui::Event>| {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, 300.0),
            )),
            time: Some(time),
            events,
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| look::show_toasts(ui.ctx()));
        output.textures_delta.clear();
        let nodes = output.platform_output.accesskit_update.unwrap().nodes;
        nodes
            .iter()
            .find(|(_, node)| node.role() == egui::Role::Status)
            .map(|(_, node)| {
                assert_eq!(node.live(), Some(egui::accesskit::Live::Polite));
                assert_eq!(node.label(), Some("Saved"));
                node.bounds().unwrap()
            })
    };
    run(0.0, vec![]);
    look::toast(&ctx, "Saved");
    // The toasts' area measures itself in its first frame.
    run(0.9, vec![]);
    let bounds = run(1.0, vec![]).expect("the toast is announced");
    let over = egui::pos2(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
    // Held under the pointer well past its time.
    let mut time = 1.0;
    while time < 8.0 {
        time += 0.1;
        assert!(
            run(time, vec![egui::Event::PointerMoved(over)]).is_some(),
            "gone under the pointer at {time}"
        );
    }
    run(time, vec![egui::Event::PointerGone]);
    assert!(run(time + 4.5, vec![]).is_none(), "the toast stayed");
}

#[test]
fn a_popup_button_shows_the_apps_own_list() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut units = 0;
        let add = |ui: &mut egui::Ui, units: &mut usize| {
            look::PopupButton::new("units", "Metres")
                .width(160.0)
                .show(ui, |ui| {
                    look::menu_value(ui, units, 0, "Metres");
                    look::menu_value(ui, units, 1, "Feet");
                })
        };
        let mut rows = Vec::new();
        for events in click(egui::pos2(40.0, 10.0)) {
            frame(&ctx, events, |ui| add(ui, &mut units));
        }
        // The list is open now; find the second row and click it.
        frame(&ctx, vec![], |ui| {
            if let Some(()) = add(ui, &mut units).inner {
                rows.push(());
            }
        });
        assert!(!rows.is_empty(), "{chosen}: the list did not open");
        let feet = popup_rect(&ctx).map(|r| egui::pos2(r.center().x, r.bottom() - 16.0));
        let feet = feet.expect("the list has an area");
        for events in click(feet) {
            frame(&ctx, events, |ui| add(ui, &mut units));
        }
        assert_eq!(units, 1, "{chosen}: the row did not set the value");
    }
}

#[test]
fn a_slider_takes_any_number_logarithmic_and_in_steps() {
    let ctx = context(look::Look::Mac, true);
    let mut samples: u32 = 1;
    for events in click(egui::pos2(100.0, 10.0)) {
        frame(&ctx, events, |ui| {
            ui.add(look::Slider::new(&mut samples, 0..=64).width(200.0))
        });
    }
    assert!(
        (30..=34).contains(&samples),
        "the middle of 0 to 64 is not {samples}"
    );
    let mut exposure: f64 = 1.0;
    for events in click(egui::pos2(100.0, 10.0)) {
        frame(&ctx, events, |ui| {
            ui.add(
                look::Slider::new(&mut exposure, 0.01..=100.0)
                    .logarithmic(true)
                    .width(200.0),
            )
        });
    }
    assert!(
        (0.8..=1.25).contains(&exposure),
        "the middle of 0.01 to 100 on a log scale is not {exposure}"
    );
    let mut stepped: f64 = 0.0;
    for events in click(egui::pos2(97.0, 10.0)) {
        frame(&ctx, events, |ui| {
            ui.add(
                look::Slider::new(&mut stepped, 0.0..=100.0)
                    .step_by(25.0)
                    .width(200.0),
            )
        });
    }
    assert_eq!(stepped, 50.0);
}

#[test]
fn tabs_one_by_one() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut shown = 0;
        let mut rects = Vec::new();
        for events in click(egui::pos2(110.0, 12.0)) {
            rects = frame(&ctx, events, |ui| {
                look::TabBar::new()
                    .show(ui, |bar| {
                        ["One", "Two", "Three"]
                            .into_iter()
                            .enumerate()
                            .map(|(i, name)| {
                                let tab = bar.tab(name, shown == i);
                                if tab.clicked() {
                                    shown = i;
                                }
                                tab.rect
                            })
                            .collect::<Vec<_>>()
                    })
                    .inner
            });
        }
        let picked = rects
            .iter()
            .position(|r| r.contains(egui::pos2(110.0, 12.0)));
        assert_eq!(
            Some(shown),
            picked,
            "{chosen}: the click did not pick the tab under it: {rects:?}"
        );
    }
}

#[test]
fn a_menu_item_is_as_wide_as_it_needs() {
    let ctx = context(look::Look::Mac, true);
    let width = frame(&ctx, vec![], |ui| {
        ui.add(look::MenuItem::new("Save").shortcut("⌘S"))
            .rect
            .width()
    });
    assert!(width < 140.0, "a short item is {width} wide");
}

#[test]
fn a_section_answers_and_opens_from_code() {
    let ctx = context(look::Look::Gtk, true);
    let shown = frame(&ctx, vec![], |ui| {
        look::Section::new("Layers")
            .open(Some(true))
            .show(ui, |ui| ui.label("body"))
    });
    assert!(shown.body_returned.is_some() && shown.header_response.rect.width() > 0.0);
    // Closed from code, it goes on closing over the next frames.
    for _ in 0..20 {
        frame(&ctx, vec![], |ui| {
            look::Section::new("Layers")
                .open(Some(false))
                .show(ui, |ui| ui.label("body"))
        });
    }
    let shown = frame(&ctx, vec![], |ui| {
        look::Section::new("Layers").show(ui, |ui| ui.label("body"))
    });
    assert!(shown.body_returned.is_none() && shown.openness == 0.0);
}

#[test]
fn a_text_field_can_select_all_on_focus() {
    let ctx = context(look::Look::Windows, true);
    let mut text = String::from("12.5");
    let mut id = egui::Id::NULL;
    for events in click(egui::pos2(40.0, 12.0)) {
        id = frame(&ctx, events, |ui| {
            ui.add(
                look::TextField::new(&mut text)
                    .select_all_on_focus(true)
                    .width(200.0),
            )
            .id
        });
    }
    let range = egui::TextEdit::load_state(&ctx, id)
        .and_then(|s| s.cursor.char_range())
        .expect("a cursor");
    assert_eq!(
        (
            range.primary.index.min(range.secondary.index),
            range.primary.index.max(range.secondary.index)
        ),
        (0.into(), 4.into())
    );
    // Without it, a click puts the caret down and selects nothing.
    let ctx = context(look::Look::Windows, true);
    for events in click(egui::pos2(40.0, 12.0)) {
        id = frame(&ctx, events, |ui| {
            ui.add(look::TextField::new(&mut text).width(200.0)).id
        });
    }
    let range = egui::TextEdit::load_state(&ctx, id)
        .and_then(|s| s.cursor.char_range())
        .expect("a cursor");
    assert_eq!(range.primary.index, range.secondary.index);
}

#[test]
fn metrics_scale_and_a_touch_look_is_finger_sized() {
    let metrics = look::Metrics::of(look::Look::Mac).scaled(2.0);
    assert_eq!(
        (metrics.control_height, metrics.radius, metrics.body.size),
        (48.0, 12, 26.0)
    );
    // A touch look draws its own size (iOS 34 points) but reserves a finger's target on a
    // touch screen.
    let ctx = context(look::Look::touch(), false);
    let at = egui::pos2(300.0, 500.0);
    let tap = |phase| egui::Event::Touch {
        device_id: egui::TouchDeviceId(1),
        id: egui::TouchId(1),
        phase,
        pos: at,
        force: None,
    };
    frame(
        &ctx,
        vec![tap(egui::TouchPhase::Start), tap(egui::TouchPhase::End)],
        |_| {},
    );
    let target = frame(&ctx, vec![], |ui| ui.add(look::Button::new("Save")).rect);
    assert!(target.height() >= 48.0, "{target:?}");
}

#[test]
fn material_sizes_its_buttons_and_fields_as_material_3() {
    let ctx = context(look::Look::Material, false);
    let (button, field) = frame(&ctx, vec![], |ui| {
        let button = ui.add(look::Button::new("OK")).rect;
        let field = ui
            .add(look::TextField::new(&mut String::new()).width(200.0))
            .rect;
        (button, field)
    });
    // A short label still makes a 64-point button; a filled field without a label is 48.
    assert!(button.width() >= 64.0, "{button:?}");
    assert_eq!(field.height(), 48.0);
}

#[test]
fn gtk_joins_its_spin_buttons_to_the_field() {
    let ctx = context(look::Look::Gtk, false);
    let mut value = 8.0;
    let field = frame(&ctx, vec![], |ui| {
        ui.add(
            look::NumberField::new(&mut value)
                .range(0.0..=10.0)
                .width(106.0),
        )
        .rect
    });
    // One 34-point box as wide as asked: the number, then − and + with no gap between.
    assert_eq!((field.width(), field.height()), (106.0, 34.0));
    let plus = field.right_center() - egui::vec2(16.0, 0.0);
    for events in click(plus) {
        frame(&ctx, events, |ui| {
            ui.add(
                look::NumberField::new(&mut value)
                    .range(0.0..=10.0)
                    .width(106.0),
            )
        });
    }
    assert_eq!(value, 9.0);
}

#[test]
fn each_look_opens_in_its_systems_window_colours() {
    // A theme starts from its system's window and text colours; the plain look from the
    // shared default palette.
    let rgb = |c: u32| egui::Color32::from_rgb((c >> 16) as u8, (c >> 8) as u8, c as u8);
    for (chosen, light, dark) in [
        (look::Look::Mac, 0xffffff, 0x1e1e1e),
        (look::Look::Windows, 0xf3f3f3, 0x202020),
        (look::Look::Gtk, 0xfafafb, 0x222226),
        (look::Look::Ios, 0xffffff, 0x000000),
        (look::Look::Material, 0xfef7ff, 0x141218),
    ] {
        let theme = look::Theme::new(chosen);
        assert_eq!(theme.schemes.light.bg, rgb(light), "{chosen}");
        assert_eq!(theme.schemes.dark.bg, rgb(dark), "{chosen}");
    }
    assert_eq!(
        look::Theme::new(look::Look::Plain).schemes,
        look::Schemes::default()
    );
}

#[test]
fn shared_dialog_actions_are_equal_from_the_first_frame() {
    for chosen in [look::Look::Gtk, look::Look::Windows, look::Look::Ios] {
        let ctx = context(chosen, false);
        let mut open = true;
        let mut buttons = Vec::new();
        frame(&ctx, vec![], |_ui| {
            buttons.clear();
            look::Dialog::new("first", "Save changes?").show(
                &ctx,
                &mut open,
                |ui| {
                    ui.label("Save your changes?");
                },
                |ui| {
                    buttons.push(ui.add(look::Button::new("Save").accent()).rect);
                    buttons.push(ui.add(look::Button::new("Don't save")).rect);
                },
            );
        });
        // libadwaita, WinUI and iOS share the row equally, without a frame of natural widths.
        assert_eq!(buttons.len(), 2, "{chosen}");
        assert!(
            (buttons[0].width() - buttons[1].width()).abs() < 0.5,
            "{chosen}: {buttons:?}"
        );
    }
}

#[test]
fn more_contrast_outlines_controls_and_darkens_secondary_text() {
    for chosen in [
        look::Look::Gtk,
        look::Look::Mac,
        look::Look::Ios,
        look::Look::Material,
    ] {
        let normal = look::Colors::resolve(&look::Theme::new(chosen), false);
        let more = look::Colors::resolve(&look::Theme::new(chosen).high_contrast(true), false);
        // Secondary text closer to the text colour than before.
        let distance = |a: egui::Color32, b: egui::Color32| {
            (i32::from(a.r()) - i32::from(b.r())).abs()
                + (i32::from(a.g()) - i32::from(b.g())).abs()
        };
        assert!(
            distance(more.dim, more.fg) < distance(normal.dim, normal.fg),
            "{chosen}"
        );
        // A button takes an edge it had not.
        let ctx = egui::Context::default();
        look::install(&ctx, look::Theme::new(chosen).high_contrast(true));
        let (rect, output) = {
            let mut rect = egui::Rect::NOTHING;
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                rect = ui.add(look::Button::new("Edge")).rect;
            });
            output.textures_delta.clear();
            (rect, output)
        };
        let edged = output.shapes.iter().any(|shape| {
            matches!(&shape.shape, egui::Shape::Rect(r)
                if r.rect == rect && r.stroke.width >= 1.0 && r.stroke.color.a() > 0)
        });
        assert!(edged, "{chosen}: no edge round the button");
    }
}

#[test]
fn a_tool_window_shows_its_contents() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut open = true;
        let shown = frame(&ctx, vec![], |_ui| {
            look::ToolWindow::new("tool", "Fillet").show(&ctx, &mut open, |ui| ui.label("Radius"))
        });
        assert!(
            shown.is_some_and(|s| s.inner.is_some()),
            "{chosen}: nothing shown"
        );
    }
}

#[test]
fn a_number_field_without_steppers_is_the_field_alone_and_need_not_have_a_range() {
    let ctx = context(look::Look::Mac, true);
    let mut value = 5000.0_f64;
    let with = frame(&ctx, vec![], |ui| {
        ui.add(look::NumberField::new(&mut value).width(140.0))
            .rect
            .width()
    });
    let without = frame(&ctx, vec![], |ui| {
        ui.add(
            look::NumberField::new(&mut value)
                .steppers(false)
                .width(140.0),
        )
        .rect
        .width()
    });
    assert!(
        (with - 140.0).abs() < 1.0 && (without - 140.0).abs() < 1.0,
        "{with} {without}"
    );
    let alone = frame(&ctx, vec![], |ui| {
        ui.add(look::NumberField::new(&mut value).steppers(false))
            .rect
            .width()
    });
    assert!(alone < 70.0, "{alone}");
    // No range: a value far beyond any default stays.
    let mut far = 1.0e9_f64;
    frame(&ctx, vec![], |ui| ui.add(look::NumberField::new(&mut far)));
    assert_eq!(far, 1.0e9);
    let mut bounded = 200_i32;
    frame(&ctx, vec![], |ui| {
        ui.add(look::NumberField::new(&mut bounded).range(0..=100))
    });
    assert_eq!(bounded, 100);
}

#[test]
fn a_sliders_number_takes_a_suffix_and_a_formatter() {
    let ctx = context(look::Look::Gtk, false);
    let mut angle = 90.0_f32;
    let plain = frame(&ctx, vec![], |ui| {
        ui.add(look::Slider::new(&mut angle, 0.0..=360.0).show_value(true))
            .rect
            .width()
    });
    let long = frame(&ctx, vec![], |ui| {
        ui.add(
            look::Slider::new(&mut angle, 0.0..=360.0)
                .show_value(true)
                .suffix(" degrees of turn"),
        )
        .rect
        .width()
    });
    assert!(
        long > plain + 20.0,
        "the suffix should widen the number: {plain} {long}"
    );
    let mut scale = 1.5_f64;
    frame(&ctx, vec![], |ui| {
        ui.add(
            look::Slider::new(&mut scale, 0.5..=4.0)
                .show_value(true)
                .custom_formatter(|v, _| format!("{v:.1}×"))
                .fixed_decimals(1),
        )
    });
}

#[test]
fn a_submenu_item_keeps_its_icon() {
    let ctx = context(look::Look::Windows, true);
    let font = egui::FontId::proportional(14.0);
    let (with_icon, plain) = frame(&ctx, vec![], |ui| {
        let a = look::MenuItem::new("Export")
            .icon("E", font.clone())
            .submenu(ui, |ui| ui.add(look::MenuItem::new("Godot")))
            .response;
        let b = look::submenu(ui, "Export", |ui| ui.add(look::MenuItem::new("Godot"))).response;
        (a.rect.width(), b.rect.width())
    });
    assert_eq!(
        with_icon, plain,
        "the icon sits in the column the label already leaves"
    );
}

#[test]
fn a_palette_changes_a_few_colours_and_install_keeps_the_apps_style() {
    let dark = look::Palette {
        bg: egui::Color32::from_rgb(10, 20, 30),
        surface: egui::Color32::from_rgb(40, 50, 60),
        ..look::Palette::of(true)
    };
    assert_eq!(dark.fg, look::Palette::of(true).fg);
    let ctx = egui::Context::default();
    let shadow = egui::Shadow {
        offset: [0, 2],
        blur: 3,
        spread: 0,
        color: egui::Color32::RED,
    };
    ctx.all_styles_mut(|style| {
        style.visuals.window_shadow = shadow;
        style.spacing.item_spacing = egui::vec2(11.0, 7.0);
        style
            .text_styles
            .insert(egui::TextStyle::Heading, egui::FontId::proportional(31.0));
    });
    look::install(
        &ctx,
        look::Theme::new(look::Look::Plain).schemes(look::Schemes {
            dark,
            light: look::Palette::of(false),
        }),
    );
    let style = ctx.style_of(egui::Theme::Dark);
    assert_eq!(style.visuals.window_shadow, shadow);
    assert_eq!(style.spacing.item_spacing, egui::vec2(11.0, 7.0));
    assert_eq!(style.text_styles[&egui::TextStyle::Heading].size, 31.0);
    assert_eq!(
        style.visuals.panel_fill,
        egui::Color32::from_rgb(10, 20, 30)
    );
    assert_eq!(
        style.visuals.window_fill,
        egui::Color32::from_rgb(40, 50, 60)
    );
    assert!(!ctx.style_of(egui::Theme::Light).visuals.dark_mode);
}

#[test]
fn pop_up_buttons_of_the_same_id_in_a_loop_open_one_list() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let rows = |ui: &mut egui::Ui| {
            (0..3)
                .map(|i| {
                    ui.push_id(i, |ui| {
                        look::PopupButton::new("units", "Metres")
                            .width(160.0)
                            .show(ui, |ui| ui.label("Feet"))
                            .inner
                            .is_some()
                    })
                    .inner
                })
                .collect::<Vec<bool>>()
        };
        for events in click(egui::pos2(40.0, 10.0)) {
            frame(&ctx, events, rows);
        }
        assert_eq!(
            frame(&ctx, vec![], rows),
            [true, false, false],
            "{chosen}: one click, one list"
        );
    }
}

#[test]
fn the_arrow_keys_move_a_slider_and_a_segmented_choice_more_than_once() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut value = 0.0_f64;
        let add = |ui: &mut egui::Ui, value: &mut f64| {
            ui.add(
                look::Slider::new(value, 0.0..=10.0)
                    .step_by(1.0)
                    .width(200.0),
            )
        };
        frame(&ctx, vec![], |ui| add(ui, &mut value));
        frame(&ctx, key(egui::Key::Tab), |ui| add(ui, &mut value));
        frame(&ctx, vec![], |ui| add(ui, &mut value));
        for _ in 0..3 {
            frame(&ctx, key(egui::Key::ArrowRight), |ui| add(ui, &mut value));
        }
        assert_eq!(value, 3.0, "{chosen}: the slider lost the focus");
        // Up raises and Down lowers the value in every look, keeping the focus.
        for _ in 0..2 {
            frame(&ctx, key(egui::Key::ArrowUp), |ui| add(ui, &mut value));
        }
        frame(&ctx, key(egui::Key::ArrowDown), |ui| add(ui, &mut value));
        assert_eq!(value, 4.0, "{chosen}: vertical arrows");
        let mut axis = 0;
        let add = |ui: &mut egui::Ui, axis: &mut usize| {
            ui.add(look::Segmented::new("axis", axis, &["X", "Y", "Z"]).width(240.0))
        };
        frame(&ctx, vec![], |ui| add(ui, &mut axis));
        frame(&ctx, key(egui::Key::Tab), |ui| add(ui, &mut axis));
        // The key's release: a frame with the focus settled.
        frame(&ctx, vec![], |ui| add(ui, &mut axis));
        // GTK's toggle group moves the focus with the arrows and chooses with Space.
        let gtk = chosen == look::Look::Gtk;
        for _ in 0..2 {
            frame(&ctx, key(egui::Key::ArrowRight), |ui| add(ui, &mut axis));
        }
        if gtk {
            frame(&ctx, key(egui::Key::Space), |ui| add(ui, &mut axis));
        }
        assert_eq!(
            axis, 2,
            "{chosen}: the segmented choice did not follow the arrows"
        );
        frame(&ctx, key(egui::Key::ArrowLeft), |ui| add(ui, &mut axis));
        if gtk {
            frame(&ctx, key(egui::Key::Space), |ui| add(ui, &mut axis));
        }
        assert_eq!(axis, 1, "{chosen}");
    }
}

#[test]
fn a_switch_and_a_segment_name_themselves_to_a_screen_reader() {
    let ctx = context(look::Look::Gtk, true);
    let mut on = false;
    frame(&ctx, vec![], |ui| {
        ui.add(look::Switch::new(&mut on).accessible_label("Snap"))
    });
    let input = egui::RawInput {
        events: key(egui::Key::Tab),
        ..Default::default()
    };
    let mut output = ctx.run_ui(input, |ui| {
        ui.add(look::Switch::new(&mut on).accessible_label("Snap"));
    });
    output.textures_delta.clear();
    let named = output.platform_output.events.iter().any(|e| matches!(e, egui::output::OutputEvent::FocusGained(info) if info.label.as_deref() == Some("Snap")));
    assert!(named, "{:?}", output.platform_output.events);
}

#[test]
fn a_form_row_names_its_switch_for_a_screen_reader() {
    let ctx = context(look::Look::Mac, true);
    let mut on = false;
    let row = |ui: &mut egui::Ui, on: &mut bool| {
        look::Form::new("named").show(ui, |form| {
            form.row("Snap to grid", |ui| ui.add(look::Switch::new(on)));
        });
        // Outside the form a switch has no name of its own.
        ui.add(look::Switch::new(&mut false));
    };
    frame(&ctx, vec![], |ui| row(ui, &mut on));
    let input = egui::RawInput {
        events: key(egui::Key::Tab),
        ..Default::default()
    };
    let mut output = ctx.run_ui(input, |ui| row(ui, &mut on));
    output.textures_delta.clear();
    let named = output.platform_output.events.iter().any(|e| matches!(e, egui::output::OutputEvent::FocusGained(info) if info.label.as_deref() == Some("Snap to grid")));
    assert!(named, "{:?}", output.platform_output.events);
}

#[test]
fn a_number_field_keeps_to_its_type_and_any_step() {
    let ctx = context(look::Look::Mac, true);
    let mut full: u8 = 255;
    // A click on Increase must not wrap the value.
    let (_, plus) = step_points(&ctx, |ui| {
        ui.add(look::NumberField::new(&mut full).width(140.0));
    });
    for events in click(plus) {
        frame(&ctx, events, |ui| {
            ui.add(look::NumberField::new(&mut full).width(140.0))
        });
    }
    assert_eq!(full, 255);
    let mut value = 0.0_f64;
    for events in click(plus) {
        frame(&ctx, events, |ui| {
            ui.add(look::NumberField::new(&mut value).step(-2.0).width(140.0))
        });
    }
    assert_eq!(value, 2.0, "a negative step counts as its size");
    for events in click(plus) {
        frame(&ctx, events, |ui| {
            ui.add(look::NumberField::new(&mut value).step(0.0).width(140.0))
        });
    }
    assert_eq!(value, 3.0, "a zero step counts as 1");
}

#[test]
fn nonsense_sizes_and_values_are_held_in_bounds() {
    for factor in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        let metrics = look::Metrics::of(look::Look::Gtk).scaled(factor);
        assert!(
            metrics.body.size.is_finite()
                && metrics.body.size > 0.0
                && metrics.control_height >= 1.0,
            "{factor}: {metrics:?}"
        );
    }
    let ctx = context(look::Look::Windows, false);
    frame(&ctx, vec![], |ui| {
        ui.add(look::ProgressBar::new(f32::NAN).width(100.0))
    });
    // A range from high to low runs the other way.
    let mut value = 0.0_f64;
    for events in click(egui::pos2(190.0, 10.0)) {
        frame(&ctx, events, |ui| {
            ui.add(look::Slider::new(&mut value, 100.0..=0.0).width(200.0))
        });
    }
    assert!(value < 10.0, "{value}");
}

#[test]
fn text_on_a_light_accent_is_dark() {
    let mut theme = look::Theme::new(look::Look::Gtk);
    theme.accent = Some([0xf6d32d, 0xf6d32d]);
    assert_eq!(
        look::Colors::resolve(&theme, false).on_accent,
        egui::Color32::BLACK
    );
    theme.accent = Some([0x3584e4, 0x3584e4]);
    assert_eq!(
        look::Colors::resolve(&theme, false).on_accent,
        egui::Color32::WHITE
    );
    let mac = look::Theme {
        accent: None,
        ..look::Theme::new(look::Look::Mac)
    };
    assert_eq!(
        look::Colors::resolve(&mac, true).on_accent,
        egui::Color32::WHITE
    );
}

#[test]
fn escape_closes_the_top_dialog_alone_and_not_while_a_field_has_the_focus() {
    let ctx = context(look::Look::Mac, true);
    let (mut first, mut second) = (true, true);
    let mut name = String::new();
    let run = |events: Vec<egui::Event>, first: &mut bool, second: &mut bool, name: &mut String| {
        frame(&ctx, events, |_ui| {
            look::Dialog::new("first", "First").show(
                &ctx,
                first,
                |ui| {
                    ui.label("…");
                },
                |_| {},
            );
            look::Dialog::new("second", "Second").show(
                &ctx,
                second,
                |ui| {
                    ui.add(look::TextField::new(name));
                },
                |_| {},
            );
        })
    };
    run(vec![], &mut first, &mut second, &mut name);
    run(vec![], &mut first, &mut second, &mut name);
    run(key(egui::Key::Escape), &mut first, &mut second, &mut name);
    assert!(
        first && !second,
        "Escape closes the one on top: {first} {second}"
    );
    second = true;
    run(vec![], &mut first, &mut second, &mut name);
    // Focus the field, then Escape: it takes the focus off the field and closes nothing.
    ctx.memory_mut(|m| m.request_focus(egui::Id::NULL));
    run(key(egui::Key::Tab), &mut first, &mut second, &mut name);
    let focused = ctx.memory(|m| m.focused().is_some());
    run(key(egui::Key::Escape), &mut first, &mut second, &mut name);
    if focused {
        assert!(first && second, "Escape only took the focus");
    }
}

#[test]
fn toasts_show_in_the_viewport_they_were_asked_for_in() {
    let ctx = context(look::Look::Gtk, true);
    let shapes = |mut output: egui::FullOutput| {
        output.textures_delta.clear();
        output.shapes.len()
    };
    let before = shapes(ctx.run_ui(egui::RawInput::default(), |ui| look::show_toasts(ui.ctx())));
    look::toast(&ctx, "Saved");
    let shown = shapes(ctx.run_ui(egui::RawInput::default(), |ui| look::show_toasts(ui.ctx())));
    assert!(shown > before);
    let tool = egui::ViewportId::from_hash_of("tool");
    let other = egui::RawInput {
        viewport_id: tool,
        viewports: [(tool, Default::default())].into_iter().collect(),
        ..Default::default()
    };
    let elsewhere = shapes(ctx.run_ui(other, |ui| look::show_toasts(ui.ctx())));
    assert_eq!(elsewhere, before, "another viewport drew the toast");
}

#[test]
fn a_dropdown_can_be_chosen_and_cancelled_with_the_keyboard() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut selected = 1;
        let draw = |ui: &mut egui::Ui, selected: &mut usize| {
            ui.add(
                look::Dropdown::new("keyboard", selected, &["First", "Second", "Third"])
                    .width(180.0),
            )
        };
        frame(&ctx, vec![], |ui| draw(ui, &mut selected));
        frame(&ctx, key(egui::Key::Tab), |ui| draw(ui, &mut selected));
        frame(&ctx, key(egui::Key::Enter), |ui| draw(ui, &mut selected));
        frame(&ctx, vec![], |ui| draw(ui, &mut selected));
        assert!(
            egui::Popup::is_any_open(&ctx),
            "{chosen}: Enter did not open the list"
        );
        frame(&ctx, key(egui::Key::ArrowDown), |ui| {
            draw(ui, &mut selected)
        });
        assert_eq!(
            selected, 1,
            "{chosen}: navigation must not commit the choice"
        );
        frame(&ctx, vec![], |ui| draw(ui, &mut selected));
        frame(&ctx, key(egui::Key::Enter), |ui| draw(ui, &mut selected));
        assert_eq!(
            selected, 2,
            "{chosen}: Enter did not choose the focused row"
        );
        assert!(!egui::Popup::is_any_open(&ctx));
        // Reopen after selection without requiring another Tab through the page.
        frame(&ctx, vec![], |ui| draw(ui, &mut selected));
        frame(&ctx, key(egui::Key::Enter), |ui| draw(ui, &mut selected));
        frame(&ctx, vec![], |ui| draw(ui, &mut selected));
        assert!(
            egui::Popup::is_any_open(&ctx),
            "{chosen}: focus did not return to the button"
        );
        // The list wraps: Down from the last row (the chosen one) reaches the first.
        frame(&ctx, key(egui::Key::ArrowDown), |ui| {
            draw(ui, &mut selected)
        });
        frame(&ctx, vec![], |ui| draw(ui, &mut selected));
        frame(&ctx, key(egui::Key::Enter), |ui| draw(ui, &mut selected));
        assert_eq!(selected, 0, "{chosen}: Down from the last row");
        frame(&ctx, vec![], |ui| draw(ui, &mut selected));
        frame(&ctx, key(egui::Key::Enter), |ui| draw(ui, &mut selected));
        frame(&ctx, vec![], |ui| draw(ui, &mut selected));
        frame(&ctx, key(egui::Key::ArrowUp), |ui| draw(ui, &mut selected));
        frame(&ctx, vec![], |ui| draw(ui, &mut selected));
        frame(&ctx, key(egui::Key::Enter), |ui| draw(ui, &mut selected));
        assert_eq!(selected, 2, "{chosen}: Up from the first row");
        frame(&ctx, vec![], |ui| draw(ui, &mut selected));
        frame(&ctx, key(egui::Key::Enter), |ui| draw(ui, &mut selected));
        frame(&ctx, vec![], |ui| draw(ui, &mut selected));
        frame(&ctx, key(egui::Key::Home), |ui| draw(ui, &mut selected));
        frame(&ctx, key(egui::Key::Escape), |ui| draw(ui, &mut selected));
        assert_eq!(selected, 2, "{chosen}: Escape committed the pending choice");
        assert!(!egui::Popup::is_any_open(&ctx));
    }
}

#[test]
fn text_fields_have_form_names_and_explicit_names_override_them() {
    let ctx = context(look::Look::Mac, true);
    ctx.enable_accesskit();
    let mut text = String::new();
    let mut ids = Vec::new();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        look::Form::new("names").show(ui, |form| {
            ids.push(
                form.row("Name", |ui| ui.add(look::TextField::new(&mut text)))
                    .id,
            );
            ids.push(
                form.row("Ignored", |ui| {
                    ui.add(look::TextField::new(&mut text).accessible_label("Search projects"))
                })
                .id,
            );
        });
        ids.push(
            ui.add(look::TextField::new(&mut text).hint("Placeholder"))
                .id,
        );
    });
    output.textures_delta.clear();
    let update = output.platform_output.accesskit_update.unwrap();
    for (id, expected) in ids
        .into_iter()
        .zip([Some("Name"), Some("Search projects"), None])
    {
        let node = &update
            .nodes
            .iter()
            .find(|(node_id, _)| *node_id == id.accesskit_id())
            .unwrap()
            .1;
        assert_eq!(node.label(), expected);
    }
}

#[test]
fn buttons_and_switches_cancel_a_press_released_outside() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut on = false;
        let mut clicked = false;
        let mut draw = |ui: &mut egui::Ui| {
            let response = ui.add(look::Button::new("Action"));
            let button = response.rect;
            clicked |= response.clicked();
            clicked |= ui.add(look::Switch::new(&mut on)).changed();
            button
        };
        let button = frame(&ctx, vec![], &mut draw);
        for start in [button.center(), egui::pos2(10.0, button.bottom() + 12.0)] {
            let press = |pos, pressed| egui::Event::PointerButton {
                pos,
                pressed,
                button: egui::PointerButton::Primary,
                modifiers: Default::default(),
            };
            frame(
                &ctx,
                vec![egui::Event::PointerMoved(start), press(start, true)],
                &mut draw,
            );
            let outside = egui::pos2(300.0, 500.0);
            frame(&ctx, vec![egui::Event::PointerMoved(outside)], &mut draw);
            frame(&ctx, vec![press(outside, false)], &mut draw);
        }
        assert!(
            !on && !clicked,
            "{chosen}: moving away must cancel activation"
        );
    }
}

#[test]
fn a_default_dialog_fits_a_phone_and_a_short_viewport() {
    for chosen in LOOKS {
        for size in [egui::vec2(320.0, 600.0), egui::vec2(320.0, 240.0)] {
            let ctx = context(chosen, true);
            let mut open = true;
            let mut actual = None;
            for _ in 0..4 {
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                        ..Default::default()
                    },
                    |_ui| {
                        look::Dialog::new("phone", "Settings").show(
                            &ctx,
                            &mut open,
                            |ui| {
                                for _ in 0..20 {
                                    ui.label("Settings content that can scroll.");
                                }
                            },
                            |ui| {
                                ui.add(look::Button::new("Done"));
                            },
                        );
                        actual = ctx.memory(|memory| memory.area_rect(egui::Id::unique("phone")));
                    },
                );
                output.textures_delta.clear();
            }
            let rect = actual.unwrap();
            assert!(
                rect.width() <= size.x && rect.height() <= size.y,
                "{chosen}: {rect:?} exceeds {size:?}"
            );
        }
    }
}

#[test]
fn dialog_actions_sit_side_by_side_and_stack_only_when_they_do_not_fit() {
    for chosen in LOOKS {
        for (width, labels, stacked) in [
            (360.0, ["Save", "Cancel"], false),
            (
                320.0,
                ["Save changes to the document", "Discard unsaved changes"],
                true,
            ),
        ] {
            let ctx = context(chosen, true);
            let mut open = true;
            let mut buttons = Vec::new();
            // The arrangement holds from the first frame: a row that does not fit is redrawn
            // stacked within it. The centred dialog settles its position a frame later.
            for frame in 0..3 {
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width + 40.0, 600.0),
                        )),
                        ..Default::default()
                    },
                    |_ui| {
                        buttons.clear();
                        look::Dialog::new("actions", "Save changes?")
                            .size([width, 80.0])
                            .show(
                                &ctx,
                                &mut open,
                                |ui| {
                                    ui.label("Save your changes?");
                                },
                                |ui| {
                                    for label in labels {
                                        buttons.push(ui.add(look::Button::new(label)).rect);
                                    }
                                },
                            );
                    },
                );
                output.textures_delta.clear();
                let [first, second] = [buttons[0], buttons[1]];
                if stacked {
                    assert!(
                        second.top() >= first.bottom()
                            && (first.right() - second.right()).abs() < 0.5,
                        "{chosen}, frame {frame}: long actions must stack right-aligned: \
                         {first:?} {second:?}"
                    );
                } else {
                    assert!(
                        (first.center().y - second.center().y).abs() < 0.5
                            && second.right() <= first.left(),
                        "{chosen}, frame {frame}: short actions must share a row, first on the \
                         right: {first:?} {second:?}"
                    );
                }
            }
            let dialog = ctx
                .memory(|memory| memory.area_rect(egui::Id::unique("actions")))
                .unwrap();
            for button in &buttons {
                assert!(
                    dialog.contains_rect(*button),
                    "{chosen}: {button:?} outside {dialog:?}"
                );
            }
            let lowest = buttons
                .iter()
                .map(|button| button.bottom())
                .fold(0.0, f32::max);
            // At most the 24-point margin and the frame's one-point edge.
            assert!(
                dialog.bottom() - lowest <= 25.0,
                "{chosen}: the dialog must end below its actions: {dialog:?} {buttons:?}"
            );
        }
    }
}

#[test]
fn a_dialog_grows_with_content_added_after_it_opened() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut open = true;
        let mut last = egui::Rect::NOTHING;
        for frame in 0..6 {
            let rows = if frame < 2 { 1 } else { 9 };
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(480.0, 600.0),
                    )),
                    ..Default::default()
                },
                |_ui| {
                    look::Dialog::new("growing", "Details")
                        .size([320.0, 80.0])
                        .show(
                            &ctx,
                            &mut open,
                            |ui| {
                                for row in 0..rows {
                                    last = ui.label(format!("Detail {row}")).rect;
                                }
                            },
                            |ui| {
                                ui.add(look::Button::new("Done"));
                            },
                        );
                },
            );
            output.textures_delta.clear();
        }
        let dialog = ctx
            .memory(|memory| memory.area_rect(egui::Id::unique("growing")))
            .unwrap();
        assert!(
            dialog.contains_rect(last),
            "{chosen}: the last row {last:?} must fit the grown dialog {dialog:?}"
        );
    }
}

#[test]
fn system_dialogs_close_on_their_actions_without_a_close_button() {
    // macOS's sheet, WinUI's content dialog, libadwaita's and iOS's alerts and Material's
    // dialog close on their actions; the plain look keeps a close button.
    for (chosen, has_close) in [
        (look::Look::Mac, false),
        (look::Look::Windows, false),
        (look::Look::Gtk, false),
        (look::Look::Material, false),
        (look::Look::Ios, false),
        (look::Look::Plain, true),
    ] {
        let ctx = context(chosen, false);
        ctx.enable_accesskit();
        let mut open = true;
        let mut update = None;
        for _ in 0..2 {
            let mut output = ctx.run_ui(egui::RawInput::default(), |_ui| {
                look::Dialog::new("sheet", "Save changes?").show(
                    &ctx,
                    &mut open,
                    |ui| {
                        ui.label("Save your changes?");
                    },
                    |ui| {
                        ui.add(look::Button::new("Save"));
                    },
                );
            });
            output.textures_delta.clear();
            update = output.platform_output.accesskit_update;
        }
        let close =
            update.unwrap().nodes.iter().any(|(_, node)| {
                node.role() == egui::Role::Button && node.label() == Some("Close")
            });
        assert_eq!(close, has_close, "{chosen}");
    }
}

#[test]
fn a_search_field_clears_from_its_button_and_keeps_the_focus() {
    for chosen in LOOKS {
        let ctx = context(chosen, false);
        ctx.enable_accesskit();
        let mut text = String::from("query");
        let draw = |ui: &mut egui::Ui, text: &mut String| {
            let response = ui.add(look::TextField::new(text).search().width(240.0));
            (response.id, response.changed())
        };
        let mut changed = false;
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(340.0, 600.0),
            )),
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| {
            draw(ui, &mut text);
        });
        output.textures_delta.clear();
        let nodes = output.platform_output.accesskit_update.unwrap().nodes;
        let bounds = nodes
            .iter()
            .find(|(_, node)| node.role() == egui::Role::Button && node.label() == Some("Clear"))
            .and_then(|(_, node)| node.bounds())
            .unwrap_or_else(|| panic!("{chosen}: no Clear button"));
        let at = egui::pos2(
            ((bounds.x0 + bounds.x1) * 0.5) as f32,
            ((bounds.y0 + bounds.y1) * 0.5) as f32,
        );
        for events in click(at) {
            changed |= frame(&ctx, events, |ui| draw(ui, &mut text)).1;
        }
        let (id, _) = frame(&ctx, vec![], |ui| draw(ui, &mut text));
        assert!(text.is_empty(), "{chosen}: {text:?}");
        assert!(changed, "{chosen}: clearing is a change");
        assert_eq!(ctx.memory(|memory| memory.focused()), Some(id), "{chosen}");
        // An empty search field offers nothing to clear.
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            draw(ui, &mut text);
        });
        output.textures_delta.clear();
        let clear = output
            .platform_output
            .accesskit_update
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.label() == Some("Clear"));
        assert!(!clear, "{chosen}");
    }
}

#[test]
fn an_opened_dialog_focuses_its_first_field_except_on_a_touch_screen() {
    for touch in [false, true] {
        let ctx = context(look::Look::Gtk, true);
        if touch {
            let at = egui::pos2(300.0, 500.0);
            let tap = |phase| egui::Event::Touch {
                device_id: egui::TouchDeviceId(1),
                id: egui::TouchId(1),
                phase,
                pos: at,
                force: None,
            };
            frame(
                &ctx,
                vec![tap(egui::TouchPhase::Start), tap(egui::TouchPhase::End)],
                |_| {},
            );
        }
        let mut open = true;
        let mut name = String::new();
        let mut field = egui::Id::NULL;
        for _ in 0..3 {
            frame(&ctx, vec![], |_ui| {
                look::Dialog::new("rename", "Rename").show(
                    &ctx,
                    &mut open,
                    |ui| {
                        ui.label("New name");
                        field = ui.add(look::TextField::new(&mut name)).id;
                    },
                    |ui| {
                        ui.add(look::Button::new("Rename").default_action());
                    },
                );
            });
        }
        let focused = ctx.memory(|memory| memory.focused());
        if touch {
            assert_ne!(focused, Some(field), "no keyboard raised unasked");
        } else {
            assert_eq!(focused, Some(field));
        }
    }
}

/// The menu items' enabled states, by label, from the accessibility tree.
fn menu_items(output: &egui::FullOutput) -> Vec<(String, bool, egui::Pos2)> {
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    nodes
        .iter()
        .filter(|(_, node)| node.role() == egui::Role::MenuItem)
        .map(|(_, node)| {
            let bounds = node.bounds().unwrap();
            (
                node.label().unwrap_or_default().to_owned(),
                !node.is_disabled(),
                egui::pos2(
                    ((bounds.x0 + bounds.x1) * 0.5) as f32,
                    ((bounds.y0 + bounds.y1) * 0.5) as f32,
                ),
            )
        })
        .collect()
}

#[test]
fn a_text_field_offers_cut_copy_paste_and_select_all() {
    for password in [false, true] {
        let ctx = context(look::Look::Gtk, true);
        ctx.enable_accesskit();
        let mut text = String::from("secret words");
        let draw = |ui: &mut egui::Ui, text: &mut String| {
            ui.add(look::TextField::new(text).password(password).width(240.0));
        };
        let run = |events: Vec<egui::Event>, text: &mut String| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(340.0, 600.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| draw(ui, text),
            );
            output.textures_delta.clear();
            output
        };
        let at = egui::pos2(60.0, 12.0);
        let secondary = |pressed| egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Secondary,
            pressed,
            modifiers: Default::default(),
        };
        let primary = |at, pressed| egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        run(vec![egui::Event::PointerMoved(at)], &mut text);
        run(vec![secondary(true)], &mut text);
        run(vec![secondary(false)], &mut text);
        let output = run(vec![], &mut text);
        let items = menu_items(&output);
        let enabled = |label: &str| items.iter().find(|item| item.0 == label).map(|item| item.1);
        assert_eq!(enabled("Cut"), Some(false), "nothing selected yet");
        assert_eq!(enabled("Copy"), Some(false));
        assert_eq!(enabled("Paste"), Some(true));
        let all = items.iter().find(|item| item.0 == "Select All").unwrap().2;
        run(vec![egui::Event::PointerMoved(all)], &mut text);
        run(vec![primary(all, true)], &mut text);
        run(vec![primary(all, false)], &mut text);
        run(vec![], &mut text);
        // Open it again over the selection; Copy needs a selection and no password.
        run(vec![egui::Event::PointerMoved(at)], &mut text);
        run(vec![secondary(true)], &mut text);
        run(vec![secondary(false)], &mut text);
        let output = run(vec![], &mut text);
        let items = menu_items(&output);
        let copy = items.iter().find(|item| item.0 == "Copy").unwrap().clone();
        assert_eq!(
            copy.1, !password,
            "Copy with a selection, password={password}"
        );
        if !password {
            run(vec![egui::Event::PointerMoved(copy.2)], &mut text);
            run(vec![primary(copy.2, true)], &mut text);
            let output = run(vec![primary(copy.2, false)], &mut text);
            let asked = output.viewport_output.values().any(|viewport| {
                viewport
                    .commands
                    .iter()
                    .any(|command| matches!(command, egui::ViewportCommand::RequestCopy))
            });
            assert!(asked, "Copy asks the integration to copy");
            // The system copies from the focused field's selection, so both are back.
            run(vec![], &mut text);
            let field = ctx.memory(|memory| memory.focused()).unwrap();
            let range = egui::TextEdit::load_state(&ctx, field)
                .and_then(|state| state.cursor.char_range())
                .unwrap();
            let sorted = range.as_sorted_char_range();
            assert_eq!(
                (sorted.start, sorted.end),
                (0.into(), text.chars().count().into())
            );
        }
    }
}

#[test]
fn text_fields_keep_their_edge_where_buttons_have_none() {
    for chosen in [look::Look::Mac, look::Look::Windows, look::Look::Plain] {
        let ctx = context(chosen, false);
        let mut text = String::new();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.add(look::TextField::new(&mut text).width(200.0));
        });
        output.textures_delta.clear();
        let edge = look::colors(&ctx).control_border;
        let edged = output.shapes.iter().any(|shape| {
            matches!(&shape.shape, egui::Shape::Rect(rect) if rect.stroke.width >= 1.0
                && (rect.stroke.color == edge || chosen == look::Look::Plain))
        });
        assert!(edged, "{chosen}: the text field lost its edge");
    }
}

#[test]
fn disabled_mac_controls_are_drawn_in_their_own_colours_not_faded() {
    for (chosen, opaque) in [(look::Look::Mac, true), (look::Look::Gtk, false)] {
        let ctx = context(chosen, false);
        let mut on = true;
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.add_enabled(false, look::Switch::new(&mut on).accessible_label("Power"));
        });
        output.textures_delta.clear();
        // The knob: the last light disc or pill drawn.
        let knob = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Circle(circle) => Some(circle.fill),
                egui::Shape::Rect(rect) if rect.rect.width() < 40.0 => Some(rect.fill),
                _ => None,
            })
            .rfind(|fill| fill.r() > 120 && fill.g() > 120 && fill.b() > 120)
            .expect("a knob");
        assert_eq!(knob.a() == 255, opaque, "{chosen}: {knob:?}");
    }
}

#[test]
fn a_mac_list_greys_its_selection_without_the_focus() {
    let ctx = context(look::Look::Mac, false);
    let run = |events: Vec<egui::Event>| {
        let mut rows = (egui::Rect::NOTHING, egui::Rect::NOTHING);
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(340.0, 600.0),
                )),
                events,
                ..Default::default()
            },
            |ui| {
                let first = ui.add(look::ListRow::new(false).accessible_label("First"));
                let second = ui.add(look::ListRow::new(true).accessible_label("Second"));
                rows = (first.rect, second.rect);
            },
        );
        output.textures_delta.clear();
        let fill = output.shapes.iter().find_map(|shape| match &shape.shape {
            egui::Shape::Rect(shape) if shape.rect == rows.1 => Some(shape.fill),
            _ => None,
        });
        (rows, fill)
    };
    let (rows, quiet) = run(vec![]);
    // Read once a frame has told egui the window is active.
    let accent = look::colors(&ctx).accent;
    assert_ne!(
        quiet,
        Some(accent),
        "without the focus the selection is grey"
    );
    // A click on a row gives the list the focus, and the selection the accent.
    for events in click(rows.0.center()) {
        run(events);
    }
    let (_, lit) = run(vec![]);
    assert_eq!(lit, Some(accent));
}

#[test]
fn an_inactive_mac_window_greys_its_accent() {
    let ctx = context(look::Look::Mac, false);
    let run = |focused: bool, on: &mut bool| {
        let mut output = ctx.run_ui(
            egui::RawInput {
                focused,
                ..Default::default()
            },
            |ui| {
                ui.add(look::Switch::new(on).accessible_label("Power"));
            },
        );
        output.textures_delta.clear();
        (look::colors(&ctx), output)
    };
    let mut on = true;
    let (active, _) = run(true, &mut on);
    let (inactive, output) = run(false, &mut on);
    assert!(active.active && !inactive.active);
    assert_ne!(active.accent, inactive.accent);
    assert_eq!(inactive.on_accent, inactive.fg);
    let grey = inactive.accent;
    let track = output
        .shapes
        .iter()
        .any(|shape| matches!(&shape.shape, egui::Shape::Rect(rect) if rect.fill == grey));
    assert!(track, "the switched-on track turns grey");
    // Other looks keep their accent.
    let ctx = context(look::Look::Gtk, false);
    let mut output = ctx.run_ui(
        egui::RawInput {
            focused: false,
            ..Default::default()
        },
        |_| {},
    );
    output.textures_delta.clear();
    assert_ne!(look::colors(&ctx).accent, look::colors(&ctx).fg);
}

#[test]
fn a_stepped_mac_slider_marks_its_steps() {
    for (step, dots) in [(Some(10.0), 11), (None, 0), (Some(0.5), 0)] {
        let ctx = context(look::Look::Mac, false);
        let mut value = 40.0;
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let slider = look::Slider::new(&mut value, 0.0..=100.0).width(200.0);
            let slider = match step {
                Some(step) => slider.step_by(step),
                None => slider,
            };
            ui.add(slider.accessible_label("Volume"));
        });
        output.textures_delta.clear();
        let marks = output
            .shapes
            .iter()
            .filter(|shape| {
                matches!(&shape.shape, egui::Shape::Rect(rect)
                    if rect.rect.size() == egui::Vec2::splat(2.0))
            })
            .count();
        assert_eq!(marks, dots, "step {step:?}");
    }
}

#[test]
fn mac_control_sizes_follow_swiftui() {
    let small = look::Metrics::small(look::Look::Mac);
    let large = look::Metrics::large(look::Look::Mac);
    assert_eq!((small.control_height, small.body.size), (20.0, 11.0));
    assert_eq!((large.control_height, large.body.size), (28.0, 13.0));
    // Width of a switch and height of a text field, as SwiftUI draws them.
    for (metrics, switch, field) in [
        (small, 45.0, 23.0),
        (look::Metrics::of(look::Look::Mac), 54.0, 26.0),
        (large, 63.0, 26.0),
    ] {
        let ctx = egui::Context::default();
        look::install(&ctx, look::Theme::new(look::Look::Mac).metrics(metrics));
        let mut on = true;
        let mut text = String::new();
        let mut sizes = (0.0, 0.0);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let switch = ui.add(look::Switch::new(&mut on).accessible_label("Power"));
            let field = ui.add(look::TextField::new(&mut text).width(200.0));
            sizes = (switch.rect.width(), field.rect.height());
        });
        output.textures_delta.clear();
        assert_eq!(sizes, (switch, field));
    }
    // The other looks scale in proportion.
    let windows = look::Metrics::small(look::Look::Windows);
    assert!(windows.control_height < look::Metrics::of(look::Look::Windows).control_height);
}

#[test]
fn tabbing_into_a_text_field_selects_its_text() {
    for chosen in LOOKS {
        let ctx = context(chosen, false);
        let mut first = String::from("first");
        let mut second = String::from("second field");
        let mut id = egui::Id::NULL;
        let mut draw = |ui: &mut egui::Ui| {
            ui.add(look::TextField::new(&mut first).width(120.0));
            id = ui.add(look::TextField::new(&mut second).width(120.0)).id;
        };
        frame(&ctx, vec![], &mut draw);
        frame(&ctx, key(egui::Key::Tab), &mut draw);
        frame(&ctx, vec![], &mut draw);
        frame(&ctx, key(egui::Key::Tab), &mut draw);
        frame(&ctx, vec![], &mut draw);
        assert_eq!(ctx.memory(|memory| memory.focused()), Some(id), "{chosen}");
        let range = egui::TextEdit::load_state(&ctx, id)
            .and_then(|state| state.cursor.char_range())
            .expect("a cursor")
            .as_sorted_char_range();
        assert_eq!(
            (range.start, range.end),
            (0.into(), "second field".len().into()),
            "{chosen}: the whole text is selected"
        );
    }
}

#[test]
fn switch_width_is_the_width_a_switch_takes() {
    for chosen in LOOKS {
        let ctx = context(chosen, false);
        let mut on = false;
        let mut width = 0.0;
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            width = ui
                .add(look::Switch::new(&mut on).accessible_label("Power"))
                .rect
                .width();
        });
        output.textures_delta.clear();
        assert_eq!(width, look::switch_width(chosen), "{chosen}");
    }
}

#[test]
fn mac_number_fields_stand_as_tall_as_text_fields() {
    let ctx = context(look::Look::Mac, false);
    let (mut number, mut text) = (8, String::new());
    let mut heights = (0.0, 0.0, 0.0);
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        let plain = ui.add(
            look::NumberField::new(&mut number)
                .steppers(false)
                .width(64.0),
        );
        let stepped = ui.add(look::NumberField::new(&mut number).width(88.0));
        let field = ui.add(look::TextField::new(&mut text).width(64.0));
        heights = (
            plain.rect.height(),
            stepped.rect.height(),
            field.rect.height(),
        );
    });
    output.textures_delta.clear();
    assert_eq!(heights, (26.0, 26.0, 26.0));
}

#[test]
fn a_pressed_mac_default_button_darkens_on_light_and_lightens_on_dark() {
    for dark in [false, true] {
        let ctx = context(look::Look::Mac, dark);
        let run = |events: Vec<egui::Event>| {
            let mut rect = egui::Rect::NOTHING;
            let mut output = ctx.run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| rect = ui.add(look::Button::new("Default").accent()).rect,
            );
            output.textures_delta.clear();
            (rect, output)
        };
        let (rect, _) = run(vec![]);
        let at = rect.center();
        run(vec![egui::Event::PointerMoved(at)]);
        let (_, output) = run(vec![egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        }]);
        let accent = look::colors(&ctx).accent;
        let fill = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Rect(shape) if shape.rect == rect => Some(shape.fill),
                _ => None,
            })
            .expect("the button's body");
        let brightness = |c: egui::Color32| u32::from(c.r()) + u32::from(c.g()) + u32::from(c.b());
        if dark {
            assert!(
                brightness(fill) > brightness(accent),
                "{fill:?} lighter than {accent:?}"
            );
        } else {
            assert!(
                brightness(fill) < brightness(accent),
                "{fill:?} darker than {accent:?}"
            );
        }
    }
}

#[test]
fn a_dialog_blocks_clicks_and_keyboard_focus_behind_it() {
    let ctx = context(look::Look::Mac, true);
    let mut open = true;
    let mut background_clicked = false;
    let mut background_id = egui::Id::NULL;
    let mut action_id = egui::Id::NULL;
    let mut draw = |ui: &mut egui::Ui| {
        let response = ui.add(look::Button::new("Background"));
        background_id = response.id;
        background_clicked |= response.clicked();
        look::Dialog::new("modal", "Settings").show(
            &ctx,
            &mut open,
            |ui| {
                action_id = ui.add(look::Button::new("Inside")).id;
            },
            |_| {},
        );
    };
    for _ in 0..3 {
        frame(&ctx, vec![], &mut draw);
    }
    for events in click(egui::pos2(10.0, 10.0)) {
        frame(&ctx, events, &mut draw);
    }
    let mut focused = Vec::new();
    for _ in 0..5 {
        frame(&ctx, key(egui::Key::Tab), &mut draw);
        focused.push(ctx.memory(|m| m.focused()));
        frame(&ctx, vec![], &mut draw);
    }
    assert!(!background_clicked, "the modal allowed a background click");
    assert!(
        !focused.contains(&Some(background_id)),
        "Tab escaped to the background"
    );
    assert!(
        focused.contains(&Some(action_id)),
        "Tab never reached the dialog action"
    );
}

#[test]
fn an_open_dropdown_exposes_named_items_and_expanded_state() {
    let ctx = context(look::Look::Material, true);
    ctx.enable_accesskit();
    let mut selected = 1;
    let mut draw = |ui: &mut egui::Ui| {
        ui.add(
            look::Dropdown::new("semantics", &mut selected, &["First", "Second", "Third"])
                .width(180.0),
        )
    };
    let button = frame(&ctx, vec![], &mut draw);
    for events in click(button.rect.center()) {
        frame(&ctx, events, &mut draw);
    }
    // Request a complete tree so entries created during the opening frame are included.
    ctx.enable_accesskit();
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(340.0, 600.0),
            )),
            ..Default::default()
        },
        |ui| {
            draw(ui);
        },
    );
    output.textures_delta.clear();
    let update = output.platform_output.accesskit_update.unwrap();
    let node = &update
        .nodes
        .iter()
        .find(|(id, _)| *id == button.id.accesskit_id())
        .unwrap()
        .1;
    assert_eq!(node.is_expanded(), Some(true));
    for label in ["First", "Second", "Third"] {
        assert!(update.nodes.iter().any(|(_, node)| node.role() == egui::Role::MenuItem && node.label() == Some(label)), "missing menu item {label}");
    }
}

#[test]
fn a_long_dropdown_stays_in_the_viewport_and_reaches_the_last_item() {
    let ctx = context(look::Look::Material, true);
    let names: Vec<String> = (0..50).map(|i| format!("Item {i}")).collect();
    let labels: Vec<&str> = names.iter().map(String::as_str).collect();
    let mut selected = 0;
    let mut draw = |ui: &mut egui::Ui| {
        ui.add(look::Dropdown::new("long", &mut selected, &labels).width(180.0))
    };
    frame(&ctx, vec![], &mut draw);
    frame(&ctx, key(egui::Key::Tab), &mut draw);
    frame(&ctx, key(egui::Key::Enter), &mut draw);
    for _ in 0..3 {
        frame(&ctx, vec![], &mut draw);
    }
    let rect = popup_rect(&ctx).unwrap();
    assert!(rect.top() >= 0.0 && rect.bottom() <= 600.0, "{rect:?}");
    frame(&ctx, key(egui::Key::End), &mut draw);
    for _ in 0..3 {
        frame(&ctx, vec![], &mut draw);
    }
    frame(&ctx, key(egui::Key::Enter), &mut draw);
    assert_eq!(selected, 49);
    assert!(!egui::Popup::is_any_open(&ctx));
}

#[test]
fn integer_sliders_move_by_default_and_reversed_sliders_follow_the_rail() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut value = 3_i32;
        let draw = |ui: &mut egui::Ui, value: &mut i32| ui.add(look::Slider::new(value, 0..=10));
        frame(&ctx, vec![], |ui| draw(ui, &mut value));
        frame(&ctx, key(egui::Key::Tab), |ui| draw(ui, &mut value));
        frame(&ctx, vec![], |ui| draw(ui, &mut value));
        frame(&ctx, key(egui::Key::ArrowRight), |ui| draw(ui, &mut value));
        assert_eq!(value, 4, "{chosen}: the default integer step did not move");
        let ctx = context(chosen, true);
        let mut value = 5.0;
        let draw = |ui: &mut egui::Ui, value: &mut f64| {
            ui.add(look::Slider::new(value, 10.0..=0.0).step_by(1.0))
        };
        frame(&ctx, vec![], |ui| draw(ui, &mut value));
        frame(&ctx, key(egui::Key::Tab), |ui| draw(ui, &mut value));
        frame(&ctx, vec![], |ui| draw(ui, &mut value));
        frame(&ctx, key(egui::Key::ArrowRight), |ui| draw(ui, &mut value));
        assert_eq!(
            value, 4.0,
            "{chosen}: Right moved away from the right endpoint"
        );
        frame(&ctx, key(egui::Key::ArrowLeft), |ui| draw(ui, &mut value));
        assert_eq!(value, 5.0);
    }
}

#[test]
fn segmented_choices_accept_extreme_indices_and_leave_empty_choices_unchanged() {
    for chosen in LOOKS {
        for labels in [&["First", "Second"][..], &[][..]] {
            let ctx = context(chosen, true);
            let mut selected = usize::MAX;
            let draw = |ui: &mut egui::Ui, selected: &mut usize| {
                ui.add(look::Segmented::new("bounds", selected, labels).width(180.0))
            };
            frame(&ctx, vec![], |ui| draw(ui, &mut selected));
            frame(&ctx, key(egui::Key::Tab), |ui| draw(ui, &mut selected));
            frame(&ctx, vec![], |ui| draw(ui, &mut selected));
            frame(&ctx, key(egui::Key::ArrowRight), |ui| {
                draw(ui, &mut selected)
            });
            if chosen == look::Look::Gtk {
                frame(&ctx, key(egui::Key::Space), |ui| draw(ui, &mut selected));
            }
            assert_eq!(
                selected,
                if labels.is_empty() { usize::MAX } else { 1 },
                "{chosen}"
            );
        }
    }
}

#[test]
fn choices_support_home_and_end_and_keep_keyboard_focus_after_clicking() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut selected = 0;
        let draw = |ui: &mut egui::Ui, selected: &mut usize| {
            ui.add(look::Segmented::new("keyboard", selected, &["A", "B", "C"]).width(240.0))
        };
        frame(&ctx, vec![], |ui| draw(ui, &mut selected));
        for events in click(egui::pos2(120.0, 10.0)) {
            frame(&ctx, events, |ui| draw(ui, &mut selected));
        }
        assert_eq!(selected, 1);
        let gtk = chosen == look::Look::Gtk;
        let press = |name, selected: &mut usize| {
            frame(&ctx, key(name), |ui| draw(ui, selected));
            if gtk {
                frame(&ctx, key(egui::Key::Space), |ui| draw(ui, selected));
            }
        };
        press(egui::Key::ArrowRight, &mut selected);
        assert_eq!(selected, 2, "{chosen}: the click did not focus the choices");
        press(egui::Key::Home, &mut selected);
        assert_eq!(selected, 0, "{chosen}: Home did not reach the first choice");
        press(egui::Key::End, &mut selected);
        assert_eq!(selected, 2, "{chosen}: End did not reach the last choice");
    }
}

#[test]
fn gtk_toggle_groups_move_the_focus_and_choose_with_space_or_enter() {
    let ctx = context(look::Look::Gtk, true);
    let mut selected = 0;
    let draw = |ui: &mut egui::Ui, selected: &mut usize| {
        ui.add(look::Segmented::new("toggles", selected, &["A", "B", "C"]).width(240.0))
    };
    frame(&ctx, vec![], |ui| draw(ui, &mut selected));
    frame(&ctx, key(egui::Key::Tab), |ui| draw(ui, &mut selected));
    frame(&ctx, vec![], |ui| draw(ui, &mut selected));
    for name in [
        egui::Key::ArrowRight,
        egui::Key::End,
        egui::Key::Home,
        egui::Key::ArrowRight,
    ] {
        frame(&ctx, key(name), |ui| draw(ui, &mut selected));
        assert_eq!(selected, 0, "{name:?} moves only the focus");
    }
    let response = frame(&ctx, key(egui::Key::Enter), |ui| draw(ui, &mut selected));
    assert_eq!(selected, 1, "Enter chooses the focused toggle");
    assert!(response.has_focus());
}

#[test]
fn tabs_support_home_and_end_in_every_look() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut selected = 1;
        let draw = |ui: &mut egui::Ui, selected: &mut usize| {
            ui.add(look::Tabs::new(
                "ends",
                selected,
                &["First", "Middle", "Last"],
            ))
        };
        frame(&ctx, vec![], |ui| draw(ui, &mut selected));
        frame(&ctx, key(egui::Key::Tab), |ui| draw(ui, &mut selected));
        frame(&ctx, vec![], |ui| draw(ui, &mut selected));
        // GTK's view switcher moves the focus with the keys and chooses with Space.
        let gtk = chosen == look::Look::Gtk;
        let press = |name, selected: &mut usize| {
            frame(&ctx, key(name), |ui| draw(ui, selected));
            if gtk {
                frame(&ctx, key(egui::Key::Space), |ui| draw(ui, selected));
            }
        };
        press(egui::Key::Home, &mut selected);
        assert_eq!(
            selected, 0,
            "{chosen}: Home must select even when the first tab already has focus"
        );
        press(egui::Key::End, &mut selected);
        assert_eq!(selected, 2, "{chosen}: End did not reach the last tab");
        press(egui::Key::Home, &mut selected);
        assert_eq!(selected, 0, "{chosen}: Home did not reach the first tab");
    }
}

#[test]
fn a_short_logarithmic_slider_does_not_lose_rounded_keyboard_steps() {
    let ctx = context(look::Look::Plain, true);
    let mut value = 3_i32;
    let draw = |ui: &mut egui::Ui, value: &mut i32| {
        ui.add(look::Slider::new(value, 1..=10).logarithmic(true))
    };
    frame(&ctx, vec![], |ui| draw(ui, &mut value));
    frame(&ctx, key(egui::Key::Tab), |ui| draw(ui, &mut value));
    frame(&ctx, vec![], |ui| draw(ui, &mut value));
    frame(&ctx, key(egui::Key::ArrowRight), |ui| draw(ui, &mut value));
    assert_eq!(value, 4);
    frame(&ctx, key(egui::Key::ArrowLeft), |ui| draw(ui, &mut value));
    assert_eq!(value, 3);
}

#[test]
fn invalid_slider_steps_do_not_produce_non_finite_values() {
    for step in [f64::INFINITY, f64::NAN, 0.0, -1.0] {
        let ctx = context(look::Look::Plain, true);
        let mut value = 5.0;
        let draw = |ui: &mut egui::Ui, value: &mut f64| {
            ui.add(
                look::Slider::new(value, 0.0..=10.0)
                    .step_by(step)
                    .width(200.0),
            )
        };
        for events in click(egui::pos2(150.0, 10.0)) {
            frame(&ctx, events, |ui| draw(ui, &mut value));
        }
        assert!(
            value.is_finite() && value > 5.0 && value <= 10.0,
            "step {step}: {value}"
        );
    }
}

#[test]
fn number_field_integer_steps_and_reversed_bounds_stay_usable() {
    for chosen in LOOKS {
        for (step, amount) in [(0.25, 1), (1.5, 2), (2.7, 3)] {
            for reversed in [false, true] {
                let ctx = context(chosen, true);
                let mut value = 5_i32;
                let draw = |ui: &mut egui::Ui, value: &mut i32| {
                    ui.add(
                        look::NumberField::new(value)
                            .range(if reversed {
                                std::ops::RangeInclusive::new(10, 0)
                            } else {
                                0..=10
                            })
                            .step(step)
                            .width(160.0),
                    )
                };
                let (minus, plus) = step_points(&ctx, |ui| {
                    draw(ui, &mut value);
                });
                for events in click(plus) {
                    frame(&ctx, events, |ui| draw(ui, &mut value));
                }
                assert_eq!(
                    value,
                    5 + amount,
                    "{chosen}: the increase step was lost, reversed={reversed}"
                );
                for events in click(minus) {
                    frame(&ctx, events, |ui| draw(ui, &mut value));
                }
                assert_eq!(value, 5, "{chosen}: decrease and increase were asymmetric");
            }
        }
    }
}

#[test]
fn stepping_an_existing_out_of_range_value_returns_it_to_the_range() {
    let ctx = context(look::Look::Mac, true);
    for initial in [-20.0, 20.0] {
        let mut value = initial;
        let draw = |ui: &mut egui::Ui, value: &mut f64| {
            ui.add(
                look::NumberField::new(value)
                    .range(0.0..=10.0)
                    .clamp_existing_to_range(false)
                    .width(160.0),
            )
        };
        let (minus, plus) = step_points(&ctx, |ui| {
            draw(ui, &mut value);
        });
        assert_eq!(value, initial);
        let at = if initial < 0.0 { plus } else { minus };
        for events in click(at) {
            frame(&ctx, events, |ui| draw(ui, &mut value));
        }
        assert_eq!(value, if initial < 0.0 { 0.0 } else { 10.0 });
    }
}

#[test]
fn mac_number_fields_step_with_a_stepper_after_the_field() {
    let ctx = context(look::Look::Mac, false);
    let mut value = 9;
    let draw = |ui: &mut egui::Ui, value: &mut i32| {
        ui.add(look::NumberField::new(value).range(0..=10).width(120.0))
    };
    let field = frame(&ctx, vec![], |ui| draw(ui, &mut value)).rect;
    let (minus, plus) = step_points(&ctx, |ui| {
        draw(ui, &mut value);
    });
    assert!(
        plus.y < minus.y && (plus.x - minus.x).abs() < 0.5 && minus.x > field.center().x,
        "the stepper stacks Increase over Decrease at the end: {minus:?} {plus:?} in {field:?}"
    );
    for events in click(plus) {
        frame(&ctx, events, |ui| draw(ui, &mut value));
    }
    assert_eq!(value, 10);
    // At the maximum Increase is disabled and a click leaves the value alone.
    for events in click(plus) {
        frame(&ctx, events, |ui| draw(ui, &mut value));
    }
    assert_eq!(value, 10);
    for events in click(minus) {
        frame(&ctx, events, |ui| draw(ui, &mut value));
    }
    assert_eq!(value, 9);
}

#[test]
fn holding_a_step_button_repeats_after_a_delay() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut value = 0;
        let draw = |ui: &mut egui::Ui, value: &mut i32| {
            ui.add(look::NumberField::new(value).range(0..=100).width(160.0));
        };
        let (_, plus) = step_points(&ctx, |ui| draw(ui, &mut value));
        let at_time = |time: f64, events: Vec<egui::Event>| egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(340.0, 600.0),
            )),
            time: Some(time),
            events,
            ..Default::default()
        };
        let press = |pressed| egui::Event::PointerButton {
            pos: plus,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        let run = |input: egui::RawInput, value: &mut i32| {
            let mut output = ctx.run_ui(input, |ui| draw(ui, value));
            output.textures_delta.clear();
        };
        run(
            at_time(10.0, vec![egui::Event::PointerMoved(plus)]),
            &mut value,
        );
        run(at_time(10.0, vec![press(true)]), &mut value);
        // Held but not yet repeating.
        run(at_time(10.4, vec![]), &mut value);
        assert_eq!(value, 0, "{chosen}: no step before the delay");
        for step in 1..=15 {
            run(at_time(10.4 + 0.056 * f64::from(step), vec![]), &mut value);
        }
        run(at_time(11.25, vec![press(false)]), &mut value);
        // Eight repeats from 0.5 s to 1.2 s; egui reports no click after so long a press.
        assert_eq!(value, 8, "{chosen}: held for 1.25 s");
        run(at_time(12.0, vec![press(true)]), &mut value);
        run(at_time(12.1, vec![press(false)]), &mut value);
        assert_eq!(value, 9, "{chosen}: a short click is one step");
    }
}

#[test]
fn number_field_step_buttons_have_translatable_names() {
    let ctx = context(look::Look::Mac, true);
    ctx.enable_accesskit();
    let mut value = 5;
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        look::Form::new("step-names").show(ui, |form| {
            form.row("Radius", |ui| {
                ui.add(
                    look::NumberField::new(&mut value)
                        .range(0..=10)
                        .step_labels("Less", "More"),
                )
            });
        });
    });
    output.textures_delta.clear();
    let tree = output.platform_output.accesskit_update.unwrap();
    for label in ["Less Radius", "More Radius"] {
        assert!(
            tree.nodes
                .iter()
                .any(|(_, node)| node.role() == egui::Role::Button && node.label() == Some(label)),
            "missing {label}"
        );
    }
}

#[test]
fn text_field_ids_keep_focus_attached_when_rows_reorder() {
    let ctx = context(look::Look::Gtk, true);
    let mut values = [String::new(), String::new()];
    let draw = |ui: &mut egui::Ui, values: &mut [String; 2], reverse: bool| {
        let mut first = egui::Id::NULL;
        for i in if reverse { [1, 0] } else { [0, 1] } {
            let response = ui.add(
                look::TextField::new(&mut values[i])
                    .id_salt(("field", i))
                    .width(180.0),
            );
            if i == 0 {
                first = response.id;
            }
        }
        first
    };
    let first = frame(&ctx, vec![], |ui| draw(ui, &mut values, false));
    ctx.memory_mut(|m| m.request_focus(first));
    frame(&ctx, vec![], |ui| draw(ui, &mut values, false));
    let moved = frame(&ctx, vec![egui::Event::Text("hello".into())], |ui| {
        draw(ui, &mut values, true)
    });
    assert_eq!(moved, first);
    assert_eq!(values, ["hello", ""]);
}

#[test]
fn password_fields_mask_accessibility_and_do_not_copy_the_secret() {
    let ctx = context(look::Look::Mac, true);
    ctx.enable_accesskit();
    let secret = "private-test-value";
    let mut text = secret.to_string();
    let draw = |ui: &mut egui::Ui, text: &mut String| {
        ui.add(
            look::TextField::new(text)
                .password(true)
                .select_all_on_focus(true)
                .accessible_label("Password")
                .width(200.0),
        )
    };
    let response = frame(&ctx, vec![], |ui| draw(ui, &mut text));
    ctx.memory_mut(|m| m.request_focus(response.id));
    frame(&ctx, vec![], |ui| draw(ui, &mut text));
    let mut output = ctx.run_ui(
        egui::RawInput {
            events: vec![egui::Event::Copy],
            ..Default::default()
        },
        |ui| {
            draw(ui, &mut text);
        },
    );
    output.textures_delta.clear();
    assert!(
        !format!(
            "{:?}{:?}{:?}",
            output.platform_output.commands,
            output.platform_output.events,
            output.platform_output.accesskit_update
        )
        .contains(secret),
        "password leaked through platform output"
    );
    let tree = output.platform_output.accesskit_update.unwrap();
    assert!(tree
        .nodes
        .iter()
        .any(|(_, node)| node.role() == egui::Role::PasswordInput
            && node.label() == Some("Password")));
    assert_eq!(text, secret);
}

#[test]
fn fractional_integer_slider_steps_are_symmetric() {
    let ctx = context(look::Look::Plain, true);
    let mut value = 5_i32;
    let draw =
        |ui: &mut egui::Ui, value: &mut i32| ui.add(look::Slider::new(value, 0..=10).step_by(1.5));
    frame(&ctx, vec![], |ui| draw(ui, &mut value));
    frame(&ctx, key(egui::Key::Tab), |ui| draw(ui, &mut value));
    frame(&ctx, vec![], |ui| draw(ui, &mut value));
    frame(&ctx, key(egui::Key::ArrowRight), |ui| draw(ui, &mut value));
    assert_eq!(value, 7);
    frame(&ctx, key(egui::Key::ArrowLeft), |ui| draw(ui, &mut value));
    assert_eq!(value, 5);
}

#[test]
fn maximum_corner_radius_does_not_overflow_in_choice_tracks() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut theme = look::Theme::new(chosen);
        theme.metrics.radius = u8::MAX;
        look::install(&ctx, theme);
        let mut selected = 0;
        frame(&ctx, vec![], |ui| {
            ui.add(look::Segmented::new(
                "radius",
                &mut selected,
                &["First", "Second"],
            ));
            look::TabBar::new().show(ui, |bar| {
                bar.tab("First", true);
                bar.tab("Second", false);
            });
        });
    }
}

#[test]
fn title_bar_buttons_report_the_expected_window_actions() {
    for chosen in LOOKS {
        if chosen == look::Look::Mac {
            continue;
        }
        let actions = if chosen == look::Look::Windows {
            vec![
                (297.0, look::WindowAction::Close),
                (251.0, look::WindowAction::ToggleMaximize),
                (205.0, look::WindowAction::Minimize),
            ]
        } else {
            vec![(302.0, look::WindowAction::Close)]
        };
        for (x, expected) in actions {
            let ctx = context(chosen, true);
            let mut actual = None;
            let mut draw = |ui: &mut egui::Ui| {
                let action = look::TitleBar::new("Editor").maximized(true).show(
                    ui,
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(320.0, 32.0)),
                );
                actual = action.or(actual);
            };
            frame(&ctx, vec![], &mut draw);
            for events in click(egui::pos2(x, 16.0)) {
                frame(&ctx, events, &mut draw);
            }
            assert_eq!(actual, Some(expected), "{chosen}: wrong title bar action");
        }
    }
}

#[test]
fn visible_spinners_request_animation_and_progress_bars_stay_sized() {
    for chosen in LOOKS {
        for dark in [false, true] {
            let ctx = context(chosen, dark);
            for time in [0.0, 0.5, 1.0] {
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(340.0, 600.0),
                        )),
                        time: Some(time),
                        ..Default::default()
                    },
                    |ui| {
                        let rect = ui.add(look::Spinner::default().size(24.0)).rect;
                        assert_eq!(rect.size(), egui::vec2(24.0, 24.0));
                        for progress in [0.0, 0.5, 1.0] {
                            assert_eq!(
                                ui.add(look::ProgressBar::new(progress).width(180.0))
                                    .rect
                                    .width(),
                                180.0
                            );
                        }
                    },
                );
                output.textures_delta.clear();
                assert_eq!(
                    output.viewport_output[&egui::ViewportId::ROOT].repaint_delay,
                    std::time::Duration::ZERO,
                    "{chosen}: a visible spinner stopped animating"
                );
            }
        }
    }
}

#[test]
fn empty_choices_disable_interaction_and_preserve_the_model() {
    for chosen in LOOKS {
        for kind in 0..3 {
            let ctx = context(chosen, false);
            let mut selected = usize::MAX;
            let mut add = |ui: &mut egui::Ui| match kind {
                0 => ui.add(look::Segmented::new("empty", &mut selected, &[])),
                1 => ui.add(look::Dropdown::new("empty", &mut selected, &[])),
                _ => ui.add(look::Tabs::new("empty", &mut selected, &[])),
            };
            let response = frame(&ctx, vec![], &mut add);
            assert!(!response.enabled(), "{chosen}, kind {kind}");
            let at = response.rect.center();
            for events in click(at).into_iter().chain([key(egui::Key::ArrowRight)]) {
                let response = frame(&ctx, events, &mut add);
                assert!(!response.changed());
            }
            assert_eq!(selected, usize::MAX);
            assert!(!egui::Popup::is_any_open(&ctx));
        }
    }
}

#[test]
fn nonfinite_slider_ranges_disable_without_changing_the_model() {
    for chosen in LOOKS {
        for endpoint in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for invalid_start in [false, true] {
                for show_value in [false, true] {
                    let ctx = context(chosen, false);
                    let mut value = 42.0;
                    let (start, end) = if invalid_start {
                        (endpoint, 100.0)
                    } else {
                        (0.0, endpoint)
                    };
                    let response = frame(&ctx, vec![], |ui| {
                        ui.add(
                            look::Slider::new(&mut value, start..=end)
                                .clamping(egui::SliderClamping::Always)
                                .show_value(show_value),
                        )
                    });
                    assert_eq!(value, 42.0);
                    assert!(!response.enabled());
                    assert!(!response.changed());
                    assert!(response.rect.is_finite());
                }
            }
        }
    }
}

#[test]
fn nan_numeric_models_change_only_when_normalization_is_requested() {
    for chosen in LOOKS {
        for normalize in [false, true] {
            let ctx = context(chosen, false);
            let mut slider = f64::NAN;
            let mut number = f64::NAN;
            for _ in 0..2 {
                let (slider_response, number_response) = frame(&ctx, vec![], |ui| {
                    (
                        ui.add(
                            look::Slider::new(&mut slider, 5.0..=10.0).clamping(if normalize {
                                egui::SliderClamping::Always
                            } else {
                                egui::SliderClamping::Never
                            }),
                        ),
                        ui.add(
                            look::NumberField::new(&mut number)
                                .range(5.0..=10.0)
                                .clamp_existing_to_range(normalize),
                        ),
                    )
                });
                assert!(slider_response.rect.is_finite());
                assert!(number_response.rect.is_finite());
                if normalize {
                    assert_eq!(slider, 5.0);
                    assert_eq!(number, 5.0);
                } else {
                    assert!(slider.is_nan());
                    assert!(number.is_nan());
                    assert!(!slider_response.changed());
                    assert!(!number_response.changed());
                }
            }
        }
    }
}

#[test]
fn equal_slider_bounds_only_normalize_with_always_clamping() {
    for chosen in LOOKS {
        for clamping in [egui::SliderClamping::Always, egui::SliderClamping::Never] {
            let ctx = context(chosen, false);
            let mut value = 42.0;
            let response = frame(&ctx, vec![], |ui| {
                ui.add(look::Slider::new(&mut value, 7.0..=7.0).clamping(clamping))
            });
            assert!(!response.enabled());
            let normalize = clamping == egui::SliderClamping::Always;
            assert_eq!(value, if normalize { 7.0 } else { 42.0 });
            assert_eq!(response.changed(), normalize);
        }
    }
}

#[test]
fn invalid_number_range_keeps_the_previous_range() {
    let ctx = context(look::Look::Gtk, false);
    for endpoint in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for invalid_start in [false, true] {
            let mut value = 42.0;
            let range = if invalid_start {
                endpoint..=100.0
            } else {
                0.0..=endpoint
            };
            let response = frame(&ctx, vec![], |ui| {
                ui.add(
                    look::NumberField::new(&mut value)
                        .range(5.0..=10.0)
                        .range(range.clone()),
                )
            });
            assert_eq!(value, 10.0);
            assert!(response.changed());
        }
    }
}

#[test]
fn malformed_dimensions_keep_widget_geometry_finite() {
    for chosen in LOOKS {
        let ctx = context(chosen, false);
        for size in [
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            -1.0,
            0.0,
            f32::MIN_POSITIVE,
            f32::MAX,
        ] {
            let mut selected = 0;
            let mut number = 5.0;
            let mut text = String::new();
            let responses = frame(&ctx, vec![], |ui| {
                vec![
                    ui.add(look::Button::new("OK").width(size)),
                    ui.add(look::Segmented::new("s", &mut selected, &["A", "B"]).width(size)),
                    ui.add(look::Dropdown::new("d", &mut selected, &["A", "B"]).width(size)),
                    ui.add(look::Slider::new(&mut number, 0.0..=10.0).width(size)),
                    ui.add(look::TextField::new(&mut text).width(size)),
                    ui.add(look::NumberField::new(&mut number).width(size)),
                    ui.add(look::ProgressBar::new(0.5).width(size)),
                    ui.add(look::Spinner::new().size(size)),
                    ui.add(look::ListRow::new(false).height(size)),
                ]
            });
            for response in responses {
                assert!(response.rect.is_finite(), "{chosen}, size {size}");
                assert!(response.rect.width() >= 0.0 && response.rect.height() >= 0.0);
            }
        }
    }
}

#[test]
fn installed_metrics_and_direct_styles_normalize_invalid_sizes() {
    for chosen in LOOKS {
        for size in [
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            0.0,
            -1.0,
            f32::MIN_POSITIVE,
            f32::MAX,
        ] {
            let mut theme = look::Theme::new(chosen);
            theme.metrics.control_height = size;
            theme.metrics.body.size = size;
            theme.metrics.bold.size = size;
            let expected_height = if size.is_finite() && size > 0.0 {
                size.clamp(1.0, 1_000_000.0)
            } else {
                look::Metrics::of(chosen).control_height
            };
            let expected_font = if size.is_finite() && size > 0.0 {
                size.clamp(1.0, 256.0)
            } else {
                look::Metrics::of(chosen).body.size
            };
            let mut style = egui::Style::default();
            look::apply_style(&mut style, &theme, false);
            assert_eq!(style.spacing.interact_size.y, expected_height);
            assert_eq!(
                style.text_styles[&egui::TextStyle::Body].size,
                expected_font
            );
            let ctx = egui::Context::default();
            look::install(&ctx, theme.clone());
            let installed = look::theme(&ctx);
            assert_eq!(installed.metrics.control_height, expected_height);
            assert_eq!(installed.metrics.body.size, expected_font);
            assert_eq!(installed.metrics.bold.size, expected_font);
            assert_eq!(
                look::Theme::new(chosen).metrics(theme.metrics).metrics,
                installed.metrics
            );
        }
    }
}

#[test]
fn clearing_an_open_dropdown_closes_its_popup_without_changing_selection() {
    for chosen in LOOKS {
        let ctx = context(chosen, false);
        let mut selected = 1;
        let draw = |ui: &mut egui::Ui, selected: &mut usize, items: &[&str]| {
            ui.add(look::Dropdown::new("dynamic", selected, items))
        };
        let response = frame(&ctx, vec![], |ui| draw(ui, &mut selected, &["A", "B"]));
        for events in click(response.rect.center()) {
            frame(&ctx, events, |ui| draw(ui, &mut selected, &["A", "B"]));
        }
        assert!(egui::Popup::is_any_open(&ctx));
        let response = frame(&ctx, vec![], |ui| draw(ui, &mut selected, &[]));
        assert!(!egui::Popup::is_any_open(&ctx));
        assert!(!response.enabled());
        assert!(!response.changed());
        assert_eq!(selected, 1);
    }
}

#[test]
fn keyboard_input_recovers_a_nan_slider_model() {
    for chosen in LOOKS {
        for show_value in [false, true] {
            let ctx = context(chosen, false);
            let mut value = f64::NAN;
            let draw = |ui: &mut egui::Ui, value: &mut f64| {
                ui.add(
                    look::Slider::new(value, 5.0..=10.0)
                        .step_by(1.0)
                        .clamping(egui::SliderClamping::Edits)
                        .show_value(show_value),
                )
            };
            let response = frame(&ctx, vec![], |ui| draw(ui, &mut value));
            assert!(value.is_nan());
            assert!(!response.changed());
            ctx.memory_mut(|m| m.request_focus(response.id));
            frame(&ctx, vec![], |ui| draw(ui, &mut value));
            let response = frame(&ctx, key(egui::Key::ArrowRight), |ui| draw(ui, &mut value));
            assert_eq!(value, 6.0, "{chosen}, show value {show_value}");
            assert!(response.changed());
        }
    }
}

fn modified_key(key: egui::Key, modifiers: egui::Modifiers) -> Vec<egui::Event> {
    vec![egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }]
}

fn primary_button(pos: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    }
}

#[test]
fn gtk_slider_shortcuts_move_values_without_moving_focus() {
    use egui::{Key, Modifiers};
    for reversed in [false, true] {
        for (key, modifiers, ordinary, reversed_value) in [
            (Key::ArrowRight, Modifiers::NONE, 51, 49),
            (Key::ArrowLeft, Modifiers::NONE, 49, 51),
            (Key::ArrowUp, Modifiers::NONE, 51, 51),
            (Key::ArrowDown, Modifiers::NONE, 49, 49),
            (Key::ArrowRight, Modifiers::CTRL, 60, 40),
            (Key::ArrowLeft, Modifiers::CTRL, 40, 60),
            (Key::ArrowUp, Modifiers::CTRL, 60, 60),
            (Key::ArrowDown, Modifiers::CTRL, 40, 40),
            (Key::PageUp, Modifiers::NONE, 60, 60),
            (Key::PageDown, Modifiers::NONE, 40, 40),
            (Key::PageUp, Modifiers::CTRL, 40, 60),
            (Key::PageDown, Modifiers::CTRL, 60, 40),
            (Key::Plus, Modifiers::NONE, 51, 51),
            (Key::Plus, Modifiers::SHIFT, 51, 51),
            (Key::Minus, Modifiers::NONE, 49, 49),
            (Key::Plus, Modifiers::CTRL, 60, 60),
            (Key::Minus, Modifiers::CTRL, 40, 40),
            (Key::Home, Modifiers::NONE, 0, 0),
            (Key::End, Modifiers::NONE, 100, 100),
        ] {
            let ctx = context(look::Look::Gtk, false);
            let mut value = 50;
            let draw = |ui: &mut egui::Ui, value: &mut i32| {
                let (start, end) = if reversed { (100, 0) } else { (0, 100) };
                let response = ui.add(look::Slider::new(value, start..=end).show_value(false));
                ui.add(look::Button::new("Next"));
                response
            };
            let response = frame(&ctx, vec![], |ui| draw(ui, &mut value));
            ctx.memory_mut(|m| m.request_focus(response.id));
            frame(&ctx, vec![], |ui| draw(ui, &mut value));
            let (response, consumed) = frame(&ctx, modified_key(key, modifiers), |ui| {
                let response = draw(ui, &mut value);
                (response, !ui.input(|i| i.key_pressed(key)))
            });
            assert_eq!(
                value,
                if reversed { reversed_value } else { ordinary },
                "{key:?}, {modifiers:?}"
            );
            assert!(response.has_focus());
            assert!(response.changed());
            assert!(consumed);
        }
    }
}

#[test]
fn gtk_slider_leaves_application_shortcuts_and_other_widgets_alone() {
    for modifiers in [
        egui::Modifiers::ALT,
        egui::Modifiers::SHIFT,
        egui::Modifiers::MAC_CMD,
    ] {
        let ctx = context(look::Look::Gtk, false);
        let mut value = 50;
        let draw = |ui: &mut egui::Ui, value: &mut i32| {
            ui.add(look::Slider::new(value, 0..=100).show_value(false))
        };
        let response = frame(&ctx, vec![], |ui| draw(ui, &mut value));
        ctx.memory_mut(|m| m.request_focus(response.id));
        frame(&ctx, vec![], |ui| draw(ui, &mut value));
        let remains = frame(&ctx, modified_key(egui::Key::PageUp, modifiers), |ui| {
            let response = draw(ui, &mut value);
            assert!(!response.changed());
            ui.input(|i| i.key_pressed(egui::Key::PageUp))
        });
        assert_eq!(value, 50);
        assert!(remains);
    }
    let ctx = context(look::Look::Gtk, false);
    let mut value = 50;
    let response = frame(&ctx, key(egui::Key::End), |ui| {
        ui.add(look::Slider::new(&mut value, 0..=100))
    });
    assert_eq!(value, 50);
    assert!(!response.changed());
}

#[test]
fn gtk_slider_pages_respect_steps_boundaries_and_logarithmic_ranges() {
    for logarithmic in [false, true] {
        let ctx = context(look::Look::Gtk, false);
        let mut value = if logarithmic { 10.0 } else { 5.0 };
        let draw = |ui: &mut egui::Ui, value: &mut f64| {
            let slider =
                look::Slider::new(value, if logarithmic { 1.0..=100.0 } else { 0.0..=10.0 })
                    .logarithmic(logarithmic)
                    .show_value(false);
            ui.add(if logarithmic {
                slider
            } else {
                slider.step_by(3.0)
            })
        };
        let response = frame(&ctx, vec![], |ui| draw(ui, &mut value));
        ctx.memory_mut(|m| m.request_focus(response.id));
        frame(&ctx, vec![], |ui| draw(ui, &mut value));
        let response = frame(&ctx, key(egui::Key::PageUp), |ui| draw(ui, &mut value));
        if logarithmic {
            assert!((value - 100.0_f64.powf(0.6)).abs() < 1e-10);
        } else {
            assert_eq!(value, 10.0);
        }
        assert!(response.changed());
        frame(&ctx, key(egui::Key::End), |ui| draw(ui, &mut value));
        let response = frame(&ctx, key(egui::Key::PageUp), |ui| draw(ui, &mut value));
        assert_eq!(value, if logarithmic { 100.0 } else { 10.0 });
        assert!(!response.changed());
    }
}

#[test]
fn switch_drag_commits_only_on_release_and_uses_the_nearest_state() {
    for chosen in SYSTEM_LOOKS {
        for (initial, distances, expected) in [
            (false, vec![18.0], true),
            (true, vec![-18.0], false),
            (false, vec![8.0], false),
            (true, vec![-8.0], true),
            (false, vec![18.0, 0.0], false),
            (true, vec![-18.0, 0.0], true),
        ] {
            let ctx = context(chosen, false);
            let mut on = initial;
            let draw = |ui: &mut egui::Ui, on: &mut bool| ui.add(look::Switch::new(on));
            let response = frame(&ctx, vec![], |ui| draw(ui, &mut on));
            let start = egui::pos2(
                response.rect.center().x + if initial { 10.0 } else { -10.0 },
                response.rect.center().y,
            );
            frame(&ctx, vec![egui::Event::PointerMoved(start)], |ui| {
                draw(ui, &mut on)
            });
            let response = frame(&ctx, vec![primary_button(start, true)], |ui| {
                draw(ui, &mut on)
            });
            assert!(response.has_focus());
            let mut end = start;
            for distance in distances {
                end = start + egui::vec2(distance, 0.0);
                let response = frame(&ctx, vec![egui::Event::PointerMoved(end)], |ui| {
                    draw(ui, &mut on)
                });
                assert_eq!(on, initial);
                assert!(!response.changed());
            }
            let response = frame(&ctx, vec![primary_button(end, false)], |ui| {
                draw(ui, &mut on)
            });
            assert_eq!(on, expected);
            assert_eq!(response.changed(), expected != initial);
        }
    }
}

#[test]
fn switch_does_not_start_a_drag_from_the_empty_half() {
    for chosen in SYSTEM_LOOKS {
        for initial in [false, true] {
            let ctx = context(chosen, false);
            let mut on = initial;
            let draw = |ui: &mut egui::Ui, on: &mut bool| ui.add(look::Switch::new(on));
            let response = frame(&ctx, vec![], |ui| draw(ui, &mut on));
            let start =
                response.rect.center() + egui::vec2(if initial { -10.0 } else { 10.0 }, 0.0);
            frame(&ctx, vec![egui::Event::PointerMoved(start)], |ui| {
                draw(ui, &mut on)
            });
            frame(&ctx, vec![primary_button(start, true)], |ui| {
                draw(ui, &mut on)
            });
            let end = start + egui::vec2(if initial { 18.0 } else { -18.0 }, 0.0);
            frame(&ctx, vec![egui::Event::PointerMoved(end)], |ui| {
                draw(ui, &mut on)
            });
            let response = frame(&ctx, vec![primary_button(end, false)], |ui| {
                draw(ui, &mut on)
            });
            assert_eq!(on, initial);
            assert!(!response.changed());
        }
    }
}

#[test]
fn switches_and_mixed_checkboxes_expose_their_actual_accessible_states() {
    for chosen in LOOKS {
        let ctx = context(chosen, false);
        ctx.enable_accesskit();
        for on in [false, true] {
            let mut on = on;
            let mut checked = false;
            let mut ids = Vec::new();
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                ids.push(
                    ui.add(look::Switch::new(&mut on).accessible_label("Power"))
                        .id,
                );
                ids.push(
                    ui.add(look::Checkbox::new(&mut checked, "Mixed").indeterminate(true))
                        .id,
                );
            });
            output.textures_delta.clear();
            let update = output.platform_output.accesskit_update.unwrap();
            let nodes: Vec<_> = ids
                .iter()
                .map(|id| {
                    &update
                        .nodes
                        .iter()
                        .find(|(node_id, _)| *node_id == id.accesskit_id())
                        .unwrap()
                        .1
                })
                .collect();
            assert_eq!(nodes[0].role(), egui::Role::Switch);
            assert_eq!(
                nodes[0].toggled(),
                Some(if on {
                    egui::accesskit::Toggled::True
                } else {
                    egui::accesskit::Toggled::False
                })
            );
            assert_eq!(nodes[0].label(), Some("Power"));
            assert_eq!(nodes[1].toggled(), Some(egui::accesskit::Toggled::Mixed));
        }
    }
}

#[test]
fn vertical_touch_drags_over_a_gtk_switch_scroll_its_container() {
    let ctx = context(look::Look::Gtk, false);
    let mut on = false;
    let draw = |ui: &mut egui::Ui, on: &mut bool| {
        let area = egui::ScrollArea::vertical()
            .scroll_source(egui::scroll_area::ScrollSource::ALL)
            .max_height(120.0)
            .show(ui, |ui| {
                ui.add_space(40.0);
                let response = ui.add(look::Switch::new(on));
                ui.add_space(400.0);
                response
            });
        (area.inner, area.state.clamped_offset().y)
    };
    let touch = |phase, pos| egui::Event::Touch {
        device_id: egui::TouchDeviceId(1),
        id: egui::TouchId(1),
        phase,
        pos,
        force: None,
    };
    let (response, _) = frame(&ctx, vec![], |ui| draw(ui, &mut on));
    frame(&ctx, vec![], |ui| draw(ui, &mut on));
    let start = response.rect.center() - egui::vec2(10.0, 0.0);
    frame(
        &ctx,
        vec![
            touch(egui::TouchPhase::Start, start),
            egui::Event::PointerMoved(start),
            primary_button(start, true),
        ],
        |ui| draw(ui, &mut on),
    );
    let mut offset = 0.0;
    let mut end = start;
    for distance in [12.0, 24.0, 40.0] {
        end = start - egui::vec2(0.0, distance);
        let (response, scroll) = frame(
            &ctx,
            vec![
                touch(egui::TouchPhase::Move, end),
                egui::Event::PointerMoved(end),
            ],
            |ui| draw(ui, &mut on),
        );
        offset = scroll;
        assert!(!response.changed());
    }
    frame(
        &ctx,
        vec![
            touch(egui::TouchPhase::End, end),
            primary_button(end, false),
            egui::Event::PointerGone,
        ],
        |ui| draw(ui, &mut on),
    );
    assert!(offset > 20.0, "vertical gesture failed to scroll: {offset}");
    assert!(!on);
}

#[test]
fn interrupted_switch_drags_do_not_commit() {
    for chosen in SYSTEM_LOOKS {
        for interrupt in 0..3 {
            let ctx = context(chosen, false);
            let mut on = false;
            let draw = |ui: &mut egui::Ui, on: &mut bool, enabled: bool| {
                ui.add_enabled(enabled, look::Switch::new(on))
            };
            let response = frame(&ctx, vec![], |ui| draw(ui, &mut on, true));
            let start = response.rect.center() - egui::vec2(10.0, 0.0);
            let end = start + egui::vec2(18.0, 0.0);
            frame(&ctx, vec![egui::Event::PointerMoved(start)], |ui| {
                draw(ui, &mut on, true)
            });
            frame(&ctx, vec![primary_button(start, true)], |ui| {
                draw(ui, &mut on, true)
            });
            frame(&ctx, vec![egui::Event::PointerMoved(end)], |ui| {
                draw(ui, &mut on, true)
            });
            let events = match interrupt {
                0 => vec![],
                1 => vec![egui::Event::WindowFocused(false)],
                _ => vec![egui::Event::Touch {
                    device_id: egui::TouchDeviceId(1),
                    id: egui::TouchId(1),
                    phase: egui::TouchPhase::Cancel,
                    pos: end,
                    force: None,
                }],
            };
            frame(&ctx, events, |ui| draw(ui, &mut on, interrupt != 0));
            let response = frame(
                &ctx,
                vec![egui::Event::WindowFocused(true), primary_button(end, false)],
                |ui| draw(ui, &mut on, true),
            );
            assert!(!on, "interruption {interrupt}");
            assert!(!response.changed());
            assert_ne!(ctx.dragged_id(), Some(response.id));
        }
    }
}

#[test]
fn gtk_slider_takes_focus_when_pressed_and_ignores_keys_while_disabled() {
    let ctx = context(look::Look::Gtk, false);
    let mut value = 5;
    let draw = |ui: &mut egui::Ui, value: &mut i32, enabled: bool| {
        ui.add_enabled(enabled, look::Slider::new(value, 0..=10).show_value(false))
    };
    let response = frame(&ctx, vec![], |ui| draw(ui, &mut value, true));
    let at = response.rect.center();
    frame(&ctx, vec![egui::Event::PointerMoved(at)], |ui| {
        draw(ui, &mut value, true)
    });
    let response = frame(&ctx, vec![primary_button(at, true)], |ui| {
        draw(ui, &mut value, true)
    });
    assert!(response.has_focus());
    frame(&ctx, vec![primary_button(at, false)], |ui| {
        draw(ui, &mut value, true)
    });
    let previous = value;
    let response = frame(&ctx, key(egui::Key::End), |ui| draw(ui, &mut value, false));
    assert_eq!(value, previous);
    assert!(!response.changed());
}

#[test]
fn checkbox_accessibility_reports_the_new_state_on_the_click_frame() {
    for chosen in LOOKS {
        let ctx = context(chosen, false);
        ctx.enable_accesskit();
        let mut checked = false;
        let draw =
            |ui: &mut egui::Ui, checked: &mut bool| ui.add(look::Checkbox::new(checked, "Active"));
        let response = frame(&ctx, vec![], |ui| draw(ui, &mut checked));
        let at = response.rect.center();
        frame(&ctx, vec![egui::Event::PointerMoved(at)], |ui| {
            draw(ui, &mut checked)
        });
        frame(&ctx, vec![primary_button(at, true)], |ui| {
            draw(ui, &mut checked)
        });
        let mut output = ctx.run_ui(
            egui::RawInput {
                events: vec![primary_button(at, false)],
                ..Default::default()
            },
            |ui| {
                draw(ui, &mut checked);
            },
        );
        output.textures_delta.clear();
        assert!(checked);
        let update = output.platform_output.accesskit_update.unwrap();
        let node = &update
            .nodes
            .iter()
            .find(|(id, _)| *id == response.id.accesskit_id())
            .unwrap()
            .1;
        assert_eq!(node.toggled(), Some(egui::accesskit::Toggled::True));
    }
}

#[test]
fn switch_keyboard_activation_and_single_frame_clicks_keep_working() {
    for chosen in LOOKS {
        let ctx = context(chosen, false);
        let mut on = false;
        let draw = |ui: &mut egui::Ui, on: &mut bool| ui.add(look::Switch::new(on));
        let response = frame(&ctx, vec![], |ui| draw(ui, &mut on));
        ctx.memory_mut(|m| m.request_focus(response.id));
        frame(&ctx, vec![], |ui| draw(ui, &mut on));
        for (activation, expected) in [(egui::Key::Space, true), (egui::Key::Enter, false)] {
            let response = frame(&ctx, key(activation), |ui| draw(ui, &mut on));
            assert_eq!(on, expected, "{chosen}, {activation:?}");
            assert!(response.changed());
        }
        let at = response.rect.center();
        frame(&ctx, vec![egui::Event::PointerMoved(at)], |ui| {
            draw(ui, &mut on)
        });
        let response = frame(
            &ctx,
            vec![primary_button(at, true), primary_button(at, false)],
            |ui| draw(ui, &mut on),
        );
        assert!(on, "{chosen}");
        assert!(response.changed());
    }
}

#[test]
fn releasing_a_switch_drag_does_not_rewind_the_thumb_animation() {
    for chosen in SYSTEM_LOOKS {
        for initial in [false, true] {
            let ctx = context(chosen, false);
            let mut on = initial;
            let draw = |events: Vec<egui::Event>, on: &mut bool| {
                let mut response = None;
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(340.0, 600.0),
                        )),
                        events,
                        ..Default::default()
                    },
                    |ui| {
                        response = Some(ui.add(look::Switch::new(on)));
                    },
                );
                // The thumb is drawn last: a filled disc, or macOS's narrow pill.
                let thumb = output
                    .shapes
                    .iter()
                    .filter_map(|shape| match &shape.shape {
                        egui::Shape::Circle(circle) if circle.fill.a() == 255 => {
                            Some(circle.center.x)
                        }
                        egui::Shape::Rect(rect)
                            if rect.fill.a() == 255 && rect.rect.width() < 40.0 =>
                        {
                            Some(rect.rect.center().x)
                        }
                        _ => None,
                    })
                    .next_back()
                    .expect("visible thumb");
                output.textures_delta.clear();
                (response.unwrap(), thumb)
            };
            let (response, _) = draw(vec![], &mut on);
            let start =
                response.rect.center() + egui::vec2(if initial { 10.0 } else { -10.0 }, 0.0);
            draw(vec![egui::Event::PointerMoved(start)], &mut on);
            draw(vec![primary_button(start, true)], &mut on);
            let end = start + egui::vec2(if initial { -18.0 } else { 18.0 }, 0.0);
            let (_, dragging) = draw(vec![egui::Event::PointerMoved(end)], &mut on);
            let (_, released) = draw(vec![primary_button(end, false)], &mut on);
            let (_, settled) = draw(vec![], &mut on);
            assert_eq!(on, !initial);
            assert!(if initial {
                released <= dragging
            } else {
                released >= dragging
            });
            assert_eq!(released, settled);
        }
    }
}

#[test]
fn grabbing_the_edge_of_a_gtk_or_ios_thumb_keeps_the_value() {
    for chosen in [look::Look::Gtk, look::Look::Ios] {
        for reversed in [false, true] {
            let ctx = context(chosen, false);
            let mut value = 50.0;
            let (start, end) = if reversed { (100.0, 0.0) } else { (0.0, 100.0) };
            let draw = |ui: &mut egui::Ui, value: &mut f64| {
                ui.add(look::Slider::new(value, start..=end).width(200.0))
            };
            let response = frame(&ctx, vec![], |ui| draw(ui, &mut value));
            let at = response.rect.center() + egui::vec2(6.0, 0.0);
            frame(&ctx, vec![egui::Event::PointerMoved(at)], |ui| {
                draw(ui, &mut value)
            });
            let response = frame(&ctx, vec![primary_button(at, true)], |ui| {
                draw(ui, &mut value)
            });
            assert_eq!(value, 50.0);
            assert!(!response.changed());
            // A tenth of the knob's travel: the rail less the knob's width (38 on iOS).
            let delta = if chosen == look::Look::Ios {
                16.2
            } else {
                18.0
            };
            let end = at + egui::vec2(delta, 0.0);
            let response = frame(&ctx, vec![egui::Event::PointerMoved(end)], |ui| {
                draw(ui, &mut value)
            });
            assert!(
                (value - if reversed { 40.0 } else { 60.0 }).abs() < 1e-5,
                "{chosen}: {value}"
            );
            assert!(response.changed());
            assert!(response.drag_started());
            assert!(response.dragged());
            let response = frame(&ctx, vec![primary_button(end, false)], |ui| {
                draw(ui, &mut value)
            });
            assert!(response.drag_stopped());
            assert!(!response.changed());
        }
    }
}

#[test]
fn small_slider_moves_accumulate_across_integer_steps() {
    let ctx = context(look::Look::Gtk, false);
    let mut value = 5;
    let draw =
        |ui: &mut egui::Ui, value: &mut i32| ui.add(look::Slider::new(value, 0..=10).width(200.0));
    let response = frame(&ctx, vec![], |ui| draw(ui, &mut value));
    let start = response.rect.center();
    frame(&ctx, vec![egui::Event::PointerMoved(start)], |ui| {
        draw(ui, &mut value)
    });
    frame(&ctx, vec![primary_button(start, true)], |ui| {
        draw(ui, &mut value)
    });
    for distance in [2.0, 4.0, 6.0, 8.0, 10.0] {
        frame(
            &ctx,
            vec![egui::Event::PointerMoved(start + egui::vec2(distance, 0.0))],
            |ui| draw(ui, &mut value),
        );
    }
    assert_eq!(value, 6);
}

#[test]
fn changing_fine_drag_mode_keeps_the_thumb_stationary_until_the_pointer_moves() {
    let ctx = context(look::Look::Gtk, false);
    let mut value = 50.0;
    let draw = |ui: &mut egui::Ui, value: &mut f64| {
        ui.add(look::Slider::new(value, 0.0..=100.0).width(200.0))
    };
    let response = frame(&ctx, vec![], |ui| draw(ui, &mut value));
    let start = response.rect.center();
    frame(&ctx, vec![egui::Event::PointerMoved(start)], |ui| {
        draw(ui, &mut value)
    });
    frame(&ctx, vec![primary_button(start, true)], |ui| {
        draw(ui, &mut value)
    });
    frame(
        &ctx,
        vec![egui::Event::PointerMoved(start + egui::vec2(18.0, 0.0))],
        |ui| draw(ui, &mut value),
    );
    assert!((value - 60.0).abs() < 1e-6);
    let response = frame(
        &ctx,
        vec![egui::Event::ModifiersChanged(egui::Modifiers::SHIFT)],
        |ui| draw(ui, &mut value),
    );
    assert!((value - 60.0).abs() < 1e-6);
    assert!(!response.changed());
    frame(
        &ctx,
        vec![egui::Event::PointerMoved(start + egui::vec2(36.0, 0.0))],
        |ui| draw(ui, &mut value),
    );
    assert!((value - 62.5).abs() < 1e-6);
    let response = frame(
        &ctx,
        vec![egui::Event::ModifiersChanged(egui::Modifiers::NONE)],
        |ui| draw(ui, &mut value),
    );
    assert!((value - 62.5).abs() < 1e-6);
    assert!(!response.changed());
    frame(
        &ctx,
        vec![egui::Event::PointerMoved(start + egui::vec2(54.0, 0.0))],
        |ui| draw(ui, &mut value),
    );
    assert!((value - 72.5).abs() < 1e-6);
}

#[test]
fn interrupted_slider_drags_stop_at_the_last_value() {
    for chosen in LOOKS {
        for interrupt in 0..4 {
            if interrupt == 3 && chosen != look::Look::Gtk {
                continue;
            }
            let ctx = context(chosen, false);
            let mut value = 50.0;
            let draw = |ui: &mut egui::Ui, value: &mut f64, enabled: bool| {
                ui.add_enabled(enabled, look::Slider::new(value, 0.0..=100.0).width(200.0))
            };
            let response = frame(&ctx, vec![], |ui| draw(ui, &mut value, true));
            let start = response.rect.center();
            frame(&ctx, vec![egui::Event::PointerMoved(start)], |ui| {
                draw(ui, &mut value, true)
            });
            frame(&ctx, vec![primary_button(start, true)], |ui| {
                draw(ui, &mut value, true)
            });
            let end = start + egui::vec2(20.0, 0.0);
            frame(&ctx, vec![egui::Event::PointerMoved(end)], |ui| {
                draw(ui, &mut value, true)
            });
            let previous = value;
            let events = match interrupt {
                0 => vec![],
                1 => vec![egui::Event::WindowFocused(false)],
                2 => vec![egui::Event::Touch {
                    device_id: egui::TouchDeviceId(1),
                    id: egui::TouchId(1),
                    phase: egui::TouchPhase::Cancel,
                    pos: end,
                    force: None,
                }],
                _ => key(egui::Key::Escape),
            };
            let response = frame(&ctx, events, |ui| draw(ui, &mut value, interrupt != 0));
            if interrupt == 3 {
                assert!(response.has_focus());
            }
            frame(
                &ctx,
                vec![
                    egui::Event::WindowFocused(true),
                    egui::Event::PointerMoved(end + egui::vec2(30.0, 0.0)),
                ],
                |ui| draw(ui, &mut value, true),
            );
            let response = frame(
                &ctx,
                vec![primary_button(end + egui::vec2(30.0, 0.0), false)],
                |ui| draw(ui, &mut value, true),
            );
            assert_eq!(value, previous, "{chosen}, interruption {interrupt}");
            assert!(!response.changed());
        }
    }
}

#[test]
fn vertical_touch_drags_over_sliders_scroll_without_changing_the_value() {
    for chosen in LOOKS {
        let ctx = context(chosen, false);
        let mut value = 50.0;
        let draw = |ui: &mut egui::Ui, value: &mut f64| {
            let area = egui::ScrollArea::vertical()
                .scroll_source(egui::scroll_area::ScrollSource::ALL)
                .max_height(120.0)
                .show(ui, |ui| {
                    ui.add_space(40.0);
                    let response = ui.add(look::Slider::new(value, 0.0..=100.0).width(200.0));
                    ui.add_space(400.0);
                    response
                });
            (area.inner, area.state.clamped_offset().y)
        };
        let touch = |phase, pos| egui::Event::Touch {
            device_id: egui::TouchDeviceId(1),
            id: egui::TouchId(1),
            phase,
            pos,
            force: None,
        };
        let (response, _) = frame(&ctx, vec![], |ui| draw(ui, &mut value));
        frame(&ctx, vec![], |ui| draw(ui, &mut value));
        let start = response.rect.center() + egui::vec2(30.0, 0.0);
        frame(
            &ctx,
            vec![
                touch(egui::TouchPhase::Start, start),
                egui::Event::PointerMoved(start),
                primary_button(start, true),
            ],
            |ui| draw(ui, &mut value),
        );
        assert_eq!(
            value, 50.0,
            "{chosen}: touch-down changed the model before gesture recognition"
        );
        let mut offset = 0.0;
        let mut end = start;
        for distance in [12.0, 24.0, 40.0] {
            end = start - egui::vec2(0.0, distance);
            let (response, scroll) = frame(
                &ctx,
                vec![
                    touch(egui::TouchPhase::Move, end),
                    egui::Event::PointerMoved(end),
                ],
                |ui| draw(ui, &mut value),
            );
            offset = scroll;
            assert!(!response.changed());
        }
        frame(
            &ctx,
            vec![
                touch(egui::TouchPhase::End, end),
                primary_button(end, false),
                egui::Event::PointerGone,
            ],
            |ui| draw(ui, &mut value),
        );
        assert!(offset > 20.0, "{chosen}: scroll offset {offset}");
        assert_eq!(value, 50.0);
    }
}

#[test]
fn slider_touch_taps_commit_on_release_and_horizontal_drags_still_work() {
    for chosen in LOOKS {
        let ctx = context(chosen, false);
        let mut value = 50.0;
        let draw = |ui: &mut egui::Ui, value: &mut f64| {
            ui.add(look::Slider::new(value, 0.0..=100.0).width(200.0))
        };
        let touch = |phase, pos| egui::Event::Touch {
            device_id: egui::TouchDeviceId(1),
            id: egui::TouchId(1),
            phase,
            pos,
            force: None,
        };
        let response = frame(&ctx, vec![], |ui| draw(ui, &mut value));
        let start = egui::pos2(response.rect.right() - 16.0, response.rect.center().y);
        frame(
            &ctx,
            vec![
                touch(egui::TouchPhase::Start, start),
                egui::Event::PointerMoved(start),
                primary_button(start, true),
            ],
            |ui| draw(ui, &mut value),
        );
        assert_eq!(value, 50.0);
        let response = frame(
            &ctx,
            vec![
                touch(egui::TouchPhase::End, start),
                primary_button(start, false),
                egui::Event::PointerGone,
            ],
            |ui| draw(ui, &mut value),
        );
        assert!(value > 90.0, "{chosen}: {value}");
        assert!(response.changed());
        let previous = value;
        let start = response.rect.center();
        frame(
            &ctx,
            vec![
                touch(egui::TouchPhase::Start, start),
                egui::Event::PointerMoved(start),
                primary_button(start, true),
            ],
            |ui| draw(ui, &mut value),
        );
        let end = start - egui::vec2(35.0, 0.0);
        let response = frame(
            &ctx,
            vec![
                touch(egui::TouchPhase::Move, end),
                egui::Event::PointerMoved(end),
            ],
            |ui| draw(ui, &mut value),
        );
        assert!(
            value < previous,
            "{chosen}: horizontal drag did not change the value"
        );
        assert!(response.dragged());
    }
}

#[test]
fn gtk_number_field_keyboard_steps_are_independent_of_drag_speed() {
    use egui::{Key, Modifiers};
    for reversed in [false, true] {
        for (key, modifiers, expected) in [
            (Key::ArrowUp, Modifiers::NONE, 53.0),
            (Key::ArrowDown, Modifiers::NONE, 47.0),
            (Key::PageUp, Modifiers::NONE, 80.0),
            (Key::PageDown, Modifiers::NONE, 20.0),
            (Key::Home, Modifiers::CTRL, 0.0),
            (Key::End, Modifiers::CTRL, 100.0),
            (Key::PageDown, Modifiers::CTRL, 0.0),
            (Key::PageUp, Modifiers::CTRL, 100.0),
        ] {
            let ctx = context(look::Look::Gtk, false);
            let mut value = 50.0;
            let (lo, hi) = if reversed { (100.0, 0.0) } else { (0.0, 100.0) };
            let draw = |ui: &mut egui::Ui, value: &mut f64| {
                ui.add(
                    look::NumberField::new(value)
                        .range(lo..=hi)
                        .step(3.0)
                        .speed(0.1)
                        .steppers(false),
                )
            };
            let response = frame(&ctx, vec![], |ui| draw(ui, &mut value));
            ctx.memory_mut(|m| m.request_focus(response.id));
            frame(&ctx, vec![], |ui| draw(ui, &mut value));
            let response = frame(&ctx, modified_key(key, modifiers), |ui| {
                draw(ui, &mut value)
            });
            assert_eq!(value, expected, "{key:?}, {modifiers:?}");
            assert!(response.changed());
            assert!(response.has_focus());
        }
    }
}

#[test]
fn gtk_number_field_shortcuts_do_not_restore_stale_edit_text() {
    let ctx = context(look::Look::Gtk, false);
    let mut value = 5.0;
    let draw = |ui: &mut egui::Ui, value: &mut f64| {
        ui.add(
            look::NumberField::new(value)
                .range(0.0..=100.0)
                .step(3.0)
                .speed(0.1)
                .steppers(false),
        )
    };
    let response = frame(&ctx, vec![], |ui| draw(ui, &mut value));
    ctx.memory_mut(|m| m.request_focus(response.id));
    frame(&ctx, vec![], |ui| draw(ui, &mut value));
    frame(
        &ctx,
        modified_key(
            egui::Key::A,
            egui::Modifiers {
                ctrl: true,
                command: true,
                ..egui::Modifiers::NONE
            },
        ),
        |ui| draw(ui, &mut value),
    );
    frame(&ctx, vec![egui::Event::Text("12".into())], |ui| {
        draw(ui, &mut value)
    });
    assert_eq!(value, 12.0);
    frame(&ctx, key(egui::Key::PageUp), |ui| draw(ui, &mut value));
    assert_eq!(value, 42.0);
    frame(&ctx, key(egui::Key::Enter), |ui| draw(ui, &mut value));
    assert_eq!(value, 42.0);
}

#[test]
fn gtk_number_field_keys_normalize_nan_and_leave_boundary_values_unchanged() {
    for (initial, key, expected, changed) in [
        (10.0, egui::Key::ArrowUp, 10.0, false),
        (0.0, egui::Key::ArrowDown, 0.0, false),
        (f64::NAN, egui::Key::ArrowUp, 3.0, true),
        (f64::NAN, egui::Key::ArrowDown, 0.0, true),
    ] {
        let ctx = context(look::Look::Gtk, false);
        let mut value = initial;
        let draw = |ui: &mut egui::Ui, value: &mut f64| {
            ui.add(
                look::NumberField::new(value)
                    .range(0.0..=10.0)
                    .step(3.0)
                    .speed(0.1)
                    .clamp_existing_to_range(false)
                    .steppers(false),
            )
        };
        let response = frame(&ctx, vec![], |ui| draw(ui, &mut value));
        ctx.memory_mut(|m| m.request_focus(response.id));
        frame(&ctx, vec![], |ui| draw(ui, &mut value));
        let response = frame(&ctx, modified_key(key, egui::Modifiers::NONE), |ui| {
            draw(ui, &mut value)
        });
        assert_eq!(value, expected);
        assert_eq!(response.changed(), changed);
    }
}

#[test]
fn numeric_accessibility_uses_the_step_and_ignores_changes_to_disabled_fields() {
    use egui::accesskit::{Action, ActionData, ActionRequest, TreeId};
    let ctx = context(look::Look::Gtk, false);
    ctx.enable_accesskit();
    let mut value = 50.0;
    let draw = |ui: &mut egui::Ui, value: &mut f64, enabled| {
        ui.add_enabled_ui(enabled, |ui| {
            ui.add(
                look::NumberField::new(value)
                    .range(0.0..=100.0)
                    .step(3.0)
                    .speed(0.1)
                    .clamp_existing_to_range(false)
                    .steppers(false),
            )
        })
        .inner
    };
    let response = frame(&ctx, vec![], |ui| draw(ui, &mut value, true));
    let request = |id: egui::Id, action, data| {
        egui::Event::AccessKitActionRequest(ActionRequest {
            target_tree: TreeId::ROOT,
            target_node: id.accesskit_id(),
            action,
            data,
        })
    };
    for (action, data, expected) in [
        (Action::Increment, None, 53.0),
        (
            Action::SetValue,
            Some(ActionData::NumericValue(200.0)),
            100.0,
        ),
        (Action::Decrement, None, 97.0),
    ] {
        let response = frame(&ctx, vec![request(response.id, action, data)], |ui| {
            draw(ui, &mut value, true)
        });
        assert_eq!(value, expected);
        assert!(response.changed());
    }
    let unchanged = frame(
        &ctx,
        vec![request(
            response.id,
            Action::SetValue,
            Some(ActionData::NumericValue(f64::NAN)),
        )],
        |ui| draw(ui, &mut value, true),
    );
    assert_eq!(value, 97.0);
    assert!(!unchanged.changed());
    let mut id = response.id;
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        id = draw(ui, &mut value, false).id;
    });
    output.textures_delta.clear();
    let update = output.platform_output.accesskit_update.unwrap();
    let node = &update
        .nodes
        .iter()
        .find(|(node_id, _)| *node_id == id.accesskit_id())
        .unwrap()
        .1;
    assert_eq!(node.numeric_value_step(), Some(3.0));
    assert_eq!(node.numeric_value(), Some(97.0));
    for (action, data) in [
        (Action::Increment, None),
        (Action::Decrement, None),
        (Action::SetValue, Some(ActionData::NumericValue(0.0))),
    ] {
        let response = frame(&ctx, vec![request(id, action, data)], |ui| {
            draw(ui, &mut value, false)
        });
        assert_eq!(value, 97.0);
        assert!(!response.changed());
    }
}

#[test]
fn hidden_controls_do_not_resume_a_previous_pointer_sequence() {
    for switch in [false, true] {
        for scenario in 0..3 {
            let ctx = context(look::Look::Gtk, false);
            let mut number = 50.0;
            let mut on = false;
            let draw = |ui: &mut egui::Ui, number: &mut f64, on: &mut bool, visible| {
                let target = ui
                    .push_id("target", |ui| {
                        if visible {
                            Some(if switch {
                                ui.add(look::Switch::new(on))
                            } else {
                                ui.add(look::Slider::new(number, 0.0..=100.0).width(200.0))
                            })
                        } else {
                            ui.allocate_space(egui::vec2(if switch { 42.0 } else { 200.0 }, 34.0));
                            None
                        }
                    })
                    .inner;
                let other = ui.add(look::Button::new("Other"));
                (target, other)
            };
            let (target, _) = frame(&ctx, vec![], |ui| draw(ui, &mut number, &mut on, true));
            let start =
                target.unwrap().rect.center() - egui::vec2(if switch { 10.0 } else { 0.0 }, 0.0);
            frame(&ctx, vec![egui::Event::PointerMoved(start)], |ui| {
                draw(ui, &mut number, &mut on, true)
            });
            frame(&ctx, vec![primary_button(start, true)], |ui| {
                draw(ui, &mut number, &mut on, true)
            });
            let end = start + egui::vec2(18.0, 0.0);
            frame(&ctx, vec![egui::Event::PointerMoved(end)], |ui| {
                draw(ui, &mut number, &mut on, true)
            });
            let previous = number;
            let (_, other) = frame(&ctx, vec![], |ui| draw(ui, &mut number, &mut on, false));
            if scenario != 2 {
                frame(&ctx, vec![primary_button(end, false)], |ui| {
                    draw(ui, &mut number, &mut on, false)
                });
            }
            let elsewhere = other.rect.center();
            if scenario == 1 {
                frame(&ctx, vec![egui::Event::PointerMoved(elsewhere)], |ui| {
                    draw(ui, &mut number, &mut on, false)
                });
                frame(&ctx, vec![primary_button(elsewhere, true)], |ui| {
                    draw(ui, &mut number, &mut on, false)
                });
            }
            frame(&ctx, vec![], |ui| draw(ui, &mut number, &mut on, true));
            let (target, _) = frame(&ctx, vec![primary_button(elsewhere, false)], |ui| {
                draw(ui, &mut number, &mut on, true)
            });
            assert_eq!(number, previous);
            assert!(!on);
            assert!(!target.unwrap().changed());
        }
    }
}

fn accessibility_request(
    target_node: egui::accesskit::NodeId,
    action: egui::accesskit::Action,
    data: Option<egui::accesskit::ActionData>,
) -> egui::Event {
    egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
        action,
        target_tree: egui::accesskit::TreeId::ROOT,
        target_node,
        data,
    })
}

#[test]
fn slider_accessibility_actions_follow_numeric_direction_and_bounds() {
    use egui::accesskit::{Action, ActionData};
    for chosen in LOOKS {
        for reversed in [false, true] {
            let ctx = context(chosen, false);
            let mut value = 50.0;
            let (start, end) = if reversed { (100.0, 0.0) } else { (0.0, 100.0) };
            let draw = |ui: &mut egui::Ui, value: &mut f64| {
                ui.add(look::Slider::new(value, start..=end).step_by(3.0))
            };
            let response = frame(&ctx, vec![], |ui| draw(ui, &mut value));
            for (action, data, expected, changed) in [
                (Action::Increment, None, 53.0, true),
                (Action::Decrement, None, 50.0, true),
                (
                    Action::SetValue,
                    Some(ActionData::NumericValue(200.0)),
                    100.0,
                    true,
                ),
                (Action::Increment, None, 100.0, false),
                (
                    Action::SetValue,
                    Some(ActionData::NumericValue(-200.0)),
                    0.0,
                    true,
                ),
                (Action::Decrement, None, 0.0, false),
                (
                    Action::SetValue,
                    Some(ActionData::NumericValue(f64::NAN)),
                    0.0,
                    false,
                ),
                (Action::SetValue, None, 0.0, false),
            ] {
                let response = frame(
                    &ctx,
                    vec![accessibility_request(
                        response.id.accesskit_id(),
                        action,
                        data,
                    )],
                    |ui| draw(ui, &mut value),
                );
                assert_eq!(value, expected, "{chosen}, reversed {reversed}, {action:?}");
                assert_eq!(response.changed(), changed);
                assert!(!response.has_focus());
            }
        }
    }
}

#[test]
fn logarithmic_slider_accessibility_steps_keep_the_numeric_direction() {
    use egui::accesskit::Action;
    let ctx = context(look::Look::Gtk, false);
    let mut value = 10.0;
    let draw = |ui: &mut egui::Ui, value: &mut f64| {
        ui.add(look::Slider::new(value, 1.0..=1000.0).logarithmic(true))
    };
    let response = frame(&ctx, vec![], |ui| draw(ui, &mut value));
    frame(
        &ctx,
        vec![accessibility_request(
            response.id.accesskit_id(),
            Action::Increment,
            None,
        )],
        |ui| draw(ui, &mut value),
    );
    assert!((value - 10.0 * 1000.0_f64.powf(0.01)).abs() < 1e-10);
    frame(
        &ctx,
        vec![accessibility_request(
            response.id.accesskit_id(),
            Action::Decrement,
            None,
        )],
        |ui| draw(ui, &mut value),
    );
    assert!((value - 10.0).abs() < 1e-10);
}

#[test]
fn slider_nodes_expose_numeric_bounds_steps_and_boundary_actions() {
    use egui::accesskit::Action;
    for chosen in LOOKS {
        let ctx = context(chosen, false);
        ctx.enable_accesskit();
        for initial in [0, 5, 10] {
            let mut value = initial;
            let mut id = egui::Id::NULL;
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                id = ui
                    .add(
                        look::Slider::new(&mut value, 0..=10)
                            .step_by(2.0)
                            .accessible_label("Zoom"),
                    )
                    .id;
            });
            output.textures_delta.clear();
            let update = output.platform_output.accesskit_update.unwrap();
            let node = &update
                .nodes
                .iter()
                .find(|(node_id, _)| *node_id == id.accesskit_id())
                .unwrap()
                .1;
            assert_eq!(node.role(), egui::Role::Slider);
            assert_eq!(node.label(), Some("Zoom"));
            assert_eq!(node.min_numeric_value(), Some(0.0));
            assert_eq!(node.max_numeric_value(), Some(10.0));
            assert_eq!(node.numeric_value_step(), Some(2.0));
            assert_eq!(node.numeric_value(), Some(f64::from(value)));
            assert!(node.supports_action(Action::SetValue));
            assert_eq!(node.supports_action(Action::Increment), initial < 10);
            assert_eq!(node.supports_action(Action::Decrement), initial > 0);
        }
    }
}

#[test]
fn slider_nodes_keep_unclamped_nonfinite_models_out_of_accessibility_data() {
    for initial in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 42.0] {
        let ctx = context(look::Look::Gtk, false);
        ctx.enable_accesskit();
        let mut value = initial;
        let mut id = egui::Id::NULL;
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let response = ui.add(
                look::Slider::new(&mut value, 0.0..=10.0).clamping(egui::SliderClamping::Never),
            );
            assert!(!response.changed());
            id = response.id;
        });
        output.textures_delta.clear();
        if initial.is_nan() {
            assert!(value.is_nan());
        } else {
            assert_eq!(value, initial);
        }
        let update = output.platform_output.accesskit_update.unwrap();
        let node = &update
            .nodes
            .iter()
            .find(|(node_id, _)| *node_id == id.accesskit_id())
            .unwrap()
            .1;
        let displayed = node.numeric_value().unwrap();
        assert!(displayed.is_finite());
        assert!((0.0..=10.0).contains(&displayed));
        let action = if displayed == 10.0 {
            egui::accesskit::Action::Decrement
        } else {
            egui::accesskit::Action::Increment
        };
        assert!(node.supports_action(action));
        frame(
            &ctx,
            vec![accessibility_request(id.accesskit_id(), action, None)],
            |ui| {
                ui.add(
                    look::Slider::new(&mut value, 0.0..=10.0).clamping(egui::SliderClamping::Never),
                )
            },
        );
        assert!((value - if displayed == 10.0 { 9.9 } else { 0.1 }).abs() < 1e-10);
    }
}

#[test]
fn disabled_sliders_and_their_number_editors_ignore_accessibility_edits() {
    use egui::accesskit::{Action, ActionData};
    for chosen in LOOKS {
        let ctx = context(chosen, false);
        ctx.enable_accesskit();
        let mut value = 42.0;
        let draw = |ui: &mut egui::Ui, value: &mut f64| {
            ui.add_enabled_ui(false, |ui| {
                ui.add(look::Slider::new(value, 0.0..=100.0).show_value(true))
            })
            .inner
        };
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            draw(ui, &mut value);
        });
        output.textures_delta.clear();
        let update = output.platform_output.accesskit_update.unwrap();
        let targets: Vec<_> = update
            .nodes
            .iter()
            .filter(|(_, node)| matches!(node.role(), egui::Role::Slider | egui::Role::SpinButton))
            .map(|(id, node)| {
                if node.role() == egui::Role::Slider {
                    assert!(!node.supports_action(Action::Increment));
                    assert!(!node.supports_action(Action::Decrement));
                    assert!(!node.supports_action(Action::SetValue));
                }
                *id
            })
            .collect();
        assert_eq!(targets.len(), 2);
        for target in targets {
            for (action, data) in [
                (Action::Increment, None),
                (Action::Decrement, None),
                (Action::SetValue, Some(ActionData::NumericValue(80.0))),
            ] {
                let response = frame(
                    &ctx,
                    vec![accessibility_request(target, action, data)],
                    |ui| draw(ui, &mut value),
                );
                assert_eq!(value, 42.0);
                assert!(!response.changed());
            }
        }
    }
}

#[test]
fn slider_accessibility_metadata_refreshes_after_its_number_field_changes() {
    use egui::accesskit::{Action, ActionData};
    let ctx = context(look::Look::Gtk, false);
    ctx.enable_accesskit();
    let mut value = 50.0;
    let draw = |ui: &mut egui::Ui, value: &mut f64| {
        ui.add(look::Slider::new(value, 0.0..=100.0).show_value(true))
    };
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        draw(ui, &mut value);
    });
    output.textures_delta.clear();
    let nodes = output.platform_output.accesskit_update.unwrap().nodes;
    let field = nodes
        .iter()
        .find(|(_, node)| node.role() == egui::Role::SpinButton)
        .unwrap()
        .0;
    let mut output = ctx.run_ui(
        egui::RawInput {
            events: vec![accessibility_request(
                field,
                Action::SetValue,
                Some(ActionData::NumericValue(100.0)),
            )],
            ..Default::default()
        },
        |ui| {
            draw(ui, &mut value);
        },
    );
    output.textures_delta.clear();
    assert_eq!(value, 100.0);
    let update = output.platform_output.accesskit_update.unwrap();
    let node = &update
        .nodes
        .iter()
        .find(|(_, node)| node.role() == egui::Role::Slider)
        .unwrap()
        .1;
    assert_eq!(node.numeric_value(), Some(100.0));
    assert!(!node.supports_action(Action::Increment));
    assert!(node.supports_action(Action::Decrement));
}

#[test]
fn logarithmic_slider_nodes_do_not_advertise_a_constant_numeric_step() {
    let ctx = context(look::Look::Gtk, false);
    ctx.enable_accesskit();
    let mut value = 2;
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        ui.add(look::Slider::new(&mut value, 1..=10).logarithmic(true));
    });
    output.textures_delta.clear();
    let update = output.platform_output.accesskit_update.unwrap();
    let (id, node) = update
        .nodes
        .iter()
        .find(|(_, node)| node.role() == egui::Role::Slider)
        .unwrap();
    assert_eq!(node.numeric_value_step(), None);
    frame(
        &ctx,
        vec![accessibility_request(
            *id,
            egui::accesskit::Action::Increment,
            None,
        )],
        |ui| ui.add(look::Slider::new(&mut value, 1..=10).logarithmic(true)),
    );
    assert_eq!(value, 3);
}

#[test]
fn invalid_and_constant_slider_ranges_reject_all_accessibility_edits() {
    use egui::accesskit::{Action, ActionData};
    for (start, end) in [(f64::NAN, 10.0), (0.0, f64::INFINITY), (7.0, 7.0)] {
        let ctx = context(look::Look::Gtk, false);
        ctx.enable_accesskit();
        let mut value = 42.0;
        let draw = |ui: &mut egui::Ui, value: &mut f64| {
            ui.add(
                look::Slider::new(value, start..=end)
                    .show_value(true)
                    .clamping(egui::SliderClamping::Never),
            )
        };
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            draw(ui, &mut value);
        });
        output.textures_delta.clear();
        let update = output.platform_output.accesskit_update.unwrap();
        for (target, node) in update
            .nodes
            .iter()
            .filter(|(_, node)| matches!(node.role(), egui::Role::Slider | egui::Role::SpinButton))
        {
            assert!(node.numeric_value().is_some_and(f64::is_finite));
            for (action, data) in [
                (Action::Increment, None),
                (Action::Decrement, None),
                (Action::SetValue, Some(ActionData::NumericValue(1.0))),
            ] {
                let response = frame(
                    &ctx,
                    vec![accessibility_request(*target, action, data)],
                    |ui| draw(ui, &mut value),
                );
                assert_eq!(value, 42.0);
                assert!(!response.changed());
            }
        }
    }
}

#[test]
fn select_all_focus_replaces_text_on_the_first_keyboard_input_frame() {
    for chosen in LOOKS {
        for salted in [false, true] {
            let ctx = context(chosen, false);
            let mut text = "Původní 🦀".to_string();
            let draw = |ui: &mut egui::Ui, text: &mut String| {
                let field = look::TextField::new(text)
                    .select_all_on_focus(true)
                    .width(200.0);
                ui.add(if salted { field.id_salt("text") } else { field })
            };
            frame(&ctx, vec![], |ui| draw(ui, &mut text));
            frame(&ctx, key(egui::Key::Tab), |ui| draw(ui, &mut text));
            let response = frame(&ctx, vec![egui::Event::Text("Žluťoučký".into())], |ui| {
                draw(ui, &mut text)
            });
            assert_eq!(text, "Žluťoučký", "{chosen}, salted {salted}");
            assert!(response.changed());
            frame(&ctx, vec![egui::Event::Text(" kůň".into())], |ui| {
                draw(ui, &mut text)
            });
            assert_eq!(text, "Žluťoučký kůň");
        }
    }
}

#[test]
fn ime_confirmation_does_not_submit_a_text_field() {
    for chosen in LOOKS {
        let ctx = context(chosen, false);
        let mut text = String::new();
        let draw = |ui: &mut egui::Ui, text: &mut String| {
            ui.add(look::TextField::new(text).id_salt("ime").width(200.0))
        };
        let response = frame(&ctx, vec![], |ui| draw(ui, &mut text));
        ctx.memory_mut(|m| m.request_focus(response.id));
        frame(&ctx, vec![], |ui| draw(ui, &mut text));
        frame(
            &ctx,
            vec![egui::Event::Ime(egui::ImeEvent::Preedit {
                text: "にほん".into(),
                active_range_chars: Some(0..3),
            })],
            |ui| draw(ui, &mut text),
        );
        let response = frame(
            &ctx,
            vec![
                egui::Event::Ime(egui::ImeEvent::Commit("日本".into())),
                key(egui::Key::Enter).remove(0),
            ],
            |ui| draw(ui, &mut text),
        );
        assert_eq!(text, "日本");
        assert!(
            response.has_focus(),
            "{chosen}: confirmation also submitted the field"
        );
    }
}

fn preedit(text: &str) -> egui::Event {
    egui::Event::Ime(egui::ImeEvent::Preedit {
        text: text.into(),
        active_range_chars: None,
    })
}

#[test]
fn select_all_happens_before_input_when_focus_is_requested_during_drawing() {
    for chosen in LOOKS {
        for ime in [false, true] {
            let ctx = context(chosen, false);
            let mut text = "Původní 🦀".to_string();
            let draw = |ui: &mut egui::Ui, text: &mut String| {
                ui.add(
                    look::TextField::new(text)
                        .id_salt("first-input")
                        .select_all_on_focus(true)
                        .width(200.0),
                )
            };
            let response = frame(&ctx, vec![], |ui| draw(ui, &mut text));
            let input = if ime {
                preedit("に")
            } else {
                egui::Event::Text("Ž".into())
            };
            frame(&ctx, vec![input], |ui| {
                ui.memory_mut(|memory| memory.request_focus(response.id));
                draw(ui, &mut text)
            });
            assert_eq!(text, if ime { "に" } else { "Ž" });
            if ime {
                frame(&ctx, vec![preedit("日本")], |ui| draw(ui, &mut text));
                assert_eq!(text, "日本");
                frame(
                    &ctx,
                    vec![egui::Event::Ime(egui::ImeEvent::Commit("日本語".into()))],
                    |ui| draw(ui, &mut text),
                );
                assert_eq!(text, "日本語");
            }
            frame(&ctx, vec![egui::Event::Text("!".into())], |ui| {
                draw(ui, &mut text)
            });
            assert_eq!(text, if ime { "日本語!" } else { "Ž!" });
        }
    }
}

#[test]
fn ime_enter_before_commit_keeps_focus_and_does_not_leak_a_submit_key() {
    let ctx = context(look::Look::Gtk, false);
    let mut text = String::new();
    let draw = |ui: &mut egui::Ui, text: &mut String| {
        ui.add(look::TextField::new(text).id_salt("ime-enter"))
    };
    let response = frame(&ctx, vec![], |ui| draw(ui, &mut text));
    ctx.memory_mut(|memory| memory.request_focus(response.id));
    frame(&ctx, vec![preedit("にほん")], |ui| draw(ui, &mut text));
    let response = frame(
        &ctx,
        vec![
            key(egui::Key::Enter).remove(0),
            egui::Event::Ime(egui::ImeEvent::Commit("日本".into())),
        ],
        |ui| {
            let response = draw(ui, &mut text);
            assert!(!ui.input(|input| input.key_pressed(egui::Key::Enter)));
            response
        },
    );
    assert_eq!(text, "日本");
    assert!(response.has_focus());
    let response = frame(&ctx, key(egui::Key::Enter), |ui| draw(ui, &mut text));
    assert!(!response.has_focus());
    assert_eq!(text, "日本");
}

#[test]
fn escape_interrupts_composition_before_leaving_the_text_field() {
    let ctx = context(look::Look::Gtk, false);
    let mut text = "prefix ".to_string();
    let draw = |ui: &mut egui::Ui, text: &mut String| {
        ui.add(look::TextField::new(text).id_salt("ime-escape"))
    };
    let response = frame(&ctx, vec![], |ui| draw(ui, &mut text));
    ctx.memory_mut(|memory| memory.request_focus(response.id));
    frame(&ctx, vec![preedit("日本")], |ui| draw(ui, &mut text));
    assert_eq!(text, "prefix 日本");
    let mut focused = false;
    let mut output = ctx.run_ui(
        egui::RawInput {
            events: key(egui::Key::Escape),
            ..Default::default()
        },
        |ui| {
            focused = draw(ui, &mut text).has_focus();
            assert!(!ui.input(|input| input.key_pressed(egui::Key::Escape)));
        },
    );
    output.textures_delta.clear();
    assert!(focused);
    assert!(
        output
            .platform_output
            .ime
            .unwrap()
            .should_interrupt_composition
    );
    frame(&ctx, vec![preedit("")], |ui| draw(ui, &mut text));
    assert_eq!(text, "prefix ");
    let response = frame(&ctx, key(egui::Key::Escape), |ui| draw(ui, &mut text));
    assert!(!response.has_focus());
    assert_eq!(text, "prefix ");
}

#[test]
fn committed_unicode_text_survives_focus_changes_and_normal_escape() {
    for chosen in LOOKS {
        let ctx = context(chosen, false);
        let mut texts = ["Žluťoučký 🦀".to_string(), String::new()];
        let draw = |ui: &mut egui::Ui, texts: &mut [String; 2]| {
            [
                ui.add(look::TextField::new(&mut texts[0]).id_salt("first")),
                ui.add(look::TextField::new(&mut texts[1]).id_salt("second")),
            ]
        };
        let fields = frame(&ctx, vec![], |ui| draw(ui, &mut texts));
        ctx.memory_mut(|memory| memory.request_focus(fields[0].id));
        frame(&ctx, vec![egui::Event::Text(" kůň".into())], |ui| {
            draw(ui, &mut texts)
        });
        frame(&ctx, key(egui::Key::Tab), |ui| draw(ui, &mut texts));
        frame(&ctx, vec![egui::Event::Text("東京".into())], |ui| {
            draw(ui, &mut texts)
        });
        let fields = frame(&ctx, key(egui::Key::Escape), |ui| draw(ui, &mut texts));
        assert_eq!(texts, ["Žluťoučký 🦀 kůň", "東京"]);
        assert!(!fields[1].has_focus());
    }
}

#[test]
fn unicode_selection_replaces_characters_instead_of_utf8_bytes() {
    let ctx = context(look::Look::Gtk, false);
    let mut text = "ž🦀東京".to_string();
    let draw = |ui: &mut egui::Ui, text: &mut String| {
        ui.add(look::TextField::new(text).id_salt("unicode-selection"))
    };
    let response = frame(&ctx, vec![], |ui| draw(ui, &mut text));
    ctx.memory_mut(|memory| memory.request_focus(response.id));
    frame(&ctx, vec![], |ui| draw(ui, &mut text));
    let mut state = egui::TextEdit::load_state(&ctx, response.id).unwrap();
    state
        .cursor
        .set_char_range(Some(egui::text::CCursorRange::two(
            egui::text::CCursor::new(1),
            egui::text::CCursor::new(3),
        )));
    state.store(&ctx, response.id);
    frame(&ctx, vec![egui::Event::Text("é".into())], |ui| {
        draw(ui, &mut text)
    });
    assert_eq!(text, "žé京");
}

#[test]
fn disabled_text_fields_ignore_accessibility_edits() {
    for chosen in LOOKS {
        let ctx = context(chosen, false);
        let mut text = "Keep me".to_string();
        let draw = |ui: &mut egui::Ui, text: &mut String| {
            ui.add_enabled_ui(false, |ui| {
                ui.add(look::TextField::new(text).id_salt("disabled-text"))
            })
            .inner
        };
        let response = frame(&ctx, vec![], |ui| draw(ui, &mut text));
        for action in [
            egui::accesskit::Action::SetValue,
            egui::accesskit::Action::ReplaceSelectedText,
        ] {
            let response = frame(
                &ctx,
                vec![accessibility_request(
                    response.id.accesskit_id(),
                    action,
                    Some(egui::accesskit::ActionData::Value("Changed".into())),
                )],
                |ui| draw(ui, &mut text),
            );
            assert_eq!(text, "Keep me");
            assert!(!response.changed());
        }
    }
}

#[test]
fn ime_state_follows_a_named_field_when_rows_reorder() {
    let ctx = context(look::Look::Gtk, false);
    let mut texts = [String::new(), "Other".to_string()];
    let draw = |ui: &mut egui::Ui, texts: &mut [String; 2], reverse| {
        let mut target = egui::Id::NULL;
        for index in if reverse { [1, 0] } else { [0, 1] } {
            let response =
                ui.add(look::TextField::new(&mut texts[index]).id_salt(("ime-row", index)));
            if index == 0 {
                target = response.id;
            }
        }
        target
    };
    let id = frame(&ctx, vec![], |ui| draw(ui, &mut texts, false));
    ctx.memory_mut(|memory| memory.request_focus(id));
    frame(&ctx, vec![preedit("にほん")], |ui| {
        draw(ui, &mut texts, false)
    });
    let reordered = frame(
        &ctx,
        vec![
            egui::Event::Ime(egui::ImeEvent::Commit("日本".into())),
            key(egui::Key::Enter).remove(0),
        ],
        |ui| draw(ui, &mut texts, true),
    );
    assert_eq!(reordered, id);
    assert_eq!(texts, ["日本", "Other"]);
    assert!(ctx.memory(|memory| memory.has_focus(id)));
}

#[test]
fn password_composition_keeps_password_ime_purpose_and_masked_accessibility() {
    let ctx = context(look::Look::Gtk, false);
    ctx.enable_accesskit();
    let mut text = String::new();
    let draw = |ui: &mut egui::Ui, text: &mut String| {
        ui.add(
            look::TextField::new(text)
                .id_salt("password-ime")
                .password(true),
        )
    };
    let response = frame(&ctx, vec![], |ui| draw(ui, &mut text));
    ctx.memory_mut(|memory| memory.request_focus(response.id));
    let secret = "秘密";
    let mut output = ctx.run_ui(
        egui::RawInput {
            events: vec![preedit(secret)],
            ..Default::default()
        },
        |ui| {
            draw(ui, &mut text);
        },
    );
    output.textures_delta.clear();
    assert_eq!(text, secret);
    assert_eq!(
        output.platform_output.ime.unwrap().purpose,
        egui::IMEPurpose::Password
    );
    for (_, node) in output.platform_output.accesskit_update.unwrap().nodes {
        assert!(!node.value().is_some_and(|value| value.contains(secret)));
    }
    let mut output = ctx.run_ui(
        egui::RawInput {
            events: vec![
                egui::Event::Ime(egui::ImeEvent::Commit(secret.into())),
                egui::Event::Copy,
            ],
            ..Default::default()
        },
        |ui| {
            draw(ui, &mut text);
        },
    );
    output.textures_delta.clear();
    assert_eq!(text, secret);
    for command in output.platform_output.commands {
        if let egui::OutputCommand::CopyText(copied) = command {
            assert!(copied.is_empty());
        }
    }
}

#[test]
fn native_window_focus_loss_clears_stale_composition_confirmation_state() {
    let ctx = context(look::Look::Gtk, false);
    let mut text = String::new();
    let draw = |ui: &mut egui::Ui, text: &mut String| {
        ui.add(look::TextField::new(text).id_salt("ime-window"))
    };
    let response = frame(&ctx, vec![], |ui| draw(ui, &mut text));
    ctx.memory_mut(|memory| memory.request_focus(response.id));
    frame(&ctx, vec![preedit("に")], |ui| draw(ui, &mut text));
    frame(&ctx, vec![egui::Event::WindowFocused(false)], |ui| {
        draw(ui, &mut text)
    });
    frame(
        &ctx,
        vec![egui::Event::WindowFocused(true), preedit("")],
        |ui| draw(ui, &mut text),
    );
    let response = frame(&ctx, key(egui::Key::Enter), |ui| draw(ui, &mut text));
    assert!(!response.has_focus());
}

#[test]
fn select_all_applies_to_focus_requests_made_between_frames() {
    for chosen in LOOKS {
        let ctx = context(chosen, false);
        let mut text = "Původní 🦀".to_string();
        let draw = |ui: &mut egui::Ui, text: &mut String| {
            ui.add(
                look::TextField::new(text)
                    .id_salt("programmatic-select")
                    .select_all_on_focus(true),
            )
        };
        let response = frame(&ctx, vec![], |ui| draw(ui, &mut text));
        ctx.memory_mut(|memory| memory.request_focus(response.id));
        frame(&ctx, vec![egui::Event::Text("Nový".into())], |ui| {
            draw(ui, &mut text)
        });
        assert_eq!(text, "Nový");
        frame(&ctx, vec![egui::Event::Text(" text".into())], |ui| {
            draw(ui, &mut text)
        });
        assert_eq!(text, "Nový text");
    }
}

#[test]
fn dropdown_typing_moves_focus_without_committing_and_resets_on_reopen() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut selected = 2;
        let draw = |ui: &mut egui::Ui, selected: &mut usize| {
            ui.add(look::Dropdown::new(
                "search",
                selected,
                &["Alpha", "Alpine", "Žába"],
            ))
        };
        frame(&ctx, vec![], |ui| draw(ui, &mut selected));
        frame(&ctx, key(egui::Key::Tab), |ui| draw(ui, &mut selected));
        frame(&ctx, key(egui::Key::Enter), |ui| draw(ui, &mut selected));
        frame(&ctx, vec![], |ui| draw(ui, &mut selected));
        for text in ["a", "a", "l"] {
            frame(&ctx, vec![egui::Event::Text(text.into())], |ui| {
                draw(ui, &mut selected)
            });
            assert_eq!(selected, 2);
        }
        frame(&ctx, key(egui::Key::Enter), |ui| draw(ui, &mut selected));
        assert_eq!(selected, 1, "{chosen}");
        frame(&ctx, vec![], |ui| draw(ui, &mut selected));
        frame(&ctx, key(egui::Key::Enter), |ui| draw(ui, &mut selected));
        frame(&ctx, vec![], |ui| draw(ui, &mut selected));
        frame(&ctx, vec![egui::Event::Text("Ž".into())], |ui| {
            draw(ui, &mut selected)
        });
        frame(&ctx, key(egui::Key::Escape), |ui| draw(ui, &mut selected));
        assert_eq!(selected, 1);
        assert!(!egui::Popup::is_any_open(&ctx));
    }
}

#[test]
fn closing_dropdown_outside_preserves_clicked_field_focus() {
    for chosen in LOOKS {
        for field_first in [false, true] {
            let ctx = context(chosen, true);
            let mut selected = 0;
            let mut text = String::new();
            let draw = |ui: &mut egui::Ui, selected: &mut usize, text: &mut String| {
                let field = |ui: &mut egui::Ui, text: &mut String| {
                    ui.add(look::TextField::new(text).id_salt("outside"))
                };
                let first = field_first.then(|| field(ui, text));
                let button = ui.add(look::Dropdown::new(
                    "outside-menu",
                    selected,
                    &["One", "Two"],
                ));
                let field = first.unwrap_or_else(|| field(ui, text));
                (button, field)
            };
            let (button, _) = frame(&ctx, vec![], |ui| draw(ui, &mut selected, &mut text));
            for events in click(button.rect.center()) {
                frame(&ctx, events, |ui| draw(ui, &mut selected, &mut text));
            }
            let (_, field) = frame(&ctx, vec![], |ui| draw(ui, &mut selected, &mut text));
            for events in click(field.rect.center()) {
                frame(&ctx, events, |ui| draw(ui, &mut selected, &mut text));
            }
            let (_, field) = frame(&ctx, vec![], |ui| draw(ui, &mut selected, &mut text));
            assert!(!egui::Popup::is_any_open(&ctx));
            assert!(field.has_focus(), "{chosen}, field_first={field_first}");
            assert_eq!(selected, 0);
        }
    }
}

#[test]
fn focused_text_field_stays_visible_when_keyboard_reduces_viewport() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut text = String::from("Unchanged");
        let mut draw = |height: f32, focus: bool, offset: Option<f32>| {
            let mut result = None;
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(390.0, height),
                    )),
                    ..Default::default()
                },
                |ui| {
                    egui::CentralPanel::default().show(ui, |ui| {
                        let mut scroll = egui::ScrollArea::vertical().id_salt("keyboard-resize");
                        if let Some(offset) = offset {
                            scroll = scroll.vertical_scroll_offset(offset);
                        }
                        let area = scroll.show(ui, |ui| {
                            ui.add_space(440.0);
                            let response =
                                ui.add(look::TextField::new(&mut text).id_salt("keyboard-field"));
                            if focus {
                                response.request_focus();
                            }
                            ui.add_space(200.0);
                            response
                        });
                        result = Some((
                            area.inner.rect,
                            area.inner.has_focus(),
                            area.state.clamped_offset().y,
                        ));
                    });
                },
            );
            output.textures_delta.clear();
            result.unwrap()
        };
        draw(700.0, true, None);
        for _ in 0..3 {
            draw(700.0, false, None);
        }
        for _ in 0..4 {
            draw(320.0, false, None);
        }
        let (rect, focused, _) = draw(320.0, false, None);
        assert!(focused, "{chosen}: resize lost focus");
        assert!(
            rect.top() >= 0.0 && rect.bottom() <= 320.0,
            "{chosen}: {rect:?}"
        );
        // A user may scroll away while keeping keyboard focus.
        draw(320.0, false, Some(0.0));
        let (rect, focused, offset) = draw(320.0, false, None);
        assert!(focused);
        assert_eq!(offset, 0.0);
        assert!(rect.top() > 320.0);
        for _ in 0..3 {
            draw(700.0, false, None);
        }
        let (rect, focused, _) = draw(700.0, false, None);
        assert!(focused);
        assert!(rect.top() >= 0.0 && rect.bottom() <= 700.0);
        assert_eq!(text, "Unchanged");
    }
}

#[test]
fn dialog_default_action_respects_focused_buttons_ime_and_disabled_state() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut open = true;
        let mut text = String::new();
        let mut field = egui::Id::NULL;
        let mut cancel = egui::Id::NULL;
        let mut accepted = 0;
        let mut cancelled = 0;
        let mut draw = |events, enabled| {
            frame(&ctx, events, |_ui| {
                look::Dialog::new("default", "Save").show(
                    &ctx,
                    &mut open,
                    |ui| {
                        field = ui.add(look::TextField::new(&mut text)).id;
                    },
                    |ui| {
                        accepted += usize::from(
                            ui.add_enabled(enabled, look::Button::new("Save").default_action())
                                .clicked(),
                        );
                        let response = ui.add(look::Button::new("Cancel"));
                        cancel = response.id;
                        cancelled += usize::from(response.clicked());
                    },
                );
            });
            (field, cancel, accepted, cancelled)
        };
        draw(vec![], true);
        let (field, cancel, _, _) = draw(vec![], true);
        ctx.memory_mut(|memory| memory.request_focus(field));
        assert_eq!(
            draw(key(egui::Key::Enter), true).2,
            1,
            "{chosen}: field Enter"
        );
        ctx.memory_mut(|memory| memory.request_focus(cancel));
        let (_, _, accepted, cancelled) = draw(key(egui::Key::Enter), true);
        assert_eq!(
            (accepted, cancelled),
            (1, 1),
            "{chosen}: focused button precedence"
        );
        ctx.memory_mut(|memory| memory.request_focus(field));
        draw(vec![preedit("東")], true);
        let mut confirmation = vec![egui::Event::Ime(egui::ImeEvent::Commit("東京".into()))];
        confirmation.extend(key(egui::Key::Enter));
        assert_eq!(draw(confirmation, true).2, 1, "{chosen}: IME submitted");
        let mut release = key(egui::Key::Enter);
        if let egui::Event::Key { pressed, .. } = &mut release[0] {
            *pressed = false;
        }
        draw(release, true);
        assert_eq!(draw(key(egui::Key::Enter), true).2, 2);
        ctx.memory_mut(|memory| memory.request_focus(field));
        assert_eq!(draw(key(egui::Key::Enter), false).2, 2);
    }
}

#[test]
fn dialog_restores_opener_focus_after_escape_and_application_close() {
    for chosen in LOOKS {
        for application_close in [false, true] {
            let ctx = context(chosen, true);
            let mut open = false;
            let draw = |ui: &mut egui::Ui, open: &mut bool| {
                let opener = ui.add(look::Button::new("Open"));
                if opener.clicked() {
                    *open = true;
                }
                look::Dialog::new("restore", "Settings").show(
                    &ctx,
                    open,
                    |_| {},
                    |ui| {
                        ui.add(look::Button::new("Done"));
                    },
                );
                opener
            };
            let opener = frame(&ctx, vec![], |ui| draw(ui, &mut open));
            ctx.memory_mut(|memory| memory.request_focus(opener.id));
            frame(&ctx, key(egui::Key::Enter), |ui| draw(ui, &mut open));
            assert!(open);
            for _ in 0..3 {
                frame(&ctx, vec![], |ui| draw(ui, &mut open));
            }
            frame(&ctx, key(egui::Key::Tab), |ui| draw(ui, &mut open));
            if application_close {
                open = false;
            }
            frame(
                &ctx,
                if application_close {
                    vec![]
                } else {
                    key(egui::Key::Escape)
                },
                |ui| draw(ui, &mut open),
            );
            assert!(!open);
            frame(&ctx, vec![], |ui| draw(ui, &mut open));
            assert_eq!(
                ctx.memory(|memory| memory.focused()),
                Some(opener.id),
                "{chosen}, application_close={application_close}"
            );
        }
    }
}

#[test]
fn dialog_escape_can_be_disabled_and_ui_close_still_restores_focus() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut open = false;
        let mut close = false;
        let draw = |ui: &mut egui::Ui, open: &mut bool, close: bool| {
            let opener = ui.add(look::Button::new("Open"));
            look::Dialog::new("locked", "Confirm")
                .close_on_escape(false)
                .show(
                    &ctx,
                    open,
                    |_| {},
                    |ui| {
                        if close {
                            ui.close();
                        }
                    },
                );
            opener
        };
        let opener = frame(&ctx, vec![], |ui| draw(ui, &mut open, close));
        ctx.memory_mut(|memory| memory.request_focus(opener.id));
        open = true;
        frame(&ctx, vec![], |ui| draw(ui, &mut open, close));
        frame(&ctx, key(egui::Key::Escape), |ui| {
            draw(ui, &mut open, close)
        });
        assert!(open, "{chosen}: Escape ignored close_on_escape(false)");
        close = true;
        frame(&ctx, vec![], |ui| draw(ui, &mut open, close));
        assert!(!open);
        frame(&ctx, vec![], |ui| draw(ui, &mut open, false));
        assert_eq!(ctx.memory(|memory| memory.focused()), Some(opener.id));
    }
}

#[test]
fn dialog_tab_navigation_stays_inside_in_both_directions() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut open = true;
        let mut background = egui::Id::NULL;
        let mut inside = Vec::new();
        let mut text = String::new();
        let mut draw = |ui: &mut egui::Ui| {
            background = ui.add(look::Button::new("Behind")).id;
            inside.clear();
            let mut field_id = egui::Id::NULL;
            let mut action_id = egui::Id::NULL;
            look::Dialog::new("tab-cycle", "Settings").show(
                &ctx,
                &mut open,
                |ui| {
                    field_id = ui.add(look::TextField::new(&mut text)).id;
                },
                |ui| {
                    action_id = ui.add(look::Button::new("OK")).id;
                },
            );
            inside.extend([field_id, action_id]);
            (background, inside.clone())
        };
        for _ in 0..3 {
            frame(&ctx, vec![], &mut draw);
        }
        for modifiers in [egui::Modifiers::NONE, egui::Modifiers::SHIFT] {
            let mut reached = std::collections::HashSet::new();
            for _ in 0..12 {
                let (background, _) =
                    frame(&ctx, modified_key(egui::Key::Tab, modifiers), &mut draw);
                frame(&ctx, vec![], &mut draw);
                let focus = ctx.memory(|memory| memory.focused());
                assert_ne!(focus, Some(background), "{chosen}: focus escaped");
                if let Some(focus) = focus {
                    reached.insert(focus);
                }
            }
            let (_, inside) = frame(&ctx, vec![], &mut draw);
            assert!(
                inside.iter().all(|id| reached.contains(id)),
                "{chosen}: unreachable control with {modifiers:?}"
            );
        }
    }
}

#[test]
fn dialog_actions_scroll_into_view_with_phone_keyboard_open() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut open = true;
        let mut action = None;
        for pass in 0..25 {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(320.0, 240.0),
                    )),
                    events: if pass > 3 {
                        key(egui::Key::Tab)
                    } else {
                        vec![]
                    },
                    ..Default::default()
                },
                |_ui| {
                    look::Dialog::new("phone-focus", "Settings").show(
                        &ctx,
                        &mut open,
                        |ui| {
                            for _ in 0..25 {
                                ui.label("Scrollable settings");
                            }
                        },
                        |ui| {
                            action = Some(ui.add(look::Button::new("Save")));
                        },
                    );
                },
            );
            output.textures_delta.clear();
            let response = action.as_ref().unwrap();
            if response.has_focus() && response.rect.top() >= 0.0 && response.rect.bottom() <= 240.0
            {
                break;
            }
            assert!(
                pass < 24,
                "{chosen}: focused action stayed outside viewport: {:?}",
                response.rect
            );
        }
    }
}

#[test]
fn dialog_default_does_not_override_standard_egui_controls_or_opening_enter() {
    for chosen in LOOKS {
        for control_first in [false, true] {
            let ctx = context(chosen, true);
            let mut open = false;
            let mut accepted = 0;
            let mut other = 0;
            let mut other_id = egui::Id::NULL;
            let mut draw = |events| {
                frame(&ctx, events, |ui| {
                    let opener = ui.add(look::Button::new("Open"));
                    if opener.clicked() {
                        open = true;
                    }
                    look::Dialog::new("mixed-buttons", "Save").show(
                        &ctx,
                        &mut open,
                        |_| {},
                        |ui| {
                            if control_first {
                                let response = ui.button("Other");
                                other_id = response.id;
                                other += usize::from(response.clicked());
                            }
                            accepted += usize::from(
                                ui.add(look::Button::new("Save").default_action()).clicked(),
                            );
                            if !control_first {
                                let response = ui.button("Other");
                                other_id = response.id;
                                other += usize::from(response.clicked());
                            }
                        },
                    );
                    (opener.id, accepted, other, other_id)
                })
            };
            let opener = draw(vec![]).0;
            ctx.memory_mut(|memory| memory.request_focus(opener));
            assert_eq!(draw(key(egui::Key::Enter)).1, 0, "opening Enter submitted");
            draw(vec![]);
            let other_id = draw(vec![]).3;
            ctx.memory_mut(|memory| memory.request_focus(other_id));
            let (_, accepted, other, _) = draw(key(egui::Key::Enter));
            assert_eq!(
                (accepted, other),
                (0, 1),
                "{chosen}: control_first={control_first}"
            );
        }
    }
}

#[test]
fn nested_dialog_escape_restores_parent_focus_and_respects_locked_child() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut parent = true;
        let mut child = false;
        let mut locked = true;
        let mut child_opener = egui::Id::NULL;
        let draw = |parent: &mut bool, child: &mut bool, locked: bool, opener: &mut egui::Id| {
            frame(&ctx, vec![], |_ui| {
                look::Dialog::new("parent", "Parent").show(
                    &ctx,
                    parent,
                    |_| {},
                    |ui| {
                        *opener = ui.add(look::Button::new("Open child")).id;
                    },
                );
                look::Dialog::new("child", "Child")
                    .close_on_escape(!locked)
                    .show(&ctx, child, |_| {}, |_| {});
            });
        };
        for _ in 0..3 {
            draw(&mut parent, &mut child, locked, &mut child_opener);
        }
        ctx.memory_mut(|memory| memory.request_focus(child_opener));
        child = true;
        for _ in 0..3 {
            draw(&mut parent, &mut child, locked, &mut child_opener);
        }
        let run_escape = |parent: &mut bool, child: &mut bool, locked: bool| {
            frame(&ctx, key(egui::Key::Escape), |_ui| {
                look::Dialog::new("parent", "Parent").show(
                    &ctx,
                    parent,
                    |_| {},
                    |ui| {
                        ui.add(look::Button::new("Open child"));
                    },
                );
                look::Dialog::new("child", "Child")
                    .close_on_escape(!locked)
                    .show(&ctx, child, |_| {}, |_| {});
            });
        };
        run_escape(&mut parent, &mut child, locked);
        assert!(
            parent && child,
            "{chosen}: Escape reached parent through locked child"
        );
        locked = false;
        run_escape(&mut parent, &mut child, locked);
        assert!(parent && !child);
        draw(&mut parent, &mut child, locked, &mut child_opener);
        assert_eq!(ctx.memory(|memory| memory.focused()), Some(child_opener));
        run_escape(&mut parent, &mut child, locked);
        assert!(
            !parent,
            "{chosen}: stale closed child blocked parent Escape"
        );
    }
}

#[test]
fn dialog_popup_uses_enter_and_escape_before_default_action_or_dialog() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut open = true;
        let mut selected = 0;
        let mut accepted = 0;
        let mut dropdown_id = egui::Id::NULL;
        let mut draw = |events| {
            frame(&ctx, events, |_ui| {
                look::Dialog::new("popup-dialog", "Settings").show(
                    &ctx,
                    &mut open,
                    |ui| {
                        dropdown_id = ui
                            .add(look::Dropdown::new(
                                "options",
                                &mut selected,
                                &["One", "Two"],
                            ))
                            .id;
                    },
                    |ui| {
                        accepted += usize::from(
                            ui.add(look::Button::new("Save").default_action()).clicked(),
                        );
                    },
                );
            });
            (open, selected, accepted, dropdown_id)
        };
        draw(vec![]);
        let dropdown = draw(vec![]).3;
        ctx.memory_mut(|memory| memory.request_focus(dropdown));
        draw(key(egui::Key::Enter));
        draw(vec![]);
        assert!(egui::Popup::is_any_open(&ctx));
        draw(key(egui::Key::ArrowDown));
        assert_eq!(draw(key(egui::Key::Enter)).2, 0);
        assert!(!egui::Popup::is_any_open(&ctx));
        draw(key(egui::Key::Enter));
        draw(vec![]);
        assert!(egui::Popup::is_any_open(&ctx));
        let (open, selected, accepted, _) = draw(key(egui::Key::Escape));
        assert!(open);
        assert_eq!((selected, accepted), (1, 0));
        assert!(!egui::Popup::is_any_open(&ctx));
    }
}

#[test]
fn dialog_escape_leaves_text_editing_before_closing() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut open = true;
        let mut text = String::new();
        let mut field = egui::Id::NULL;
        let mut draw = |events| {
            frame(&ctx, events, |_ui| {
                look::Dialog::new("editing-escape", "Edit").show(
                    &ctx,
                    &mut open,
                    |ui| {
                        field = ui.add(look::TextField::new(&mut text)).id;
                    },
                    |_| {},
                );
            });
            (open, field)
        };
        draw(vec![]);
        let field = draw(vec![]).1;
        ctx.memory_mut(|memory| memory.request_focus(field));
        draw(vec![]);
        assert!(
            draw(key(egui::Key::Escape)).0,
            "{chosen}: first Escape closed editor"
        );
        assert_ne!(ctx.memory(|memory| memory.focused()), Some(field));
        assert!(
            !draw(key(egui::Key::Escape)).0,
            "{chosen}: second Escape did not close dialog"
        );
    }
}

#[test]
fn dialog_does_not_restore_focus_to_disabled_opener() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut open = false;
        let draw = |ui: &mut egui::Ui, open: &mut bool, enabled| {
            let opener = ui.add_enabled(enabled, look::Button::new("Open"));
            look::Dialog::new("disabled-opener", "Settings").show(&ctx, open, |_| {}, |_| {});
            opener.id
        };
        let opener = frame(&ctx, vec![], |ui| draw(ui, &mut open, true));
        ctx.memory_mut(|memory| memory.request_focus(opener));
        open = true;
        frame(&ctx, vec![], |ui| draw(ui, &mut open, true));
        frame(&ctx, vec![], |ui| draw(ui, &mut open, false));
        open = false;
        frame(&ctx, vec![], |ui| draw(ui, &mut open, false));
        assert_ne!(ctx.memory(|memory| memory.focused()), Some(opener));
    }
}

#[test]
fn dialog_focus_return_is_independent_of_background_draw_order() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut open = false;
        let draw = |ui: &mut egui::Ui, open: &mut bool| {
            look::Dialog::new("draw-order", "Settings").show(&ctx, open, |_| {}, |_| {});
            ui.add(look::Button::new("Open")).id
        };
        let opener = frame(&ctx, vec![], |ui| draw(ui, &mut open));
        ctx.memory_mut(|memory| memory.request_focus(opener));
        open = true;
        for _ in 0..3 {
            frame(&ctx, vec![], |ui| draw(ui, &mut open));
        }
        open = false;
        frame(&ctx, vec![], |ui| draw(ui, &mut open));
        assert_eq!(
            ctx.memory(|memory| memory.focused()),
            Some(opener),
            "{chosen}: focus lost after later background draw"
        );
        frame(&ctx, vec![], |ui| draw(ui, &mut open));
        assert_eq!(ctx.memory(|memory| memory.focused()), Some(opener));
    }
}
