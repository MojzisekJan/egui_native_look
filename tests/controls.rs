//! Every control draws and takes a click in every look and scheme.

use egui_native_look as look;

fn frame<R>(ctx: &egui::Context, events: Vec<egui::Event>, mut add: impl FnMut(&mut egui::Ui) -> R) -> R {
    let mut result = None;
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(340.0, 600.0))),
        events,
        ..Default::default()
    };
    let mut output = ctx.run_ui(input, |ui| result = Some(add(ui)));
    // Nothing renders the font atlas here; egui asks that its updates are taken.
    output.textures_delta.clear();
    result.unwrap()
}

fn click(at: egui::Pos2) -> Vec<Vec<egui::Event>> {
    let press = |pressed| egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
    vec![vec![egui::Event::PointerMoved(at)], vec![press(true)], vec![press(false)]]
}

/// A context with `chosen` installed, in the dark scheme or the light one.
fn context(chosen: look::Look, dark: bool) -> egui::Context {
    let ctx = egui::Context::default();
    look::install(&ctx, look::Theme::new(chosen));
    ctx.set_theme(if dark { egui::Theme::Dark } else { egui::Theme::Light });
    ctx
}

/// The open pop-up's rect: the topmost foreground area.
fn popup_rect(ctx: &egui::Context) -> Option<egui::Rect> {
    let layer = ctx.memory(|m| m.layer_ids().filter(|l| l.order == egui::Order::Foreground).last())?;
    ctx.memory(|m| m.area_rect(layer.id))
}

fn key(key: egui::Key) -> Vec<egui::Event> {
    vec![egui::Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers: Default::default() }]
}

const LOOKS: [look::Look; 6] = [look::Look::Plain, look::Look::Mac, look::Look::Windows, look::Look::Gtk, look::Look::Ios, look::Look::Material];

#[test]
fn controls_take_clicks_in_every_look() {
    for chosen in LOOKS {
        for dark in [true, false] {
            let ctx = context(chosen, dark);
            let mut checked = false;
            for events in click(egui::pos2(6.0, 10.0)) {
                frame(&ctx, events, |ui| ui.add(look::Checkbox::new(&mut checked, "Snap")));
            }
            assert!(checked, "{chosen}: the check box did not take the click");
            let mut chosen_value = 0;
            for events in click(egui::pos2(6.0, 10.0)) {
                frame(&ctx, events, |ui| look::radio_value(ui, &mut chosen_value, 1, "Rotate"));
            }
            assert_eq!(chosen_value, 1, "{chosen}: the radio button did not take the click");
            let mut on = false;
            for events in click(egui::pos2(15.0, 10.0)) {
                frame(&ctx, events, |ui| ui.add(look::Switch::new(&mut on)));
            }
            assert!(on, "{chosen}: the switch did not take the click");
            let mut segment = 0;
            for events in click(egui::pos2(250.0, 10.0)) {
                frame(&ctx, events, |ui| ui.add(look::Segmented::new("axis", &mut segment, &["X", "Y", "Z"]).width(300.0)));
            }
            assert_eq!(segment, 2, "{chosen}: the segmented choice did not take the click");
            let mut pressed = false;
            for events in click(egui::pos2(20.0, 10.0)) {
                pressed |= frame(&ctx, events, |ui| ui.add(look::Button::new("Apply")).clicked());
            }
            assert!(pressed, "{chosen}: the button did not take the click");
            let mut value = 50.0;
            for events in click(egui::pos2(190.0, 10.0)) {
                frame(&ctx, events, |ui| ui.add(look::Slider::new(&mut value, 0.0..=100.0).width(200.0)));
            }
            assert!(value > 90.0, "{chosen}: the slider did not follow the click ({value})");
        }
    }
}

