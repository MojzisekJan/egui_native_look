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
        ui.add(look::NumberField::new(&mut s.radius).range(0.0..=1024.0));
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

/// The toolbar, a form and a colour well.
fn more(ui: &mut egui::Ui, s: &mut State) {
    ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        ui.add(look::IconButton::new("↖").selected(true));
        ui.add(look::IconButton::new("↔"));
        ui.add(look::IconButton::new("↕"));
    });
    look::Form::new("form").show(ui, |form| {
        let width = form.control_width();
        form.row("Radius", |ui| ui.add(look::NumberField::new(&mut s.radius).range(0.0..=1024.0).width(width)));
        form.row("Corners", |ui| ui.add(look::Dropdown::new("corners", &mut s.shape, &["Mitred", "Rounded"]).width(width)));
        let mut tint = egui::Color32::from_rgb(0xe0, 0x8a, 0x3c);
        form.row("Colour", |ui| ui.add(look::ColorWell::new(&mut tint)));
        form.row("Snap to grid", |ui| ui.add(look::Switch::new(&mut s.on)));
    });
}

/// Draws `add` in every look and scheme and compares each with its picture, named after
/// the look, the scheme and `suffix`.
fn every_look_and_scheme_of(add: fn(&mut egui::Ui, &mut State), height: f32, suffix: &str) {
    let mut failures = Vec::new();
    for chosen in [look::Look::Mac, look::Look::Windows, look::Look::Gtk, look::Look::Ios, look::Look::Material, look::Look::Plain] {
        for (scheme, name) in [(egui::Theme::Dark, "dark"), (egui::Theme::Light, "light")] {
            let state = State { on: true, checked: true, mode: 1, axis: 0, tab: 0, shape: 0, radius: 8.0, volume: 40.0, name: String::new() };
            let mut harness = egui_kittest::Harness::builder().with_size(egui::vec2(420.0, height)).build_ui_state(
                move |ui, state: &mut State| {
                    egui::Frame::new().fill(ui.visuals().panel_fill).inner_margin(12).show(ui, |ui| add(ui, state));
                },
                state,
            );
            look::install(&harness.ctx, theme(chosen));
            harness.ctx.set_theme(scheme);
            harness.run();
            if let Err(error) = harness.try_snapshot(format!("{}-{name}{suffix}", chosen.label().to_lowercase())) {
                failures.push(error.to_string());
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_look_and_scheme() {
    every_look_and_scheme_of(controls, 470.0, "");
}

#[test]
fn the_toolbar_and_form_in_every_look_and_scheme() {
    every_look_and_scheme_of(more, 330.0, "-more");
}
