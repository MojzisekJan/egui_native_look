use eframe::egui;
use egui_native_look as look;
use serde_json::json;
use std::{collections::HashMap, path::Path};

struct App {
    sequence: u64,
    password: bool,
    controls_enabled: bool,
    radio: usize,
    list_selected: usize,
    list_disabled: bool,
    list_activated: u32,
    list_scroll: f32,
    menu_selected: usize,
    check: bool,
    switch: bool,
    entry: String,
    invalid_entry: String,
    dialog_entry: String,
    form_number: f64,
    number: f64,
    slider: f64,
    dropdown: usize,
    button: u32,
    dialog_save: u32,
    dialog: bool,
    ids: HashMap<String, egui::Id>,
}
impl Default for App {
    fn default() -> Self {
        Self {
            sequence: 0,
            password: false,
            controls_enabled: true,
            radio: 0,
            list_selected: 0,
            list_disabled: true,
            list_activated: 0,
            list_scroll: 0.0,
            menu_selected: 0,
            check: false,
            switch: false,
            entry: "Example".into(),
            invalid_entry: "Example".into(),
            dialog_entry: String::new(),
            form_number: 4.,
            number: 4.,
            slider: 40.,
            dropdown: 0,
            button: 0,
            dialog_save: 0,
            dialog: false,
            ids: HashMap::new(),
        }
    }
}
impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let root = std::env::var("GTK_PARITY_OUT").expect("GTK_PARITY_OUT");
        let ctx = ui.ctx().clone();
        ctx.accesskit_node_builder(egui::accesskit_root_id(), |node| {
            node.set_label("egui comparison")
        });
        let wheel = ctx.input(|input| {
            input
                .events
                .iter()
                .filter(|event| matches!(event, egui::Event::MouseWheel { .. }))
                .map(|event| format!("{event:?}"))
                .collect::<Vec<_>>()
        });
        if !wheel.is_empty() {
            use std::io::Write;
            let mut log = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(format!("{root}/wheel-events.log"))
                .unwrap();
            writeln!(log, "{wheel:?}").unwrap();
        }
        let ime = ctx.input(|input| {
            input
                .events
                .iter()
                .filter(|event| matches!(event, egui::Event::Ime(_)))
                .map(|event| format!("{event:?}"))
                .collect::<Vec<_>>()
        });
        if !ime.is_empty() {
            use std::io::Write;
            let mut log = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(format!("{root}/ime-events.jsonl"))
                .unwrap();
            writeln!(
                log,
                "{}",
                json!({"frame":ctx.cumulative_frame_nr(),"events":ime})
            )
            .unwrap();
        }
        let path = format!("{root}/egui-command.json");
        if Path::new(&path).exists() {
            let msg: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            std::fs::remove_file(&path).unwrap();
            if let Some(seq) = msg["seq"].as_u64() {
                self.sequence = seq;
            }
            if msg["reset"].as_bool() == Some(true) {
                self.check = false;
                self.switch = false;
                self.number = 4.;
                self.slider = 40.;
                self.dropdown = 0;
                self.entry = "Example".into();
                self.button = 0;
                self.dialog_save = 0;
            }
            if let Some(dark) = msg["dark"].as_bool() {
                ctx.set_theme(if dark {
                    egui::Theme::Dark
                } else {
                    egui::Theme::Light
                });
            }
            if let Some(accent) = msg["accent"].as_u64() {
                let mut theme = look::theme(&ctx);
                theme = theme.accent(Some([accent as u32; 2]));
                look::install(&ctx, theme);
            }
            if let Some(password) = msg["password"].as_bool() {
                self.password = password;
            }
            if let Some(enabled) = msg["controls_enabled"].as_bool() {
                self.controls_enabled = enabled;
            }
            if let Some(text) = msg["text"].as_str() {
                self.entry = text.into();
            }
            if let Some(n) = msg["number"].as_f64() {
                self.number = n;
            }
            if let Some(n) = msg["dropdown"].as_u64() {
                self.dropdown = n as usize;
            }
            if let Some(n) = msg["slider"].as_f64() {
                self.slider = n;
            }
            if let Some(disabled) = msg["list_disabled"].as_bool() {
                self.list_disabled = disabled;
            }
            if let Some(n) = msg["list_selected"].as_u64() {
                self.list_selected = n as usize;
            }
            if let Some(n) = msg["radio"].as_u64() {
                self.radio = n as usize;
            }
            if let Some(name) = msg["focus"].as_str() {
                if let Some(id) = self.ids.get(name) {
                    ctx.memory_mut(|m| m.request_focus(*id));
                }
            }
        }
        ui.painter()
            .rect_filled(ctx.content_rect(), 0., look::colors(&ctx).bg);
        if std::env::var("STATE").unwrap_or_default() == "disabled" || !self.controls_enabled {
            ui.disable();
        }
        if std::env::var_os("LONG_DIALOG").is_some() {
            let mut open = true;
            look::Dialog::new("long","Opravdu chcete uložit všechny neuložené změny v tomto dokumentu?").show(&ctx,&mut open,|ui|{ui.label("Před zavřením dokumentu můžete uložit změny, pokračovat v úpravách nebo změny zahodit. Tento delší český text musí zůstat celý čitelný i v úzkém okně.");},|ui|{
    ui.add(look::Button::new("Uložit všechny provedené změny").accent().default_action());
    ui.add(look::Button::new("Pokračovat v úpravách dokumentu"));
   });
            ctx.request_repaint_after(std::time::Duration::from_millis(30));
            return;
        }
        let mut widgets = json!({});
        ui.painter().text(
            egui::pos2(20., 15.),
            egui::Align2::LEFT_TOP,
            "egui_native_look — GTK",
            egui::FontId::proportional(14.67),
            ui.visuals().text_color(),
        );
        for (i, (name, label)) in [
            ("button", "Button"),
            ("check", "Checkbox"),
            ("switch", "Switch"),
            ("entry", "Text field"),
            ("number", "Number"),
            ("slider", "Slider"),
            ("dropdown", "Dropdown"),
            ("progress", "Progress"),
            ("dialog", "Dialog"),
            ("menu", "Nested menu"),
        ]
        .into_iter()
        .enumerate()
        {
            let y = 60. + 50. * i as f32;
            ui.painter().text(
                egui::pos2(20., y),
                egui::Align2::LEFT_TOP,
                label,
                egui::FontId::proportional(14.67),
                ui.visuals().text_color(),
            );
            let r = ui
                .scope_builder(
                    egui::UiBuilder::new()
                        .id_salt(name)
                        .max_rect(egui::Rect::from_min_size(
                            egui::pos2(180., y),
                            egui::vec2(240., 40.),
                        ))
                        .layout(egui::Layout::left_to_right(egui::Align::Min)),
                    |ui| match name {
                        "button" => {
                            let r = ui.add(look::Button::new("Save").accent().width(240.));
                            if r.clicked() {
                                self.button += 1
                            };
                            r
                        }
                        "check" => ui.add_enabled(
                            std::env::var_os("DISABLED_CHECK").is_none(),
                            look::Checkbox::new(&mut self.check, "Notifications"),
                        ),
                        "switch" => {
                            ui.add(look::Switch::new(&mut self.switch).accessible_label("Switch"))
                        }
                        "entry" => ui.add(
                            look::TextField::new(&mut self.entry)
                                .password(self.password)
                                .accessible_label("Text field")
                                .id_salt("text")
                                .width(240.),
                        ),
                        "number" => {
                            let field = look::NumberField::new(&mut self.number)
                                .accessible_label("Number")
                                .range(-10.0..=10.0)
                                .fixed_decimals(2)
                                .step(1.)
                                .width(240.);
                            if std::env::var("LC_ALL")
                                .unwrap_or_default()
                                .starts_with("cs")
                            {
                                ui.add(
                                    field
                                        .custom_formatter(|value, _| {
                                            format!("{value:.2}").replace('.', ",")
                                        })
                                        .custom_parser(|text| text.replace(',', ".").parse().ok()),
                                )
                            } else if std::env::var_os("SPIN_ROW").is_some() {
                                look::Form::new("row")
                                    .show(ui, |form| form.row("Number", |ui| ui.add(field)))
                            } else {
                                ui.add(field)
                            }
                        }
                        "slider" => ui.add(
                            look::Slider::new(&mut self.slider, -10.0..=100.0)
                                .accessible_label("Slider")
                                .step_by(1.)
                                .show_value(false)
                                .width(240.),
                        ),
                        "dropdown" => ui.add(
                            look::Dropdown::new(
                                "options",
                                &mut self.dropdown,
                                &["Alpha", "Beta", "Gamma"],
                            )
                            .width(240.),
                        ),
                        "progress" => ui.add(look::ProgressBar::new(0.4).width(240.)),
                        "dialog" => {
                            let r = ui.add(look::Button::new("Open dialog").width(240.));
                            if r.clicked() {
                                self.dialog_entry.clear();
                                self.dialog = true
                            };
                            r
                        }
                        _ => {
                            look::PopupButton::new("nested-menu", "Nested menu")
                                .width(240.)
                                .show(ui, |ui| {
                                    ui.add_enabled(false, look::MenuItem::new("Unavailable"));
                                    look::MenuItem::new("More").submenu(ui, |ui| {
                                        if ui.add(look::MenuItem::new("Alpha")).clicked() {
                                            self.menu_selected = 1;
                                        }
                                        if ui.add(look::MenuItem::new("Beta")).clicked() {
                                            self.menu_selected = 2;
                                        }
                                    });
                                })
                                .response
                        }
                    },
                )
                .inner;
            self.ids.insert(name.into(), r.id);
            widgets[name] = json!({"x":r.rect.min.x,"y":r.rect.min.y,"width":r.rect.width(),"height":r.rect.height(),"focus":r.has_focus(),"down":r.is_pointer_button_down_on(),"hover":r.hovered()});
        }
        for (i, name) in [
            "plain",
            "icon",
            "icon_text",
            "long",
            "form",
            "list",
            "destructive",
            "invalid",
        ]
        .into_iter()
        .enumerate()
        {
            let y = 60. + 60. * i as f32;
            let r = ui
                .scope_builder(
                    egui::UiBuilder::new()
                        .id_salt(name)
                        .max_rect(egui::Rect::from_min_size(
                            egui::pos2(500., y),
                            egui::vec2(320., 54.),
                        ))
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    |ui| match name {
                        "plain" => ui.add(look::Button::new("Save")),
                        "icon" => ui.add(look::IconButton::new("+").tooltip("Add")),
                        "icon_text" => ui.add(
                            look::Button::new("Add item")
                                .icon("+", egui::FontId::proportional(16.)),
                        ),
                        "long" => ui.add(look::Button::new("Uložit všechny provedené změny")),
                        "destructive" => ui.add(look::Button::new("Delete").destructive()),
                        "invalid" => ui.add(
                            look::TextField::new(&mut self.invalid_entry)
                                .invalid(true)
                                .width(240.),
                        ),
                        "form" => look::Form::new("form").show(ui, |f| {
                            f.row("Count", |ui| {
                                ui.add(
                                    look::NumberField::new(&mut self.form_number)
                                        .range(-10.0..=10.0)
                                        .fixed_decimals(2),
                                )
                            })
                        }),
                        _ => {
                            let r = ui.add(
                                look::ListRow::new(self.check)
                                    .accessible_label("Documents")
                                    .height(34.),
                            );
                            ui.painter().text(
                                r.rect.left_center() + egui::vec2(12., 0.),
                                egui::Align2::LEFT_CENTER,
                                "Documents",
                                look::theme(&ctx).metrics.body.clone(),
                                ui.visuals().text_color(),
                            );
                            if r.clicked() {
                                self.check = !self.check
                            };
                            r
                        }
                    },
                )
                .inner;
            self.ids.insert(name.into(), r.id);
            widgets[name] = json!({"x":r.rect.min.x,"y":r.rect.min.y,"width":r.rect.width(),"height":r.rect.height(),"focus":r.has_focus(),"down":r.is_pointer_button_down_on(),"hover":r.hovered()});
        }
        let mut radio_focus = false;
        let radio = ui
            .scope_builder(
                egui::UiBuilder::new()
                    .id_salt("radio")
                    .max_rect(egui::Rect::from_min_size(
                        egui::pos2(180., 580.),
                        egui::vec2(360., 40.),
                    )),
                |ui| {
                    let selected = self.radio;
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        look::RadioGroup::new("radio", &mut self.radio).show(ui, |group| {
                            let mut first = None;
                            for (i, label) in ["First", "Second", "Third"].into_iter().enumerate() {
                                let r = group.option(i, label);
                                if i == selected {
                                    first = Some(r.id);
                                }
                                radio_focus |= r.has_focus();
                            }
                            first
                        })
                    })
                    .inner
                },
            )
            .inner;
        if let Some(id) = radio.inner {
            self.ids.insert("radio".into(), id);
        }
        widgets["radio"] = json!({"x":radio.response.rect.min.x,"y":radio.response.rect.min.y,"width":radio.response.rect.width(),"height":radio.response.rect.height(),"focus":radio_focus});
        let list = ui
            .scope_builder(
                egui::UiBuilder::new()
                    .id_salt("listbox")
                    .max_rect(egui::Rect::from_min_size(
                        egui::pos2(180., 650.),
                        egui::vec2(240., 160.),
                    )),
                |ui| {
                    ui.spacing_mut().interact_size.y = 30.;
                    ui.spacing_mut().item_spacing.y = 0.;
                    let scroll = look::ScrollArea::vertical()
                        .max_height(160.)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            look::ListBox::new("files", &mut self.list_selected).show(ui, |list| {
                                for i in 0..20 {
                                    list.item_enabled(
                                        i != 2 || !self.list_disabled,
                                        i,
                                        format!("File {i:02}"),
                                    );
                                }
                            })
                        });
                    self.list_scroll = scroll.state.clamped_offset().y;
                    scroll.inner
                },
            )
            .inner;
        if list.activated.is_some() {
            self.list_activated += 1;
        }
        self.ids.insert("listbox".into(), list.response.id);
        widgets["listbox"] = json!({"x":list.response.rect.min.x,"y":list.response.rect.min.y,"width":list.response.rect.width(),"height":list.response.rect.height(),"focus":list.response.has_focus()});
        look::Dialog::new("save-dialog", "Save changes?").show(
            &ctx,
            &mut self.dialog,
            |ui| {
                ui.label("Your preferences will be saved.");
                if std::env::var_os("IME_DIALOG").is_some() {
                    let r = ui.add(
                        look::TextField::new(&mut self.dialog_entry)
                            .id_salt("dialog-entry")
                            .accessible_label("Name"),
                    );
                    self.ids.insert("dialog-entry".into(), r.id);
                }
            },
            |ui| {
                if ui
                    .add(look::Button::new("Save").accent().default_action())
                    .clicked()
                {
                    self.dialog_save += 1;
                    ui.close();
                }
                if ui.add(look::Button::new("Cancel")).clicked() {
                    ui.close();
                }
            },
        );
        let mut offsets = [0.0, 0.0];
        let mut scroll_geometry = [0.0, 0.0];
        ui.scope_builder(
            egui::UiBuilder::new()
                .id_salt("scrolls")
                .max_rect(egui::Rect::from_min_size(
                    egui::pos2(500., 650.),
                    egui::vec2(300., 180.),
                )),
            |ui| {
                ui.spacing_mut().item_spacing.y = 6.;
                let outer = look::ScrollArea::vertical()
                    .id_salt("outer")
                    .max_height(180.)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let inner = look::ScrollArea::vertical()
                            .id_salt("inner")
                            .max_height(90.)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                for i in 0..8 {
                                    ui.add(
                                        look::Button::new(format!("Nested item {i}")).width(280.),
                                    );
                                }
                            });
                        offsets[1] = inner.state.clamped_offset().y;
                        scroll_geometry[1] = inner.inner_rect.height();
                        for i in 0..12 {
                            ui.add(look::Button::new(format!("Item {i}")).width(280.));
                        }
                    });
                offsets[0] = outer.state.clamped_offset().y;
                scroll_geometry[0] = outer.inner_rect.height();
            },
        );
        let selection = self
            .ids
            .get("entry")
            .and_then(|id| egui::TextEdit::load_state(&ctx, *id))
            .and_then(|s| s.cursor.char_range())
            .map(|r| [r.primary.index.0, r.secondary.index.0]);
        let data = json!({"frame":ctx.cumulative_frame_nr(),"sequence":self.sequence,"radio":self.radio,"list_selected":self.list_selected,"list_activated":self.list_activated,"list_scroll":self.list_scroll,"dropdown_open":egui::Popup::is_any_open(&ctx),"scroll":offsets,"scroll_geometry":scroll_geometry,"selection":selection,"ime_output":ctx.output(|o|format!("{:?}",o.ime)),"window_focused":ctx.input(|i|i.focused),"dialog_text":self.dialog_entry,"dialog_text_focused":self.ids.get("dialog-entry").is_some_and(|id|ctx.memory(|m|m.has_focus(*id))),"check":self.check,"switch":self.switch,"entry":self.entry,"number":self.number,"slider":self.slider,"dropdown":self.dropdown,"button":self.button,"dialog_save":self.dialog_save,"dialog":self.dialog,"menu_selected":self.menu_selected,"widgets":widgets,"dark":ui.visuals().dark_mode,"accent":format!("{:?}",look::colors(&ctx).accent),"pixels_per_point":ctx.pixels_per_point(),"screen":[ctx.content_rect().width(),ctx.content_rect().height()]});
        std::fs::write(
            format!("{root}/egui-state.json"),
            serde_json::to_string_pretty(&data).unwrap(),
        )
        .unwrap();
        ctx.request_repaint_after(std::time::Duration::from_millis(30));
    }
}
fn main() -> eframe::Result {
    let dark = std::env::args().any(|x| x == "dark");
    eframe::run_native(
        "egui comparison",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([
                    std::env::var("TEST_WIDTH")
                        .ok()
                        .and_then(|s| s.parse::<f32>().ok())
                        .unwrap_or(900.),
                    std::env::var("TEST_HEIGHT")
                        .ok()
                        .and_then(|s| s.parse::<f32>().ok())
                        .unwrap_or(900.),
                ])
                .with_position([0., 0.])
                .with_decorations(false),
            ..Default::default()
        },
        Box::new(move |cc| {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "dejavu".into(),
                egui::FontData::from_owned(
                    std::fs::read("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf").unwrap(),
                )
                .into(),
            );
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "dejavu".into());
            fonts.font_data.insert(
                "dejavu-bold".into(),
                egui::FontData::from_owned(
                    std::fs::read("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf").unwrap(),
                )
                .into(),
            );
            fonts
                .families
                .insert(look::bold_family(), vec!["dejavu-bold".into()]);
            cc.egui_ctx.set_fonts(fonts);
            let mut theme = look::Theme::new(look::Look::Gtk);
            theme.accent = Some([0x3584e4, 0x3584e4]);
            theme.metrics = theme.metrics.system_bold();
            look::install(&cc.egui_ctx, theme);
            cc.egui_ctx.set_theme(if dark {
                egui::Theme::Dark
            } else {
                egui::Theme::Light
            });
            Ok(Box::new(App::default()))
        }),
    )
}
