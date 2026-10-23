//! Windows: dialogs and tool windows with the look's header, and the title bar of an
//! undecorated window.

use egui::{Color32, CornerRadius, Rect, Sense, Stroke, Ui};

use super::Ctx;
use crate::Look;

/// A dialog: a window inside the app with the look's header, a body, and a row of
/// actions along the bottom. Each system draws its own: macOS's sheet, WinUI's content
/// dialog, libadwaita's and iOS's alerts and Material's dialog, all closing on their
/// actions; the plain look's window with its title and a close button.
///
/// ```no_run
/// # fn demo(ctx: &egui::Context, open: &mut bool) {
/// use egui_native_look as look;
/// look::Dialog::new("export", "Export").show(ctx, open, |ui| { ui.label("…"); }, |ui| {
///     if ui.add(look::Button::new("Export").accent().default_action()).clicked() {
///         ui.close();
///     }
/// });
/// # }
/// ```
///
/// The builder must be added to a UI or shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::Dialog;
/// Dialog::new("export", "Export");
/// ```
#[must_use = "add the widget to a UI or call its show method"]
pub struct Dialog {
    id: egui::Id,
    title: String,
    /// The size asked for; by default the look's.
    size: Option<egui::Vec2>,
    close_label: String,
    close_on_escape: bool,
}

impl Dialog {
    /// The size a dialog opens at unless asked otherwise: Material 3's 280-point minimum
    /// width, libadwaita's 300-point alert, 420 by 240 elsewhere.
    fn default_size(look: Look) -> egui::Vec2 {
        match look {
            Look::Material => egui::vec2(280.0, 1.0),
            Look::Gtk => egui::vec2(300.0, 1.0),
            // WinUI's content dialog is at least 320 wide.
            Look::Windows => egui::vec2(320.0, 1.0),
            // iOS 26's alert on an iPhone: 316 points.
            Look::Ios => egui::vec2(316.0, 1.0),
            _ => egui::vec2(420.0, 240.0),
        }
    }

    /// A dialog titled `title`; `id` keeps its place from frame to frame.
    pub fn new(id: impl egui::AsIdSalt, title: impl Into<String>) -> Self {
        Dialog {
            id: egui::Id::unique(id),
            title: title.into(),
            size: None,
            close_label: "Close".into(),
            close_on_escape: true,
        }
    }
    /// The close button's name, under the pointer and for a screen reader; "Close" by
    /// default, for an app to translate.
    pub fn close_label(mut self, label: impl Into<String>) -> Self {
        self.close_label = label.into();
        self
    }
    /// Whether an unhandled Escape closes this dialog. Enabled by default.
    pub fn close_on_escape(mut self, enabled: bool) -> Self {
        self.close_on_escape = enabled;
        self
    }

    /// Its size when it opens. Invalid axes keep their previous size; positive axes
    /// are capped at 1,000,000 points.
    pub fn size(mut self, size: impl Into<egui::Vec2>) -> Self {
        let size = size.into();
        let previous = self.size.unwrap_or(egui::vec2(420.0, 240.0));
        self.size = Some(egui::vec2(
            super::dimension(size.x).unwrap_or(previous.x),
            super::dimension(size.y).unwrap_or(previous.y),
        ));
        self
    }

