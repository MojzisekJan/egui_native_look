//! Every control in every look: `cargo run --example gallery [mac|windows|gtk|ios|material|plain] [light]`.
//! The look and the scheme can be switched in the window too; `dialog` opens the dialog, `toast` shows a toast, `window` the tool window, `contrast` asks for more contrast. With `GALLERY_SHOT=file.png`
//! it saves its window to that file and quits. In the browser (`web/build.sh`) the same words go in the page's query: `?mac&light`.

use eframe::egui;
use egui_native_look as look;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let shot = std::env::var_os("GALLERY_SHOT").map(Into::into);
    eframe::run_native(
        "egui_native_look gallery",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1040.0, 800.0]),
            ..Default::default()
        },
        Box::new(move |cc| Ok(Box::new(Gallery::new(cc, &args, shot)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast;
    let window = web_sys::window().expect("a window");
    let query = window.location().search().unwrap_or_default();
    let args: Vec<String> = query
        .trim_start_matches('?')
        .split('&')
        .filter(|a| !a.is_empty())
        .map(String::from)
        .collect();
    let canvas = window
        .document()
        .and_then(|d| d.get_element_by_id("gallery"))
        .expect("the page has a #gallery canvas");
    let canvas = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("#gallery is a canvas");
    wasm_bindgen_futures::spawn_local(async move {
        eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(move |cc| Ok(Box::new(Gallery::new(cc, &args, None)))),
            )
            .await
            .expect("started the gallery");
    });
}

/// Whether the gallery was asked for more contrast (the word `contrast`).
static CONTRAST: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// `look` with its system's sizes and, where the system has it, its semibold; with more
/// contrast when asked.
fn theme(look: look::Look) -> look::Theme {
    let metrics = look::Metrics::of(look);
    let metrics = if look::system_fonts(look).is_some() {
        metrics.system_bold()
    } else {
        metrics
    };
    look::Theme::new(look)
        .metrics(metrics)
        .high_contrast(CONTRAST.load(std::sync::atomic::Ordering::Relaxed))
}

#[derive(Default)]
struct Gallery {
    look: usize,
    light: bool,
    on: bool,
    checked: bool,
    mixed: bool,
    mode: usize,
    size: usize,
    tab: usize,
    language: usize,
    delay: f64,
    volume: f64,
    name: String,
    query: String,
    picked: usize,
    added_items: Vec<String>,
    last_action: String,
    dialog: bool,
    details: bool,
    notifications: bool,
    tool: usize,
    retries: u32,
    startup: usize,
    tint: egui::Color32,
    units: usize,
    page: usize,
    speed: f32,
    items: u32,
    limits: [f32; 3],
    interval: f64,
    tool_window: bool,
    #[cfg_attr(
        target_arch = "wasm32",
        expect(dead_code, reason = "the browser takes no shots")
    )]
    shot: Option<std::path::PathBuf>,
    #[cfg_attr(
        target_arch = "wasm32",
        expect(dead_code, reason = "the browser takes no shots")
    )]
    frames: u32,
}

impl Gallery {
    /// The gallery as `args` ask for it: a look's name, `light`, `dialog`, `toast`, `window`.
    fn new(
        cc: &eframe::CreationContext,
        args: &[String],
        shot: Option<std::path::PathBuf>,
    ) -> Self {
        let has = |word: &str| args.iter().any(|a| a == word);
        CONTRAST.store(has("contrast"), std::sync::atomic::Ordering::Relaxed);
        let chosen = args
            .first()
            .and_then(|name| look::Look::from_name(name))
            .unwrap_or_else(|| {
                if look::touch_screen() {
                    look::Look::touch()
                } else {
                    look::Look::system()
                }
            });
        let light = has("light");
        let mut fonts = egui::FontDefinitions::default();
        look::install_fonts(&mut fonts, chosen);
        cc.egui_ctx.set_fonts(fonts);
        look::install(&cc.egui_ctx, theme(chosen));
        cc.egui_ctx.set_theme(if light {
            egui::Theme::Light
        } else {
            egui::Theme::Dark
        });
        if has("toast") {
            look::toast(&cc.egui_ctx, "Example notification");
        }
        Gallery {
            look: LOOKS.iter().position(|l| *l == chosen).unwrap_or(0),
            light,
            shot,
            delay: 8.0,
            volume: 40.0,
            retries: 4,
            details: true,
            tint: egui::Color32::from_rgb(0xe0, 0x8a, 0x3c),
            speed: 1.0,
            items: 16,
            limits: [0.0, 100.0, 1.0],
            interval: 250.0,
            tool_window: has("window"),
            dialog: has("dialog"),
            ..Default::default()
        }
    }
}

