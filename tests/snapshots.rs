//! Every control in every look and scheme, against the pictures in `tests/snapshots`.
//! `UPDATE_SNAPSHOTS=1 cargo test --test snapshots` writes new ones after a deliberate
//! change.

use egui_native_look as look;

/// A theme that reads nothing from this system, so the pictures are the same everywhere.
fn theme(chosen: look::Look) -> look::Theme {
    let mut theme = look::Theme::new(chosen);
    theme.accent = Some(match chosen {
        look::Look::Windows => [0x60cdff, 0x005fb8],
        look::Look::Gtk => [0x3584e4, 0x3584e4],
        look::Look::Material => [0xd0bcff, 0x6750a4],
        _ => [0x0a84ff, 0x007aff],
    });
    theme
}

struct State {
    on: bool,
    checked: bool,
    mode: usize,
    axis: usize,
    tab: usize,
    shape: usize,
    radius: f64,
    volume: f64,
    name: String,
}

fn controls(ui: &mut egui::Ui, s: &mut State) {
    ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);
    ui.horizontal(|ui| {
        ui.add(look::Button::new("Button"));
        ui.add_enabled(false, look::Button::new("Disabled"));
        ui.add(look::Button::new("Default").accent());
    });
    ui.horizontal(|ui| {
        ui.add(look::Switch::new(&mut s.on));
        ui.add(look::Checkbox::new(&mut s.checked, "Check box"));
        let mut mixed = false;
        ui.add(look::Checkbox::new(&mut mixed, "Mixed").indeterminate(true));
    });
    ui.horizontal(|ui| {
        look::radio_value(ui, &mut s.mode, 0, "Move");
        look::radio_value(ui, &mut s.mode, 1, "Rotate");
    });
    ui.add(look::Segmented::new("axis", &mut s.axis, &["X", "Y", "Z"]).width(240.0));
    ui.add(look::Tabs::new("tabs", &mut s.tab, &["General", "Keyboard"]));
    ui.horizontal(|ui| {
        ui.add(look::Dropdown::new("shape", &mut s.shape, &["Cuboid", "Sphere"]).width(150.0));
        ui.add(look::NumberField::new(&mut s.radius, 0.0..=1024.0));
    });
    ui.horizontal(|ui| {
        ui.add(look::Slider::new(&mut s.volume, 0.0..=100.0).width(160.0));
        ui.add(look::ProgressBar::new(0.4).width(120.0));
    });
    ui.add(look::TextField::new(&mut s.name).hint("Name").width(240.0));
    look::section(ui, "section", "Section", true, |ui| {
        ui.add(look::ListRow::new(true));
        ui.add(look::ListRow::new(false));
    });
}

#[test]
fn every_look_and_scheme() {
    let mut failures = Vec::new();
    for chosen in [look::Look::Mac, look::Look::Windows, look::Look::Gtk, look::Look::Ios, look::Look::Material, look::Look::Plain] {
        for (scheme, name) in [(egui::Theme::Dark, "dark"), (egui::Theme::Light, "light")] {
            let state = State { on: true, checked: true, mode: 1, axis: 0, tab: 0, shape: 0, radius: 8.0, volume: 40.0, name: String::new() };
            let mut harness = egui_kittest::Harness::builder().with_size(egui::vec2(420.0, 470.0)).build_ui_state(
                |ui, state: &mut State| {
                    egui::Frame::new().fill(ui.visuals().panel_fill).inner_margin(12).show(ui, |ui| controls(ui, state));
                },
                state,
            );
            look::install(&harness.ctx, theme(chosen));
            harness.ctx.set_theme(scheme);
            harness.run();
            if let Err(error) = harness.try_snapshot(format!("{}-{name}", chosen.label().to_lowercase())) {
                failures.push(error.to_string());
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
