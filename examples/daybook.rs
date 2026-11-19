//! A working task, notes and focus demo. Run `cargo run --example daybook`.
use eframe::egui::{self, Color32, RichText};
use egui_native_look as look;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

const PROJECTS: [&str; 3] = ["Design studio", "Engineering", "Personal"];
const PRIORITIES: [&str; 3] = ["High", "Medium", "Low"];
#[derive(Clone, Serialize, Deserialize)]
struct Task {
    id: u64,
    title: String,
    project: usize,
    done: bool,
    priority: usize,
}
#[derive(Serialize, Deserialize)]
struct Document {
    tasks: Vec<Task>,
    notes: String,
    minutes: u32,
    dark: bool,
    look_index: usize,
}
impl Document {
    fn seed() -> Self {
        serde_json::from_str(include_str!("daybook/seed.json")).unwrap()
    }
    fn normalize(&mut self) {
        self.minutes = self.minutes.clamp(5, 60);
        self.look_index = self.look_index.min(1);
        for task in &mut self.tasks {
            task.project = task.project.min(2);
            task.priority = task.priority.min(2);
        }
    }
    fn add(&mut self, title: &str, project: usize, priority: usize) -> bool {
        let title = title.trim();
        if title.is_empty() {
            return false;
        }
        let id = self
            .tasks
            .iter()
            .map(|t| t.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        self.tasks.push(Task {
            id,
            title: title.into(),
            project: project.min(2),
            done: false,
            priority: priority.min(2),
        });
        true
    }
}
struct Daybook {
    data: Document,
    page: usize,
    filter: usize,
    search: String,
    draft: String,
    project: usize,
    priority: usize,
    adding: bool,
    deleting: Option<u64>,
    remaining: Duration,
    started: Option<Instant>,
    message: String,
    path: Option<PathBuf>,
    serialized: String,
    save_error: Option<String>,
    shot: Option<PathBuf>,
    frames: u32,
}
fn data_path() -> PathBuf {
    if let Some(path) = std::env::var_os("DAYBOOK_DATA") {
        return path.into();
    }
    let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_else(|| ".".into()));
    if cfg!(any(target_os = "macos", target_os = "ios")) {
        home.join("Library/Application Support/Daybook/egui.json")
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/share"))
            .join("daybook/egui.json")
    }
}
fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}
fn card(ui: &egui::Ui) -> egui::Frame {
    egui::Frame::new()
        .fill(if ui.visuals().dark_mode {
            rgb(35, 38, 43)
        } else {
            Color32::WHITE
        })
        .stroke(egui::Stroke::new(
            1.,
            if ui.visuals().dark_mode {
                rgb(56, 60, 67)
            } else {
                rgb(229, 232, 236)
            },
        ))
        .corner_radius(14)
        .inner_margin(20)
}
// Measure the whole text column before placing siblings. Immediate-mode horizontal
// layout cannot retroactively re-center an earlier checkbox when a later label grows.
fn task_row(ui: &mut egui::Ui, task: &mut Task) -> bool {
    let metrics = look::theme(ui.ctx()).metrics;
    let remove_width = ui
        .fonts_mut(|f| {
            f.layout_no_wrap(
                "Remove".into(),
                metrics.body.clone(),
                ui.visuals().text_color(),
            )
            .size()
            .x
        })
        .ceil()
        + 24.0;
    let checkbox_width = if look::look(ui.ctx()) == look::Look::Ios {
        28.0
    } else {
        21.0
    };
    let gap = 8.0;
    let text_width = (ui.available_width() - checkbox_width - remove_width - gap * 2.0).max(32.0);
    let text = RichText::new(&task.title).size(14.0);
    let text = if task.done {
        text.strikethrough().weak()
    } else {
        text
    };
    let title = egui::WidgetText::from(text).into_galley(
        ui,
        Some(egui::TextWrapMode::Wrap),
        text_width,
        egui::FontSelection::Default,
    );
    let mut metadata = egui::text::LayoutJob::default();
    metadata.wrap.max_width = text_width;
    metadata.append(
        PROJECTS[task.project],
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::proportional(11.0),
            color: ui.visuals().weak_text_color(),
            ..Default::default()
        },
    );
    metadata.append(
        &format!("  · {} priority", PRIORITIES[task.priority]),
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::proportional(11.0),
            color: if task.priority == 0 {
                rgb(192, 103, 52)
            } else {
                ui.visuals().weak_text_color()
            },
            ..Default::default()
        },
    );
    let metadata = ui.fonts_mut(|f| f.layout_job(metadata));
    let text_height = title.size().y + 5.0 + metadata.size().y;
    let height = text_height.max(ui.spacing().interact_size.y);
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), height),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = gap;
            let completion = ui.add(look::Checkbox::new(&mut task.done, ""));
            completion.widget_info(|| {
                egui::WidgetInfo::selected(egui::Role::CheckBox, true, task.done, &task.title)
            });
            completion.on_hover_text("Mark task complete");
            ui.allocate_ui_with_layout(
                egui::vec2(text_width, text_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.add(egui::Label::new(title));
                    ui.add_space(5.0);
                    ui.add(egui::Label::new(metadata));
                },
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(look::Button::new("Remove").width(remove_width))
                    .clicked()
            })
            .inner
        },
    )
    .inner
}