const LOOKS: [look::Look; 6] = [
    look::Look::Mac,
    look::Look::Windows,
    look::Look::Gtk,
    look::Look::Ios,
    look::Look::Material,
    look::Look::Plain,
];

/// Custom widgets allocate complete rectangles, so narrow rows stack explicitly.
/// A vertical scroll area that a touch screen scrolls by dragging from its first touch.
/// egui enables dragging only once it has seen a touch, which would leave the first
/// swipe on a phone without effect.
fn scroll_area(id: &str, touch: bool) -> egui::ScrollArea {
    let drag = if touch {
        egui::scroll_area::DragScroll::Always
    } else {
        egui::scroll_area::DragScroll::OnTouch
    };
    egui::ScrollArea::vertical()
        .id_salt(id)
        .scroll_source(egui::scroll_area::ScrollSource {
            drag,
            ..Default::default()
        })
}

fn responsive_row<R>(
    ui: &mut egui::Ui,
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    if ui.available_width() < 560.0 {
        ui.vertical(add)
    } else {
        ui.horizontal(add)
    }
}

impl eframe::App for Gallery {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        #[cfg(not(target_arch = "wasm32"))]
        self.save_shot(&ctx);
        look::follow_system(&ctx);
        let compact = ui.available_width() < 960.0;
        if !compact {
            egui::Panel::right("more")
                .exact_size(400.0)
                .frame(egui::Frame::central_panel(&ctx.global_style()))
                .show(ui, |ui| {
                    scroll_area("more-scroll", look::touch_screen()).show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);
                        self.more(ui);
                    });
                });
        }
        look::show_toasts(&ctx);
        egui::CentralPanel::default().show(ui, |ui| {
            scroll_area("gallery-scroll", look::touch_screen()).show(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);
                self.controls(ui);
                if compact {
                    ui.separator();
                    self.more(ui);
                }
            });
        });
    }
}

