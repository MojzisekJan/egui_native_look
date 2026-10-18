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