#[test]
fn a_dropdown_picks_from_its_list() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut shape = 0;
        let add = |ui: &mut egui::Ui, shape: &mut usize| ui.add(look::Dropdown::new("shape", shape, &["Cuboid", "Cylinder", "Sphere"]).width(160.0));
        for events in click(egui::pos2(40.0, 10.0)) {
            frame(&ctx, events, |ui| add(ui, &mut shape));
        }
        // The list opens under the button; its third row.
        let row = (look::theme(&ctx).metrics.control_height - 4.0).max(20.0);
        let third = egui::pos2(40.0, look::theme(&ctx).metrics.control_height + 8.0 + row * 2.5);
        for events in click(third) {
            frame(&ctx, events, |ui| add(ui, &mut shape));
        }
        assert_eq!(shape, 2, "{chosen}: the dropdown did not pick from its list");
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
    assert!(!look::colors(&gtk).dark && look::colors(&mac).dark, "the scheme follows each context's egui theme");
}

#[test]
fn looks_have_names() {
    for (name, chosen, label) in [("plain", look::Look::Plain, "Plain"), ("mac", look::Look::Mac, "macOS"), ("windows", look::Look::Windows, "Windows"), ("gtk", look::Look::Gtk, "GTK"), ("ios", look::Look::Ios, "iOS"), ("material", look::Look::Material, "Material")] {
        assert_eq!(look::Look::from_name(name), Some(chosen));
        assert_eq!(chosen.label(), label);
    }
    assert_eq!(look::Look::from_name("motif"), None);
}

#[test]
fn shortcuts_read_as_each_system_writes_them() {
    let save_as = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND | egui::Modifiers::SHIFT, egui::Key::S);
    assert_eq!(look::shortcut_text(look::Look::Mac, &save_as), "⇧⌘S");
    assert_eq!(look::shortcut_text(look::Look::Windows, &save_as), "Ctrl+Shift+S");
}

#[test]
fn an_icon_button_and_a_colour_well_take_clicks() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut pressed = false;
        for events in click(egui::pos2(10.0, 10.0)) {
            pressed |= frame(&ctx, events, |ui| ui.add(look::IconButton::new("#").tooltip("Grid")).clicked());
        }
        assert!(pressed, "{chosen}: the icon button did not take the click");
        let mut tint = egui::Color32::RED;
        let mut opened = false;
        for events in click(egui::pos2(12.0, 10.0)) {
            frame(&ctx, events, |ui| ui.add(look::ColorWell::new(&mut tint)));
            opened |= egui::Popup::is_any_open(&ctx);
        }
        frame(&ctx, vec![], |ui| ui.add(look::ColorWell::new(&mut tint)));
        assert!(opened || egui::Popup::is_any_open(&ctx), "{chosen}: the colour well did not open its picker");
    }
}

#[test]
fn a_form_lines_its_controls_up() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut rects = Vec::new();
        for _ in 0..3 {
            rects = frame(&ctx, vec![], |ui| {
                look::Form::new("form")
                    .show(ui, |form| {
                        let a = form.row("Radius", |ui| ui.add(look::Button::new("A").width(60.0))).rect;
                        let b = form.row("A much longer label", |ui| ui.add(look::Button::new("B").width(60.0))).rect;
                        vec![a, b]
                    })
            });
        }
        assert!((rects[0].left() - rects[1].left()).abs() < 0.5, "{chosen}: the controls start at different places: {rects:?}");
        assert!(rects[1].top() > rects[0].bottom(), "{chosen}: the rows overlap");
    }
}