impl Gallery {
    fn notify(&mut self, ctx: &egui::Context, message: impl Into<String>) {
        self.last_action = message.into();
        look::toast(ctx, self.last_action.clone());
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        responsive_row(ui, |ui| {
            let labels: Vec<&str> = LOOKS.iter().map(|l| l.label()).collect();
            let changed = if ui.available_width() < 620.0 {
                ui.add(look::Dropdown::new("look", &mut self.look, &labels).width(180.0))
                    .changed()
            } else {
                ui.add(look::Segmented::new("look", &mut self.look, &labels).width(500.0))
                    .changed()
            };
            if changed {
                let mut fonts = egui::FontDefinitions::default();
                look::install_fonts(&mut fonts, LOOKS[self.look]);
                ctx.set_fonts(fonts);
                look::install(&ctx, theme(LOOKS[self.look]));
            }
            if ui
                .add(look::Switch::new(&mut self.light).accessible_label("Light"))
                .changed()
            {
                ctx.set_theme(if self.light {
                    egui::Theme::Light
                } else {
                    egui::Theme::Dark
                });
            }
            ui.label("Light");
        });
        ui.heading(LOOKS[self.look].label());
        responsive_row(ui, |ui| {
            if ui.add(look::Button::new("Button")).clicked() {
                self.notify(&ctx, "Button activated");
            }
            ui.add_enabled(false, look::Button::new("Disabled"));
            if ui.add(look::Button::new("Default").accent()).clicked() {
                self.notify(&ctx, "Primary action activated");
            }
        });
        responsive_row(ui, |ui| {
            let font = egui::FontId::proportional(16.0);
            if ui
                .add(look::Button::new("Add item").icon("+", font.clone()))
                .clicked()
            {
                let item = format!("Item {}", self.added_items.len() + 4);
                self.picked = self.added_items.len() + 3;
                self.added_items.push(item.clone());
                self.notify(&ctx, format!("Added {item}"));
            }
            if ui
                .add(look::Button::new("Save").icon("💾", font.clone()).accent())
                .clicked()
            {
                self.notify(&ctx, "Save activated");
            }
            ui.add_enabled(false, look::Button::new("Refresh").icon("🔄", font));
        });
        if !self.last_action.is_empty() {
            ui.label(format!("Last action: {}", self.last_action));
        }
        responsive_row(ui, |ui| {
            ui.add(look::Switch::new(&mut self.on).accessible_label("Switch"));
            ui.label("Switch");
            ui.add(look::Checkbox::new(&mut self.checked, "Check box"));
            let mixed = !self.mixed;
            ui.add(look::Checkbox::new(&mut self.mixed, "Mixed").indeterminate(mixed));
        });
        responsive_row(ui, |ui| {
            look::RadioGroup::new("mode", &mut self.mode)
                .accessible_label("Mode")
                .show(ui, |group| {
                    group.option(0, "Automatic");
                    group.option(1, "Manual");
                    group.option(2, "Custom");
                });
        });
        ui.add(
            look::Segmented::new("size", &mut self.size, &["Small", "Medium", "Large"])
                .width(240.0),
        );
        if ui.available_width() < 360.0 {
            ui.add(
                look::Dropdown::new(
                    "tabs",
                    &mut self.tab,
                    &["General", "Appearance", "Shortcuts"],
                )
                .width(240.0),
            );
        } else {
            ui.add(look::Tabs::new(
                "tabs",
                &mut self.tab,
                &["General", "Appearance", "Shortcuts"],
            ));
        }
        responsive_row(ui, |ui| {
            ui.add(
                look::Dropdown::new(
                    "language",
                    &mut self.language,
                    &["English", "Deutsch", "Français", "Čeština"],
                )
                .width(180.0),
            );
            ui.add(
                look::NumberField::new(&mut self.delay)
                    .range(0.0..=1024.0)
                    .accessible_label("Delay"),
            );
        });
        responsive_row(ui, |ui| {
            ui.add(
                look::Slider::new(&mut self.volume, 0.0..=100.0)
                    .width(200.0)
                    .accessible_label("Volume"),
            );
            ui.add(look::ProgressBar::new((self.volume / 100.0) as f32).width(140.0));
            ui.add(look::Spinner::new());
        });
        responsive_row(ui, |ui| {
            ui.add(
                look::TextField::new(&mut self.name)
                    .hint("Name")
                    .width(260.0),
            );
            if ui.add(look::Button::new("Dialog…")).clicked() {
                self.dialog = true;
            }
        });
        let mut save_confirmed = false;
        look::Dialog::new("dialog", "Save changes?")
            .size([
                380.0_f32.min((ctx.content_rect().width() - 32.0).max(0.0)),
                160.0,
            ])
            .show(
                &ctx,
                &mut self.dialog,
                |ui| {
                    ui.label("Save your changes before closing this dialog?");
                },
                |ui| {
                    let save = ui
                        .add(look::Button::new("Save").accent().default_action())
                        .clicked();
                    let cancel = ui.add(look::Button::new("Cancel")).clicked();
                    save_confirmed |= save;
                    if save || cancel {
                        ui.close();
                    }
                },
            );
        if save_confirmed {
            self.notify(&ctx, "Save confirmed");
        }
        responsive_row(ui, |ui| {
            let search = ui.add(
                look::TextField::new(&mut self.query)
                    .id_salt("gallery-search")
                    .accessible_label("Search items")
                    .hint("Search items")
                    .search()
                    .width(260.0),
            );
            if ui
                .add_enabled(!self.query.is_empty(), look::Button::new("Clear search"))
                .clicked()
            {
                self.query.clear();
                search.request_focus();
            }
        });
        let query = self.query.trim().to_lowercase();
        look::section(ui, "section", "Items", true, |ui| {
            let mut matches = 0;
            for (i, item) in ["First row", "Second row", "Third row"]
                .into_iter()
                .map(String::from)
                .chain(self.added_items.iter().cloned())
                .enumerate()
            {
                if !item.to_lowercase().contains(&query) {
                    continue;
                }
                matches += 1;
                // Filtering must not transfer a row's focus or identity to another item.
                let id = ui.make_persistent_id(("gallery-item", i));
                let response = ui
                    .scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
                        ui.add(look::ListRow::new(i == self.picked).accessible_label(&item))
                    })
                    .inner;
                let colour = if i == self.picked {
                    look::picked_text(ui)
                } else {
                    look::colors(ui.ctx()).fg
                };
                look::text(
                    ui.painter(),
                    response.rect.left_center() + egui::vec2(10.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    &item,
                    egui::FontId::proportional(13.0),
                    colour,
                );
                if response.clicked() {
                    self.picked = i;
                }
            }
            if matches == 0 {
                ui.label("No matching items");
            }
        });
    }
}