    /// Shows a centred modal while `open`, blocking input to the UI behind it.
    /// A close button, where the look has one, and Escape clear `open`. The size is limited
    /// to the viewport, with scrolling for tall content. Actions run right to left side by
    /// side (sharing the row equally in libadwaita, WinUI and iOS), and top to bottom
    /// aligned to the right when they do not fit. An unhandled Escape closes
    /// only the top dialog when enabled; fields and popups handle it first.
    /// Closing restores focus to the enabled widget focused when the dialog opened.
    /// Keep calling `show` with `open = false` after closing through application state.
    pub fn show(
        self,
        ctx: &egui::Context,
        open: &mut bool,
        body: impl FnOnce(&mut Ui),
        actions: impl FnOnce(&mut Ui),
    ) {
        if ctx.plugin_opt::<DialogFocusReturn>().is_none() {
            ctx.add_plugin(DialogFocusReturn);
        }
        let state_id = self.id.with("dialog-lifecycle");
        let mut state = ctx.data(|data| data.get_temp::<DialogLifecycle>(state_id));
        if !*open {
            forget_dialog(ctx, self.id);
            if let Some(state) = state.take() {
                restore_dialog_focus(ctx, state.opener);
            }
            ctx.data_mut(|data| data.remove::<DialogLifecycle>(state_id));
            return;
        }
        let frame_nr = ctx.cumulative_frame_nr();
        let newly_opened = state
            .as_ref()
            .is_none_or(|state| frame_nr.saturating_sub(state.frame) > 1);
        if newly_opened {
            state = Some(DialogLifecycle {
                opener: ctx.memory(|memory| memory.focused()),
                frame: frame_nr,
                editing: false,
                settled: false,
            });
            ctx.memory_mut(|memory| memory.stop_text_input());
        }
        let mut state = state.unwrap();
        state.frame = frame_nr;
        ctx.data_mut(|data| data.insert_temp(state_id, state.clone()));
        let keyboard_id = self.id.with("dialog-keyboard");
        let popup_open = egui::Popup::is_any_open(ctx);
        let enter = !newly_opened && !popup_open && ctx.input(|input| {
            input.focused
                && !input.events.iter().any(|event| matches!(event, egui::Event::WindowFocused(false)))
                && input.events.iter().any(|event| matches!(event,
                egui::Event::Key { key: egui::Key::Enter, pressed: true, repeat: false, modifiers, .. }
                    if modifiers.is_none()))
        });
        let focused = ctx.memory(|memory| memory.focused());
        let keyboard_pass = ctx.cumulative_pass_nr();
        ctx.data_mut(|data| {
            data.insert_temp(
                keyboard_id,
                DialogKeyboard {
                    enter,
                    blocked: false,
                    focused,
                    text_input: false,
                    first_field: None,
                    pass: keyboard_pass,
                },
            )
        });
        let x = {
            let theme = crate::theme(ctx);
            let c = crate::Colors::resolve(&theme, ctx.theme() == egui::Theme::Dark);
            Ctx {
                look: theme.look,
                c,
                m: theme.metrics,
            }
        };
        let top_escape = escape_closes(ctx, self.id);
        let escape_allowed =
            self.close_on_escape && !newly_opened && !popup_open && !state.editing && top_escape;
        let mut close = false;
        let frame = dialog_frame(ctx);
        let available =
            (ctx.content_rect().size() - egui::vec2(32.0, 32.0)).max(egui::Vec2::splat(1.0));
        let size = self
            .size
            .unwrap_or_else(|| Self::default_size(x.look))
            .min(available);
        let mut modal = egui::Modal::new(self.id)
            .area(egui::Modal::default_area(self.id).accessible_name(self.title.clone()))
            .frame(frame);
        match x.look {
            // A macOS sheet dims the window behind it only lightly.
            Look::Mac => {
                modal =
                    modal.backdrop_color(Color32::from_black_alpha(if x.c.dark { 80 } else { 50 }))
            }
            // Android's dialogs dim it to about half.
            Look::Material => modal = modal.backdrop_color(Color32::from_black_alpha(138)),
            // WinUI's smoke: black at 30 %; iOS dims to a fifth.
            Look::Windows => modal = modal.backdrop_color(Color32::from_black_alpha(77)),
            Look::Ios => modal = modal.backdrop_color(Color32::from_black_alpha(51)),
            // libadwaita dims it with twice its shade colour.
            Look::Gtk => {
                modal = modal.backdrop_color(Color32::from_rgba_unmultiplied(
                    0,
                    0,
                    6,
                    if x.c.dark { 128 } else { 36 },
                ))
            }
            _ => {}
        }
        let margin = if x.look == Look::Mac {
            egui::Margin {
                left: 20,
                right: 20,
                top: 0,
                bottom: 20,
            }
        } else if matches!(x.look, Look::Material | Look::Gtk | Look::Windows) {
            egui::Margin {
                left: 24,
                right: 24,
                top: 0,
                bottom: 24,
            }
        } else if x.look == Look::Ios {
            // An iOS alert's buttons run 14 points from its sides and 16 from its foot.
            egui::Margin {
                left: 14,
                right: 14,
                top: 0,
                bottom: 16,
            }
        } else {
            egui::Margin::same(16)
        };
        let modal = modal.show(ctx, |ui| {
            ui.set_width(size.x);
            // The area offers last frame's size; let content that grew since then
            // grow the dialog up to the viewport before it scrolls.
            ui.set_max_height(available.y);
            egui::ScrollArea::vertical()
                .max_height(available.y)
                .show(ui, |ui| {
                    ui.set_min_width(size.x);
                    ui.set_min_height(size.y);
                    close |= header(ui, &x, &self.title, &self.close_label, false);
                    egui::Frame::new().inner_margin(margin).show(ui, |ui| {
                        if x.look == Look::Gtk {
                            // An alert's body is centred under its heading.
                            ui.vertical_centered(body);
                        } else if x.look == Look::Ios {
                            // iOS sets an alert's message 28 points in, in its 15-point
                            // secondary text.
                            egui::Frame::new()
                                .inner_margin(egui::Margin::symmetric(14, 0))
                                .show(ui, |ui| {
                                    ui.style_mut().override_font_id =
                                        Some(egui::FontId::new(15.0, x.m.body.family.clone()));
                                    ui.visuals_mut().override_text_color = Some(x.c.dim);
                                    body(ui);
                                });
                        } else {
                            body(ui);
                        }
                        ui.add_space(match x.look {
                            Look::Material | Look::Windows => 24.0,
                            Look::Gtk => 21.0,
                            Look::Ios => 18.0,
                            _ => 12.0,
                        });
                        // WinUI sets its buttons apart on the window's own colour, under a
                        // faint line, 24 points round them.
                        let command = (x.look == Look::Windows).then(|| {
                            let top = ui.cursor().top();
                            ui.add_space(24.0);
                            (ui.painter().add(egui::Shape::Noop), top)
                        });
                        // The row's width is only known after the actions are drawn, so it is
                        // kept from the last side-by-side pass; a row that no longer fits
                        // discards the pass and stacks the actions in the next one.
                        let row_id = self.id.with("dialog-actions-width");
                        let actions = |ui: &mut Ui| as_dialog_actions(ui, actions);
                        let room = ui.available_width();
                        let needed = ui.ctx().data(|d| d.get_temp::<f32>(row_id));
                        if needed.is_some_and(|needed| needed > room) {
                            ui.with_layout(egui::Layout::top_down(egui::Align::Max), actions);
                        } else {
                            // Sized like `ui.horizontal`, so the row is centred on its
                            // own height rather than on the room left in the dialog.
                            let row = egui::vec2(room, ui.spacing().interact_size.y);
                            let width = ui
                                .allocate_ui_with_layout(
                                    row,
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        actions(ui);
                                        ui.min_rect().width()
                                    },
                                )
                                .inner;
                            ui.ctx().data_mut(|d| d.insert_temp(row_id, width));
                            if width > room {
                                ui.ctx().request_discard("egui_native_look::Dialog actions");
                            }
                        }
                        if let Some((shape, top)) = command {
                            let inner = ui.max_rect();
                            let area = Rect::from_min_max(
                                egui::pos2(inner.left() - 24.0, top),
                                egui::pos2(inner.right() + 24.0, ui.min_rect().bottom() + 24.0),
                            );
                            let line = Color32::from_black_alpha(if x.c.dark { 25 } else { 15 });
                            ui.painter().set(
                                shape,
                                egui::Shape::Vec(vec![
                                    egui::Shape::rect_filled(
                                        area,
                                        CornerRadius {
                                            sw: 8,
                                            se: 8,
                                            ..Default::default()
                                        },
                                        x.c.bg,
                                    ),
                                    egui::Shape::hline(
                                        area.x_range(),
                                        area.top() + 0.5,
                                        Stroke::new(1.0, line),
                                    ),
                                ]),
                            );
                        }
                    });
                });
        });
        let focused = ctx.memory(|memory| memory.focused());
        let keyboard = ctx.data(|data| data.get_temp::<DialogKeyboard>(keyboard_id));
        state.editing = keyboard.as_ref().is_some_and(|keyboard| {
            keyboard.text_input && focused.is_some() && focused == keyboard.focused
        });
        // The first frame only measures the dialog, so its fields take no focus yet.
        if !state.settled && !newly_opened {
            // An opened dialog puts the focus in its first text field, as GNOME's and
            // macOS's do, unless the focus already moved into it. Not on a touch screen,
            // where it would raise the software keyboard unasked.
            let touch = crate::touch_screen() || ctx.input(|input| input.has_touch_screen());
            if let Some(field) = keyboard.and_then(|keyboard| keyboard.first_field) {
                if !touch && (focused.is_none() || focused == state.opener) {
                    ctx.memory_mut(|memory| memory.request_focus(field));
                }
            }
            state.settled = true;
        }
        ctx.data_mut(|data| data.insert_temp(state_id, state.clone()));
        close |= modal.response.should_close();
        close |= escape_allowed
            && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        if close {
            *open = false;
            forget_dialog(ctx, self.id);
            restore_dialog_focus(ctx, state.opener);
            // The previous modal layer still filters focus during the next pass.
            // Keep the opener until the first closed pass has completed too.
        }
    }
}