#[test]
fn a_toast_goes_after_a_while() {
    let ctx = context(look::Look::Gtk, true);
    let run = |time: f64| {
        let input = egui::RawInput { time: Some(time), ..Default::default() };
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
fn a_popup_button_shows_the_apps_own_list() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut units = 0;
        let add = |ui: &mut egui::Ui, units: &mut usize| {
            look::PopupButton::new("units", "Metres").width(160.0).show(ui, |ui| {
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
        frame(&ctx, events, |ui| ui.add(look::Slider::new(&mut samples, 0..=64).width(200.0)));
    }
    assert!((30..=34).contains(&samples), "the middle of 0 to 64 is not {samples}");
    let mut exposure: f64 = 1.0;
    for events in click(egui::pos2(100.0, 10.0)) {
        frame(&ctx, events, |ui| ui.add(look::Slider::new(&mut exposure, 0.01..=100.0).logarithmic(true).width(200.0)));
    }
    assert!((0.8..=1.25).contains(&exposure), "the middle of 0.01 to 100 on a log scale is not {exposure}");
    let mut stepped: f64 = 0.0;
    for events in click(egui::pos2(97.0, 10.0)) {
        frame(&ctx, events, |ui| ui.add(look::Slider::new(&mut stepped, 0.0..=100.0).step_by(25.0).width(200.0)));
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
        let picked = rects.iter().position(|r| r.contains(egui::pos2(110.0, 12.0)));
        assert_eq!(Some(shown), picked, "{chosen}: the click did not pick the tab under it: {rects:?}");
    }
}

#[test]
fn a_menu_item_is_as_wide_as_it_needs() {
    let ctx = context(look::Look::Mac, true);
    let width = frame(&ctx, vec![], |ui| ui.add(look::MenuItem::new("Save").shortcut("⌘S")).rect.width());
    assert!(width < 140.0, "a short item is {width} wide");
}

#[test]
fn a_section_answers_and_opens_from_code() {
    let ctx = context(look::Look::Gtk, true);
    let shown = frame(&ctx, vec![], |ui| look::Section::new("Layers").open(Some(true)).show(ui, |ui| ui.label("body")));
    assert!(shown.body_returned.is_some() && shown.header_response.rect.width() > 0.0);
    // Closed from code, it goes on closing over the next frames.
    for _ in 0..20 {
        frame(&ctx, vec![], |ui| look::Section::new("Layers").open(Some(false)).show(ui, |ui| ui.label("body")));
    }
    let shown = frame(&ctx, vec![], |ui| look::Section::new("Layers").show(ui, |ui| ui.label("body")));
    assert!(shown.body_returned.is_none() && shown.openness == 0.0);
}

#[test]
fn a_text_field_can_select_all_on_focus() {
    let ctx = context(look::Look::Windows, true);
    let mut text = String::from("12.5");
    let mut id = egui::Id::NULL;
    for events in click(egui::pos2(40.0, 12.0)) {
        id = frame(&ctx, events, |ui| ui.add(look::TextField::new(&mut text).select_all_on_focus(true).width(200.0)).id);
    }
    let range = egui::TextEdit::load_state(&ctx, id).and_then(|s| s.cursor.char_range()).expect("a cursor");
    assert_eq!((range.primary.index.min(range.secondary.index), range.primary.index.max(range.secondary.index)), (0.into(), 4.into()));
    // Without it, a click puts the caret down and selects nothing.
    let ctx = context(look::Look::Windows, true);
    for events in click(egui::pos2(40.0, 12.0)) {
        id = frame(&ctx, events, |ui| ui.add(look::TextField::new(&mut text).width(200.0)).id);
    }
    let range = egui::TextEdit::load_state(&ctx, id).and_then(|s| s.cursor.char_range()).expect("a cursor");
    assert_eq!(range.primary.index, range.secondary.index);
}

#[test]
fn metrics_scale_and_a_touch_look_is_finger_sized() {
    let metrics = look::Metrics::of(look::Look::Mac).scaled(2.0);
    assert_eq!((metrics.control_height, metrics.radius, metrics.body.size), (44.0, 12, 26.0));
    assert!(look::Metrics::of(look::Look::touch()).control_height >= 40.0);
}

#[test]
fn a_tool_window_shows_its_contents() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut open = true;
        let shown = frame(&ctx, vec![], |_ui| look::ToolWindow::new("tool", "Fillet").show(&ctx, &mut open, |ui| ui.label("Radius")));
        assert!(shown.is_some_and(|s| s.inner.is_some()), "{chosen}: nothing shown");
    }
}

#[test]
fn a_number_field_without_steppers_is_the_field_alone_and_need_not_have_a_range() {
    let ctx = context(look::Look::Mac, true);
    let mut value = 5000.0_f64;
    let with = frame(&ctx, vec![], |ui| ui.add(look::NumberField::new(&mut value).width(140.0)).rect.width());
    let without = frame(&ctx, vec![], |ui| ui.add(look::NumberField::new(&mut value).steppers(false).width(140.0)).rect.width());
    assert!((with - 140.0).abs() < 1.0 && (without - 140.0).abs() < 1.0, "{with} {without}");
    let alone = frame(&ctx, vec![], |ui| ui.add(look::NumberField::new(&mut value).steppers(false)).rect.width());
    assert!(alone < 70.0, "{alone}");
    // No range: a value far beyond any default stays.
    let mut far = 1.0e9_f64;
    frame(&ctx, vec![], |ui| ui.add(look::NumberField::new(&mut far)));
    assert_eq!(far, 1.0e9);
    let mut bounded = 200_i32;
    frame(&ctx, vec![], |ui| ui.add(look::NumberField::new(&mut bounded).range(0..=100)));
    assert_eq!(bounded, 100);
}

#[test]
fn a_sliders_number_takes_a_suffix_and_a_formatter() {
    let ctx = context(look::Look::Gtk, false);
    let mut angle = 90.0_f32;
    let plain = frame(&ctx, vec![], |ui| ui.add(look::Slider::new(&mut angle, 0.0..=360.0).show_value(true)).rect.width());
    let long = frame(&ctx, vec![], |ui| ui.add(look::Slider::new(&mut angle, 0.0..=360.0).show_value(true).suffix(" degrees of turn")).rect.width());
    assert!(long > plain + 20.0, "the suffix should widen the number: {plain} {long}");
    let mut scale = 1.5_f64;
    frame(&ctx, vec![], |ui| ui.add(look::Slider::new(&mut scale, 0.5..=4.0).show_value(true).custom_formatter(|v, _| format!("{v:.1}×")).fixed_decimals(1)));
}

#[test]
fn a_submenu_item_keeps_its_icon() {
    let ctx = context(look::Look::Windows, true);
    let font = egui::FontId::proportional(14.0);
    let (with_icon, plain) = frame(&ctx, vec![], |ui| {
        let a = look::MenuItem::new("Export").icon("E", font.clone()).submenu(ui, |ui| ui.add(look::MenuItem::new("Godot"))).response;
        let b = look::submenu(ui, "Export", |ui| ui.add(look::MenuItem::new("Godot"))).response;
        (a.rect.width(), b.rect.width())
    });
    assert_eq!(with_icon, plain, "the icon sits in the column the label already leaves");
}

#[test]
fn a_palette_changes_a_few_colours_and_install_keeps_the_apps_style() {
    let dark = look::Palette { bg: egui::Color32::from_rgb(10, 20, 30), surface: egui::Color32::from_rgb(40, 50, 60), ..look::Palette::of(true) };
    assert_eq!(dark.fg, look::Palette::of(true).fg);
    let ctx = egui::Context::default();
    let shadow = egui::Shadow { offset: [0, 2], blur: 3, spread: 0, color: egui::Color32::RED };
    ctx.all_styles_mut(|style| {
        style.visuals.window_shadow = shadow;
        style.spacing.item_spacing = egui::vec2(11.0, 7.0);
        style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::proportional(31.0));
    });
    look::install(&ctx, look::Theme::new(look::Look::Plain).schemes(look::Schemes { dark, light: look::Palette::of(false) }));
    let style = ctx.style_of(egui::Theme::Dark);
    assert_eq!(style.visuals.window_shadow, shadow);
    assert_eq!(style.spacing.item_spacing, egui::vec2(11.0, 7.0));
    assert_eq!(style.text_styles[&egui::TextStyle::Heading].size, 31.0);
    assert_eq!(style.visuals.panel_fill, egui::Color32::from_rgb(10, 20, 30));
    assert_eq!(style.visuals.window_fill, egui::Color32::from_rgb(40, 50, 60));
    assert!(!ctx.style_of(egui::Theme::Light).visuals.dark_mode);
}

