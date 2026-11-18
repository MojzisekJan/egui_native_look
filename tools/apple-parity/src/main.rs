use egui_native_look as look;
use serde_json::{json, Value};

fn option(name: &str, default: &str) -> String {
    let args: Vec<String> = std::env::args().collect();
    args.windows(2)
        .find(|a| a[0] == name)
        .map_or_else(|| default.into(), |a| a[1].clone())
}

struct Fixture {
    checked: bool,
    switched: bool,
    segment: usize,
    choice: usize,
    slider: f64,
    text: String,
    password: String,
    number: i32,
    count: usize,
    alert: bool,
    geometry: Value,
    disabled: bool,
}

impl Default for Fixture {
    fn default() -> Self {
        Self {
            checked: true,
            switched: true,
            segment: 1,
            choice: 1,
            slider: 0.4,
            text: "Hello".into(),
            password: "secret".into(),
            number: 3,
            count: 0,
            alert: false,
            geometry: json!({}),
            disabled: std::env::args().any(|a| a == "--disabled"),
        }
    }
}

fn setup(ctx: &egui::Context) {
    let chosen = if option("--look", "mac") == "ios" {
        look::Look::Ios
    } else {
        look::Look::Mac
    };
    let mut fonts = egui::FontDefinitions::default();
    look::install_fonts(&mut fonts, chosen);
    ctx.set_fonts(fonts);
    let mut theme = look::Theme::new(chosen);
    theme.metrics = match option("--size", "regular").as_str() {
        "small" => look::Metrics::small(chosen),
        "large" => look::Metrics::large(chosen),
        _ => look::Metrics::of(chosen),
    }
    .system_bold();
    look::install(ctx, theme);
    ctx.set_theme(if option("--scheme", "light") == "dark" {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    });
}

impl Fixture {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        ui.painter()
            .rect_filled(ctx.content_rect(), 0., look::colors(&ctx).bg);
        let annotations = ui.painter().clone();
        if self.disabled {
            ui.disable();
        }
        for (index, (name, label)) in [
            ("button", "Button"),
            ("default", "Default"),
            ("destructive", "Destructive"),
            ("checkbox", "Checkbox"),
            ("switch", "Switch"),
            ("segmented", "Segmented"),
            ("picker", "Picker"),
            ("slider", "Slider"),
            ("text", "Text field"),
            ("password", "Password"),
            ("stepper", "Stepper"),
            ("progress", "Progress"),
            ("alert", "Alert"),
        ]
        .into_iter()
        .enumerate()
        {
            let y = 16. + 42. * index as f32;
            annotations.text(
                egui::pos2(16., y + 21.),
                egui::Align2::LEFT_CENTER,
                label,
                egui::FontId::proportional(11.),
                look::colors(&ctx).dim,
            );
            let response = ui
                .scope_builder(
                    egui::UiBuilder::new()
                        .id_salt(name)
                        .max_rect(egui::Rect::from_min_size(
                            egui::pos2(120., y),
                            egui::vec2(224., 42.),
                        ))
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    |ui| match name {
                        "button" => {
                            let r = ui.add(look::Button::new("Save"));
                            self.count += usize::from(r.clicked());
                            r
                        }
                        "default" => {
                            let r = ui.add(look::Button::new("Continue").accent().default_action());
                            self.count += usize::from(r.clicked());
                            r
                        }
                        "destructive" => ui.add(look::Button::new("Delete").destructive()),
                        "checkbox" => ui.add(look::Checkbox::new(&mut self.checked, "Enabled")),
                        "switch" => {
                            ui.add(look::Switch::new(&mut self.switched).accessible_label("Switch"))
                        }
                        "segmented" => ui.add(
                            look::Segmented::new(name, &mut self.segment, &["One", "Two", "Three"])
                                .width(216.),
                        ),
                        "picker" => ui.add(
                            look::Dropdown::new(name, &mut self.choice, &["One", "Two", "Three"])
                                .width(160.),
                        ),
                        "slider" => ui.add(
                            look::Slider::new(&mut self.slider, 0.0..=1.0)
                                .width(216.)
                                .show_value(false)
                                .accessible_label("Slider"),
                        ),
                        "text" => ui.add(
                            look::TextField::new(&mut self.text)
                                .hint("Name")
                                .width(216.),
                        ),
                        "password" => ui.add(
                            look::TextField::new(&mut self.password)
                                .password(true)
                                .hint("Password")
                                .width(216.),
                        ),
                        "stepper" => ui.add(
                            look::NumberField::new(&mut self.number)
                                .range(0..=10)
                                .width(160.)
                                .accessible_label("Stepper"),
                        ),
                        "progress" => ui.add(look::ProgressBar::new(0.4).width(216.)),
                        _ => {
                            let r = ui.add(look::Button::new("Show alert"));
                            self.alert |= r.clicked();
                            r
                        }
                    },
                )
                .inner;
            let r = response.rect;
            self.geometry[name] = json!([r.min.x, r.min.y, r.width(), r.height()]);
        }
        annotations.text(
            egui::pos2(120., 583.),
            egui::Align2::LEFT_CENTER,
            format!(
                "clicks={} check={} value={:.2}",
                self.count,
                usize::from(self.checked),
                self.slider
            ),
            egui::FontId::proportional(11.),
            look::colors(&ctx).fg,
        );
        look::Dialog::new("confirm", "Save changes?").show(
            &ctx,
            &mut self.alert,
            |ui| {
                ui.label("Your changes will be saved.");
            },
            |ui| {
                if ui.add(look::Button::new("Cancel")).clicked() {
                    ui.close();
                }
                if ui
                    .add(look::Button::new("Save").accent().default_action())
                    .clicked()
                {
                    self.count += 1;
                    ui.close();
                }
                if ui.add(look::Button::new("Delete").destructive()).clicked() {
                    self.count += 10;
                    ui.close();
                }
            },
        );
    }
}

impl eframe::App for Fixture {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        self.ui(ui);
    }
}

fn main() {
    if std::env::args().any(|a| a == "--live") {
        eframe::run_native(
            "egui Apple parity",
            eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default().with_inner_size([360., 620.]),
                ..Default::default()
            },
            Box::new(|cc| {
                setup(&cc.egui_ctx);
                Ok(Box::new(Fixture::default()))
            }),
        )
        .unwrap();
        return;
    }
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(360., 620.))
        .with_pixels_per_point(
            option("--scale", "2")
                .parse::<f32>()
                .expect("positive display scale"),
        )
        .build_ui_state(|ui, state: &mut Fixture| state.ui(ui), Fixture::default());
    setup(&harness.ctx);
    for _ in 0..30 {
        harness.step();
    }
    let out = std::path::PathBuf::from(option("--out", "/tmp/egui-reference"));
    std::fs::create_dir_all(&out).unwrap();
    harness
        .render()
        .unwrap()
        .save(out.join("egui.png"))
        .unwrap();
    std::fs::write(
        out.join("egui.json"),
        serde_json::to_vec_pretty(&json!({"geometry": harness.state().geometry})).unwrap(),
    )
    .unwrap();
}
