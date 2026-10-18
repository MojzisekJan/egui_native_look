//! The cost of a frame of controls in each look, against egui's own widgets.

use criterion::{criterion_group, criterion_main, Criterion};
use egui_native_look as look;

const ROWS: usize = 200;

fn frame(ctx: &egui::Context, mut add: impl FnMut(&mut egui::Ui, usize)) {
    let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0))), ..Default::default() };
    let mut output = ctx.run_ui(input, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for row in 0..ROWS {
                ui.horizontal(|ui| add(ui, row));
            }
        });
    });
    output.textures_delta.clear();
    let _ = ctx.tessellate(output.shapes, output.pixels_per_point);
}

fn controls(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("{ROWS} rows of five controls"));
    let mut state = vec![(false, 0usize, 0usize, 4.0f64); ROWS];
    {
        let ctx = egui::Context::default();
        group.bench_function("egui", |b| b.iter(|| frame(&ctx, |ui, row| {
            // The same controls as the looks draw: three segments, − value +.
            let (on, choice, segment, value) = &mut state[row];
            let _ = ui.button("Apply");
            ui.checkbox(on, "Snap");
            ui.radio_value(choice, 1, "Rotate");
            for (i, axis) in ["X", "Y", "Z"].into_iter().enumerate() {
                ui.selectable_value(segment, i, axis);
            }
            let _ = ui.button("−");
            ui.add(egui::DragValue::new(value));
            let _ = ui.button("+");
        })));
    }
    for chosen in [look::Look::Plain, look::Look::Mac, look::Look::Windows, look::Look::Gtk, look::Look::Ios, look::Look::Material] {
        let ctx = egui::Context::default();
        look::install(&ctx, look::Theme::new(chosen).metrics(look::Metrics::of(look::Look::Plain)));
        group.bench_function(format!("{chosen}"), |b| b.iter(|| frame(&ctx, |ui, row| {
            let (on, choice, segment, value) = &mut state[row];
            ui.add(look::Button::new("Apply"));
            ui.add(look::Checkbox::new(on, "Snap"));
            look::radio_value(ui, choice, 1, "Rotate");
            ui.add(look::Segmented::new(("axis", row), segment, &["X", "Y", "Z"]).width(120.0));
            ui.add(look::NumberField::new(value).range(0.0..=100.0));
        })));
    }
    group.finish();
}

criterion_group!(benches, controls);
criterion_main!(benches);
