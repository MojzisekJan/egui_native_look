//! Every control in every look and scheme, against the pictures in `tests/snapshots`.
//! They render with wgpu, which needs a GPU (or a software adapter), so they are left out
//! of a plain `cargo test`: `cargo test --test snapshots -- --include-ignored` runs them,
//! and `UPDATE_SNAPSHOTS=1` with it writes new ones after a deliberate change.

use egui_native_look as look;

fn snapshot_options(name: &str) -> egui_kittest::SnapshotOptions {
    // Measured software-renderer differences: one antialiased pixel in each affected image.
    // Keep every pixel checked; only these references need a wider colour tolerance.
    let threshold = match name {
        "windows-dark-menu" => 3.0,
        "macos-light" | "macos-light-more" => 1.0,
        _ => return egui_kittest::SnapshotOptions::default(),
    };
    egui_kittest::SnapshotOptions::default().threshold(threshold)
}

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
        ui.add(look::Switch::new(&mut s.on).accessible_label("Switch"));
        ui.add(look::Checkbox::new(&mut s.checked, "Check box"));
        let mut mixed = false;
        ui.add(look::Checkbox::new(&mut mixed, "Mixed").indeterminate(true));
    });
    ui.horizontal(|ui| {
        look::radio_value(ui, &mut s.mode, 0, "Move");
        look::radio_value(ui, &mut s.mode, 1, "Rotate");
    });
    ui.add(look::Segmented::new("axis", &mut s.axis, &["X", "Y", "Z"]).width(240.0));
    ui.add(look::Tabs::new(
        "tabs",
        &mut s.tab,
        &["General", "Keyboard"],
    ));
    ui.horizontal(|ui| {
        ui.add(look::Dropdown::new("shape", &mut s.shape, &["Cuboid", "Sphere"]).width(150.0));
        ui.add(
            look::NumberField::new(&mut s.radius)
                .range(0.0..=1024.0)
                .accessible_label("Radius"),
        );
    });
    ui.horizontal(|ui| {
        ui.add(
            look::Slider::new(&mut s.volume, 0.0..=100.0)
                .width(160.0)
                .accessible_label("Volume"),
        );
        ui.add(look::ProgressBar::new(0.4).width(120.0));
    });
    ui.add(look::TextField::new(&mut s.name).hint("Name").width(240.0));
    look::section(ui, "section", "Section", true, |ui| {
        ui.add(look::ListRow::new(true).accessible_label("Selected row"));
        ui.add(look::ListRow::new(false).accessible_label("Other row"));
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
        form.row("Radius", |ui| {
            ui.add(
                look::NumberField::new(&mut s.radius)
                    .range(0.0..=1024.0)
                    .width(width),
            )
        });
        form.row("Corners", |ui| {
            ui.add(
                look::Dropdown::new("corners", &mut s.shape, &["Mitred", "Rounded"]).width(width),
            )
        });
        let mut tint = egui::Color32::from_rgb(0xe0, 0x8a, 0x3c);
        form.row("Colour", |ui| ui.add(look::ColorWell::new(&mut tint)));
        form.row("Snap to grid", |ui| ui.add(look::Switch::new(&mut s.on)));
    });
}