#[derive(Clone)]
struct DialogLifecycle {
    opener: Option<egui::Id>,
    frame: u64,
    editing: bool,
    /// Whether the dialog has given out its initial focus.
    settled: bool,
}

#[derive(Clone)]
struct DialogKeyboard {
    enter: bool,
    blocked: bool,
    focused: Option<egui::Id>,
    text_input: bool,
    /// The first enabled text field laid out in the dialog this pass.
    first_field: Option<egui::Id>,
    pass: u64,
}

struct DialogFocusReturn;

impl egui::Plugin for DialogFocusReturn {
    fn debug_name(&self) -> &'static str {
        "DialogFocusReturn"
    }

    fn on_end_pass(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx();
        let key = egui::Id::unique("egui_native_look::DialogFocusReturn").with(ctx.viewport_id());
        let pending = ctx.data_mut(|data| {
            let pending = data.get_temp::<egui::Id>(key);
            data.remove::<egui::Id>(key);
            pending
        });
        if let Some(id) = pending {
            if ctx
                .read_response(id)
                .is_some_and(|response| response.enabled())
            {
                ctx.memory_mut(|memory| memory.request_focus(id));
            }
        }
    }
}

fn restore_dialog_focus(ctx: &egui::Context, opener: Option<egui::Id>) {
    if let Some(id) = opener {
        // Restore after all widgets draw so modal filtering cannot clear the focus again.
        let key = egui::Id::unique("egui_native_look::DialogFocusReturn").with(ctx.viewport_id());
        ctx.data_mut(|data| data.insert_temp(key, id));
    }
}

/// Records an enabled text field `id` in the dialog `ui` belongs to, if any; the first is
/// the one an opened dialog focuses.
pub(super) fn dialog_field(ui: &Ui, id: egui::Id) {
    let key = ui.layer_id().id.with("dialog-keyboard");
    let pass = ui.ctx().cumulative_pass_nr();
    ui.data_mut(|data| {
        if let Some(mut keyboard) = data.get_temp::<DialogKeyboard>(key) {
            if keyboard.pass == pass && keyboard.first_field.is_none() {
                keyboard.first_field = Some(id);
                data.insert_temp(key, keyboard);
            }
        }
    });
}

pub(super) fn dialog_text_input(ui: &Ui, composing: bool) {
    let id = ui.layer_id().id.with("dialog-keyboard");
    let pass = ui.ctx().cumulative_pass_nr();
    ui.data_mut(|data| {
        if let Some(mut keyboard) = data.get_temp::<DialogKeyboard>(id) {
            if keyboard.pass == pass {
                keyboard.text_input = true;
                keyboard.blocked |= composing;
                data.insert_temp(id, keyboard);
            }
        }
    });
}

fn dialog_actions_id() -> egui::Id {
    egui::Id::unique("egui_native_look::Dialog actions")
}

