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
///     ui.add(look::Button::new("Export").accent());
/// });
/// # }
/// ```
pub struct Dialog {
    id: egui::Id,
    title: String,
    size: egui::Vec2,
    close_label: String,
}

impl Dialog {
    /// A dialog titled `title`; `id` keeps its place from frame to frame.
    pub fn new(id: impl std::hash::Hash + std::fmt::Debug, title: impl Into<String>) -> Self {
        Dialog { id: egui::Id::new(id), title: title.into(), size: egui::vec2(420.0, 240.0), close_label: "Close".into() }
    }
    /// The close button's name, under the pointer and for a screen reader; "Close" by
    /// default, for an app to translate.
    pub fn close_label(mut self, label: impl Into<String>) -> Self {
        self.close_label = label.into();
        self
    }
    /// Its size when it opens.
    pub fn size(mut self, size: impl Into<egui::Vec2>) -> Self {
        self.size = size.into();
        self
    }

    /// Shows the dialog while `open`, centred when it opens; the close button and Escape
    /// clear `open`. `actions` are laid out right to left, the first one rightmost. Escape
    /// closes only the dialog on top, and only while nothing had the focus (Escape first
    /// takes it off a field) and no pop-up is open.
    pub fn show(self, ctx: &egui::Context, open: &mut bool, body: impl FnOnce(&mut Ui), actions: impl FnOnce(&mut Ui)) {
        if !*open {
            return;
        }
        let x = {
            let theme = crate::theme(ctx);
            let c = crate::Colors::resolve(&theme, ctx.theme() == egui::Theme::Dark);
            Ctx { look: theme.look, c, m: theme.metrics }
        };
        let mut close = escape_closes(ctx, self.id);
        let frame = window_frame(ctx).inner_margin(0);
        egui::Window::new(&self.title)
            .id(self.id)
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .frame(frame)
            .default_size(self.size)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ctx.content_rect().center())
            .show(ctx, |ui| {
                ui.set_min_width(self.size.x);
                close |= header(ui, &x, &self.title, &self.close_label, false);
                egui::Frame::new().inner_margin(16).show(ui, |ui| {
                    body(ui);
                    ui.add_space(12.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), actions);
                });
            });
        let focused = ctx.memory(|m| m.focused().is_some());
        ctx.data_mut(|d| d.insert_temp(self.id.with("egui_native_look-focused"), focused));
        if close {
            *open = false;
        }
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
    pub fn new(id: impl std::hash::Hash + std::fmt::Debug, title: impl Into<String>) -> Self {
        ToolWindow { id: egui::Id::new(id), title: title.into(), size: egui::vec2(280.0, 200.0), pos: None, resizable: true, close_label: "Close".into() }
    }
    /// The close button's name, under the pointer and for a screen reader; "Close" by
    /// default, for an app to translate.
    pub fn close_label(mut self, label: impl Into<String>) -> Self {
        self.close_label = label.into();
        self
    }
    /// Its size when it first opens.
    pub fn default_size(mut self, size: impl Into<egui::Vec2>) -> Self {
        self.size = size.into();
        self
    }
    /// Where its top left corner is when it first opens; by default egui's choice.
    pub fn default_pos(mut self, pos: impl Into<egui::Pos2>) -> Self {
        self.pos = Some(pos.into());
        self
    }
    /// Whether its edges resize it; they do by default.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Shows the window while `open`; its close button clears `open`. The inner value is
    /// what `add` returned, while shown.
    pub fn show<R>(self, ctx: &egui::Context, open: &mut bool, add: impl FnOnce(&mut Ui) -> R) -> Option<egui::InnerResponse<Option<R>>> {
        if !*open {
            return None;
        }
        let x = {
            let theme = crate::theme(ctx);
            let c = crate::Colors::resolve(&theme, ctx.theme() == egui::Theme::Dark);
            Ctx { look: theme.look, c, m: theme.metrics }
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

/// Whether Escape closes the dialog `id` this frame: pressed, the dialog the topmost of
/// those shown, nothing focused at the end of the last frame, and no pop-up open.
fn escape_closes(ctx: &egui::Context, id: egui::Id) -> bool {
    let pass = ctx.cumulative_pass_nr();
    let registry = egui::Id::new("egui_native_look::Dialogs");
    let mut shown = ctx.data(|d| d.get_temp::<Shown>(registry)).unwrap_or_default();
    shown.0.retain(|(other, at)| *other != id && pass.saturating_sub(*at) <= 2);
    shown.0.push((id, pass));
    let layer = |id| egui::LayerId::new(egui::Order::Middle, id);
    let top = ctx.memory(|m| m.layer_ids().filter(|l| shown.0.iter().any(|(d, _)| layer(*d) == *l)).last());
    ctx.data_mut(|d| d.insert_temp(registry, shown));
    let topmost = top.is_none_or(|top| top == layer(id));
    let focused = ctx.data(|d| d.get_temp::<bool>(id.with("egui_native_look-focused"))).unwrap_or(false);
    topmost && !focused && !egui::Popup::is_any_open(ctx) && ctx.input(|i| i.key_pressed(egui::Key::Escape))
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
        .shadow(egui::Shadow { offset: [0, 8], blur: 28, spread: 0, color: Color32::from_black_alpha(alpha) })
}

/// A dialog's header in the look. Returns whether its close button was clicked.
/// Material's dialogs have none, closing on their actions; `closable` gives one anyway,
/// for a window without actions.
fn header(ui: &mut Ui, x: &Ctx, title: &str, close_label: &str, closable: bool) -> bool {
    let c = &x.c;
    let height = match x.look {
        Look::Material => 56.0,
        Look::Ios => 52.0,
        _ => 40.0,
    };
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
    let left = if left_title { bar.left() + 20.0 } else { bar.center().x - galley.size().x / 2.0 };
    let top = crate::galley_top(ui.ctx(), &galley, bar.center().y);
    painter.galley(egui::pos2(left, top), galley, c.fg);
    if !left_title {
        painter.hline(bar.x_range(), bar.bottom() - 0.5, Stroke::new(1.0, c.border));
    }
    match x.look {
        Look::Material if !closable => false,
        Look::Mac => {
            // The window's red close button at the start of the bar.
            let dot = Rect::from_center_size(egui::pos2(bar.left() + 20.0, bar.center().y), egui::Vec2::splat(12.0));
            let response = ui.interact(dot.expand(4.0), ui.id().with("close"), Sense::click());
            response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, close_label));
            ui.painter().circle_filled(dot.center(), 6.0, crate::colors::rgb(0xff5f57));
            if response.hovered() {
                let m = dot.center();
                let s = Stroke::new(1.2, Color32::from_black_alpha(150));
                ui.painter().line_segment([m + egui::vec2(-2.5, -2.5), m + egui::vec2(2.5, 2.5)], s);
                ui.painter().line_segment([m + egui::vec2(2.5, -2.5), m + egui::vec2(-2.5, 2.5)], s);
            }
            response.on_hover_text(close_label).clicked()
        }
        _ => {
            let side = 28.0;
            let rect = Rect::from_center_size(egui::pos2(bar.right() - side / 2.0 - 8.0, bar.center().y), egui::Vec2::splat(side));
            let response = ui.interact(rect, ui.id().with("close"), Sense::click());
            let round = match x.look {
                Look::Gtk | Look::Ios => CornerRadius::same((side / 2.0) as u8),
                _ => CornerRadius::same(x.m.radius),
            };
            let fill = match x.look {
                // libadwaita's and iOS's close buttons are round and tinted at rest.
                Look::Gtk | Look::Ios => Some(if response.hovered() { c.button_hover } else { c.button }),
                _ if response.hovered() => Some(c.button_hover),
                _ => None,
            };
            if let Some(fill) = fill {
                ui.painter().rect_filled(rect, round, fill);
            }
            close_cross(ui.painter(), rect.center(), if response.hovered() { c.fg } else { c.dim });
            if response.has_focus() {
                crate::paint::ring(ui.painter(), &x.paint(), rect, round);
            }
            response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, close_label));
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
pub struct TitleBar {
    title: String,
    maximized: bool,
    labels: [String; 3],
}

impl TitleBar {
    /// A bar showing `title`.
    pub fn new(title: impl Into<String>) -> Self {
        TitleBar { title: title.into(), maximized: false, labels: ["Minimize".into(), "Maximize".into(), "Close".into()] }
    }
    /// The window buttons' names, for a screen reader and under the pointer: "Minimize",
    /// "Maximize" and "Close" by default, for an app to translate.
    pub fn button_labels(mut self, minimize: impl Into<String>, maximize: impl Into<String>, close: impl Into<String>) -> Self {
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
    pub fn show(self, ui: &mut Ui, rect: Rect) -> Option<WindowAction> {
        let x = Ctx::of(ui);
        let c = &x.c;
        let drag = ui.interact(rect, ui.id().with("egui_native_look-title-bar"), Sense::click_and_drag());
        if drag.drag_started_by(egui::PointerButton::Primary) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        if drag.double_clicked() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!self.maximized));
        }
        let painter = ui.painter();
        let mut action = None;
        if x.look == Look::Windows {
            let width = 46.0;
            let (galley, _) = super::truncated(ui, &self.title, x.m.body.clone(), c.fg, rect.width() - 14.0 - 3.0 * width - 8.0);
            painter.galley(egui::pos2(rect.left() + 14.0, crate::galley_top(ui.ctx(), &galley, rect.center().y)), galley, c.fg);
            let buttons = [(WindowAction::Close, 0.0), (WindowAction::ToggleMaximize, 1.0), (WindowAction::Minimize, 2.0)];
            for (what, slot) in buttons {
                let r = Rect::from_min_size(egui::pos2(rect.right() - width * (slot + 1.0), rect.top()), egui::vec2(width, rect.height()));
                let response = ui.interact(r, ui.id().with(("window-button", slot as i32)), Sense::click());
                let close = what == WindowAction::Close;
                let label = &self.labels[2 - slot as usize];
                response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
                let response = response.on_hover_text(label);
                let mut colour = c.fg;
                if response.hovered() {
                    let fill = if close { crate::colors::rgb(0xc42b1c) } else { c.fg.gamma_multiply(0.1) };
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
                        ui.painter().line_segment([m + egui::vec2(-5.0, 0.0), m + egui::vec2(5.0, 0.0)], s);
                    }
                    WindowAction::ToggleMaximize => {
                        ui.painter().rect_stroke(Rect::from_center_size(m, egui::Vec2::splat(10.0)), CornerRadius::same(2), s, egui::StrokeKind::Inside);
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
            let (galley, _) = super::truncated(ui, &self.title, x.m.bold.clone(), c.fg, room.width() - 2.0 * 44.0);
            painter.galley(egui::pos2(room.center().x - galley.size().x / 2.0, crate::galley_top(ui.ctx(), &galley, rect.center().y)), galley, c.fg);
            if x.look != Look::Mac {
                let side = (rect.height() - 12.0).clamp(20.0, 26.0);
                let r = Rect::from_center_size(egui::pos2(rect.right() - side / 2.0 - 8.0, rect.center().y), egui::Vec2::splat(side));
                let response = ui.interact(r, ui.id().with("window-close"), Sense::click());
                let label = &self.labels[2];
                response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
                let response = response.on_hover_text(label);
                ui.painter().rect_filled(r, CornerRadius::same((side / 2.0) as u8), if response.hovered() { c.button_hover } else { c.button });
                close_cross(ui.painter(), r.center(), c.fg);
                if response.clicked() {
                    action = Some(WindowAction::Close);
                }
            }
        }
        action
    }
}