#[test]
fn pop_up_buttons_of_the_same_id_in_a_loop_open_one_list() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let rows = |ui: &mut egui::Ui| {
            (0..3)
                .map(|i| ui.push_id(i, |ui| look::PopupButton::new("units", "Metres").width(160.0).show(ui, |ui| ui.label("Feet")).inner.is_some()).inner)
                .collect::<Vec<bool>>()
        };
        for events in click(egui::pos2(40.0, 10.0)) {
            frame(&ctx, events, rows);
        }
        assert_eq!(frame(&ctx, vec![], rows), [true, false, false], "{chosen}: one click, one list");
    }
}

#[test]
fn the_arrow_keys_move_a_slider_and_a_segmented_choice_more_than_once() {
    for chosen in LOOKS {
        let ctx = context(chosen, true);
        let mut value = 0.0_f64;
        let add = |ui: &mut egui::Ui, value: &mut f64| ui.add(look::Slider::new(value, 0.0..=10.0).step_by(1.0).width(200.0));
        frame(&ctx, vec![], |ui| add(ui, &mut value));
        frame(&ctx, key(egui::Key::Tab), |ui| add(ui, &mut value));
        frame(&ctx, vec![], |ui| add(ui, &mut value));
        for _ in 0..3 {
            frame(&ctx, key(egui::Key::ArrowRight), |ui| add(ui, &mut value));
        }
        assert_eq!(value, 3.0, "{chosen}: the slider lost the focus");
        let mut axis = 0;
        let add = |ui: &mut egui::Ui, axis: &mut usize| ui.add(look::Segmented::new("axis", axis, &["X", "Y", "Z"]).width(240.0));
        frame(&ctx, vec![], |ui| add(ui, &mut axis));
        frame(&ctx, key(egui::Key::Tab), |ui| add(ui, &mut axis));
        // The key's release: a frame with the focus settled.
        frame(&ctx, vec![], |ui| add(ui, &mut axis));
        for _ in 0..2 {
            frame(&ctx, key(egui::Key::ArrowRight), |ui| add(ui, &mut axis));
        }
        assert_eq!(axis, 2, "{chosen}: the segmented choice did not follow the arrows");
        frame(&ctx, key(egui::Key::ArrowLeft), |ui| add(ui, &mut axis));
        assert_eq!(axis, 1, "{chosen}");
    }
}