/// Runs `actions`, the buttons along a dialog's bottom, marked as such for
/// [`in_dialog_actions`].
fn as_dialog_actions(ui: &mut Ui, actions: impl FnOnce(&mut Ui)) {
    let outer = in_dialog_actions(ui.ctx());
    ui.data_mut(|d| d.insert_temp(dialog_actions_id(), true));
    let look = crate::look(ui.ctx());
    if matches!(look, Look::Gtk | Look::Windows | Look::Ios) {
        // libadwaita's responses and WinUI's and iOS's buttons share the row in equal widths,
        // 12 and 8 points apart: each button takes its share of the row by the count of the
        // last pass.
        let gap = if look == Look::Gtk { 12.0 } else { 8.0 };
        ui.spacing_mut().item_spacing.x = gap;
        let room = ui.available_width();
        let count =
            ui.data(|d| d.get_temp::<usize>(dialog_actions_count_id(ui.layer_id()).with("last")));
        ui.data_mut(|d| {
            d.insert_temp(dialog_actions_count_id(ui.layer_id()), 0usize);
            d.insert_temp(
                dialog_actions_count_id(ui.layer_id()).with("share"),
                count.map(|n| (room - gap * (n.max(1) - 1) as f32) / n.max(1) as f32),
            );
        });
    }
    let last =
        ui.data(|d| d.get_temp::<usize>(dialog_actions_count_id(ui.layer_id()).with("last")));
    actions(ui);
    let n = ui.data_mut(|d| {
        d.insert_temp(dialog_actions_id(), outer);
        let n = d
            .get_temp::<usize>(dialog_actions_count_id(ui.layer_id()))
            .unwrap_or(0);
        d.insert_temp(dialog_actions_count_id(ui.layer_id()).with("last"), n);
        n
    });
    // A changed count (the first pass too) shares the row out wrongly: lay it out again.
    if matches!(look, Look::Gtk | Look::Windows | Look::Ios) && last != Some(n) {
        ui.ctx()
            .request_discard("egui_native_look::Dialog action count");
    }
}

/// Where a dialog keeps its actions' count, under the dialog's own layer.
fn dialog_actions_count_id(layer: egui::LayerId) -> egui::Id {
    layer.id.with("egui_native_look::Dialog actions count")
}

/// The width each of a libadwaita alert's responses or a WinUI dialog's buttons takes,
/// counting this one, or `None` outside a dialog's actions and on the first pass.
pub(super) fn shared_action_width(ui: &Ui) -> Option<f32> {
    if !in_dialog_actions(ui.ctx()) {
        return None;
    }
    let key = dialog_actions_count_id(ui.layer_id());
    ui.data_mut(|d| {
        let n = d.get_temp::<usize>(key).unwrap_or(0);
        d.insert_temp(key, n + 1);
        d.get_temp::<Option<f32>>(key.with("share")).flatten()
    })
}

/// Whether the buttons being added are a dialog's actions, which Material 3 draws as text
/// buttons.
pub(super) fn in_dialog_actions(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(dialog_actions_id()))
        .unwrap_or(false)
}

pub(super) fn dialog_button(ui: &Ui, response: &mut egui::Response, default: bool) {
    let id = ui.layer_id().id.with("dialog-keyboard");
    let pass = ui.ctx().cumulative_pass_nr();
    let top = ui.memory(|memory| memory.top_modal_layer());
    let clicked = ui.data_mut(|data| {
        let Some(mut keyboard) = data.get_temp::<DialogKeyboard>(id) else {
            return false;
        };
        if keyboard.pass != pass {
            return false;
        }
        let activate = default
            && response.enabled()
            && keyboard.enter
            && !keyboard.blocked
            && top == Some(ui.layer_id())
            && (keyboard.focused.is_none()
                || keyboard.text_input
                || keyboard.focused == Some(response.id));
        if activate {
            keyboard.enter = false;
        }
        data.insert_temp(id, keyboard);
        activate
    });
    if clicked {
        ui.input_mut(|input| {
            input.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
        });
        response
            .flags
            .insert(egui::response::Flags::FAKE_PRIMARY_CLICKED);
    }
}

/// A window that floats over the app and stays while the app is used, a tool's settings or
/// a palette: the look's header with its title and close button, moved by dragging it,
/// resized at its edges.
///
/// ```no_run
/// # fn demo(ctx: &egui::Context, open: &mut bool) {
/// use egui_native_look as look;
/// look::ToolWindow::new("fillet", "Fillet").show(ctx, open, |ui| { ui.label("…"); });
/// # }
/// ```
///
/// The builder must be added to a UI or shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::ToolWindow;
/// ToolWindow::new("settings", "Settings");
/// ```
#[must_use = "add the widget to a UI or call its show method"]
pub struct ToolWindow {
    id: egui::Id,
    title: String,
    size: egui::Vec2,
    pos: Option<egui::Pos2>,
    resizable: bool,
    close_label: String,
}

impl ToolWindow {
    /// A window titled `title`; `id` keeps its place and size from frame to frame.
    pub fn new(id: impl egui::AsIdSalt, title: impl Into<String>) -> Self {
        ToolWindow {
            id: egui::Id::unique(id),
            title: title.into(),
            size: egui::vec2(280.0, 200.0),
            pos: None,
            resizable: true,
            close_label: "Close".into(),
        }
    }
    /// The close button's name, under the pointer and for a screen reader; "Close" by
    /// default, for an app to translate.
    pub fn close_label(mut self, label: impl Into<String>) -> Self {
        self.close_label = label.into();
        self
    }
    /// Its size when it first opens. Invalid axes keep their previous size; positive
    /// axes are capped at 1,000,000 points.
    pub fn default_size(mut self, size: impl Into<egui::Vec2>) -> Self {
        let size = size.into();
        self.size = egui::vec2(
            super::dimension(size.x).unwrap_or(self.size.x),
            super::dimension(size.y).unwrap_or(self.size.y),
        );
        self
    }
    /// Where its top left corner is when it first opens; by default egui's choice.
    /// A non-finite coordinate restores automatic positioning.
    pub fn default_pos(mut self, pos: impl Into<egui::Pos2>) -> Self {
        let pos = pos.into();
        self.pos = pos.is_finite().then_some(pos);
        self
    }
    /// Whether its edges resize it; they do by default.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Shows the window while `open`; its close button clears `open`. The inner value is
    /// what `add` returned, while shown.
    pub fn show<R>(
        self,
        ctx: &egui::Context,
        open: &mut bool,
        add: impl FnOnce(&mut Ui) -> R,
    ) -> Option<egui::InnerResponse<Option<R>>> {
        if !*open {
            return None;
        }
        let x = {
            let theme = crate::theme(ctx);
            let c = crate::Colors::resolve(&theme, ctx.theme() == egui::Theme::Dark);
            Ctx {
                look: theme.look,
                c,
                m: theme.metrics,
            }
        };
        let mut close = false;
        let mut window = egui::Window::new(&self.title)
            .id(self.id)
            .title_bar(false)
            .collapsible(false)
            .resizable(self.resizable)
            .frame(window_frame(ctx).inner_margin(0))
            .default_size(self.size);
        if let Some(pos) = self.pos {
            window = window.default_pos(pos);
        }
        let shown = window.show(ctx, |ui| {
            close = header(ui, &x, &self.title, &self.close_label, true);
            egui::Frame::new().inner_margin(12).show(ui, add).inner
        });
        if close {
            *open = false;
        }
        shown
    }
}

