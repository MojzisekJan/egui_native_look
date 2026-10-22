//! Windows: dialogs and tool windows with the look's header, and the title bar of an
//! undecorated window.

use egui::{Color32, CornerRadius, Rect, Sense, Stroke, Ui};

use super::Ctx;
use crate::Look;

/// A dialog: a window inside the app with the look's header (its title and close
/// button), a body, and a row of actions along the bottom.
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
    size: egui::Vec2,
    close_label: String,
    close_on_escape: bool,
}

impl Dialog {
    /// A dialog titled `title`; `id` keeps its place from frame to frame.
    pub fn new(id: impl egui::AsIdSalt, title: impl Into<String>) -> Self {
        Dialog {
            id: egui::Id::unique(id),
            title: title.into(),
            size: egui::vec2(420.0, 240.0),
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
        self.size = egui::vec2(
            super::dimension(size.x).unwrap_or(self.size.x),
            super::dimension(size.y).unwrap_or(self.size.y),
        );
        self
    }

    /// Shows a centred modal while `open`, blocking input to the UI behind it.
    /// The close button and Escape clear `open`. The size is limited to the viewport,
    /// with scrolling for tall content. Actions run right to left on wide dialogs and
    /// top to bottom on narrow ones, aligned to the right. An unhandled Escape closes
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
        let size = self.size.min(available);
        let modal = egui::Modal::new(self.id)
            .area(egui::Modal::default_area(self.id).accessible_name(self.title.clone()))
            .frame(frame)
            .show(ctx, |ui| {
                ui.set_width(size.x);
                egui::ScrollArea::vertical()
                    .max_height(available.y)
                    .show(ui, |ui| {
                        ui.set_min_width(size.x);
                        ui.set_min_height(size.y);
                        close |= header(ui, &x, &self.title, &self.close_label, false);
                        egui::Frame::new().inner_margin(16).show(ui, |ui| {
                            body(ui);
                            ui.add_space(12.0);
                            let layout = if ui.available_width() < 360.0 {
                                egui::Layout::top_down(egui::Align::Max)
                            } else {
                                egui::Layout::right_to_left(egui::Align::Center)
                            };
                            ui.with_layout(layout, actions);
                        });
                    });
            });
        let focused = ctx.memory(|memory| memory.focused());
        state.editing = ctx
            .data(|data| data.get_temp::<DialogKeyboard>(keyboard_id))
            .is_some_and(|keyboard| {
                keyboard.text_input && focused.is_some() && focused == keyboard.focused
            });
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
}

#[derive(Clone)]
struct DialogKeyboard {
    enter: bool,
    blocked: bool,
    focused: Option<egui::Id>,
    text_input: bool,
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

/// Material modal surfaces use the backdrop instead of a window outline or shadow.
fn dialog_frame(ctx: &egui::Context) -> egui::Frame {
    let frame = window_frame(ctx).inner_margin(0);
    if crate::look(ctx) == Look::Material {
        frame.stroke(Stroke::NONE).shadow(egui::Shadow::NONE)
    } else {
        frame
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
/// Material's dialogs have none, closing on their actions; `closable` gives one anyway,
/// for a window without actions.
fn header(ui: &mut Ui, x: &Ctx, title: &str, close_label: &str, closable: bool) -> bool {
    let c = &x.c;
    let height: f32 = match x.look {
        Look::Material => 56.0,
        Look::Ios => 52.0,
        _ => 40.0,
    };
    let height = height.max(super::minimum_target(ui));
    let (bar, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), height), Sense::hover());
    let painter = ui.painter();
    let font = match x.look {
        // Material's dialog title is its headline-small (24), Windows' its subtitle (20).
        Look::Material => egui::FontId::new(22.0, x.m.bold.family.clone()),
        Look::Windows => egui::FontId::new(18.0, x.m.bold.family.clone()),
        _ => x.m.bold.clone(),
    };
    let left_title = matches!(x.look, Look::Windows | Look::Material);
    // The title keeps clear of the close button at either end, cut short if it must be.
    let (galley, _) = super::truncated(ui, title, font, c.fg, bar.width() - 2.0 * 48.0);
    let left = if left_title {
        bar.left() + 20.0
    } else {
        bar.center().x - galley.size().x / 2.0
    };
    let top = crate::galley_top(ui.ctx(), &galley, bar.center().y);
    painter.galley(egui::pos2(left, top), galley, c.fg);
    if !left_title {
        painter.hline(
            bar.x_range(),
            bar.bottom() - 0.5,
            Stroke::new(1.0, c.border),
        );
    }
    match x.look {
        Look::Material if !closable => false,
        Look::Mac => {
            // The window's red close button at the start of the bar.
            let dot = Rect::from_center_size(
                egui::pos2(
                    bar.left() + 20.0_f32.max(super::minimum_target(ui) * 0.5),
                    bar.center().y,
                ),
                egui::Vec2::splat(12.0),
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
            ui.painter()
                .circle_filled(dot.center(), 6.0, crate::colors::rgb(0xff5f57));
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
        for chosen in [
            crate::Look::Plain,
            crate::Look::Mac,
            crate::Look::Windows,
            crate::Look::Gtk,
            crate::Look::Ios,
        ] {
            for scheme in [egui::Theme::Dark, egui::Theme::Light] {
                let ctx = egui::Context::default();
                crate::install(&ctx, crate::Theme::new(chosen));
                ctx.set_theme(scheme);
                assert_eq!(
                    super::dialog_frame(&ctx),
                    super::window_frame(&ctx).inner_margin(0)
                );
            }
        }
    }

    #[test]
    fn invalid_window_axes_preserve_previous_sizes() {
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0, -1.0] {
            let dialog = Dialog::new("dialog", "Dialog").size((123.0, 234.0));
            assert_eq!(dialog.size((invalid, 345.0)).size, egui::vec2(123.0, 345.0));
            let window = ToolWindow::new("window", "Window").default_size((123.0, 234.0));
            assert_eq!(
                window.default_size((345.0, invalid)).size,
                egui::vec2(345.0, 234.0)
            );
        }
        let bounds = egui::vec2(f32::MIN_POSITIVE, f32::MAX);
        assert_eq!(
            Dialog::new("dialog", "Dialog").size(bounds).size,
            egui::vec2(1.0, 1_000_000.0)
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