impl Gallery {
    /// The toolbar, a form, a colour well and a toast.
    fn more(&mut self, ui: &mut egui::Ui) {
        ui.heading("Toolbar");
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            for (i, (glyph, tip)) in [
                ("📂", "Open"),
                ("💾", "Save"),
                ("🔄", "Refresh"),
                ("…", "More"),
            ]
            .into_iter()
            .enumerate()
            {
                if ui
                    .add(
                        look::IconButton::new(glyph)
                            .selected(self.tool == i)
                            .tooltip(tip),
                    )
                    .clicked()
                {
                    self.tool = i;
                }
            }
            ui.add_space(8.0);
            if ui
                .add(
                    look::IconButton::new("#")
                        .selected(self.details)
                        .tooltip("Show details"),
                )
                .clicked()
            {
                self.details = !self.details;
            }
        });
        ui.heading("Form");
        look::Form::new("form").show(ui, |form| {
            let width = form.control_width();
            form.row("Delay", |ui| {
                ui.add(
                    look::NumberField::new(&mut self.delay)
                        .range(0.0..=1024.0)
                        .width(width),
                )
            });
            form.row("Retries", |ui| {
                ui.add(
                    look::NumberField::new(&mut self.retries)
                        .range(1.0..=64.0)
                        .width(width),
                )
            });
            form.row_with_tip(
                "On startup",
                "Choose what opens when the application starts",
                |ui| {
                    ui.add(
                        look::Dropdown::new(
                            "startup",
                            &mut self.startup,
                            &["Last session", "Home", "Blank document", "File picker"],
                        )
                        .width(width),
                    )
                },
            );
            form.row("Colour", |ui| ui.add(look::ColorWell::new(&mut self.tint)));
            form.row("Notifications", |ui| {
                ui.add(look::Switch::new(&mut self.notifications))
            });
        });
        responsive_row(ui, |ui| {
            if ui.add(look::Button::new("Show a toast")).clicked() {
                look::toast(ui.ctx(), "Example notification");
            }
            if ui.add(look::Button::new("Tool window")).clicked() {
                self.tool_window = !self.tool_window;
            }
        });
        ui.heading("Menus and lists");
        responsive_row(ui, |ui| {
            let names = ["Seconds", "Minutes", "Hours"];
            let units = &mut self.units;
            look::PopupButton::new("units", names[*units])
                .width(120.0)
                .show(ui, |ui| {
                    for (i, name) in names.into_iter().enumerate() {
                        look::menu_value(ui, units, i, name);
                    }
                });
            look::PopupButton::new("menu", "Menu").show(ui, |ui| {
                let icons = look::theme(ui.ctx()).metrics.body;
                if ui
                    .add(
                        look::MenuItem::new("Save")
                            .icon("💾", icons.clone())
                            .shortcut(look::shortcut_text(
                                look::look(ui.ctx()),
                                &egui::KeyboardShortcut::new(
                                    egui::Modifiers::COMMAND,
                                    egui::Key::S,
                                ),
                            )),
                    )
                    .clicked()
                {
                    self.notify(ui.ctx(), "Save activated");
                }
                look::MenuItem::new("Save as")
                    .icon("📄", icons)
                    .submenu(ui, |ui| {
                        for format in ["Text document", "PDF document"] {
                            if ui.add(look::MenuItem::new(format)).clicked() {
                                self.notify(ui.ctx(), format!("{format} selected"));
                            }
                        }
                    });
            });
            egui::ComboBox::new("egui-combo", "Time units")
                .selected_text(names[self.units])
                .icon(look::combo_icon)
                .show_ui(ui, |ui| {
                    for (i, name) in names.into_iter().enumerate() {
                        ui.selectable_value(&mut self.units, i, name);
                    }
                });
        });
        look::TabBar::new().show(ui, |bar| {
            for (i, name) in ["Overview", "Activity", "Settings"].into_iter().enumerate() {
                if bar.tab(name, self.page == i).clicked() {
                    self.page = i;
                }
            }
        });
        ui.add(
            look::Slider::new(&mut self.speed, 0.01..=100.0)
                .logarithmic(true)
                .width(160.0)
                .text("Speed"),
        );
        ui.add(
            look::Slider::new(&mut self.items, 1..=64)
                .accessible_label("Item count")
                .width(160.0)
                .show_value(true)
                .suffix(" items"),
        );
        ui.add(
            look::NumberField::new(&mut self.interval)
                .range(0.0..=10_000.0)
                .step(10.0)
                .suffix(" ms")
                .width(160.0),
        );
        ui.horizontal(|ui| {
            for (label, value) in ["Min ", "Max ", "Step "].into_iter().zip(&mut self.limits) {
                ui.add(
                    look::NumberField::new(value)
                        .steppers(false)
                        .prefix(label)
                        .width(70.0),
                );
            }
        });
        let ctx = ui.ctx().clone();
        look::ToolWindow::new("tool", "Preferences")
            .default_pos([660.0, 470.0])
            .show(&ctx, &mut self.tool_window, |ui| {
                look::Form::new("tool-form").show(ui, |form| {
                    form.row("Delay", |ui| {
                        ui.add(look::NumberField::new(&mut self.delay).range(0.0..=1024.0))
                    });
                    form.row("Retries", |ui| {
                        ui.add(look::NumberField::new(&mut self.retries).range(1.0..=64.0))
                    });
                });
            });
    }

    /// Saves the window to `shot` and quits; the browser has no files to save to.
    #[cfg(not(target_arch = "wasm32"))]
    fn save_shot(&mut self, ctx: &egui::Context) {
        let Some(path) = &self.shot else { return };
        self.frames += 1;
        ctx.request_repaint();
        if self.frames == 10 {
            let (path, ctx2) = (path.clone(), ctx.clone());
            ctx.request_screenshot(move |image| {
                let [w, h] = image.size;
                let bytes: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
                image::save_buffer(&path, &bytes, w as u32, h as u32, image::ColorType::Rgba8)
                    .expect("saved the shot");
                ctx2.send_viewport_cmd(egui::ViewportCommand::Close);
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn install_gallery_theme(ctx: &egui::Context, chosen: look::Look) {
        let mut fonts = egui::FontDefinitions::default();
        look::install_fonts(&mut fonts, chosen);
        ctx.set_fonts(fonts);
        look::install(ctx, theme(chosen));
    }

    fn interactive_gallery(
        chosen: look::Look,
        controls: bool,
    ) -> egui_kittest::Harness<'static, Gallery> {
        egui_kittest::Harness::builder()
            .with_size(egui::vec2(800.0, 1400.0))
            .with_context_setup(move |ctx| install_gallery_theme(ctx, chosen))
            .build_ui_state(
                move |ui, gallery: &mut Gallery| {
                    if controls {
                        gallery.controls(ui);
                    } else {
                        gallery.more(ui);
                    }
                },
                Gallery {
                    look: LOOKS.iter().position(|look| *look == chosen).unwrap(),
                    ..Default::default()
                },
            )
    }

    #[test]
    fn gallery_icon_actions_add_select_and_report_without_changing_other_controls() {
        use egui_kittest::kittest::Queryable;
        for chosen in LOOKS {
            let mut harness = interactive_gallery(chosen, true);
            harness.run_steps(3);
            harness.get_by_label("Add item").click();
            harness.run_steps(3);
            assert_eq!(harness.state().added_items, ["Item 4"]);
            assert_eq!(harness.state().picked, 3);
            assert_eq!(harness.state().last_action, "Added Item 4");
            harness.get_by_label("Add item").click();
            harness.run_steps(3);
            assert_eq!(harness.state().added_items, ["Item 4", "Item 5"]);
            assert_eq!(harness.state().picked, 4);
            harness.get_by_label("Save").click();
            harness.run_steps(3);
            assert_eq!(harness.state().last_action, "Save activated");
            assert_eq!(harness.state().added_items.len(), 2);
            assert!(!harness.state().checked);
        }
    }

    #[test]
    fn gallery_dialog_cancel_escape_and_enter_have_distinct_results() {
        use egui_kittest::kittest::Queryable;
        for chosen in LOOKS {
            let mut harness = interactive_gallery(chosen, true);
            harness.run_steps(3);
            harness.get_by_label("Dialog…").click();
            harness.run_steps(3);
            assert!(harness.state().dialog);
            harness.get_by_label("Cancel").click();
            harness.run_steps(3);
            assert!(!harness.state().dialog);
            assert!(harness.state().last_action.is_empty());
            harness.get_by_label("Dialog…").click();
            harness.run_steps(3);
            harness.key_press(egui::Key::Escape);
            harness.run_steps(3);
            assert!(!harness.state().dialog);
            assert!(harness.state().last_action.is_empty());
            harness.get_by_label("Dialog…").click();
            harness.run_steps(3);
            harness.key_press(egui::Key::Enter);
            harness.run_steps(3);
            assert!(!harness.state().dialog, "{chosen}");
            assert_eq!(harness.state().last_action, "Save confirmed", "{chosen}");
        }
    }

    #[test]
    fn gallery_menu_actions_report_selection_and_close_the_menu_chain() {
        use egui_kittest::kittest::Queryable;
        for chosen in LOOKS {
            let mut harness = interactive_gallery(chosen, false);
            harness.run_steps(3);
            harness.get_by_label("Menu").click();
            harness.run_steps(3);
            harness
                .get_by_role_and_label(egui::accesskit::Role::MenuItem, "Save")
                .click();
            harness.run_steps(3);
            assert_eq!(harness.state().last_action, "Save activated");
            assert!(!egui::Popup::is_any_open(&harness.ctx));
            for format in ["Text document", "PDF document"] {
                harness.get_by_label("Menu").click();
                harness.run_steps(3);
                harness.get_by_label("Save as").click();
                harness.run_steps(3);
                harness.get_by_label(format).click();
                harness.run_steps(3);
                assert_eq!(harness.state().last_action, format!("{format} selected"));
                assert!(!egui::Popup::is_any_open(&harness.ctx), "{chosen}");
            }
        }
    }

    #[test]
    fn gallery_search_filters_typed_text_and_keeps_row_identity_and_selection() {
        use egui_kittest::kittest::{NodeT, Queryable};
        for chosen in LOOKS {
            let mut harness = interactive_gallery(chosen, true);
            harness.run_steps(3);
            let second = harness.get_by_label("Second row").accesskit_node().id();
            harness.get_by_label("Search items").click();
            harness.run_steps(3);
            harness.event(egui::Event::Text("  sEcOnD  ".into()));
            harness.run_steps(3);
            assert_eq!(harness.state().query, "  sEcOnD  ");
            assert!(harness.query_by_label("First row").is_none());
            assert!(harness.query_by_label("Third row").is_none());
            assert_eq!(
                harness.get_by_label("Second row").accesskit_node().id(),
                second
            );
            assert_eq!(harness.state().picked, 0);
            harness.get_by_label("Second row").click();
            harness.run_steps(3);
            assert_eq!(harness.state().picked, 1);
            harness.get_by_label("Clear search").click();
            harness.run_steps(3);
            assert!(harness.state().query.is_empty());
            assert!(harness
                .get_by_label("Search items")
                .accesskit_node()
                .is_focused());
            assert!(harness.query_by_label("First row").is_some());
            assert_eq!(
                harness.get_by_label("Second row").accesskit_node().id(),
                second
            );
            assert_eq!(harness.state().picked, 1);
        }
    }

    #[test]
    fn gallery_search_handles_unicode_empty_results_and_added_items() {
        use egui_kittest::kittest::Queryable;
        for chosen in LOOKS {
            let mut harness = interactive_gallery(chosen, true);
            harness.state_mut().added_items = vec!["Žlutý dokument".into(), "Invoice 2026".into()];
            harness.state_mut().picked = 4;
            harness.state_mut().query = "  ŽLUTÝ ".into();
            harness.run_steps(3);
            assert!(harness.query_by_label("Žlutý dokument").is_some());
            assert!(harness.query_by_label("Invoice 2026").is_none());
            assert_eq!(harness.state().picked, 4);
            harness.get_by_label("Žlutý dokument").click();
            harness.run_steps(3);
            assert_eq!(harness.state().picked, 3);
            harness.state_mut().query = "no such item".into();
            harness.run_steps(3);
            assert!(harness.query_by_label("No matching items").is_some());
            assert!(harness.query_by_label("Žlutý dokument").is_none());
            assert_eq!(harness.state().picked, 3);
            harness.state_mut().query = "   ".into();
            harness.run_steps(3);
            assert!(harness.query_by_label("No matching items").is_none());
            assert!(harness.query_by_label("First row").is_some());
            assert!(harness.query_by_label("Invoice 2026").is_some());
            assert_eq!(harness.state().picked, 3);
        }
    }

    #[test]
    fn adding_an_item_updates_active_search_results() {
        use egui_kittest::kittest::Queryable;
        for chosen in LOOKS {
            let mut harness = interactive_gallery(chosen, true);
            harness.state_mut().query = "Item".into();
            harness.run_steps(3);
            assert!(harness.query_by_label("No matching items").is_some());
            harness.get_by_label("Add item").click();
            harness.run_steps(3);
            assert_eq!(harness.state().query, "Item");
            assert_eq!(harness.state().picked, 3);
            assert!(harness.query_by_label("Item 4").is_some());
            assert!(harness.query_by_label("No matching items").is_none());
        }
    }

    #[test]
    fn gallery_sections_fit_phone_widths() {
        for touch_mode in [false, true] {
            for width in [288.0, 320.0, 360.0, 390.0, 768.0] {
                for chosen in LOOKS {
                    let ctx = egui::Context::default();
                    install_gallery_theme(&ctx, chosen);
                    let mut gallery = Gallery {
                        look: LOOKS.iter().position(|l| *l == chosen).unwrap(),
                        ..Default::default()
                    };
                    if touch_mode {
                        let pos = egui::pos2(1000.0, 1000.0);
                        let mut output = ctx.run_ui(
                            egui::RawInput {
                                events: [egui::TouchPhase::Start, egui::TouchPhase::End]
                                    .into_iter()
                                    .map(|phase| egui::Event::Touch {
                                        device_id: egui::TouchDeviceId(1),
                                        id: egui::TouchId(1),
                                        phase,
                                        pos,
                                        force: None,
                                    })
                                    .collect(),
                                ..Default::default()
                            },
                            |_| {},
                        );
                        output.textures_delta.clear();
                    }
                    for _ in 0..3 {
                        let mut output = ctx.run_ui(
                            egui::RawInput {
                                screen_rect: Some(egui::Rect::from_min_size(
                                    egui::Pos2::ZERO,
                                    egui::vec2(width, 640.0),
                                )),
                                ..Default::default()
                            },
                            |ui| {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    let available = ui.available_width();
                                    gallery.controls(ui);
                                    assert!(
                                        ui.min_rect().width() <= available + 1.0,
                                        "controls {chosen:?} at {width}: {}",
                                        ui.min_rect().width()
                                    );
                                    gallery.more(ui);
                                    assert!(
                                        ui.min_rect().width() <= available + 1.0,
                                        "{chosen:?} at {width}: content {} > {available}",
                                        ui.min_rect().width()
                                    );
                                });
                            },
                        );
                        output.textures_delta.clear();
                    }
                }
            }
        }
    }
    #[test]
    fn the_first_swipe_on_a_touch_screen_scrolls_the_phone_gallery() {
        use egui_kittest::kittest::Queryable;
        for chosen in LOOKS {
            // The harness also checks that every control in the phone layout has a name.
            let mut harness = egui_kittest::Harness::builder()
                .with_size(egui::vec2(390.0, 844.0))
                .with_context_setup(move |ctx| install_gallery_theme(ctx, chosen))
                .build_ui_state(
                    |ui, gallery: &mut Gallery| {
                        scroll_area("gallery-scroll", true).show(ui, |ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);
                            gallery.controls(ui);
                            ui.separator();
                            gallery.more(ui);
                        });
                    },
                    Gallery {
                        look: LOOKS.iter().position(|look| *look == chosen).unwrap(),
                        ..Default::default()
                    },
                );
            harness.run_steps(3);
            let before = harness.get_by_label("Button").rect().top();
            let x = 360.0;
            let touch = |phase, y: f32| egui::Event::Touch {
                device_id: egui::TouchDeviceId(1),
                id: egui::TouchId(1),
                phase,
                pos: egui::pos2(x, y),
                force: None,
            };
            let button = |y: f32, pressed| egui::Event::PointerButton {
                pos: egui::pos2(x, y),
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            };
            harness.input_mut().events.extend([
                touch(egui::TouchPhase::Start, 700.0),
                egui::Event::PointerMoved(egui::pos2(x, 700.0)),
                button(700.0, true),
            ]);
            harness.step();
            for step in 1..=10 {
                let y = 700.0 - 40.0 * step as f32;
                harness.input_mut().events.extend([
                    touch(egui::TouchPhase::Move, y),
                    egui::Event::PointerMoved(egui::pos2(x, y)),
                ]);
                harness.step();
            }
            harness.input_mut().events.extend([
                touch(egui::TouchPhase::End, 300.0),
                button(300.0, false),
                egui::Event::PointerGone,
            ]);
            harness.run_steps(3);
            let after = harness.get_by_label("Button").rect().top();
            assert!(
                before - after > 200.0,
                "{chosen}: the first swipe moved the content from {before} to {after}"
            );
        }
    }
}