/// The dialogs shown lately, each with the pass it was last shown in.
#[derive(Clone, Default)]
struct Shown(Vec<(egui::Id, u64)>);

fn forget_dialog(ctx: &egui::Context, id: egui::Id) {
    let registry = egui::Id::unique("egui_native_look::Dialogs");
    ctx.data_mut(|data| {
        if let Some(mut shown) = data.get_temp::<Shown>(registry) {
            shown.0.retain(|(other, _)| *other != id);
            data.insert_temp(registry, shown);
        }
    });
}

/// Whether Escape closes the dialog `id` this frame: pressed, the dialog the topmost of
/// those shown. The caller lets fields and popups handle the event before consuming it.
fn escape_closes(ctx: &egui::Context, id: egui::Id) -> bool {
    let pass = ctx.cumulative_pass_nr();
    let registry = egui::Id::unique("egui_native_look::Dialogs");
    let mut shown = ctx
        .data(|d| d.get_temp::<Shown>(registry))
        .unwrap_or_default();
    shown
        .0
        .retain(|(other, at)| *other != id && pass.saturating_sub(*at) <= 2);
    shown.0.push((id, pass));
    let layer = |id| egui::LayerId::new(egui::Order::Foreground, id);
    let top = ctx.memory(|m| {
        m.layer_ids()
            .filter(|l| shown.0.iter().any(|(d, _)| layer(*d) == *l))
            .last()
    });
    ctx.data_mut(|d| d.insert_temp(registry, shown));
    let topmost = top.is_none_or(|top| top == layer(id));
    topmost && ctx.input(|input| input.focused && input.key_pressed(egui::Key::Escape))
}

/// Material modal surfaces use the backdrop instead of a window outline or shadow; a macOS
/// sheet is rounder than a window.
fn dialog_frame(ctx: &egui::Context) -> egui::Frame {
    let frame = window_frame(ctx).inner_margin(0);
    match crate::look(ctx) {
        Look::Material => frame.stroke(Stroke::NONE).shadow(egui::Shadow::NONE),
        // iOS 26's alert: a light glass, 30-point corners, a soft shadow.
        Look::Ios => {
            let c = crate::colors(ctx);
            let ink = if c.dark {
                Color32::WHITE
            } else {
                Color32::BLACK
            };
            frame
                .corner_radius(CornerRadius::same(30))
                .stroke(Stroke::NONE)
                .fill(crate::lerp_color(
                    c.bg,
                    ink,
                    if c.dark { 0.07 } else { 0.067 },
                ))
        }
        // WinUI lays a white layer over the window's colour for the dialog's content (a
        // twentieth of white on the dark scheme); its buttons keep the window's colour.
        Look::Windows => {
            let c = crate::colors(ctx);
            frame.fill(if c.dark {
                crate::lerp_color(c.bg, Color32::WHITE, 0.05)
            } else {
                Color32::WHITE
            })
        }
        // libadwaita's alert: 18-point corners, the window's colour on the light scheme
        // and a lighter grey on the dark, a soft shade around it.
        Look::Gtk => {
            let c = crate::colors(ctx);
            frame
                .corner_radius(CornerRadius::same(18))
                .stroke(Stroke::NONE)
                .fill(if c.dark { c.surface } else { c.bg })
                .shadow(egui::Shadow {
                    offset: [0, 0],
                    blur: 5,
                    spread: 2,
                    color: Color32::from_rgba_unmultiplied(0, 0, 6, 26),
                })
        }
        // A macOS sheet has no edge, and on the dark scheme stands a shade above the window.
        Look::Mac => {
            let c = crate::colors(ctx);
            let fill = if c.dark {
                crate::lerp_color(c.surface, Color32::WHITE, 0.022)
            } else {
                c.surface
            };
            frame
                .corner_radius(CornerRadius::same(16))
                .stroke(Stroke::NONE)
                .fill(fill)
        }
        _ => frame,
    }
}

/// The frame of a window in the look installed in `ctx`: the window's background, a
/// hairline edge, the look's rounding (macOS's 10, WinUI's 8, libadwaita's 12, iOS's 14,
/// Material's 28) and a soft shadow. For an app's own `egui::Window`s, with
/// `.title_bar(false)` and a header of its own, or as they are.
pub fn window_frame(ctx: &egui::Context) -> egui::Frame {
    let resolved = crate::resolved(ctx);
    let c = resolved.colors(ctx);
    let round = match resolved.theme.look {
        Look::Mac => 10,
        Look::Windows => 8,
        Look::Gtk => 12,
        Look::Ios => 14,
        Look::Material => 28,
        Look::Plain => return egui::Frame::window(&ctx.global_style()).fill(c.surface),
    };
    let alpha = if c.dark { 110 } else { 45 };
    egui::Frame::window(&ctx.global_style())
        .fill(c.surface)
        .stroke(Stroke::new(1.0, c.border))
        .corner_radius(CornerRadius::same(round))
        .shadow(egui::Shadow {
            offset: [0, 8],
            blur: 28,
            spread: 0,
            color: Color32::from_black_alpha(alpha),
        })
}