impl Daybook {
    fn new(ctx: &egui::Context) -> Self {
        let demo = std::env::args().any(|a| a == "--demo");
        let path = (!demo).then(data_path);
        let mut save_error = None;
        let mut data = match path.as_ref().map(std::fs::read_to_string) {
            Some(Ok(text)) => serde_json::from_str(&text).unwrap_or_else(|e| {
                save_error = Some(format!(
                    "Could not read saved data: {e}. Saving is disabled to protect it."
                ));
                Document::seed()
            }),
            Some(Err(e)) if e.kind() != std::io::ErrorKind::NotFound => {
                save_error = Some(format!("Could not read saved data: {e}"));
                Document::seed()
            }
            _ => Document::seed(),
        };
        data.normalize();
        if cfg!(target_os = "ios") || std::env::args().any(|a| a == "--ios") {
            data.look_index = 1;
        }
        if std::env::args().any(|a| a == "--dark") {
            data.dark = true;
        }
        let serialized = serde_json::to_string(&data).unwrap();
        let app = Self {
            remaining: Duration::from_secs(u64::from(data.minutes) * 60),
            data,
            page: 0,
            filter: 0,
            search: String::new(),
            draft: String::new(),
            project: 0,
            priority: 1,
            adding: false,
            deleting: None,
            started: None,
            message: "One thing at a time.".into(),
            path: if save_error.is_none() { path } else { None },
            serialized,
            save_error,
            shot: std::env::var_os("DAYBOOK_SHOT").map(Into::into),
            frames: 0,
        };
        app.apply_theme(ctx);
        app
    }
    fn apply_theme(&self, ctx: &egui::Context) {
        Self::configure_theme(ctx, self.data.look_index, self.data.dark);
    }
    fn configure_theme(ctx: &egui::Context, look_index: usize, dark: bool) {
        let chosen = if look_index == 0 {
            look::Look::Mac
        } else {
            look::Look::Ios
        };
        let mut fonts = egui::FontDefinitions::default();
        look::install_fonts(&mut fonts, chosen);
        let mut heading_family = fonts
            .families
            .get(&look::bold_family())
            .cloned()
            .unwrap_or_default();
        if let Some(system) = look::system_fonts(chosen) {
            let mut heading = system.regular;
            heading.tweak.coords =
                egui::epaint::text::VariationCoords::new([(b"wght", 700.0), (b"opsz", 28.0)]);
            fonts
                .font_data
                .insert("daybook-heading".into(), heading.into());
            heading_family.insert(0, "daybook-heading".into());
        }
        fonts.families.insert(
            egui::FontFamily::Name("daybook-heading".into()),
            heading_family,
        );
        ctx.set_fonts(fonts);
        look::install(
            ctx,
            look::Theme::new(chosen).metrics(look::Metrics::of(chosen).system_bold()),
        );
        ctx.set_theme(if dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        });
    }
    fn seconds(&self) -> u64 {
        self.remaining
            .saturating_sub(self.started.map(|t| t.elapsed()).unwrap_or_default())
            .as_secs()
    }
    fn save(&mut self) {
        let text = serde_json::to_string(&self.data).unwrap();
        if text == self.serialized {
            return;
        }
        if let Some(path) = &self.path {
            let result = (|| {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let tmp = path.with_extension("tmp");
                std::fs::write(&tmp, &text)?;
                std::fs::rename(tmp, path)
            })();
            if let Err(e) = result {
                self.save_error = Some(format!("Changes could not be saved: {e}"));
                return;
            }
            self.save_error = None;
        }
        self.serialized = text;
    }
    fn sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(20.);
        ui.label(
            RichText::new("daybook.")
                .size(28.)
                .strong()
                .family(egui::FontFamily::Name("daybook-heading".into())),
        );
        ui.add_space(3.0);
        ui.label(RichText::new("A little more intention.").size(12.).weak());
        ui.add_space(36.);
        ui.label(
            RichText::new("WORKSPACE")
                .size(10.)
                .strong()
                .family(look::bold_family())
                .color(ui.visuals().weak_text_color()),
        );
        ui.add_space(8.);
        ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = 32.0;
            look::ListBox::new("navigation", &mut self.page)
                .accessible_label("Workspace")
                .show(ui, |list| {
                    list.item(0, "My day");
                    list.item(1, "All tasks");
                    list.item(2, "Notes");
                });
        });
        ui.add_space(30.);
        ui.label(
            RichText::new("PROJECTS")
                .size(10.)
                .strong()
                .family(look::bold_family())
                .color(ui.visuals().weak_text_color()),
        );
        ui.add_space(8.);
        ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = 32.0;
            look::ListBox::new("projects", &mut self.page)
                .accessible_label("Projects")
                .show(ui, |list| {
                    for (i, p) in PROJECTS.iter().enumerate() {
                        list.item(i + 3, *p);
                    }
                });
        });
        ui.add_space(40.);
        ui.separator();
        ui.add_space(14.);
        ui.label(
            RichText::new("Appearance")
                .strong()
                .family(look::bold_family()),
        );
        ui.add_space(8.);
        if ui
            .add(
                look::Segmented::new("look", &mut self.data.look_index, &["macOS", "iOS"])
                    .width(ui.available_width()),
            )
            .changed()
        {
            self.apply_theme(ui.ctx());
        }
        ui.add_space(10.);
        ui.horizontal(|ui| {
            ui.label("Dark mode");
            if ui
                .add(look::Switch::new(&mut self.data.dark).accessible_label("Dark mode"))
                .changed()
            {
                self.apply_theme(ui.ctx());
            }
        });
        ui.add_space(22.);
        ui.label(
            RichText::new("Built with egui_native_look")
                .size(10.)
                .weak(),
        );
    }
    fn focus_card(&mut self, ui: &mut egui::Ui) {
        card(ui).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new("TIME FOR FOCUS")
                    .size(10.)
                    .strong()
                    .family(look::bold_family())
                    .color(ui.visuals().weak_text_color()),
            );
            ui.add_space(12.);
            ui.label(
                RichText::new("Make room for deep work.")
                    .size(15.)
                    .strong()
                    .family(look::bold_family()),
            );
            ui.add_space(12.);
            let sec = self.seconds();
            ui.label(RichText::new(format!("{:02}:{:02}", sec / 60, sec % 60)).size(44.));
            ui.add_space(12.);
            ui.label(RichText::new(&self.message).size(12.).weak());
            ui.add_space(16.);
            ui.horizontal(|ui| {
                if ui
                    .add(
                        look::Button::new(if self.started.is_some() {
                            "Pause"
                        } else {
                            "Start focus"
                        })
                        .accent(),
                    )
                    .clicked()
                {
                    if let Some(start) = self.started.take() {
                        self.remaining = self.remaining.saturating_sub(start.elapsed());
                        self.message = "Take a breath. Resume when ready.".into();
                    } else {
                        if self.remaining.is_zero() {
                            self.remaining = Duration::from_secs(u64::from(self.data.minutes) * 60);
                        }
                        self.started = Some(Instant::now());
                        self.message = "You have this. Keep going.".into();
                    }
                }
                if ui.add(look::Button::new("Reset")).clicked() {
                    self.started = None;
                    self.remaining = Duration::from_secs(u64::from(self.data.minutes) * 60);
                    self.message = "One thing at a time.".into();
                }
            });
            ui.add_space(16.);
            ui.horizontal(|ui| {
                ui.label("Minutes");
                if ui
                    .add_enabled(
                        self.started.is_none(),
                        look::NumberField::new(&mut self.data.minutes)
                            .accessible_label("Focus minutes")
                            .range(5..=60)
                            .width(90.),
                    )
                    .changed()
                {
                    self.remaining = Duration::from_secs(u64::from(self.data.minutes) * 60);
                }
            });
        });
    }
    fn notes(&mut self, ui: &mut egui::Ui, large: bool) {
        card(ui).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new("Scratchpad")
                    .size(18.)
                    .strong()
                    .family(look::bold_family()),
            );
            ui.add_space(12.);
            ui.label(RichText::new("Ideas worth keeping.").size(12.).weak());
            ui.add_space(12.);
            let editor_height = if large { 360.0 } else { 170.0 };
            look::ScrollArea::vertical()
                .id_salt(("notes-editor", large))
                .scroll_bar_visibility(if look::look(ui.ctx()) == look::Look::Mac {
                    egui::scroll_area::ScrollBarVisibility::AlwaysVisible
                } else {
                    egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded
                })
                .max_height(editor_height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.data.notes)
                            .desired_width(f32::INFINITY)
                            .desired_rows(1)
                            .min_size(egui::vec2(0.0, editor_height))
                            .font(egui::FontId::proportional(13.0))
                            .frame(egui::Frame::NONE)
                            .margin(5)
                            .hint_text("Start writing…"),
                    );
                });
            ui.add_space(12.);
            ui.label(
                RichText::new(if self.path.is_some() {
                    "Saved on this device"
                } else {
                    "Preview session"
                })
                .size(10.)
                .weak(),
            );
        });
    }
    fn content(&mut self, ui: &mut egui::Ui) {
        let title = match self.page {
            0 => "Make today count.",
            1 => "Everything, in one place.",
            2 => "A place for your ideas.",
            p => PROJECTS[(p - 3).min(2)],
        };
        ui.label(
            RichText::new("YOUR PERSONAL WORKSPACE")
                .size(10.)
                .strong()
                .family(look::bold_family())
                .color(ui.visuals().weak_text_color()),
        );
        ui.add_space(8.);
        ui.label(
            RichText::new(title)
                .size(30.)
                .strong()
                .family(egui::FontFamily::Name("daybook-heading".into())),
        );
        ui.add_space(5.);
        ui.label(
            RichText::new("Small steps. Thoughtful work. A clearer day.")
                .size(13.)
                .weak(),
        );
        ui.add_space(26.);
        let completed = self.data.tasks.iter().filter(|t| t.done).count();
        let total = self.data.tasks.len();
        let compact_metrics = ui.available_width() < 480.0;
        ui.columns(3, |cols| {
            for (i, (label, value)) in [
                ("TASKS DONE", format!("{completed} / {total}")),
                ("FOCUS SESSION", format!("{} min", self.data.minutes)),
                ("YOUR PROJECTS", "3".into()),
            ]
            .into_iter()
            .enumerate()
            {
                card(&cols[i])
                    .inner_margin(if compact_metrics { 12 } else { 20 })
                    .show(&mut cols[i], |ui| {
                        ui.set_width(ui.available_width());
                        ui.add(egui::Label::new(RichText::new(label).size(10.).weak()).truncate());
                        ui.add_space(8.);
                        ui.label(
                            RichText::new(value)
                                .size(if compact_metrics { 22. } else { 24. })
                                .strong()
                                .family(look::bold_family()),
                        );
                    });
            }
        });
        ui.add_space(24.);
        if self.page == 2 {
            self.notes(ui, true);
            return;
        }
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Your tasks")
                    .size(20.)
                    .strong()
                    .family(look::bold_family()),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let response = ui.add(look::Button::new("+ New task").accent());
                if response.clicked() {
                    self.adding = true;
                    self.project = self.page.saturating_sub(3).min(2);
                }
            });
        });
        ui.add_space(12.);
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(12.0, 12.0);
            ui.horizontal_wrapped(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(210.0, ui.spacing().interact_size.y),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        let picker = look::Segmented::new(
                            "filter",
                            &mut self.filter,
                            &["All", "Active", "Done"],
                        );
                        ui.add(if look::look(ui.ctx()) == look::Look::Mac {
                            picker
                        } else {
                            picker.width(210.0)
                        });
                    },
                );
                ui.add(
                    look::TextField::new(&mut self.search)
                        .hint("Search tasks")
                        .accessible_label("Search tasks")
                        .width(180.),
                );
            });
        });
        ui.add_space(14.);
        let query = self.search.to_lowercase();
        let mut visible = 0;
        let filtered: Vec<usize> = self
            .data
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                (self.page < 3 || t.project == self.page - 3)
                    && (self.filter == 0 || (self.filter == 2) == t.done)
                    && t.title.to_lowercase().contains(&query)
            })
            .map(|(i, _)| i)
            .collect();
        for index in filtered {
            let task = &mut self.data.tasks[index];
            visible += 1;
            ui.push_id(task.id, |ui| {
                card(ui).inner_margin(14).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    if task_row(ui, task) {
                        self.deleting = Some(task.id);
                    }
                });
            });
            ui.add_space(8.);
        }
        if visible == 0 {
            ui.add_space(25.);
            ui.label(
                RichText::new("A clear page.")
                    .size(20.)
                    .strong()
                    .family(look::bold_family()),
            );
            ui.label("No tasks match this view. Add a task or adjust your filters.");
        }
        ui.add_space(12.);
        ui.label(
            RichText::new(format!(
                "{completed} of {total} completed — keep the momentum."
            ))
            .size(11.)
            .weak(),
        );
        if total > 0 {
            ui.add_space(8.);
            ui.add(
                look::ProgressBar::new(completed as f32 / total as f32)
                    .width(ui.available_width())
                    .accessible_label("Completed tasks"),
            );
        }
    }
    fn dialogs(&mut self, ctx: &egui::Context) {
        let mut open = self.adding;
        let mut add = false;
        let can_add = std::cell::Cell::new(!self.draft.trim().is_empty());
        look::Dialog::new("new-task", "Create a task")
            .size(egui::vec2(420., 230.))
            .show(
                ctx,
                &mut open,
                |ui| {
                    ui.label("What would you like to move forward?");
                    ui.add_space(10.);
                    ui.add(
                        look::TextField::new(&mut self.draft)
                            .hint("Task title")
                            .accessible_label("Task title")
                            .width(ui.available_width()),
                    );
                    can_add.set(!self.draft.trim().is_empty());
                    ui.add_space(12.);
                    ui.horizontal(|ui| {
                        ui.label("Project");
                        ui.add(
                            look::Dropdown::new("project", &mut self.project, &PROJECTS)
                                .width(180.),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Priority");
                        ui.add(
                            look::Segmented::new("priority", &mut self.priority, &PRIORITIES)
                                .width(220.),
                        );
                    });
                },
                |ui| {
                    if ui.add(look::Button::new("Cancel")).clicked() {
                        ui.close();
                    }
                    if ui
                        .add_enabled(
                            can_add.get(),
                            look::Button::new("Add task").accent().default_action(),
                        )
                        .clicked()
                    {
                        add = true;
                        ui.close();
                    }
                },
            );
        self.adding = open;
        if add && self.data.add(&self.draft, self.project, self.priority) {
            self.draft.clear();
            self.filter = 0;
            self.search.clear();
            self.page = 1;
        }
        if let Some(id) = self.deleting {
            let mut open = true;
            let mut remove = false;
            look::Dialog::new("remove-task", "Remove this task?").show(
                ctx,
                &mut open,
                |ui| {
                    ui.label("This task will be removed from your workspace.");
                },
                |ui| {
                    if ui.add(look::Button::new("Keep task")).clicked() {
                        ui.close();
                    }
                    if ui.add(look::Button::new("Remove").destructive()).clicked() {
                        remove = true;
                        ui.close();
                    }
                },
            );
            if remove {
                self.data.tasks.retain(|t| t.id != id);
            }
            if !open {
                self.deleting = None;
            }
        }
    }
}
impl Daybook {
    fn render(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        // Explicit gaps below mirror SwiftUI spacing; avoid adding egui defaults twice.
        ui.spacing_mut().item_spacing = egui::vec2(8., 0.);
        if self.started.is_some() {
            ctx.request_repaint_after(Duration::from_millis(200));
            if self.seconds() == 0 {
                self.started = None;
                self.remaining = Duration::ZERO;
                self.message = "Session complete. Time for a break.".into();
            }
        }
        let wide = ui.available_width() >= 980.;
        let sidebar = ui.available_width() >= 720.;
        if sidebar {
            egui::Panel::left("sidebar")
                .exact_size(210.)
                .resizable(false)
                .frame(
                    egui::Frame::new()
                        .fill(if self.data.dark {
                            rgb(27, 30, 34)
                        } else {
                            rgb(239, 242, 246)
                        })
                        .inner_margin(22),
                )
                .show(ui, |ui| {
                    self.sidebar(ui);
                });
        }
        let background = if self.data.dark {
            rgb(22, 24, 28)
        } else {
            rgb(248, 249, 251)
        };
        if wide {
            egui::Panel::right("inspector")
                .exact_size(300.)
                .resizable(false)
                .frame(egui::Frame::new().fill(background).inner_margin(20))
                .show(ui, |ui| {
                    look::ScrollArea::vertical().show(ui, |ui| {
                        ui.add_space(20.);
                        self.focus_card(ui);
                        ui.add_space(18.);
                        self.notes(ui, false);
                    });
                });
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(background))
            .show(ui, |ui| {
                look::ScrollArea::vertical().show(ui, |ui| {
                    egui::Frame::new().inner_margin(28).show(ui, |ui| {
                        if !sidebar {
                            ui.horizontal(|ui| {
                                let labels = [
                                    "My day",
                                    "All tasks",
                                    "Notes",
                                    PROJECTS[0],
                                    PROJECTS[1],
                                    PROJECTS[2],
                                ];
                                let font = look::theme(ui.ctx()).metrics.body;
                                let width = ui.fonts_mut(|fonts| {
                                    fonts
                                        .layout_no_wrap(
                                            labels[self.page.min(5)].into(),
                                            font,
                                            ui.visuals().text_color(),
                                        )
                                        .size()
                                        .x
                                }) + 48.0;
                                ui.add(
                                    look::Dropdown::new(
                                        "workspace",
                                        &mut self.page,
                                        &[
                                            "My day",
                                            "All tasks",
                                            "Notes",
                                            PROJECTS[0],
                                            PROJECTS[1],
                                            PROJECTS[2],
                                        ],
                                    )
                                    .accessible_label("Workspace")
                                    .width(width),
                                );
                                ui.label("Dark");
                                ui.add_space((ui.available_width() - 61.0).max(0.0));
                                if ui
                                    .add(
                                        look::Switch::new(&mut self.data.dark)
                                            .accessible_label("Dark mode"),
                                    )
                                    .changed()
                                {
                                    self.apply_theme(&ctx);
                                }
                            });
                            ui.add_space(24.);
                        }
                        self.content(ui);
                        if !wide {
                            ui.add_space(24.);
                            self.focus_card(ui);
                            if self.page != 2 {
                                ui.add_space(18.);
                                self.notes(ui, false);
                            }
                        }
                        if let Some(e) = &self.save_error {
                            ui.colored_label(rgb(200, 70, 50), e);
                        }
                    });
                });
            });
        self.dialogs(&ctx);
        self.save();
        if let Some(path) = &self.shot {
            self.frames += 1;
            ctx.request_repaint();
            if self.frames == 12 {
                let path = path.clone();
                let closing = ctx.clone();
                ctx.request_screenshot(move |image| {
                    let [w, h] = image.size;
                    let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                    image::save_buffer(path, &bytes, w as u32, h as u32, image::ColorType::Rgba8)
                        .expect("save screenshot");
                    closing.send_viewport_cmd(egui::ViewportCommand::Close);
                });
            }
        }
    }
}
impl eframe::App for Daybook {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        #[cfg(target_os = "ios")]
        {
            let bounds = ui.ctx().content_rect();
            let background = if self.data.dark {
                rgb(22, 24, 28)
            } else {
                rgb(248, 249, 251)
            };
            ui.painter()
                .rect_filled(ui.ctx().viewport_rect(), 0.0, background);
            ui.scope_builder(egui::UiBuilder::new().max_rect(bounds), |ui| {
                self.render(ui)
            });
        }
        #[cfg(not(target_os = "ios"))]
        self.render(ui);
    }
}
// Retain backend errors in the app sandbox: iOS event-loop failures may exit
// before a console debugger is attached.
#[cfg(target_os = "ios")]
fn install_ios_log() {
    struct FileLog;
    impl log::Log for FileLog {
        fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
            metadata.level() <= log::Level::Info
        }
        fn log(&self, record: &log::Record<'_>) {
            use std::io::Write;
            if self.enabled(record.metadata()) {
                let path =
                    PathBuf::from(std::env::var_os("HOME").unwrap()).join("Documents/backend.log");
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                {
                    let _ = writeln!(
                        file,
                        "{} {}: {}",
                        record.level(),
                        record.target(),
                        record.args()
                    );
                }
            }
        }
        fn flush(&self) {}
    }
    static LOGGER: FileLog = FileLog;
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Info);
}
fn main() -> eframe::Result {
    #[cfg(target_os = "ios")]
    install_ios_log();
    #[cfg(not(target_os = "ios"))]
    let args: Vec<String> = std::env::args().collect();
    #[cfg(not(target_os = "ios"))]
    if let Some(index) = args.iter().position(|s| s == "--render") {
        let output = args.get(index + 1).expect("--render requires a PNG path");
        let mut app = Daybook::new(&egui::Context::default());
        app.path = None;
        app.shot = None;
        let size = if args.iter().any(|s| s == "--compact") {
            egui::vec2(390., 800.)
        } else {
            egui::vec2(1180., 800.)
        };
        let size = if let Some(index) = args.iter().position(|s| s == "--size") {
            let value = args
                .get(index + 1)
                .expect("--size requires WIDTHxHEIGHT in points");
            let (w, h) = value.split_once('x').expect("--size requires WIDTHxHEIGHT");
            let (w, h): (f32, f32) = (w.parse().unwrap(), h.parse().unwrap());
            assert!(
                w.is_finite()
                    && h.is_finite()
                    && (320.0..=4096.0).contains(&w)
                    && (320.0..=4096.0).contains(&h)
            );
            egui::vec2(w, h)
        } else {
            size
        };
        let (look_index, dark) = (app.data.look_index, app.data.dark);
        let mut harness = egui_kittest::Harness::builder()
            .with_theme(if dark {
                egui::Theme::Dark
            } else {
                egui::Theme::Light
            })
            .with_size(size)
            .build_eframe(move |cc| {
                Daybook::configure_theme(&cc.egui_ctx, look_index, dark);
                app
            });
        harness.run_steps(12);
        harness.render().unwrap().save(output).unwrap();
        return Ok(());
    }

    eframe::run_native(
        "Daybook · egui_native_look",
        eframe::NativeOptions {
            #[cfg(target_os = "ios")]
            wgpu_options: {
                let mut config = eframe::egui_wgpu::WgpuConfiguration::default();
                if let eframe::egui_wgpu::WgpuSetup::CreateNew(setup) = &mut config.wgpu_setup {
                    let default_descriptor = setup.device_descriptor.clone();
                    setup.device_descriptor = std::sync::Arc::new(move |adapter| {
                        let mut descriptor = default_descriptor(adapter);
                        // The iOS simulator exposes 15 inter-stage variables rather
                        // than WebGPU's default 16; egui's shader only needs two.
                        descriptor.required_limits.max_inter_stage_shader_variables = descriptor
                            .required_limits
                            .max_inter_stage_shader_variables
                            .min(adapter.limits().max_inter_stage_shader_variables);
                        descriptor
                    });
                }
                config
            },
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1180., 800.])
                .with_min_inner_size([390., 620.]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(Daybook::new(&cc.egui_ctx)))),
    )
}
#[cfg(all(test, not(target_os = "ios")))]
mod tests {
    use super::*;
    #[test]
    fn adding_tasks_rejects_blank_and_keeps_unique_ids() {
        let mut d = Document::seed();
        assert!(!d.add("  ", 0, 1));
        let old = d.tasks.len();
        assert!(d.add("  New task  ", 1, 0));
        assert_eq!(d.tasks.len(), old + 1);
        assert_eq!(d.tasks.last().unwrap().title, "New task");
        let id = d.tasks.last().unwrap().id;
        d.tasks.retain(|t| t.id != 1);
        d.add("Another", 0, 2);
        assert!(d.tasks.last().unwrap().id > id);
    }
    #[test]
    fn document_roundtrip_preserves_edits() {
        let mut d = Document::seed();
        d.tasks[0].done = true;
        d.notes = "Draft\nsecond line".into();
        d.dark = true;
        let s = serde_json::to_string(&d).unwrap();
        let r: Document = serde_json::from_str(&s).unwrap();
        assert!(r.tasks[0].done && r.dark);
        assert_eq!(r.notes, d.notes);
    }
    #[test]
    fn task_dialog_adds_and_filter_search_find_the_result() {
        use egui_kittest::kittest::Queryable;
        let mut app = Daybook::new(&egui::Context::default());
        app.path = None;
        app.data = Document::seed();
        app.save_error = None;
        app.shot = None;
        let mut h = egui_kittest::Harness::builder()
            .with_theme(egui::Theme::Light)
            .with_context_setup(|ctx| Daybook::configure_theme(ctx, 0, false))
            .with_size(egui::vec2(1180., 800.))
            .build_ui_state(|ui, state: &mut Daybook| state.render(ui), app);
        h.run_steps(3);
        h.get_by_label("+ New task").click();
        h.run_steps(3);
        h.get_by_label("Task title").click();
        h.run_steps(2);
        h.event(egui::Event::Text("Review tomorrow".into()));
        h.run_steps(3);
        h.get_by_label("Add task").click();
        h.run_steps(3);
        assert_eq!(h.state().data.tasks.len(), 7);
        assert_eq!(
            h.state().data.tasks.last().unwrap().title,
            "Review tomorrow"
        );
        h.get_by_label("Search tasks").click();
        h.run_steps(2);
        h.event(egui::Event::Text("tomorrow".into()));
        h.run_steps(3);
        assert!(h.query_by_label("Polish the onboarding flow").is_none());
        assert!(h.query_all_by_label("Review tomorrow").next().is_some());
    }
    #[test]
    fn task_controls_stay_centered_for_wrapped_titles_and_both_apple_looks() {
        use egui_kittest::kittest::Queryable;
        for look_index in [0, 1] {
            for width in [310.0, 580.0] {
                for title in ["A short task", "Review the accessibility and keyboard navigation of every onboarding screen before the next release"] {
                    let task=Task{id:1,title:title.into(),project:1,done:false,priority:0};
                    let mut h=egui_kittest::Harness::builder().with_theme(egui::Theme::Light)
                        .with_context_setup(move |ctx| Daybook::configure_theme(ctx,look_index,false))
                        .with_size(egui::vec2(width+16.0,400.0))
                        .build_ui_state(|ui,task:&mut Task| {ui.spacing_mut().item_spacing.y=0.0;task_row(ui,task);},task);
                    h.run_steps(3);
                    let checkbox=h.get_by_role_and_label(egui::accesskit::Role::CheckBox,title).rect();
                    let remove=h.get_by_label("Remove").rect();
                    assert!((checkbox.center().y-remove.center().y).abs()<0.5,"misaligned: {checkbox:?} / {remove:?}");
                    assert!(remove.right()<=width+8.5,"button escaped the row: {remove:?}");
                    h.get_by_role_and_label(egui::accesskit::Role::CheckBox,title).click();h.run_steps(3);
                    assert!(h.state().done,"centered checkbox must remain clickable");
                }
            }
        }
    }
}