#[test]
fn a_switch_and_a_segment_name_themselves_to_a_screen_reader() {
    let ctx = context(look::Look::Gtk, true);
    let mut on = false;
    frame(&ctx, vec![], |ui| ui.add(look::Switch::new(&mut on).accessible_label("Snap")));
    let input = egui::RawInput { events: key(egui::Key::Tab), ..Default::default() };
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
    let input = egui::RawInput { events: key(egui::Key::Tab), ..Default::default() };
    let mut output = ctx.run_ui(input, |ui| row(ui, &mut on));
    output.textures_delta.clear();
    let named = output.platform_output.events.iter().any(|e| matches!(e, egui::output::OutputEvent::FocusGained(info) if info.label.as_deref() == Some("Snap to grid")));
    assert!(named, "{:?}", output.platform_output.events);
}

#[test]
fn a_number_field_keeps_to_its_type_and_any_step() {
    let ctx = context(look::Look::Mac, true);
    let mut full: u8 = 255;
    // The + button is the last control of the row; a click there must not wrap the value.
    let plus = frame(&ctx, vec![], |ui| ui.add(look::NumberField::new(&mut full).width(140.0)).rect.right_center() - egui::vec2(8.0, 0.0));
    for events in click(plus) {
        frame(&ctx, events, |ui| ui.add(look::NumberField::new(&mut full).width(140.0)));
    }
    assert_eq!(full, 255);
    let mut value = 0.0_f64;
    for events in click(plus) {
        frame(&ctx, events, |ui| ui.add(look::NumberField::new(&mut value).step(-2.0).width(140.0)));
    }
    assert_eq!(value, 2.0, "a negative step counts as its size");
    for events in click(plus) {
        frame(&ctx, events, |ui| ui.add(look::NumberField::new(&mut value).step(0.0).width(140.0)));
    }
    assert_eq!(value, 3.0, "a zero step counts as 1");
}