/// A dialog's header in the look. Returns whether its close button was clicked.
/// Material's dialogs and macOS's sheets have none, closing on their actions; `closable`
/// gives one anyway, for a window without actions.
fn header(ui: &mut Ui, x: &Ctx, title: &str, close_label: &str, closable: bool) -> bool {
    let c = &x.c;
    // A macOS sheet has no title bar: its title is bold text at the top of the body;
    // a libadwaita alert has its heading centred 32 points down, ten above its body.
    let sheet = x.look == Look::Mac && !closable;
    // WinUI's content dialog: its title 24 points in, without a close button or a line.
    let content_dialog = x.look == Look::Windows && !closable;
    let alert = x.look == Look::Gtk && !closable;
    // iOS 26's alert: its bold title 28 points in, without a bar.
    let ios_alert = x.look == Look::Ios && !closable;
    let height: f32 = match x.look {
        // Material's title sits 24 points in, its 32-point line 16 points above the body.
        Look::Material => 72.0,
        _ if alert => 66.0,
        _ if ios_alert => 48.0,
        _ if content_dialog => 64.0,
        Look::Ios => 52.0,
        _ if sheet => 48.0,
        // macOS's title bar.
        Look::Mac => 32.0,
        _ => 40.0,
    };
    let height = height.max(super::minimum_target(ui));
    let (bar, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), height), Sense::hover());
    let painter = ui.painter();
    let font = match x.look {
        // Material's dialog title is its headline-small (24, regular), Windows' its
        // subtitle (20).
        Look::Material => egui::FontId::new(24.0, x.m.body.family.clone()),
        Look::Windows if content_dialog => egui::FontId::new(20.0, x.m.bold.family.clone()),
        Look::Windows => egui::FontId::new(18.0, x.m.bold.family.clone()),
        _ if ios_alert => x.m.bold.clone(),
        // libadwaita's title-2: 136 % of the body, in its heaviest weight.
        _ if alert => egui::FontId::new(x.m.body.size * 1.36, x.m.bold.family.clone()),
        _ => x.m.bold.clone(),
    };
    let left_title = sheet || matches!(x.look, Look::Windows | Look::Material);
    // The title keeps clear of the close button at either end, cut short if it must be.
    let (galley, _) = super::truncated(ui, title, font, c.fg, bar.width() - 2.0 * 48.0);
    let left = if ios_alert {
        bar.left() + 28.0
    } else if x.look == Look::Material || content_dialog {
        bar.left() + 24.0
    } else if left_title {
        bar.left() + 20.0
    } else {
        bar.center().x - galley.size().x / 2.0
    };
    let middle = if sheet {
        bar.center().y + 4.0
    } else if x.look == Look::Material {
        bar.top() + 38.5
    } else if alert {
        bar.top() + 44.0
    } else if content_dialog {
        bar.top() + 38.0
    } else if ios_alert {
        bar.top() + 31.0
    } else {
        bar.center().y
    };
    let top = crate::galley_top(ui.ctx(), &galley, middle);
    painter.galley(egui::pos2(left, top), galley, c.fg);
    if !left_title && !alert && !ios_alert {
        let line = if x.look == Look::Mac {
            let ink = if c.dark {
                Color32::WHITE
            } else {
                Color32::BLACK
            };
            crate::lerp_color(c.surface, ink, if c.dark { 0.1 } else { 0.09 })
        } else {
            c.border
        };
        painter.hline(bar.x_range(), bar.bottom() - 0.5, Stroke::new(1.0, line));
    }
    match x.look {
        Look::Material | Look::Mac | Look::Gtk | Look::Windows | Look::Ios if !closable => false,
        Look::Mac => {
            // The window's red close button at the start of the bar: fourteen points,
            // centred sixteen from the corner.
            let dot = Rect::from_center_size(
                egui::pos2(
                    bar.left() + 16.0_f32.max(super::minimum_target(ui) * 0.5),
                    bar.center().y,
                ),
                egui::Vec2::splat(14.0),
            );
            let response = super::click_response(
                ui,
                ui.interact(
                    Rect::from_center_size(
                        dot.center(),
                        super::target_size(ui, dot.expand(4.0).size()),
                    )
                    .intersect(bar),
                    ui.scope_id().with("close"),
                    Sense::click(),
                ),
            );
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), close_label)
            });
            ui.painter().circle_filled(
                dot.center(),
                7.0,
                crate::colors::rgb(if c.dark { 0xf2605a } else { 0xf9807a }),
            );
            if response.hovered() {
                let m = dot.center();
                let s = Stroke::new(1.2, Color32::from_black_alpha(150));
                ui.painter()
                    .line_segment([m + egui::vec2(-2.5, -2.5), m + egui::vec2(2.5, 2.5)], s);
                ui.painter()
                    .line_segment([m + egui::vec2(2.5, -2.5), m + egui::vec2(-2.5, 2.5)], s);
            }
            response.on_hover_text(close_label).clicked()
        }
        _ => {
            let side: f32 = 28.0;
            let rect = Rect::from_center_size(
                egui::pos2(
                    bar.right() - (side / 2.0 + 8.0).max(super::minimum_target(ui) * 0.5),
                    bar.center().y,
                ),
                egui::Vec2::splat(side),
            );
            let response = super::click_response(
                ui,
                ui.interact(
                    Rect::from_center_size(rect.center(), super::target_size(ui, rect.size()))
                        .intersect(bar),
                    ui.scope_id().with("close"),
                    Sense::click(),
                ),
            );
            let round = match x.look {
                Look::Gtk | Look::Ios => CornerRadius::same((side / 2.0) as u8),
                _ => CornerRadius::same(x.m.radius),
            };
            let fill = match x.look {
                // libadwaita's and iOS's close buttons are round and tinted at rest.
                Look::Gtk | Look::Ios => Some(if response.hovered() {
                    c.button_hover
                } else {
                    c.button
                }),
                _ if response.hovered() => Some(c.button_hover),
                _ => None,
            };
            if let Some(fill) = fill {
                ui.painter().rect_filled(rect, round, fill);
            }
            close_cross(
                ui.painter(),
                rect.center(),
                if response.hovered() { c.fg } else { c.dim },
            );
            if response.has_focus() {
                crate::paint::ring(ui.painter(), &x.paint(), rect, round);
            }
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), close_label)
            });
            response.on_hover_text(close_label).clicked()
        }
    }
}

