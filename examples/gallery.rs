//! Every control in every look: `cargo run --example gallery [mac|windows|gtk|ios|material|plain] [light]`.
//! The look and the scheme can be switched in the window too; `dialog` opens the dialog. With `GALLERY_SHOT=file.png`
//! it saves its window to that file and quits.

use eframe::egui;
use egui_native_look as look;

fn main() -> eframe::Result {
    let forced = std::env::args().nth(1).and_then(|name| look::Look::from_name(&name));
    let light = std::env::args().any(|a| a == "light");
    let dialog = std::env::args().any(|a| a == "dialog");
    eframe::run_native(
        "egui_native_look gallery",
        eframe::NativeOptions { viewport: egui::ViewportBuilder::default().with_inner_size([620.0, 720.0]), ..Default::default() },
        Box::new(move |cc| {
            let chosen = forced.unwrap_or_else(look::Look::system);
            let mut fonts = egui::FontDefinitions::default();
            look::install_fonts(&mut fonts, chosen);
            cc.egui_ctx.set_fonts(fonts);
            look::install(&cc.egui_ctx, theme(chosen));
            cc.egui_ctx.set_theme(if light { egui::Theme::Light } else { egui::Theme::Dark });
            Ok(Box::new(Gallery {
                look: LOOKS.iter().position(|l| *l == chosen).unwrap_or(0),
                light,
                shot: std::env::var_os("GALLERY_SHOT").map(Into::into),
                radius: 8.0,
                volume: 40.0,
                dialog,
                ..Default::default()
            }))
        }),
    )
}

/// `look` with its system's sizes and, where the system has it, its semibold.
fn theme(look: look::Look) -> look::Theme {
    let metrics = look::Metrics::of(look);
    let metrics = if look::system_fonts(look).is_some() { metrics.system_bold() } else { metrics };
    look::Theme::new(look).metrics(metrics)
}

#[derive(Default)]
struct Gallery {
    look: usize,
    light: bool,
    on: bool,
    checked: bool,
    mixed: bool,
    mode: usize,
    axis: usize,
    tab: usize,
    shape: usize,
    radius: f64,
    volume: f64,
    name: String,
    query: String,
    picked: usize,
    dialog: bool,
    shot: Option<std::path::PathBuf>,
    frames: u32,
}

const LOOKS: [look::Look; 6] = [look::Look::Mac, look::Look::Windows, look::Look::Gtk, look::Look::Ios, look::Look::Material, look::Look::Plain];

impl eframe::App for Gallery {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.save_shot(&ctx);
        egui::CentralPanel::default().show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);
            ui.horizontal(|ui| {
                let labels: Vec<&str> = LOOKS.iter().map(|l| l.label()).collect();
                if ui.add(look::Segmented::new("look", &mut self.look, &labels).width(440.0)).changed() {
                    let mut fonts = egui::FontDefinitions::default();
                    look::install_fonts(&mut fonts, LOOKS[self.look]);
                    ctx.set_fonts(fonts);
                    look::install(&ctx, theme(LOOKS[self.look]));
                }
                if ui.add(look::Switch::new(&mut self.light)).changed() {
                    ctx.set_theme(if self.light { egui::Theme::Light } else { egui::Theme::Dark });
                }
                ui.label("Light");
            });
            ui.heading(LOOKS[self.look].label());
            ui.horizontal(|ui| {
                ui.add(look::Button::new("Button"));
                ui.add_enabled(false, look::Button::new("Disabled"));
                ui.add(look::Button::new("Default").accent());
            });
            ui.horizontal(|ui| {
                ui.add(look::Switch::new(&mut self.on));
                ui.label("Switch");
                ui.add(look::Checkbox::new(&mut self.checked, "Check box"));
                let mixed = !self.mixed;
                ui.add(look::Checkbox::new(&mut self.mixed, "Mixed").indeterminate(mixed));
            });
            ui.horizontal(|ui| {
                look::radio_value(ui, &mut self.mode, 0, "Move");
                look::radio_value(ui, &mut self.mode, 1, "Rotate");
                look::radio_value(ui, &mut self.mode, 2, "Scale");
            });
            ui.add(look::Segmented::new("axis", &mut self.axis, &["X", "Y", "Z"]).width(240.0));
            ui.add(look::Tabs::new("tabs", &mut self.tab, &["General", "Rendering", "Keyboard"]));
            ui.horizontal(|ui| {
                ui.add(look::Dropdown::new("shape", &mut self.shape, &["Cuboid", "Cylinder", "Sphere", "Stairs"]).width(180.0));
                ui.add(look::NumberField::new(&mut self.radius, 0.0..=1024.0));
            });
            ui.horizontal(|ui| {
                ui.add(look::Slider::new(&mut self.volume, 0.0..=100.0).width(200.0));
                ui.add(look::ProgressBar::new((self.volume / 100.0) as f32).width(140.0));
                ui.add(look::Spinner::new());
            });
            ui.horizontal(|ui| {
                ui.add(look::TextField::new(&mut self.name).hint("Name").width(260.0));
                if ui.add(look::Button::new("Dialog…")).clicked() {
                    self.dialog = true;
                }
            });
            look::Dialog::new("dialog", "Export map").size([380.0, 160.0]).show(&ctx, &mut self.dialog, |ui| {
                ui.label("Write the level for the chosen engine?");
            }, |ui| {
                ui.add(look::Button::new("Export").accent());
                ui.add(look::Button::new("Cancel"));
            });
            ui.add(look::TextField::new(&mut self.query).hint("Search").search().width(260.0));
            look::section(ui, "section", "Section", true, |ui| {
                for (i, item) in ["First row", "Second row", "Third row"].into_iter().enumerate() {
                    let response = ui.add(look::ListRow::new(i == self.picked));
                    let colour = if i == self.picked { look::picked_text(ui) } else { look::colors(ui.ctx()).fg };
                    look::text(ui.painter(), response.rect.left_center() + egui::vec2(10.0, 0.0), egui::Align2::LEFT_CENTER, item, egui::FontId::proportional(13.0), colour);
                    if response.clicked() {
                        self.picked = i;
                    }
                }
            });
        });
    }
}

impl Gallery {
    fn save_shot(&mut self, ctx: &egui::Context) {
        let Some(path) = &self.shot else { return };
        self.frames += 1;
        ctx.request_repaint();
        if self.frames == 10 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        let image = ctx.input(|i| i.events.iter().find_map(|e| match e {
            egui::Event::Screenshot { image, .. } => Some(image.clone()),
            _ => None,
        }));
        if let Some(image) = image {
            let [w, h] = image.size;
            let bytes: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
            image::save_buffer(path, &bytes, w as u32, h as u32, image::ColorType::Rgba8).expect("saved the shot");
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