/// Draws `add` in every look and scheme and compares each with its picture, named after
/// the look, the scheme and `suffix`.
fn every_look_and_scheme_of(add: fn(&mut egui::Ui, &mut State), height: f32, suffix: &str) {
    let mut failures = Vec::new();
    for chosen in [
        look::Look::Mac,
        look::Look::Windows,
        look::Look::Gtk,
        look::Look::Ios,
        look::Look::Material,
        look::Look::Plain,
    ] {
        for (scheme, name) in [(egui::Theme::Dark, "dark"), (egui::Theme::Light, "light")] {
            let mut options = egui_kittest::SnapshotOptions::default();
            // One antialiased edge pixel can vary between graphics backends.
            if chosen == look::Look::Mac && scheme == egui::Theme::Light && suffix.is_empty() {
                options.max_failed_pixels = 1;
            }
            let state = State {
                on: true,
                checked: true,
                mode: 1,
                axis: 0,
                tab: 0,
                shape: 0,
                radius: 8.0,
                volume: 40.0,
                name: String::new(),
            };
            let mut harness = egui_kittest::Harness::builder()
                .with_options(options)
                .with_size(egui::vec2(420.0, height))
                .build_ui_state(
                    move |ui, state: &mut State| {
                        egui::Frame::new()
                            .fill(ui.visuals().panel_fill)
                            .inner_margin(12)
                            .show(ui, |ui| add(ui, state));
                    },
                    state,
                );
            look::install(&harness.ctx, theme(chosen));
            harness.ctx.set_theme(scheme);
            harness.run();
            let file = format!("{}-{name}{suffix}", chosen.label().to_lowercase());
            if let Err(error) = harness.try_snapshot_options(&file, &snapshot_options(&file)) {
                failures.push(error.to_string());
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
#[ignore = "renders with wgpu: cargo test --test snapshots -- --include-ignored"]
fn every_look_and_scheme() {
    every_look_and_scheme_of(controls, 470.0, "");
}

#[test]
#[ignore = "renders with wgpu: cargo test --test snapshots -- --include-ignored"]
fn the_toolbar_and_form_in_every_look_and_scheme() {
    every_look_and_scheme_of(more, 330.0, "-more");
}

/// A chequered window, so a dialog's corners, edge and shade show against it.
fn chequer(ui: &mut egui::Ui) {
    let area = ui.max_rect();
    for row in 0..12 {
        for column in 0..16 {
            let color = if (row + column) % 2 == 0 {
                egui::Color32::from_rgb(100, 130, 160)
            } else {
                egui::Color32::from_rgb(180, 160, 120)
            };
            let min = area.min + egui::vec2(column as f32 * 32.0, row as f32 * 32.0);
            ui.painter().rect_filled(
                egui::Rect::from_min_size(min, egui::Vec2::splat(32.0)),
                0.0,
                color,
            );
        }
    }
}

/// Every look's dialog, at its own size, in both schemes: `<look>-<scheme>-dialog`.
#[test]
#[ignore = "renders with wgpu: cargo test --test snapshots -- --include-ignored"]
fn every_looks_dialog_in_both_schemes() {
    let mut failures = Vec::new();
    for chosen in [
        look::Look::Mac,
        look::Look::Windows,
        look::Look::Gtk,
        look::Look::Ios,
        look::Look::Material,
        look::Look::Plain,
    ] {
        for (scheme, name) in [(egui::Theme::Dark, "dark"), (egui::Theme::Light, "light")] {
            let mut harness = egui_kittest::Harness::builder()
                .with_size(egui::vec2(440.0, 320.0))
                .with_theme(scheme)
                .with_context_setup(move |ctx| look::install(ctx, theme(chosen)))
                .build_ui(|ui| {
                    chequer(ui);
                    look::Dialog::new("dialog", "Save changes?").show(
                        ui.ctx(),
                        &mut true,
                        |ui| {
                            ui.label("Save changes to this document?");
                        },
                        |ui| {
                            ui.add(look::Button::new("Save").accent().default_action());
                            ui.add(look::Button::new("Cancel"));
                        },
                    );
                });
            harness.run();
            let file = format!("{}-{name}-dialog", chosen.label().to_lowercase());
            if let Err(error) = harness.try_snapshot_options(&file, &snapshot_options(&file)) {
                failures.push(error.to_string());
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Every look's open pop-up menu, a ticked item and a plain one, in both schemes:
/// `<look>-<scheme>-menu`.
#[test]
#[ignore = "renders with wgpu: cargo test --test snapshots -- --include-ignored"]
fn every_looks_open_menu_in_both_schemes() {
    use egui_kittest::kittest::Queryable;
    let mut failures = Vec::new();
    for chosen in [
        look::Look::Mac,
        look::Look::Windows,
        look::Look::Gtk,
        look::Look::Ios,
        look::Look::Material,
        look::Look::Plain,
    ] {
        for (scheme, name) in [(egui::Theme::Dark, "dark"), (egui::Theme::Light, "light")] {
            let mut harness = egui_kittest::Harness::builder()
                .with_size(egui::vec2(300.0, 260.0))
                .with_theme(scheme)
                .with_context_setup(move |ctx| look::install(ctx, theme(chosen)))
                .build_ui_state(
                    |ui, units: &mut usize| {
                        egui::Frame::new()
                            .fill(ui.visuals().panel_fill)
                            .inner_margin(12)
                            .show(ui, |ui| {
                                ui.set_min_size(ui.available_size());
                                look::PopupButton::new("units", "Metres").show(ui, |ui| {
                                    look::menu_value(ui, units, 0, "Metres");
                                    look::menu_value(ui, units, 1, "Feet");
                                    ui.separator();
                                    ui.add(look::MenuItem::new("Convert all units…"));
                                });
                            });
                    },
                    0usize,
                );
            harness.run();
            harness.get_by_label("Metres").click();
            harness.run();
            // The pointer away, so no row is lit under it.
            harness.hover_at(egui::pos2(290.0, 250.0));
            harness.run();
            let file = format!("{}-{name}-menu", chosen.label().to_lowercase());
            if let Err(error) = harness.try_snapshot_options(&file, &snapshot_options(&file)) {
                failures.push(error.to_string());
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