fn close_cross(painter: &egui::Painter, m: egui::Pos2, colour: Color32) {
    let s = Stroke::new(1.5, colour);
    painter.line_segment([m + egui::vec2(-4.5, -4.5), m + egui::vec2(4.5, 4.5)], s);
    painter.line_segment([m + egui::vec2(4.5, -4.5), m + egui::vec2(-4.5, 4.5)], s);
}

/// What a [`TitleBar`]'s buttons asked for; the bar itself already started a drag or
/// toggled maximised on a double click.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowAction {
    /// Minimise the window.
    Minimize,
    /// Maximise the window, or restore it when it is maximised.
    ToggleMaximize,
    /// Close the window.
    Close,
}

/// The title bar of a window the app draws itself (no system decorations): the title, and
/// the look's window buttons. On macOS the system's own traffic lights stay (a window with
/// a transparent title bar), so the bar only leaves room for them; on Windows the
/// minimise, maximise and close buttons, close turning red; in libadwaita and elsewhere a
/// round close button. A drag on the bar moves the window and a double click maximises it.
///
/// The builder must be added to a UI or shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::TitleBar;
/// TitleBar::new("Editor");
/// ```
#[must_use = "add the widget to a UI or call its show method"]
pub struct TitleBar {
    title: String,
    maximized: bool,
    labels: [String; 3],
}

impl TitleBar {
    /// A bar showing `title`.
    pub fn new(title: impl Into<String>) -> Self {
        TitleBar {
            title: title.into(),
            maximized: false,
            labels: ["Minimize".into(), "Maximize".into(), "Close".into()],
        }
    }
    /// The window buttons' names, for a screen reader and under the pointer: "Minimize",
    /// "Maximize" and "Close" by default, for an app to translate.
    pub fn button_labels(
        mut self,
        minimize: impl Into<String>,
        maximize: impl Into<String>,
        close: impl Into<String>,
    ) -> Self {
        self.labels = [minimize.into(), maximize.into(), close.into()];
        self
    }
    /// Whether the window is maximised now, for the maximise button's glyph and the
    /// double click.
    pub fn maximized(mut self, maximized: bool) -> Self {
        self.maximized = maximized;
        self
    }

    /// Draws the bar into `rect`. Returns what its buttons asked for.
    /// Reserve at least 48 points of height on touch devices. Button targets stay inside `rect`.
    pub fn show(self, ui: &mut Ui, rect: Rect) -> Option<WindowAction> {
        let x = Ctx::of(ui);
        let c = &x.c;
        let drag = ui.interact(
            rect,
            ui.scope_id().with("egui_native_look-title-bar"),
            Sense::click_and_drag(),
        );
        if drag.drag_started_by(egui::PointerButton::Primary) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        if drag.double_clicked() {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Maximized(!self.maximized));
        }
        let painter = ui.painter();
        let mut action = None;
        if x.look == Look::Windows {
            let width = 46.0_f32.max(super::minimum_target(ui));
            let (galley, _) = super::truncated(
                ui,
                &self.title,
                x.m.body.clone(),
                c.fg,
                rect.width() - 14.0 - 3.0 * width - 8.0,
            );
            painter.galley(
                egui::pos2(
                    rect.left() + 14.0,
                    crate::galley_top(ui.ctx(), &galley, rect.center().y),
                ),
                galley,
                c.fg,
            );
            let buttons = [
                (WindowAction::Close, 0.0),
                (WindowAction::ToggleMaximize, 1.0),
                (WindowAction::Minimize, 2.0),
            ];
            for (what, slot) in buttons {
                let r = Rect::from_min_size(
                    egui::pos2(rect.right() - width * (slot + 1.0), rect.top()),
                    egui::vec2(width, rect.height()),
                );
                let response = ui.interact(
                    r,
                    ui.scope_id().with(("window-button", slot as i32)),
                    Sense::click(),
                );
                let close = what == WindowAction::Close;
                let label = &self.labels[2 - slot as usize];
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), label)
                });
                let response = super::click_response(ui, response).on_hover_text(label);
                let mut colour = c.fg;
                if response.hovered() {
                    let fill = if close {
                        crate::colors::rgb(0xc42b1c)
                    } else {
                        c.fg.gamma_multiply(0.1)
                    };
                    ui.painter().rect_filled(r, 0.0, fill);
                    if close {
                        colour = Color32::WHITE;
                    }
                }
                let m = r.center();
                let s = Stroke::new(1.0, colour);
                match what {
                    WindowAction::Close => close_cross(ui.painter(), m, colour),
                    WindowAction::Minimize => {
                        ui.painter()
                            .line_segment([m + egui::vec2(-5.0, 0.0), m + egui::vec2(5.0, 0.0)], s);
                    }
                    WindowAction::ToggleMaximize => {
                        ui.painter().rect_stroke(
                            Rect::from_center_size(m, egui::Vec2::splat(10.0)),
                            CornerRadius::same(2),
                            s,
                            egui::StrokeKind::Inside,
                        );
                    }
                }
                if response.clicked() {
                    action = Some(what);
                }
            }
        } else {
            let inset = if x.look == Look::Mac { 72.0 } else { 0.0 };
            let room = Rect::from_min_max(egui::pos2(rect.left() + inset, rect.top()), rect.max);
            // Centred on the room, kept clear of the close button (or the traffic lights).
            let (galley, _) = super::truncated(
                ui,
                &self.title,
                x.m.bold.clone(),
                c.fg,
                room.width() - 2.0 * 44.0,
            );
            painter.galley(
                egui::pos2(
                    room.center().x - galley.size().x / 2.0,
                    crate::galley_top(ui.ctx(), &galley, rect.center().y),
                ),
                galley,
                c.fg,
            );
            if x.look != Look::Mac {
                let side = (rect.height() - 12.0).clamp(20.0, 26.0);
                let r = Rect::from_center_size(
                    egui::pos2(
                        rect.right() - (side / 2.0 + 8.0).max(super::minimum_target(ui) * 0.5),
                        rect.center().y,
                    ),
                    egui::Vec2::splat(side),
                );
                let response = ui.interact(
                    Rect::from_center_size(r.center(), super::target_size(ui, r.size()))
                        .intersect(rect),
                    ui.scope_id().with("window-close"),
                    Sense::click(),
                );
                let label = &self.labels[2];
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), label)
                });
                let response = super::click_response(ui, response).on_hover_text(label);
                ui.painter().rect_filled(
                    r,
                    CornerRadius::same((side / 2.0) as u8),
                    if response.hovered() {
                        c.button_hover
                    } else {
                        c.button
                    },
                );
                close_cross(ui.painter(), r.center(), c.fg);
                if response.clicked() {
                    action = Some(WindowAction::Close);
                }
            }
        }
        action
    }
}