#[test]
fn nonsense_sizes_and_values_are_held_in_bounds() {
    for factor in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        let metrics = look::Metrics::of(look::Look::Gtk).scaled(factor);
        assert!(metrics.body.size.is_finite() && metrics.body.size > 0.0 && metrics.control_height >= 1.0, "{factor}: {metrics:?}");
    }
    let ctx = context(look::Look::Windows, false);
    frame(&ctx, vec![], |ui| ui.add(look::ProgressBar::new(f32::NAN).width(100.0)));
    // A range from high to low runs the other way.
    let mut value = 0.0_f64;
    for events in click(egui::pos2(190.0, 10.0)) {
        frame(&ctx, events, |ui| ui.add(look::Slider::new(&mut value, 100.0..=0.0).width(200.0)));
    }
    assert!(value < 10.0, "{value}");
}

#[test]
fn text_on_a_light_accent_is_dark() {
    let mut theme = look::Theme::new(look::Look::Gtk);
    theme.accent = Some([0xf6d32d, 0xf6d32d]);
    assert_eq!(look::Colors::resolve(&theme, false).on_accent, egui::Color32::BLACK);
    theme.accent = Some([0x3584e4, 0x3584e4]);
    assert_eq!(look::Colors::resolve(&theme, false).on_accent, egui::Color32::WHITE);
    let mac = look::Theme { accent: None, ..look::Theme::new(look::Look::Mac) };
    assert_eq!(look::Colors::resolve(&mac, true).on_accent, egui::Color32::WHITE);
}

#[test]
fn escape_closes_the_top_dialog_alone_and_not_while_a_field_has_the_focus() {
    let ctx = context(look::Look::Mac, true);
    let (mut first, mut second) = (true, true);
    let mut name = String::new();
    let run = |events: Vec<egui::Event>, first: &mut bool, second: &mut bool, name: &mut String| {
        frame(&ctx, events, |_ui| {
            look::Dialog::new("first", "First").show(&ctx, first, |ui| { ui.label("…"); }, |_| {});
            look::Dialog::new("second", "Second").show(&ctx, second, |ui| { ui.add(look::TextField::new(name)); }, |_| {});
        })
    };
    run(vec![], &mut first, &mut second, &mut name);
    run(vec![], &mut first, &mut second, &mut name);
    run(key(egui::Key::Escape), &mut first, &mut second, &mut name);
    assert!(first && !second, "Escape closes the one on top: {first} {second}");
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
fn following_the_system_never_asks_for_frames() {
    let ctx = context(look::Look::Mac, true);
    // egui asks for a few frames as it starts; after those, nothing should.
    for frame in 0..6 {
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| look::follow_system(ui.ctx()));
        if frame < 3 {
            output.textures_delta.clear();
            continue;
        }
        output.textures_delta.clear();
        let delay = output.viewport_output.get(&egui::ViewportId::ROOT).map(|v| v.repaint_delay);
        assert!(delay.is_none_or(|d| d > std::time::Duration::from_secs(60)), "{delay:?}");
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
    let other = egui::RawInput { viewport_id: tool, viewports: [(tool, Default::default())].into_iter().collect(), ..Default::default() };
    let elsewhere = shapes(ctx.run_ui(other, |ui| look::show_toasts(ui.ctx())));
    assert_eq!(elsewhere, before, "another viewport drew the toast");
}