#[cfg(test)]
mod tests {
    use super::{Dialog, ToolWindow};

    #[test]
    fn material_dialogs_do_not_inherit_window_shadows_or_outlines() {
        for scheme in [egui::Theme::Dark, egui::Theme::Light] {
            let ctx = egui::Context::default();
            crate::install(&ctx, crate::Theme::new(crate::Look::Material));
            ctx.set_theme(scheme);
            let window = super::window_frame(&ctx);
            assert_ne!(window.shadow, egui::Shadow::NONE);
            assert_ne!(window.stroke, egui::Stroke::NONE);
            let dialog = super::dialog_frame(&ctx);
            assert_eq!(dialog.shadow, egui::Shadow::NONE);
            assert_eq!(dialog.stroke, egui::Stroke::NONE);
            assert_eq!(dialog.corner_radius, egui::CornerRadius::same(28));
            assert_eq!(dialog.fill, window.fill);
        }
    }

    #[test]
    fn other_dialog_looks_keep_their_window_frames() {
        for chosen in [crate::Look::Plain, crate::Look::Mac] {
            for scheme in [egui::Theme::Dark, egui::Theme::Light] {
                let ctx = egui::Context::default();
                crate::install(&ctx, crate::Theme::new(chosen));
                ctx.set_theme(scheme);
                let window = super::window_frame(&ctx).inner_margin(0);
                // A macOS sheet keeps the window's shadow, rounder and without an edge.
                let expected = if chosen == crate::Look::Mac {
                    let c = crate::colors(&ctx);
                    let fill = if c.dark {
                        crate::lerp_color(c.surface, egui::Color32::WHITE, 0.022)
                    } else {
                        c.surface
                    };
                    window
                        .corner_radius(egui::CornerRadius::same(16))
                        .stroke(egui::Stroke::NONE)
                        .fill(fill)
                } else {
                    window
                };
                assert_eq!(super::dialog_frame(&ctx), expected);
            }
        }
    }

    #[test]
    fn a_gtk_dialog_is_an_alert() {
        let ctx = egui::Context::default();
        crate::install(&ctx, crate::Theme::new(crate::Look::Gtk));
        // libadwaita's alert: 18-point corners, rounder than its windows' 12.
        assert_eq!(
            super::dialog_frame(&ctx).corner_radius,
            egui::CornerRadius::same(18)
        );
        assert_eq!(
            super::window_frame(&ctx).corner_radius,
            egui::CornerRadius::same(12)
        );
    }

    #[test]
    fn invalid_window_axes_preserve_previous_sizes() {
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0, -1.0] {
            let dialog = Dialog::new("dialog", "Dialog").size((123.0, 234.0));
            assert_eq!(
                dialog.size((invalid, 345.0)).size,
                Some(egui::vec2(123.0, 345.0))
            );
            let window = ToolWindow::new("window", "Window").default_size((123.0, 234.0));
            assert_eq!(
                window.default_size((345.0, invalid)).size,
                egui::vec2(345.0, 234.0)
            );
        }
        let bounds = egui::vec2(f32::MIN_POSITIVE, f32::MAX);
        assert_eq!(
            Dialog::new("dialog", "Dialog").size(bounds).size,
            Some(egui::vec2(1.0, 1_000_000.0))
        );
        assert_eq!(
            ToolWindow::new("window", "Window")
                .default_size(bounds)
                .size,
            egui::vec2(1.0, 1_000_000.0)
        );
    }

    #[test]
    fn invalid_positions_restore_automatic_positioning() {
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            for pos in [egui::pos2(invalid, 10.0), egui::pos2(10.0, invalid)] {
                let window = ToolWindow::new("window", "Window")
                    .default_pos((10.0, 20.0))
                    .default_pos(pos);
                assert_eq!(window.pos, None);
            }
        }
        assert_eq!(
            ToolWindow::new("window", "Window")
                .default_pos((10.0, 20.0))
                .pos,
            Some(egui::pos2(10.0, 20.0))
        );
    }
}
