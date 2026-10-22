//! Choosing: segmented choices, pop-up buttons with their list, sliders and tabs.

use egui::{Color32, CornerRadius, Rect, Response, Sense, Stroke, Ui, Widget};

use super::{truncated, Ctx};
use crate::Look;

/// A choice of one of `items` in equal segments: in the system looks a grey track with the
/// chosen segment raised out of it; in the plain look the chosen segment in the accent.
/// `response.changed()` when the choice changed. Empty choices are inactive and
/// leave the index unchanged. An out-of-range index shows no selection until input
/// chooses a valid item.
///
/// The builder must be added to a UI or shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::Segmented;
/// Segmented::new("axis", &mut 0, &["X", "Y"]);
/// ```
#[must_use = "add the widget to a UI or call its show method"]
pub struct Segmented<'a> {
    id: egui::IdSalt,
    selected: &'a mut usize,
    items: Vec<(String, String)>,
    width: Option<f32>,
    label: String,
}

impl<'a> Segmented<'a> {
    /// `items` are labels; [`Segmented::tooltips`] adds a tooltip to each.
    pub fn new(id: impl egui::AsIdSalt, selected: &'a mut usize, items: &[&str]) -> Self {
        Segmented {
            id: egui::IdSalt::new(id),
            selected,
            items: items
                .iter()
                .map(|s| (s.to_string(), String::new()))
                .collect(),
            width: None,
            label: String::new(),
        }
    }
    /// Names this group of choices; a surrounding form label is used when unset.
    pub fn accessible_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
    /// A tooltip for each item, in order.
    pub fn tooltips(mut self, tips: &[&str]) -> Self {
        for (item, tip) in self.items.iter_mut().zip(tips) {
            item.1 = tip.to_string();
        }
        self
    }
    /// The whole control's width; by default the width available.
    /// Non-positive and non-finite widths restore the automatic width. Widths above
    /// 1,000,000 points are capped to that value; positive widths below one point use one point.
    pub fn width(mut self, width: f32) -> Self {
        self.width = super::dimension(width);
        self
    }
}

impl Widget for Segmented<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        let name = super::form::accessible_name(ui, &self.label);
        let c = &x.c;
        let before = *self.selected;
        let height = ui.spacing().interact_size.y.max(super::minimum_target(ui));
        let width = self
            .width
            .unwrap_or_else(|| ui.available_width())
            .max(super::minimum_target(ui) * self.items.len() as f32);
        let enabled = ui.is_enabled() && !self.items.is_empty();
        // The control as a whole takes the focus, and the arrows move the choice inside it.
        let (rect, mut response) = super::allocate_control(
            ui,
            egui::vec2(width, height),
            if enabled && !self.items.is_empty() {
                Sense::focusable_noninteractive()
            } else {
                Sense::hover()
            },
        );
        response.flags.set(egui::response::Flags::ENABLED, enabled);
        let n = self.items.len().max(1);
        let each = rect.width() / n as f32;
        let id = response.id.with(("segmented", self.id));
        let focused = choice_keys(ui, &response, self.items.len(), self.selected, x.look);
        let r = x.m.radius;
        if x.look == Look::Material {
            let response = material_segments(
                ui,
                &x,
                rect,
                response,
                id,
                self.selected,
                &self.items,
                before,
            );
            let value = self
                .items
                .get(*self.selected)
                .map(|(label, _)| label.as_str())
                .unwrap_or_default();
            ui.ctx().accesskit_node_builder(response.id, |node| {
                if !name.is_empty() {
                    node.set_label(&name);
                }
                node.set_value(value);
            });
            return response;
        }
        let system = x.look.native();
        if system {
            ui.painter()
                .rect_filled(rect, CornerRadius::same(r.saturating_add(1)), c.track);
            for i in 1..n {
                if i != *self.selected && i != self.selected.saturating_add(1) {
                    let left = (rect.left() + each * i as f32).round() + 0.5;
                    ui.painter().line_segment(
                        [
                            egui::pos2(left, rect.top() + 7.0),
                            egui::pos2(left, rect.bottom() - 7.0),
                        ],
                        Stroke::new(1.0, c.control_border),
                    );
                }
            }
        }
        for (i, (label, tip)) in self.items.iter().enumerate() {
            let slot = Rect::from_min_size(
                egui::pos2(rect.left() + each * i as f32, rect.top()),
                egui::vec2(each, rect.height()),
            );
            let segment = ui.interact(
                slot,
                id.with(i),
                if enabled {
                    Sense::CLICK
                } else {
                    Sense::hover()
                },
            );
            let segment = super::click_response(ui, segment);
            let on = i == *self.selected;
            let colour = if system {
                if on {
                    let fill = if enabled {
                        c.raised
                    } else {
                        c.raised.gamma_multiply(0.6)
                    };
                    let inner = CornerRadius::same(r.saturating_sub(1));
                    let body = slot.shrink(2.0);
                    match x.look {
                        Look::Windows => {
                            crate::paint::fluent(ui.painter(), &x.c, body, inner, fill, enabled)
                        }
                        Look::Gtk => {
                            // libadwaita's toggle group raises its active toggle without an edge.
                            ui.painter().rect_filled(
                                body.translate(egui::vec2(0.0, 1.0)),
                                inner,
                                Color32::from_black_alpha(if c.dark { 50 } else { 20 }),
                            );
                            ui.painter().rect_filled(body, inner, fill);
                        }
                        _ => {
                            crate::paint::raised(ui.painter(), x.c.dark, body, inner, fill, enabled)
                        }
                    }
                } else if segment.hovered() && enabled {
                    ui.painter().rect_filled(
                        slot.shrink(2.0),
                        CornerRadius::same(r.saturating_sub(1)),
                        c.row_hover,
                    );
                }
                if enabled {
                    c.fg
                } else {
                    c.disabled
                }
            } else {
                let round = CornerRadius {
                    nw: if i == 0 { r } else { 0 },
                    sw: if i == 0 { r } else { 0 },
                    ne: if i + 1 == n { r } else { 0 },
                    se: if i + 1 == n { r } else { 0 },
                };
                let fill = match (on, segment.hovered() && enabled) {
                    (true, true) => c.accent_hover,
                    (true, false) => c.accent,
                    (false, true) => c.button_hover,
                    (false, false) => c.button,
                };
                ui.painter().rect_filled(
                    slot,
                    round,
                    if enabled {
                        fill
                    } else {
                        fill.gamma_multiply(0.5)
                    },
                );
                if !enabled {
                    c.disabled
                } else if on {
                    c.on_accent
                } else {
                    c.fg
                }
            };
            let font = if on && system {
                x.m.bold.clone()
            } else {
                x.m.body.clone()
            };
            let (galley, cut) = truncated(ui, label, font, colour, slot.width() - 10.0);
            let top = crate::galley_top(ui.ctx(), &galley, slot.center().y);
            ui.painter().galley(
                egui::pos2(slot.center().x - galley.size().x / 2.0, top),
                galley,
                colour,
            );
            let hover = match (cut, tip.is_empty()) {
                (true, false) => format!("{label}\n\n{tip}"),
                (true, true) => label.clone(),
                (false, _) => tip.clone(),
            };
            let segment = if hover.is_empty() {
                segment
            } else {
                segment.on_hover_text(&hover)
            };
            segment.widget_info(|| {
                egui::WidgetInfo::selected(egui::Role::RadioButton, enabled, on, label)
            });
            if segment.clicked() {
                *self.selected = i;
                choice_clicked(ui, &response, i);
                response.request_focus();
            }
            if i == focused && response.has_focus() {
                crate::paint::ring(
                    ui.painter(),
                    &x.paint(),
                    slot.shrink(2.0),
                    CornerRadius::same(r.saturating_sub(1)),
                );
            }
        }
        if !system {
            ui.painter().rect_stroke(
                rect,
                CornerRadius::same(r),
                Stroke::new(1.0, c.border),
                egui::StrokeKind::Inside,
            );
        }
        choice_accessibility(ui, &response, id, self.items.len());
        for i in 0..self.items.len() {
            ui.ctx().accesskit_node_builder(id.with(i), |node| {
                node.set_toggled(if i == *self.selected {
                    egui::accesskit::Toggled::True
                } else {
                    egui::accesskit::Toggled::False
                })
            });
        }
        let label = self
            .items
            .get(*self.selected)
            .map(|(label, _)| label.clone())
            .unwrap_or_default();
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::RadioGroup, enabled, &label));
        ui.ctx().accesskit_node_builder(response.id, |node| {
            if !name.is_empty() {
                node.set_label(&name);
            }
            node.set_value(label);
        });
        if *self.selected != before {
            response.mark_changed();
        }
        response
    }
}

/// Material 3's segmented buttons in `rect`: an outlined pill split by thin lines, the
/// chosen segment in the tonal container with a tick before its label.
#[allow(clippy::too_many_arguments)]
fn material_segments(
    ui: &mut Ui,
    x: &Ctx,
    rect: Rect,
    mut response: Response,
    id: egui::Id,
    selected: &mut usize,
    items: &[(String, String)],
    before: usize,
) -> Response {
    let c = &x.c;
    let n = items.len().max(1);
    let each = rect.width() / n as f32;
    let pill = (rect.height() / 2.0) as u8;
    let enabled = response.enabled();
    for (i, (label, tip)) in items.iter().enumerate() {
        let slot = Rect::from_min_size(
            egui::pos2(rect.left() + each * i as f32, rect.top()),
            egui::vec2(each, rect.height()),
        );
        let segment = ui.interact(
            slot,
            id.with(i),
            if enabled {
                Sense::CLICK
            } else {
                Sense::hover()
            },
        );
        let segment = super::click_response(ui, segment);
        let round = CornerRadius {
            nw: if i == 0 { pill } else { 0 },
            sw: if i == 0 { pill } else { 0 },
            ne: if i + 1 == n { pill } else { 0 },
            se: if i + 1 == n { pill } else { 0 },
        };
        let on = i == *selected;
        if on {
            ui.painter().rect_filled(slot, round, c.button);
        } else if segment.hovered() && enabled {
            ui.painter()
                .rect_filled(slot, round, c.fg.gamma_multiply(0.08));
        }
        let colour = if enabled { c.fg } else { c.disabled };
        let (galley, _) = truncated(ui, label, x.m.body.clone(), colour, slot.width() - 30.0);
        let tick = if on { 18.0 } else { 0.0 };
        let left = slot.center().x - (galley.size().x + tick) / 2.0;
        if on {
            super::basic::check_mark(
                ui.painter(),
                egui::pos2(left + 7.0, slot.center().y),
                colour,
            );
        }
        let top = crate::galley_top(ui.ctx(), &galley, slot.center().y);
        ui.painter()
            .galley(egui::pos2(left + tick, top), galley, colour);
        if i > 0 {
            ui.painter().line_segment(
                [
                    egui::pos2(slot.left(), slot.top()),
                    egui::pos2(slot.left(), slot.bottom()),
                ],
                Stroke::new(1.0, c.control_strong),
            );
        }
        let segment = if tip.is_empty() {
            segment
        } else {
            segment.on_hover_text(tip)
        };
        segment.widget_info(|| {
            egui::WidgetInfo::selected(egui::Role::RadioButton, enabled, on, label)
        });
        if segment.clicked() {
            *selected = i;
            choice_clicked(ui, &response, i);
            response.request_focus();
        }
        if on && response.has_focus() {
            crate::paint::ring(ui.painter(), &x.paint(), slot, round);
        }
    }
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(pill),
        Stroke::new(1.0, c.control_strong),
        egui::StrokeKind::Inside,
    );
    choice_accessibility(ui, &response, id, items.len());
    for i in 0..items.len() {
        ui.ctx().accesskit_node_builder(id.with(i), |node| {
            node.set_toggled(if i == *selected {
                egui::accesskit::Toggled::True
            } else {
                egui::accesskit::Toggled::False
            })
        });
    }
    let label = items
        .get(*selected)
        .map(|(label, _)| label.clone())
        .unwrap_or_default();
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::RadioGroup, enabled, &label));
    if *selected != before {
        response.mark_changed();
    }
    response
}

#[derive(Clone, Copy)]
struct ChoiceFocus {
    index: usize,
    selected: usize,
    focused: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ChoiceKey {
    First,
    Last,
    Move { forward: bool, select: bool },
    Select,
}

/// Modified navigation that belongs to the application is left untouched.
fn choice_key(key: egui::Key, modifiers: egui::Modifiers, windows: bool) -> Option<ChoiceKey> {
    if modifiers.is_none() {
        match key {
            egui::Key::Home => Some(ChoiceKey::First),
            egui::Key::End => Some(ChoiceKey::Last),
            egui::Key::Space => Some(ChoiceKey::Select),
            egui::Key::ArrowLeft => Some(ChoiceKey::Move {
                forward: false,
                select: true,
            }),
            egui::Key::ArrowRight => Some(ChoiceKey::Move {
                forward: true,
                select: true,
            }),
            egui::Key::ArrowUp if windows => Some(ChoiceKey::Move {
                forward: false,
                select: true,
            }),
            egui::Key::ArrowDown if windows => Some(ChoiceKey::Move {
                forward: true,
                select: true,
            }),
            _ => None,
        }
    } else if windows && modifiers.ctrl && !modifiers.shift && !modifiers.alt && !modifiers.mac_cmd
    {
        match key {
            egui::Key::ArrowLeft | egui::Key::ArrowUp => Some(ChoiceKey::Move {
                forward: false,
                select: false,
            }),
            egui::Key::ArrowRight | egui::Key::ArrowDown => Some(ChoiceKey::Move {
                forward: true,
                select: false,
            }),
            _ => None,
        }
    } else {
        None
    }
}

/// The group has one tab stop; focus and selection can move independently inside it.
fn choice_keys(ui: &Ui, response: &Response, n: usize, selected: &mut usize, look: Look) -> usize {
    let key = response.id.with("choice-focus");
    if n == 0 {
        ui.data_mut(|data| data.remove::<ChoiceFocus>(key));
        return 0;
    }
    let has_focus = response.has_focus() && response.enabled();
    let mut focus = ui
        .data(|data| data.get_temp::<ChoiceFocus>(key))
        .unwrap_or(ChoiceFocus {
            index: if *selected < n { *selected } else { 0 },
            selected: *selected,
            focused: false,
        });
    if focus.selected != *selected || !has_focus || !focus.focused {
        focus.index = if *selected < n { *selected } else { 0 };
    }
    focus.index = focus.index.min(n - 1);
    if has_focus {
        let windows = look == Look::Windows;
        ui.memory_mut(|memory| {
            memory.set_focus_lock_filter(
                response.id,
                egui::EventFilter {
                    horizontal_arrows: true,
                    vertical_arrows: windows,
                    ..Default::default()
                },
            )
        });
        let mut handled = false;
        ui.input_mut(|input| {
            input.events.retain(|event| {
                let egui::Event::Key {
                    key,
                    modifiers,
                    pressed: true,
                    ..
                } = event
                else {
                    return true;
                };
                let Some(action) = choice_key(*key, *modifiers, windows) else {
                    return true;
                };
                handled = true;
                match action {
                    ChoiceKey::First => {
                        focus.index = 0;
                        *selected = focus.index;
                    }
                    ChoiceKey::Last => {
                        focus.index = n - 1;
                        *selected = focus.index;
                    }
                    ChoiceKey::Move { forward, select } => {
                        focus.index = if forward {
                            focus.index.saturating_add(1).min(n - 1)
                        } else {
                            focus.index.saturating_sub(1)
                        };
                        if select {
                            *selected = focus.index;
                        }
                    }
                    ChoiceKey::Select => *selected = focus.index,
                }
                false
            })
        });
        if handled {
            ui.memory_mut(|memory| memory.move_focus(egui::FocusDirection::None));
        }
    }
    focus.selected = *selected;
    focus.focused = has_focus;
    ui.data_mut(|data| data.insert_temp(key, focus));
    focus.index
}

fn choice_clicked(ui: &Ui, response: &Response, index: usize) {
    ui.data_mut(|data| {
        data.insert_temp(
            response.id.with("choice-focus"),
            ChoiceFocus {
                index,
                selected: index,
                focused: true,
            },
        )
    });
}

fn choice_accessibility(ui: &Ui, response: &Response, children: egui::Id, count: usize) {
    let focus = ui.data(|data| data.get_temp::<ChoiceFocus>(response.id.with("choice-focus")));
    let focused = response.has_focus();
    ui.ctx().accesskit_node_builder(response.id, |node| {
        if let Some(focus) = focus.filter(|_| focused) {
            node.set_active_descendant(children.with(focus.index).accesskit_id());
        } else {
            node.clear_active_descendant();
        }
    });
    for i in 0..count {
        ui.ctx().accesskit_node_builder(children.with(i), |node| {
            node.set_position_in_set(i + 1);
            node.set_size_of_set(count);
        });
    }
}

/// While the item `focused` of a row of choices has the focus, Left and Right choose the
/// item before or after it and move the focus there; the arrows stay in the row.
fn arrow_keys(ui: &Ui, focused: Option<usize>, ids: &[egui::Id], selected: &mut usize) {
    let Some(i) = focused else { return };
    let filter = egui::EventFilter {
        horizontal_arrows: true,
        ..Default::default()
    };
    ui.memory_mut(|m| m.set_focus_lock_filter(ids[i], filter));
    let next = ui.input_mut(|input| {
        if input.consume_key(egui::Modifiers::NONE, egui::Key::Home) {
            Some(0)
        } else if input.consume_key(egui::Modifiers::NONE, egui::Key::End) {
            Some(ids.len() - 1)
        } else if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight) {
            Some((i + 1).min(ids.len() - 1))
        } else if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft) {
            Some(i.saturating_sub(1))
        } else {
            None
        }
    });
    let Some(next) = next else { return };
    *selected = next;
    if next != i {
        ui.memory_mut(|m| {
            m.request_focus(ids[next]);
            // The next frame's arrows reach it before it can ask for them itself.
            m.set_focus_lock_filter(ids[next], filter);
        });
    }
}

/// A pop-up button with its list: the chosen item on the button, the items in a list
/// under it (or over it, where there is more room). `response.changed()` when the choice
/// changed. An empty list is disabled and leaves the index unchanged. An out-of-range
/// index shows a blank button until the user chooses an item.
///
/// The builder must be added to a UI or shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::Dropdown;
/// Dropdown::new("shape", &mut 0, &["Cube", "Sphere"]);
/// ```
#[must_use = "add the widget to a UI or call its show method"]
pub struct Dropdown<'a> {
    id: egui::IdSalt,
    selected: &'a mut usize,
    items: Vec<String>,
    width: Option<f32>,
    label: String,
}

impl<'a> Dropdown<'a> {
    /// A pop-up button choosing `selected`, an index into `items`; `id` tells its list
    /// from others'.
    pub fn new(id: impl egui::AsIdSalt, selected: &'a mut usize, items: &[&str]) -> Self {
        Dropdown {
            id: egui::IdSalt::new(id),
            selected,
            items: items.iter().map(|s| s.to_string()).collect(),
            width: None,
            label: String::new(),
        }
    }
    /// Names the choice for assistive technology; a surrounding form label is used when unset.
    pub fn accessible_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
    /// The button's width; by default its longest item's.
    /// Non-positive and non-finite widths restore the automatic width. Widths above
    /// 1,000,000 points are capped to that value; positive widths below one point use one point.
    pub fn width(mut self, width: f32) -> Self {
        self.width = super::dimension(width);
        self
    }
}

#[derive(Clone, Default)]
struct DropdownSearch {
    prefix: String,
    time: f64,
    frame: u64,
}

impl DropdownSearch {
    fn find(&mut self, text: &str, items: &[String], current: usize, time: f64) -> Option<usize> {
        let text = text.to_lowercase();
        if text.is_empty() || text.chars().any(char::is_control) || items.is_empty() {
            return None;
        }
        if time - self.time > 1.0 || time < self.time {
            self.prefix.clear();
        }
        self.time = time;
        let cycle = self.prefix == text && text.chars().count() == 1;
        if !cycle {
            self.prefix.push_str(&text);
        }
        let start = if cycle || self.prefix == text {
            current + 1
        } else {
            current
        };
        (0..items.len())
            .map(|offset| (start + offset) % items.len())
            .find(|&index| items[index].to_lowercase().starts_with(&self.prefix))
    }
}

impl Widget for Dropdown<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        let c = x.c;
        let before = *self.selected;
        let height = ui.spacing().interact_size.y;
        let font = x.m.body.clone();
        let longest = self
            .items
            .iter()
            .map(|t| ui.fonts_mut(|f| f.layout_no_wrap(t.clone(), font.clone(), c.fg).size().x))
            .fold(0.0, f32::max);
        let width = self.width.unwrap_or(longest + 44.0);
        let text = self.items.get(*self.selected).cloned().unwrap_or_default();
        let row_height = height.max(20.0);
        let selected = &mut *self.selected;
        let look = x.look;
        let items = &self.items;
        let mut response = PopupButton::new(self.id, text)
            .accessible_label(self.label)
            .enabled(!items.is_empty())
            .width(width)
            .show(ui, |ui| {
                let max_height = (ui.ctx().content_rect().height() - 32.0).clamp(1.0, 320.0);
                egui::ScrollArea::vertical()
                    .max_height(max_height)
                    .show(ui, |ui| {
                        let mut ids = Vec::with_capacity(items.len());
                        let mut focused = None;
                        for (i, item) in items.iter().enumerate() {
                            let (row, hit) = super::allocate_control(
                                ui,
                                egui::vec2(ui.available_width(), row_height),
                                Sense::click(),
                            );
                            // macOS and the plain look light the row under the pointer in the accent;
                            // WinUI and libadwaita in a tint.
                            let accent_row = matches!(look, Look::Mac | Look::Plain);
                            ids.push(hit.id);
                            if hit.has_focus() {
                                focused = Some(i);
                                if hit.gained_focus() {
                                    hit.scroll_to_me(Some(egui::Align::Center));
                                }
                            }
                            hit.widget_info(|| {
                                egui::WidgetInfo::selected(
                                    egui::Role::MenuItem,
                                    ui.is_enabled(),
                                    i == *selected,
                                    item,
                                )
                            });
                            let hot = hit.hovered() || hit.has_focus();
                            if hot {
                                ui.painter().rect_filled(
                                    row,
                                    CornerRadius::same(4),
                                    if accent_row { c.accent } else { c.button_hover },
                                );
                            } else if i == *selected && look == Look::Windows {
                                ui.painter()
                                    .rect_filled(row, CornerRadius::same(4), c.row_hover);
                                // WinUI marks the chosen item with an accent bar at its start.
                                let bar = Rect::from_center_size(
                                    egui::pos2(row.left() + 2.0, row.center().y),
                                    egui::vec2(3.0, row.height() * 0.5),
                                );
                                ui.painter()
                                    .rect_filled(bar, CornerRadius::same(2), c.accent);
                            }
                            let colour = if hot && accent_row { c.on_accent } else { c.fg };
                            if i == *selected && look != Look::Windows {
                                super::basic::check_mark(
                                    ui.painter(),
                                    egui::pos2(row.left() + 12.0, row.center().y),
                                    colour,
                                );
                            }
                            crate::text(
                                ui.painter(),
                                egui::pos2(row.left() + 26.0, row.center().y),
                                egui::Align2::LEFT_CENTER,
                                item,
                                font.clone(),
                                colour,
                            );
                            if hit.clicked() {
                                *selected = i;
                                ui.close();
                            }
                        }
                        // Focus follows navigation without changing the value until activation.
                        if !ids.is_empty() {
                            let current = focused.unwrap_or((*selected).min(ids.len() - 1));
                            let next = ui.input_mut(|input| {
                                if input.consume_key(egui::Modifiers::NONE, egui::Key::Home) {
                                    Some(0)
                                } else if input.consume_key(egui::Modifiers::NONE, egui::Key::End) {
                                    Some(ids.len() - 1)
                                } else if input
                                    .consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)
                                {
                                    Some((current + 1).min(ids.len() - 1))
                                } else if input
                                    .consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)
                                {
                                    Some(current.saturating_sub(1))
                                } else {
                                    None
                                }
                            });
                            let search_id = ui.scope_id().with("dropdown-search");
                            let frame = ui.ctx().cumulative_frame_nr();
                            let mut search = ui
                                .data(|data| data.get_temp::<DropdownSearch>(search_id))
                                .unwrap_or_default();
                            if frame.saturating_sub(search.frame) > 1 {
                                search.prefix.clear();
                            }
                            search.frame = frame;
                            let mut target = next;
                            if next.is_some() {
                                search.prefix.clear();
                            }
                            ui.input_mut(|input| {
                                input.events.retain(|event| {
                                    if let egui::Event::Text(text) = event {
                                        target = search
                                            .find(
                                                text,
                                                items,
                                                target.unwrap_or(current),
                                                input.time,
                                            )
                                            .or(target);
                                        false
                                    } else {
                                        true
                                    }
                                });
                            });
                            ui.data_mut(|data| data.insert_temp(search_id, search));
                            let pointer_busy = ui.input(|input| {
                                input.pointer.any_down() || input.pointer.any_click()
                            });
                            if let Some(next) = target
                                .or_else(|| (focused.is_none() && !pointer_busy).then_some(current))
                            {
                                ui.memory_mut(|memory| {
                                    memory.move_focus(egui::FocusDirection::None);
                                    memory.request_focus(ids[next]);
                                });
                            }
                        }
                    });
            })
            .response;
        if *self.selected != before {
            response.mark_changed();
        }
        response
    }
}

#[derive(Clone)]
struct MenuFocus {
    pass: u64,
    items: Vec<(egui::Id, Rect)>,
}

pub(super) fn register_menu_item(ui: &Ui, response: &Response) -> bool {
    if !response.enabled() {
        return false;
    }
    let key = ui.layer_id().id.with("menu-focus");
    let pass = ui.ctx().cumulative_pass_nr();
    ui.data_mut(|data| {
        if let Some(mut menu) = data.get_temp::<MenuFocus>(key) {
            if menu.pass == pass {
                menu.items.push((response.id, response.rect));
                data.insert_temp(key, menu);
                return true;
            }
        }
        false
    })
}

/// Each popup keeps its own enabled rows so nested navigation cannot reach its parent.
pub(super) fn menu_contents<R>(
    ui: &mut Ui,
    launcher: egui::Id,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    let key = ui.layer_id().id.with("menu-focus");
    let pass = ui.ctx().cumulative_pass_nr();
    ui.data_mut(|data| {
        data.insert_temp(
            key,
            MenuFocus {
                pass,
                items: Vec::new(),
            },
        )
    });
    let bounds = ui.ctx().content_rect();
    let max_width = (bounds.width() - 24.0).max(1.0);
    ui.set_max_width(ui.available_width().min(max_width));
    egui::ScrollArea::vertical()
        .id_salt("menu-rows")
        .max_height((bounds.height() - 32.0).max(1.0))
        .min_scrolled_height(1.0)
        .scroll_source(egui::scroll_area::ScrollSource::ALL)
        .show(ui, |ui| {
            let result = add(ui);
            menu_keys(ui, launcher);
            result
        })
        .inner
}

fn menu_keys(ui: &Ui, launcher: egui::Id) {
    let key = ui.layer_id().id.with("menu-focus");
    let Some(menu) = ui.data(|data| data.get_temp::<MenuFocus>(key)) else {
        return;
    };
    if menu.items.is_empty() {
        return;
    }
    let focused = ui.memory(|memory| memory.focused());
    let in_menu = focused.is_some_and(|focused| menu.items.iter().any(|(id, _)| *id == focused));
    let target = if in_menu || focused.is_none() {
        ui.input_mut(|input| {
            if input.consume_key(egui::Modifiers::NONE, egui::Key::Home) {
                menu.items.first().copied()
            } else if input.consume_key(egui::Modifiers::NONE, egui::Key::End) {
                menu.items.last().copied()
            } else {
                None
            }
        })
    } else if focused == Some(launcher)
        && !ui.input(|input| input.pointer.any_down() || input.pointer.any_click())
    {
        menu.items.first().copied()
    } else {
        None
    };
    if let Some((target, rect)) = target {
        ui.memory_mut(|memory| memory.request_focus(target));
        ui.scroll_to_rect_animation(rect, None, egui::style::ScrollAnimation::none());
    }
}

/// A pop-up button whose list is the app's own: `show` draws the button with `text` on
/// it and, while it is open, whatever the closure adds under it, which can be
/// [`menu_value`] rows, egui's `selectable_value`, icons or columns. A click on a row closes
/// it, as a menu's does.
///
/// ```no_run
/// # #[derive(PartialEq, Clone, Copy)] enum Units { Metres, Feet }
/// # fn demo(ui: &mut egui::Ui, units: &mut Units) {
/// use egui_native_look as look;
/// let text = if *units == Units::Metres { "Metres" } else { "Feet" };
/// look::PopupButton::new("units", text).width(140.0).show(ui, |ui| {
///     look::menu_value(ui, units, Units::Metres, "Metres");
///     look::menu_value(ui, units, Units::Feet, "Feet");
/// });
/// # }
/// ```
///
/// The builder must be added to a UI or shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::PopupButton;
/// PopupButton::new("units", "Metres");
/// ```
#[must_use = "add the widget to a UI or call its show method"]
pub struct PopupButton {
    id: egui::IdSalt,
    text: String,
    width: Option<f32>,
    enabled: bool,
    label: String,
}

impl PopupButton {
    /// A button reading `text`, the choice it stands for; `id` tells its list from
    /// others'.
    pub fn new(id: impl egui::AsIdSalt, text: impl Into<String>) -> Self {
        PopupButton {
            id: egui::IdSalt::new(id),
            text: text.into(),
            width: None,
            enabled: true,
            label: String::new(),
        }
    }
    /// Names the popup for assistive technology independently of its current text.
    pub fn accessible_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
    /// The button's width; by default its text's and the arrows'.
    /// Non-positive and non-finite widths restore the automatic width. Widths above
    /// 1,000,000 points are capped to that value; positive widths below one point use one point.
    pub fn width(mut self, width: f32) -> Self {
        self.width = super::dimension(width);
        self
    }

    fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Draws the button and, while open, the list `add` fills. The inner value is what
    /// `add` returned, while open.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<Option<R>> {
        let x = Ctx::of(ui);
        let c = x.c;
        let height = ui.spacing().interact_size.y;
        let font = x.m.body.clone();
        let width = self.width.unwrap_or_else(|| {
            ui.fonts_mut(|f| {
                f.layout_no_wrap(self.text.clone(), font.clone(), c.fg)
                    .size()
                    .x
            }) + 44.0
        });
        let enabled = ui.is_enabled() && self.enabled;
        let (rect, mut response) = super::allocate_control(
            ui,
            egui::vec2(width, height),
            if enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        response.flags.set(egui::response::Flags::ENABLED, enabled);
        // The button's own id, unique to where it is laid out, tells the list apart from
        // another button's built with the same `id` (the rows of a loop).
        let popup = response.id.with((self.id, "egui_native_look-dropdown"));
        if !enabled {
            egui::Popup::close_id(ui.ctx(), popup);
        }
        let open = egui::Popup::is_id_open(ui.ctx(), popup);
        let arrows =
            crate::paint::popup_button(ui.painter(), &x.paint(), rect, &response, enabled, open);
        let colour = if enabled { c.fg } else { c.disabled };
        let (galley, _) = truncated(ui, &self.text, font, colour, rect.width() - 10.0 - arrows);
        let top = crate::galley_top(ui.ctx(), &galley, rect.center().y);
        ui.painter()
            .galley(egui::pos2(rect.left() + 10.0, top), galley, colour);
        let inner = egui::Popup::menu(&response)
            .id(popup)
            .width(width.min((ui.ctx().content_rect().width() - 16.0).max(1.0)))
            .show(|ui| {
                ui.set_min_width(
                    (width - 8.0).min((ui.ctx().content_rect().width() - 24.0).max(1.0)),
                );
                menu_contents(ui, response.id, add)
            });
        let expanded = egui::Popup::is_id_open(ui.ctx(), popup);
        let outside_click = ui.input(|input| {
            input.pointer.any_click()
                && input.pointer.interact_pos().is_some_and(|position| {
                    !rect.contains(position)
                        && inner
                            .as_ref()
                            .is_none_or(|inner| !inner.response.rect.contains(position))
                })
        });
        if enabled && open && !expanded && !outside_click {
            response.request_focus();
        }
        let text = self.text;
        let name = super::form::accessible_name(ui, &self.label);
        let name = if name.is_empty() { &text } else { &name };
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::ComboBox, enabled, name));
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_value(text);
            node.set_has_popup(egui::accesskit::HasPopup::Menu);
        });
        ui.ctx()
            .accesskit_node_builder(response.id, |node| node.set_expanded(expanded));
        egui::InnerResponse::new(inner.map(|i| i.inner), response)
    }
}

/// A row of a [`PopupButton`]'s list choosing `value`: a [`super::MenuItem`] ticked while
/// `*current == value`, which sets it when clicked.
pub fn menu_value<V: PartialEq>(
    ui: &mut Ui,
    current: &mut V,
    value: V,
    label: impl Into<String>,
) -> Response {
    let mut response = ui.add(super::MenuItem::new(label).checked(*current == value));
    if response.clicked() && *current != value {
        *current = value;
        response.mark_changed();
    }
    response
}

/// Draws a pop-up button's arrows in the look, for egui's own combo box:
/// `egui::ComboBox::from_id_salt("units").icon(egui_native_look::combo_icon)`. egui
/// passes the room it leaves for an icon at the button's end.
pub fn combo_icon(ui: &Ui, rect: Rect, visuals: &egui::style::WidgetVisuals, open: bool) {
    let x = Ctx::of(ui);
    let c = &x.c;
    let m = rect.center();
    if x.look == Look::Mac {
        let side = rect.height().clamp(12.0, 16.0);
        let square = Rect::from_center_size(m, egui::vec2(side, side));
        ui.painter()
            .rect_filled(square, CornerRadius::same(4), c.accent);
        let m = square.center();
        let stroke = Stroke::new(1.4, Color32::WHITE);
        ui.painter().add(egui::Shape::line(
            vec![
                m + egui::vec2(-2.8, -1.5),
                m + egui::vec2(0.0, -4.2),
                m + egui::vec2(2.8, -1.5),
            ],
            stroke,
        ));
        ui.painter().add(egui::Shape::line(
            vec![
                m + egui::vec2(-2.8, 1.5),
                m + egui::vec2(0.0, 4.2),
                m + egui::vec2(2.8, 1.5),
            ],
            stroke,
        ));
    } else {
        let flip = if open { -1.0 } else { 1.0 };
        let chevron = vec![
            m + egui::vec2(-4.0, -2.0 * flip),
            m + egui::vec2(0.0, 2.0 * flip),
            m + egui::vec2(4.0, -2.0 * flip),
        ];
        let colour = if ui.is_enabled() {
            visuals.fg_stroke.color.gamma_multiply(0.8)
        } else {
            c.disabled
        };
        ui.painter()
            .add(egui::Shape::line(chevron, Stroke::new(1.5, colour)));
    }
}

/// A slider over `range`: a thin rail filled with the accent up to a knob, white with a
/// shadow on macOS and in libadwaita, WinUI's accent dot in a ring on Windows. Any of
/// egui's numbers, linear or logarithmic, in steps, with a label and the number beside
/// it.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui, exposure: &mut f32, samples: &mut u32) {
/// use egui_native_look as look;
/// ui.add(look::Slider::new(exposure, 0.01..=100.0).logarithmic(true).text("Exposure"));
/// ui.add(look::Slider::new(samples, 1..=64).show_value(true));
/// ui.add(look::Slider::new(exposure, 0.0..=1.0).show_value(true).custom_formatter(|v, _| format!("{:.0} %", v * 100.0)));
/// # }
/// ```
///
/// The builder must be added to a UI or shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::Slider;
/// Slider::new(&mut 0.5, 0.0..=1.0);
/// ```
#[must_use = "add the widget to a UI or call its show method"]
pub struct Slider<'a, T: egui::emath::Numeric> {
    value: &'a mut T,
    range: std::ops::RangeInclusive<f64>,
    width: Option<f32>,
    text: String,
    logarithmic: bool,
    step: Option<f64>,
    clamping: egui::SliderClamping,
    show_value: bool,
    label: String,
    format: super::entry::Format<'a>,
}

impl<'a, T: egui::emath::Numeric> Slider<'a, T> {
    /// A slider moving `value` across `range`. Non-finite endpoints disable editing
    /// and preserve the value. Equal endpoints disable editing; `Always` clamping
    /// can still normalize the value. NaN values draw at the start and are replaced
    /// only by an edit or `Always` clamping.
    /// In the GTK look, arrows and +/− move one step; Ctrl moves ten steps.
    /// Page Up/Down change the value by ten steps, and Home/End select the numeric
    /// minimum/maximum. Ctrl+Page Up/Down move toward the left/right rail endpoints.
    pub fn new(value: &'a mut T, range: std::ops::RangeInclusive<T>) -> Self {
        let range = range.start().to_f64()..=range.end().to_f64();
        Slider {
            value,
            range,
            width: None,
            text: String::new(),
            logarithmic: false,
            step: None,
            clamping: egui::SliderClamping::default(),
            show_value: false,
            label: String::new(),
            format: super::entry::Format::default(),
        }
    }
    /// The rail's width; by default egui's `slider_width`.
    /// Non-positive and non-finite widths restore the automatic width. Widths above
    /// 1,000,000 points are capped to that value; positive widths below one point use one point.
    pub fn width(mut self, width: f32) -> Self {
        self.width = super::dimension(width);
        self
    }
    /// A label after it.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }
    /// What it sets, for a screen reader, when no [`Slider::text`] says so. In a
    /// [`crate::Form`] row the row's label is used when this is not set.
    pub fn accessible_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
    /// Spreads the range logarithmically, for a range of positive numbers spanning
    /// orders of magnitude.
    pub fn logarithmic(mut self, logarithmic: bool) -> Self {
        self.logarithmic = logarithmic;
        self
    }
    /// Moves in steps of `step`, and the arrow keys by one step. Non-finite and
    /// non-positive steps are ignored; integer steps are rounded and are at least one.
    pub fn step_by(mut self, step: f64) -> Self {
        self.step = (step.is_finite() && step > 0.0).then_some(if T::INTEGRAL {
            step.round().max(1.0)
        } else {
            step
        });
        self
    }
    /// When a value outside the range is pulled into it, as egui's slider does: by
    /// default only when the slider changes it.
    pub fn clamping(mut self, clamping: egui::SliderClamping) -> Self {
        self.clamping = clamping;
        self
    }
    /// The number after the rail, in a field to drag or type in, written as the
    /// methods below say: `"1.5×"`, `"90°"`, `"75 %"`.
    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }
    super::entry::format_methods!();

    fn log(&self) -> bool {
        self.logarithmic && *self.range.start() > 0.0 && self.range.end() > self.range.start()
    }

    /// Where `value` sits along the rail, from 0 to 1.
    fn fraction(&self, value: f64) -> f64 {
        let (lo, hi) = (*self.range.start(), *self.range.end());
        if value.is_nan() || hi == lo {
            0.0
        } else if self.log() {
            ((value.max(lo).ln() - lo.ln()) / (hi.ln() - lo.ln())).clamp(0.0, 1.0)
        } else if (hi - lo).is_finite() {
            ((value - lo) / (hi - lo)).clamp(0.0, 1.0)
        } else {
            ((value * 0.5 - lo * 0.5) / (hi * 0.5 - lo * 0.5)).clamp(0.0, 1.0)
        }
    }

    /// The value at `t` along the rail, in a step and a whole number where it must be.
    fn value_at(&self, t: f64) -> f64 {
        let (lo, hi) = (*self.range.start(), *self.range.end());
        if t <= 0.0 {
            return lo;
        }
        if t >= 1.0 {
            return hi;
        }
        let mut value = if self.log() {
            ((1.0 - t) * lo.ln() + t * hi.ln()).exp()
        } else {
            (1.0 - t) * lo + t * hi
        };
        if let Some(step) = self.step {
            let stepped = lo + ((value - lo) / step).round() * step;
            if stepped.is_finite() {
                value = stepped;
            }
        }
        if T::INTEGRAL {
            value = value.round();
        }
        value.clamp(lo.min(hi), hi.max(lo))
    }

    /// Move by numeric steps or fractions of the rail; visual movement follows its direction.
    fn nudged(&self, steps: f64, visual: bool) -> f64 {
        self.nudged_from(self.value.to_f64(), steps, visual)
    }

    fn nudged_from(&self, current: f64, steps: f64, visual: bool) -> f64 {
        let (lo, hi) = (*self.range.start(), *self.range.end());
        let direction = if hi < lo { -1.0 } else { 1.0 };
        let numeric_steps = steps * if visual { direction } else { 1.0 };
        let current = if current.is_nan() { lo } else { current };
        let value = match self.step {
            Some(step) if !self.log() => current + step * numeric_steps,
            None if T::INTEGRAL && !self.log() => current + numeric_steps,
            _ => self.value_at(self.fraction(current) + 0.01 * numeric_steps * direction),
        };
        // Integer rounding must not swallow a keyboard step on a short range.
        let value = if value == current {
            self.step
                .or(T::INTEGRAL.then_some(1.0))
                .map_or(value, |step| value + step * numeric_steps)
        } else {
            value
        };
        if value.is_nan() {
            if numeric_steps > 0.0 {
                hi.max(lo)
            } else {
                lo.min(hi)
            }
        } else {
            value.clamp(lo.min(hi), hi.max(lo))
        }
    }

    fn set(&mut self, value: f64, response: &mut Response) {
        let value = T::from_f64(value);
        if super::number_changed(self.value.to_f64(), value.to_f64()) {
            *self.value = value;
            response.mark_changed();
        }
    }
}

impl<T: egui::emath::Numeric> Widget for Slider<'_, T> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let (lo, hi) = (*self.range.start(), *self.range.end());
        let invalid = !lo.is_finite() || !hi.is_finite();
        if invalid {
            self.range = 0.0..=0.0;
            self.clamping = egui::SliderClamping::Never;
        }
        if invalid || lo == hi {
            ui.add_enabled_ui(false, |ui| self.controls(ui)).inner
        } else {
            self.controls(ui)
        }
    }
}

impl<T: egui::emath::Numeric> Slider<'_, T> {
    fn controls(mut self, ui: &mut Ui) -> Response {
        let (lo, hi) = (*self.range.start(), *self.range.end());
        let before = self.value.to_f64();
        if self.clamping == egui::SliderClamping::Always {
            let current = self.value.to_f64();
            let clamped = if current.is_nan() {
                lo
            } else {
                current.clamp(lo.min(hi), hi.max(lo))
            };
            *self.value = T::from_f64(clamped);
        }
        let clamped = super::number_changed(before, self.value.to_f64());
        if !self.show_value && self.text.is_empty() {
            let mut response = self.rail(ui);
            if clamped {
                response.mark_changed();
            }
            return response;
        }
        let inner = ui.horizontal(|ui| {
            let mut response = self.rail(ui);
            if self.show_value {
                let before = self.value.to_f64();
                let mut shown = if before.is_nan() {
                    T::from_f64(lo)
                } else {
                    *self.value
                };
                let speed = self.step.unwrap_or((hi / 200.0 - lo / 200.0).abs());
                let range = if self.clamping == egui::SliderClamping::Never {
                    f64::NEG_INFINITY..=f64::INFINITY
                } else {
                    lo..=hi
                };
                let format = std::mem::take(&mut self.format);
                // At least 56 points wide, wider for a long prefix or suffix.
                let field = ui
                    .scope(|ui| {
                        ui.spacing_mut().interact_size.x = 56.0;
                        super::ignore_disabled_numeric_actions(ui, ui.next_auto_id());
                        ui.add(
                            format
                                .apply(egui::DragValue::new(&mut shown).speed(speed).range(range)),
                        )
                    })
                    .inner;
                if field.changed() {
                    *self.value = shown;
                }
                if super::number_changed(before, self.value.to_f64()) {
                    response.mark_changed();
                    self.update_accessible_node(ui, &response);
                }
                response |= field;
            }
            if !self.text.is_empty() {
                ui.label(&self.text);
            }
            response
        });
        let mut response = inner.inner;
        if clamped {
            response.mark_changed();
        }
        response
    }
}

impl<T: egui::emath::Numeric> Slider<'_, T> {
    /// A finite value matching the knob position, without implicitly normalizing the model.
    fn accessible_value(&self) -> f64 {
        let (start, end) = (*self.range.start(), *self.range.end());
        let value = self.value.to_f64();
        if value.is_nan() {
            start
        } else {
            value.clamp(start.min(end), start.max(end))
        }
    }

    /// Accessibility changes use numeric direction and do not require keyboard focus.
    fn accessible_input(&mut self, ui: &mut Ui, response: &mut Response) {
        use egui::accesskit::{Action, ActionData};
        ui.input_mut(|input| {
            input.consume_accesskit_action_requests(response.id, |request| {
                let value = match request.action {
                    Action::Increment => response
                        .enabled()
                        .then(|| self.nudged_from(self.accessible_value(), 1.0, false)),
                    Action::Decrement => response
                        .enabled()
                        .then(|| self.nudged_from(self.accessible_value(), -1.0, false)),
                    Action::SetValue => {
                        if !response.enabled() {
                            return true;
                        }
                        match request.data {
                            Some(ActionData::NumericValue(value)) if !value.is_nan() => {
                                let (lo, hi) = (*self.range.start(), *self.range.end());
                                Some(value.clamp(lo.min(hi), hi.max(lo)))
                            }
                            _ => None,
                        }
                    }
                    _ => return false,
                };
                if let Some(value) = value {
                    self.set(value, response);
                }
                true
            })
        });
    }

    /// Keep advertised actions consistent with the value and representable numeric steps.
    fn update_accessible_node(&self, ui: &Ui, response: &Response) {
        use egui::accesskit::Action;
        let (start, end) = (*self.range.start(), *self.range.end());
        let (lo, hi) = (start.min(end), start.max(end));
        let value = self.accessible_value();
        ui.ctx().accesskit_node_builder(response.id, |node| {
            let increment = T::from_f64(self.nudged_from(value, 1.0, false)).to_f64();
            let decrement = T::from_f64(self.nudged_from(value, -1.0, false)).to_f64();
            let step = if self.log() {
                None
            } else {
                Some(self.step.unwrap_or_else(|| {
                    if T::INTEGRAL {
                        1.0
                    } else {
                        (hi / 100.0 - lo / 100.0).abs()
                    }
                }))
            };
            node.set_min_numeric_value(lo);
            node.set_max_numeric_value(hi);
            node.set_numeric_value(value);
            node.clear_numeric_value_step();
            if let Some(step) = step.filter(|step| *step > 0.0 && step.is_finite()) {
                node.set_numeric_value_step(step);
            }
            for action in [Action::Increment, Action::Decrement, Action::SetValue] {
                node.remove_action(action);
            }
            if response.enabled() {
                node.add_action(Action::SetValue);
                if value < hi && increment > value {
                    node.add_action(Action::Increment);
                }
                if value > lo && decrement < value {
                    node.add_action(Action::Decrement);
                }
            }
        });
    }

    /// Keep the unrounded fraction between moves so small drags accumulate across numeric steps.
    fn pointer_input(
        &mut self,
        ui: &mut Ui,
        response: &mut Response,
        rail: Rect,
        radius: f32,
        look: Look,
    ) -> bool {
        use egui::response::Flags;
        let id = response.id.with("slider-drag");
        let mut drag = ui.data(|data| data.get_temp::<SliderDrag>(id));
        let (pressed, released, down, touch, decided, cancelled, shift, press_time) =
            ui.input(|input| {
                (
                    input.pointer.primary_pressed(),
                    input.pointer.primary_released(),
                    input.pointer.primary_down(),
                    input.any_touches()
                        || input.events.iter().any(|event| {
                            matches!(
                                event,
                                egui::Event::Touch {
                                    phase: egui::TouchPhase::Start,
                                    ..
                                }
                            )
                        }),
                    input.pointer.is_decidedly_dragging(),
                    super::pointer_cancelled(input),
                    input.modifiers.shift,
                    input.pointer.press_start_time(),
                )
            });
        let frame = ui.ctx().cumulative_frame_nr();
        if drag.is_some_and(|state| {
            (down && state.press_time != press_time) || frame.saturating_sub(state.seen_frame) > 1
        }) {
            drag = None;
            ui.data_mut(|data| data.remove::<SliderDrag>(id));
            if ui.ctx().dragged_id() == Some(response.id) {
                ui.ctx().stop_dragging();
            }
        }
        let escape = look == Look::Gtk
            && down
            && drag.is_some_and(|state| !state.ignored)
            && ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        if !response.enabled() || cancelled || escape || (!down && !released) {
            if drag.is_some_and(|state| state.claimed) {
                response.flags.insert(Flags::DRAG_STOPPED);
            }
            ui.data_mut(|data| data.remove::<SliderDrag>(id));
            if ui.ctx().dragged_id() == Some(response.id) {
                ui.ctx().stop_dragging();
            }
            return false;
        }
        let Some(position) = super::pointer_position(ui, response) else {
            return drag.is_some();
        };
        if pressed
            && drag.is_none()
            && (response.is_pointer_button_down_on()
                || response.clicked_by(egui::PointerButton::Primary))
        {
            response.request_focus();
            let fraction = self.fraction(self.value.to_f64());
            let thumb_x = egui::lerp(rail.left()..=rail.right(), fraction as f32);
            let relative =
                matches!(look, Look::Gtk | Look::Ios) && (position.x - thumb_x).abs() <= radius;
            let fraction = if relative {
                fraction
            } else {
                ((position.x - rail.left()) / rail.width()).clamp(0.0, 1.0) as f64
            };
            drag = Some(SliderDrag {
                press_time,
                seen_frame: frame,
                origin: position,
                last: position,
                fraction,
                touch,
                relative,
                claimed: false,
                ignored: false,
            });
            if !touch && !relative {
                self.set(self.value_at(fraction), response);
            }
        }
        if let Some(state) = drag.as_mut() {
            let delta = position - state.origin;
            if !state.claimed && !state.ignored && decided {
                if delta.y.abs() > delta.x.abs() {
                    state.ignored = true;
                } else {
                    state.claimed = true;
                    response.flags.insert(Flags::DRAG_STARTED);
                }
            }
            if state.claimed && !state.ignored {
                let fine = if look == Look::Gtk && shift {
                    fine_drag_scale(rail.width(), *self.range.start(), *self.range.end())
                } else {
                    1.0
                };
                state.advance(position.x, rail.width(), fine);
                self.set(self.value_at(state.fraction), response);
                if down {
                    ui.ctx().set_dragged_id(response.id);
                    response
                        .flags
                        .insert(Flags::DRAGGED | Flags::IS_POINTER_BUTTON_DOWN_ON);
                }
                if released {
                    response.flags.insert(Flags::DRAG_STOPPED);
                }
            } else if released
                && state.touch
                && !state.relative
                && !state.ignored
                && response.clicked_by(egui::PointerButton::Primary)
            {
                self.set(self.value_at(state.fraction), response);
            }
        }
        let active = drag.is_some_and(|state| !state.ignored) && down;
        if down {
            if let Some(mut state) = drag {
                state.seen_frame = frame;
                ui.data_mut(|data| data.insert_temp(id, state));
            }
        } else {
            ui.data_mut(|data| data.remove::<SliderDrag>(id));
        }
        active
    }

    /// The rail and its knob.
    fn rail(&mut self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        let c = &x.c;
        let height = ui.spacing().interact_size.y;
        let width = self
            .width
            .unwrap_or_else(|| ui.spacing().slider_width.max(120.0));
        let (rect, mut response) =
            super::allocate_touch(ui, egui::vec2(width, height), Sense::click());
        let knob_r: f32 = match x.look {
            Look::Windows | Look::Gtk | Look::Material => 10.0,
            Look::Ios => 14.0,
            _ => 8.0,
        };
        let knob_r = knob_r.min((rect.width() * 0.5 - 0.5).max(0.0));
        let rail = Rect::from_min_max(
            egui::pos2(rect.left() + knob_r, rect.center().y - 2.0),
            egui::pos2(rect.right() - knob_r, rect.center().y + 2.0),
        );
        self.accessible_input(ui, &mut response);
        let dragging = self.pointer_input(ui, &mut response, rail, knob_r, x.look);
        response = super::click_response(ui, response);
        if response.has_focus() && response.enabled() {
            let gtk = x.look == Look::Gtk;
            let filter = egui::EventFilter {
                horizontal_arrows: true,
                vertical_arrows: gtk,
                escape: gtk && dragging,
                ..Default::default()
            };
            ui.memory_mut(|m| m.set_focus_lock_filter(response.id, filter));
            if gtk {
                let commands = ui.input_mut(|input| {
                    let mut commands = Vec::new();
                    input.events.retain(|event| {
                        if let egui::Event::Key {
                            key,
                            pressed: true,
                            modifiers,
                            ..
                        } = event
                        {
                            if let Some(command) = slider_key(*key, *modifiers) {
                                commands.push(command);
                                return false;
                            }
                        }
                        true
                    });
                    commands
                });
                for command in commands {
                    let (lo, hi) = (*self.range.start(), *self.range.end());
                    let value = match command {
                        SliderKey::Start => lo.min(hi),
                        SliderKey::End => hi.max(lo),
                        SliderKey::Move { steps, visual } => self.nudged(steps, visual),
                    };
                    self.set(value, &mut response);
                }
            } else {
                let steps = ui.input(|i| {
                    f64::from(u8::from(i.key_pressed(egui::Key::ArrowRight)))
                        - f64::from(u8::from(i.key_pressed(egui::Key::ArrowLeft)))
                });
                if steps != 0.0 {
                    self.set(self.nudged(steps, true), &mut response);
                }
            }
        }
        let enabled = ui.is_enabled();
        let t = self.fraction(self.value.to_f64()) as f32;
        let painter = ui.painter();
        let at = egui::pos2(egui::lerp(rail.left()..=rail.right(), t), rail.center().y);
        let accent = if enabled { c.accent } else { c.disabled };
        painter.rect_filled(rail, CornerRadius::same(2), c.track);
        painter.rect_filled(
            Rect::from_min_max(rail.min, egui::pos2(at.x, rail.max.y)),
            CornerRadius::same(2),
            accent,
        );
        match x.look {
            Look::Windows => {
                // WinUI's thumb: a ring in the button's fill round an accent dot that
                // grows under the pointer.
                painter.circle_filled(at, knob_r, c.button);
                painter.circle_stroke(at, knob_r, Stroke::new(1.0, c.control_border));
                painter.circle_filled(
                    at,
                    if response.hovered() || response.dragged() {
                        7.0
                    } else {
                        5.0
                    },
                    accent,
                );
            }
            Look::Plain | Look::Material => {
                painter.circle_filled(
                    at,
                    knob_r,
                    if response.hovered() {
                        c.accent_hover
                    } else {
                        accent
                    },
                );
            }
            Look::Mac | Look::Gtk | Look::Ios => {
                painter.circle_filled(
                    at + egui::vec2(0.0, 1.0),
                    knob_r + 0.5,
                    Color32::from_black_alpha(if enabled { 50 } else { 20 }),
                );
                painter.circle_filled(
                    at,
                    knob_r,
                    if enabled {
                        Color32::WHITE
                    } else {
                        Color32::from_gray(if c.dark { 140 } else { 240 })
                    },
                );
            }
        }
        if response.has_focus() {
            crate::paint::ring(
                painter,
                &x.paint(),
                Rect::from_center_size(at, egui::Vec2::splat(knob_r * 2.0)),
                CornerRadius::same(knob_r as u8),
            );
        }
        let value = self.accessible_value();
        let text = if self.text.is_empty() {
            super::form::accessible_name(ui, &self.label)
        } else {
            self.text.clone()
        };
        response.widget_info(|| egui::WidgetInfo::slider(enabled, value, &text));
        self.update_accessible_node(ui, &response);
        response
    }
}

#[derive(Clone, Copy)]
struct SliderDrag {
    press_time: Option<f64>,
    seen_frame: u64,
    origin: egui::Pos2,
    last: egui::Pos2,
    fraction: f64,
    touch: bool,
    relative: bool,
    claimed: bool,
    ignored: bool,
}

impl SliderDrag {
    fn advance(&mut self, x: f32, width: f32, scale: f64) {
        self.fraction += f64::from(x - self.last.x) / f64::from(width) * scale;
        self.last.x = x;
    }
}

/// Reduce sensitivity on wide numeric ranges without overflowing their span.
fn fine_drag_scale(width: f32, start: f64, end: f64) -> f64 {
    let span = (end - start).abs();
    let scale = if span.is_finite() {
        f64::from(width) / span
    } else {
        (f64::from(width) * 0.5) / (end * 0.5 - start * 0.5).abs()
    };
    if scale >= 1.0 {
        0.25
    } else {
        scale
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum SliderKey {
    Start,
    End,
    Move { steps: f64, visual: bool },
}

/// Translate supported shortcuts without capturing unrelated application shortcuts.
fn slider_key(key: egui::Key, modifiers: egui::Modifiers) -> Option<SliderKey> {
    use egui::Key;
    if modifiers.alt
        || modifiers.mac_cmd
        || (modifiers.command && !modifiers.ctrl)
        || (modifiers.shift && key != Key::Plus)
    {
        return None;
    }
    let page = if modifiers.ctrl { 10.0 } else { 1.0 };
    let (steps, visual) = match key {
        Key::Home if !modifiers.ctrl => return Some(SliderKey::Start),
        Key::End if !modifiers.ctrl => return Some(SliderKey::End),
        Key::ArrowLeft => (-page, true),
        Key::ArrowRight => (page, true),
        Key::ArrowUp | Key::Plus => (page, false),
        Key::ArrowDown | Key::Minus => (-page, false),
        Key::PageUp => (if modifiers.ctrl { -10.0 } else { 10.0 }, modifiers.ctrl),
        Key::PageDown => (if modifiers.ctrl { 10.0 } else { -10.0 }, modifiers.ctrl),
        _ => return None,
    };
    Some(SliderKey::Move { steps, visual })
}

/// A row of tabs: on macOS and in libadwaita a segmented control, on Windows WinUI's
/// pivot with an accent bar under the chosen tab, in the plain look an accent underline.
/// `response.changed()` when the tab changed.
///
/// The builder must be added to a UI or shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::Tabs;
/// Tabs::new("pages", &mut 0, &["First", "Second"]);
/// ```
#[must_use = "add the widget to a UI or call its show method"]
pub struct Tabs<'a> {
    id: egui::IdSalt,
    selected: &'a mut usize,
    labels: Vec<String>,
    label: String,
}

impl<'a> Tabs<'a> {
    /// Tabs reading `labels`, `selected` the index of the one shown. Empty lists are
    /// inactive; invalid indices are preserved until a valid choice is made.
    pub fn new(id: impl egui::AsIdSalt, selected: &'a mut usize, labels: &[&str]) -> Self {
        Tabs {
            id: egui::IdSalt::new(id),
            selected,
            labels: labels.iter().map(|s| s.to_string()).collect(),
            label: String::new(),
        }
    }
    /// Names this set of pages for assistive technology; a surrounding form label is used when unset.
    pub fn accessible_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
}

impl Widget for Tabs<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        if self.labels.is_empty() {
            return ui
                .add_enabled_ui(false, |ui| {
                    ui.allocate_response(egui::Vec2::ZERO, Sense::hover())
                })
                .inner;
        }
        let name = super::form::accessible_name(ui, &self.label);
        let x = Ctx::of(ui);
        if matches!(x.look, Look::Mac | Look::Gtk | Look::Ios) {
            let labels: Vec<&str> = self.labels.iter().map(String::as_str).collect();
            let width = labels
                .iter()
                .map(|l| {
                    ui.fonts_mut(|f| {
                        f.layout_no_wrap(l.to_string(), x.m.bold.clone(), x.c.fg)
                            .size()
                            .x
                    }) + 28.0
                })
                .sum::<f32>();
            let response = ui.add(
                Segmented::new(self.id, self.selected, &labels)
                    .width(width.min(ui.available_width())),
            );
            let children = response.id.with(("segmented", egui::IdSalt::new(self.id)));
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_role(egui::Role::TabList);
                node.clear_label();
                if !name.is_empty() {
                    node.set_label(&name);
                }
            });
            for i in 0..labels.len() {
                ui.ctx().accesskit_node_builder(children.with(i), |node| {
                    node.set_role(egui::Role::Tab);
                    node.clear_toggled();
                    node.set_selected(i == *self.selected);
                });
            }
            return response;
        }
        let before = *self.selected;
        let mut whole: Option<Response> = None;
        let (mut ids, mut focused) = (Vec::new(), None);
        let group = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for (i, label) in self.labels.iter().enumerate() {
                let response = pivot_tab(ui, &x, label, i == *self.selected);
                if response.clicked() {
                    *self.selected = i;
                }
                if response.has_focus() {
                    focused = Some(i);
                }
                ids.push(response.id);
                whole = Some(match whole.take() {
                    Some(w) => w | response,
                    None => response,
                });
            }
        });
        arrow_keys(ui, focused, &ids, self.selected);
        ui.ctx().accesskit_node_builder(group.response.id, |node| {
            node.set_role(egui::Role::TabList);
            if !name.is_empty() {
                node.set_label(&name);
            }
        });
        for (i, id) in ids.iter().enumerate() {
            ui.ctx().accesskit_node_builder(*id, |node| {
                node.set_selected(i == *self.selected);
                node.clear_toggled();
            });
        }
        let mut response =
            whole.unwrap_or_else(|| ui.allocate_response(egui::Vec2::ZERO, Sense::hover()));
        if *self.selected != before {
            response.mark_changed();
        }
        response
    }
}

/// One tab, for a row the app builds itself (filtered, or in a loop): it draws as one tab
/// of [`Tabs`] and is clicked like a button. In a [`TabBar`] the tabs of macOS,
/// libadwaita and iOS sit in the segmented track; alone, a tab is a segment without it.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui, shown: &mut usize, pages: &[(&str, bool)]) {
/// use egui_native_look as look;
/// look::TabBar::new().show(ui, |bar| {
///     for (i, (name, visible)) in pages.iter().enumerate() {
///         if *visible && bar.tab(name, *shown == i).clicked() {
///             *shown = i;
///         }
///     }
/// });
/// # }
/// ```
///
/// The builder must be added to a UI or shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::Tab;
/// Tab::new("Scene", true);
/// ```
#[must_use = "add the widget to a UI or call its show method"]
pub struct Tab {
    label: String,
    selected: bool,
}

impl Tab {
    /// A tab reading `label`, the one shown when `selected`.
    pub fn new(label: impl Into<String>, selected: bool) -> Self {
        Tab {
            label: label.into(),
            selected,
        }
    }
}

impl Widget for Tab {
    fn ui(self, ui: &mut Ui) -> Response {
        let x = Ctx::of(ui);
        if segmented_tabs(x.look) {
            segment_tab(ui, &x, &self.label, self.selected)
        } else {
            pivot_tab(ui, &x, &self.label, self.selected)
        }
    }
}

/// A row of [`Tab`]s the app adds one by one.
///
/// The builder must be added to a UI or shown to have an effect.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use egui_native_look::TabBar;
/// TabBar::new();
/// ```
#[must_use = "add the widget to a UI or call its show method"]
#[derive(Default)]
pub struct TabBar;

impl TabBar {
    /// An empty row.
    pub fn new() -> Self {
        TabBar
    }

    /// Lays out the tabs `add` adds, in the look's track where it has one.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add: impl FnOnce(&mut TabBarUi<'_>) -> R,
    ) -> egui::InnerResponse<R> {
        let x = Ctx::of(ui);
        let segmented = segmented_tabs(x.look);
        let track = ui.painter().add(egui::Shape::Noop);
        let inner = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = if segmented { 0.0 } else { 4.0 };
            let x = Ctx::of(ui);
            add(&mut TabBarUi { ui, x })
        });
        if segmented && inner.response.rect.width() > 0.0 {
            ui.painter().set(
                track,
                egui::epaint::RectShape::filled(
                    inner.response.rect,
                    CornerRadius::same(x.m.radius.saturating_add(1)),
                    x.c.track,
                ),
            );
        }
        inner
    }
}

/// The tabs of a [`TabBar`] being laid out.
pub struct TabBarUi<'a> {
    ui: &'a mut Ui,
    x: Ctx,
}

impl TabBarUi<'_> {
    /// A tab reading `label`, the one shown when `selected`.
    pub fn tab(&mut self, label: &str, selected: bool) -> Response {
        if segmented_tabs(self.x.look) {
            segment_tab(self.ui, &self.x, label, selected)
        } else {
            pivot_tab(self.ui, &self.x, label, selected)
        }
    }

    /// The row's `Ui`, for anything else in it.
    pub fn ui(&mut self) -> &mut Ui {
        self.ui
    }
}

/// Whether the look's tabs are a segmented control.
fn segmented_tabs(look: Look) -> bool {
    matches!(look, Look::Mac | Look::Gtk | Look::Ios)
}

/// A tab of WinUI's pivot, Material's tabs or the plain look's: its label, with an accent
/// bar under it when chosen.
fn pivot_tab(ui: &mut Ui, x: &Ctx, label: &str, on: bool) -> Response {
    let c = &x.c;
    let height = ui.spacing().interact_size.y + 4.0;
    let font = if on {
        x.m.bold.clone()
    } else {
        x.m.body.clone()
    };
    let width = ui.fonts_mut(|f| {
        f.layout_no_wrap(label.to_string(), font.clone(), c.fg)
            .size()
            .x
    }) + 20.0;
    let (rect, response) = super::allocate_control(ui, egui::vec2(width, height), Sense::click());
    if response.hovered() && !on {
        ui.painter()
            .rect_filled(rect.shrink(2.0), CornerRadius::same(4), c.row_hover);
    }
    // Material colours the chosen tab's label in the primary too.
    let colour = if on && x.look == Look::Material {
        c.accent
    } else if on || response.hovered() {
        c.fg
    } else {
        c.dim
    };
    crate::text(
        ui.painter(),
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        font,
        colour,
    );
    if on {
        let bar_width = if x.look == Look::Windows {
            16.0
        } else {
            width - 12.0
        };
        let bar = Rect::from_center_size(
            egui::pos2(rect.center().x, rect.bottom() - 1.5),
            egui::vec2(bar_width, 3.0),
        );
        ui.painter()
            .rect_filled(bar, CornerRadius::same(2), c.accent);
    }
    if response.has_focus() {
        crate::paint::ring(
            ui.painter(),
            &x.paint(),
            rect.shrink(2.0),
            CornerRadius::same(4),
        );
    }
    response
        .widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, ui.is_enabled(), on, label));
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_selected(on);
        node.clear_toggled();
    });
    response
}

/// A segment of a segmented control as a tab: raised out of the track when chosen, as
/// [`Segmented`] draws it.
fn segment_tab(ui: &mut Ui, x: &Ctx, label: &str, on: bool) -> Response {
    let c = &x.c;
    let height = ui.spacing().interact_size.y;
    let font = if on {
        x.m.bold.clone()
    } else {
        x.m.body.clone()
    };
    // The bold width either way, so the row stays still as the choice moves.
    let width = ui.fonts_mut(|f| {
        f.layout_no_wrap(label.to_string(), x.m.bold.clone(), c.fg)
            .size()
            .x
    }) + 28.0;
    let enabled = ui.is_enabled();
    let (slot, response) = super::allocate_control(
        ui,
        egui::vec2(width, height),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let inner = CornerRadius::same(x.m.radius.saturating_sub(1));
    let body = slot.shrink(2.0);
    if on {
        let fill = if enabled {
            c.raised
        } else {
            c.raised.gamma_multiply(0.6)
        };
        if x.look == Look::Gtk {
            ui.painter().rect_filled(
                body.translate(egui::vec2(0.0, 1.0)),
                inner,
                Color32::from_black_alpha(if c.dark { 50 } else { 20 }),
            );
            ui.painter().rect_filled(body, inner, fill);
        } else {
            crate::paint::raised(ui.painter(), c.dark, body, inner, fill, enabled);
        }
    } else if response.hovered() && enabled {
        ui.painter().rect_filled(body, inner, c.row_hover);
    }
    let colour = if enabled { c.fg } else { c.disabled };
    crate::text(
        ui.painter(),
        slot.center(),
        egui::Align2::CENTER_CENTER,
        label,
        font,
        colour,
    );
    if response.has_focus() {
        crate::paint::ring(ui.painter(), &x.paint(), body, inner);
    }
    response.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, enabled, on, label));
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_selected(on);
        node.clear_toggled();
    });
    response
}

#[cfg(test)]
mod tests {
    use super::Slider;

    #[test]
    fn accessible_values_are_finite_and_leave_the_model_unchanged() {
        for (start, end) in [(0.0, 10.0), (10.0, 0.0)] {
            for current in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -5.0, 42.0] {
                let mut value = current;
                let slider = Slider::new(&mut value, start..=end);
                let displayed = slider.accessible_value();
                assert!(displayed.is_finite() && (0.0..=10.0).contains(&displayed));
                if current.is_nan() {
                    assert_eq!(displayed, start);
                }
                assert_eq!(slider.value.to_bits(), current.to_bits());
            }
        }
    }

    #[test]
    fn accessibility_steps_start_at_the_visible_knob_for_out_of_range_models() {
        let mut value = 42.0;
        let slider = Slider::new(&mut value, 0.0..=10.0).step_by(1.0);
        assert_eq!(slider.nudged(-1.0, false), 10.0);
        assert_eq!(
            slider.nudged_from(slider.accessible_value(), -1.0, false),
            9.0
        );
        assert_eq!(*slider.value, 42.0);
    }

    #[test]
    fn fine_drag_sensitivity_stays_positive_across_extreme_finite_ranges() {
        use super::fine_drag_scale;
        assert_eq!(fine_drag_scale(180.0, 0.0, 100.0), 0.25);
        assert_eq!(fine_drag_scale(180.0, 100.0, 0.0), 0.25);
        assert!((fine_drag_scale(180.0, 0.0, 10_000.0) - 0.018).abs() < 1e-12);
        let extreme = fine_drag_scale(1.0, -f64::MAX, f64::MAX);
        assert!(extreme.is_finite() && extreme > 0.0 && extreme < 1.0);
    }

    #[test]
    fn unrounded_drag_positions_accumulate_small_integer_moves() {
        let mut value = 5;
        let slider = Slider::new(&mut value, 0..=10);
        let mut drag = super::SliderDrag {
            press_time: None,
            seen_frame: 0,
            origin: egui::Pos2::ZERO,
            last: egui::Pos2::ZERO,
            fraction: 0.5,
            touch: false,
            relative: true,
            claimed: true,
            ignored: false,
        };
        for x in 1..=100 {
            drag.advance(x as f32, 1000.0, 1.0);
            if x < 40 {
                assert_eq!(slider.value_at(drag.fraction), 5.0);
            }
        }
        assert_eq!(slider.value_at(drag.fraction), 6.0);
    }

    #[test]
    fn dragging_past_an_endpoint_keeps_the_pointer_offset_on_return() {
        let mut drag = super::SliderDrag {
            press_time: None,
            seen_frame: 0,
            origin: egui::Pos2::ZERO,
            last: egui::Pos2::ZERO,
            fraction: 0.5,
            touch: false,
            relative: true,
            claimed: true,
            ignored: false,
        };
        drag.advance(100.0, 100.0, 1.0);
        assert_eq!(drag.fraction, 1.5);
        drag.advance(60.0, 100.0, 1.0);
        assert!(drag.fraction > 1.0);
        drag.advance(40.0, 100.0, 1.0);
        assert!((drag.fraction - 0.9).abs() < 1e-12);
    }

    #[test]
    fn keyboard_nudges_distinguish_visual_and_numeric_directions() {
        for (start, end, direction) in [(0, 10, 1), (10, 0, -1)] {
            let mut value = 5;
            let slider = Slider::new(&mut value, start..=end);
            assert_eq!(slider.nudged(1.0, true), f64::from(5 + direction));
            assert_eq!(slider.nudged(1.0, false), 6.0);
            assert_eq!(slider.nudged(10.0, false), 10.0);
            assert_eq!(slider.nudged(-10.0, false), 0.0);
        }
    }

    #[test]
    fn keyboard_nudges_handle_nan_and_overflow_without_producing_nan() {
        for current in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0] {
            let mut value = current;
            let slider = Slider::new(&mut value, -f64::MAX..=f64::MAX).step_by(f64::MAX);
            for steps in [-10.0, 10.0] {
                assert!(slider.nudged(steps, false).is_finite());
            }
        }
    }

    #[test]
    fn shortcut_translation_preserves_unrelated_modifier_combinations() {
        use super::{slider_key, SliderKey};
        use egui::{Key, Modifiers};
        for modifier in [Modifiers::ALT, Modifiers::MAC_CMD, Modifiers::SHIFT] {
            assert_eq!(slider_key(Key::ArrowRight, modifier), None);
        }
        assert_eq!(slider_key(Key::Home, Modifiers::CTRL), None);
        assert_eq!(slider_key(Key::Tab, Modifiers::NONE), None);
        assert_eq!(slider_key(Key::End, Modifiers::NONE), Some(SliderKey::End));
        assert_eq!(
            slider_key(Key::Plus, Modifiers::SHIFT),
            Some(SliderKey::Move {
                steps: 1.0,
                visual: false
            })
        );
        assert_eq!(
            slider_key(
                Key::ArrowRight,
                Modifiers {
                    ctrl: true,
                    command: true,
                    ..Modifiers::NONE
                }
            ),
            Some(SliderKey::Move {
                steps: 10.0,
                visual: true
            })
        );
    }

    #[test]
    fn extreme_finite_ranges_keep_values_and_fractions_finite() {
        for (start, end, logarithmic) in [
            (-f64::MAX, f64::MAX, false),
            (f64::MAX, -f64::MAX, false),
            (f64::MIN_POSITIVE, f64::MAX, true),
        ] {
            let mut value = start;
            let slider = Slider::new(&mut value, start..=end).logarithmic(logarithmic);
            for tick in 0..=100 {
                let t = f64::from(tick) / 100.0;
                let at = slider.value_at(t);
                assert!(at.is_finite());
                assert!((slider.fraction(at) - t).abs() < 1e-12);
            }
            assert_eq!(slider.fraction(f64::NAN), 0.0);
        }
    }

    #[test]
    fn steps_smaller_than_numeric_precision_do_not_overflow() {
        let mut value = 0.0;
        let slider = Slider::new(&mut value, -f64::MAX..=f64::MAX).step_by(f64::MIN_POSITIVE);
        for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
            assert!(slider.value_at(t).is_finite());
        }
    }

    #[test]
    fn linear_slider_values_round_trip_in_both_directions() {
        for (start, end) in [(0.0, 100.0), (-50.0, 50.0), (100.0, -100.0)] {
            let mut value = start;
            let slider = Slider::new(&mut value, start..=end);
            for tick in 0..=100 {
                let fraction = f64::from(tick) / 100.0;
                let at = slider.value_at(fraction);
                assert!(
                    (slider.fraction(at) - fraction).abs() < 1e-12,
                    "range {start}..={end}, fraction {fraction}"
                );
            }
            assert_eq!(slider.value_at(0.0), start);
            assert_eq!(slider.value_at(1.0), end);
        }
    }

    #[test]
    fn logarithmic_sliders_round_trip_across_orders_of_magnitude() {
        let mut value = 1.0;
        let slider = Slider::new(&mut value, 0.01..=10000.0).logarithmic(true);
        let mut previous = 0.0;
        for tick in 0..=100 {
            let fraction = f64::from(tick) / 100.0;
            let at = slider.value_at(fraction);
            assert!(
                at > previous,
                "logarithmic values must increase at tick {tick}"
            );
            assert!((slider.fraction(at) - fraction).abs() < 1e-12);
            previous = at;
        }
    }

    #[test]
    fn equal_slider_endpoints_are_constant() {
        let mut value = 7.0;
        let slider = Slider::new(&mut value, 7.0..=7.0);
        for fraction in [0.0, 0.5, 1.0] {
            assert_eq!(slider.value_at(fraction), 7.0);
        }
        assert_eq!(slider.fraction(7.0), 0.0);
    }

    #[test]
    fn invalid_logarithmic_ranges_fall_back_to_linear_values() {
        for (start, end) in [(-10.0, 10.0), (0.0, 100.0), (100.0, 1.0)] {
            let mut value = start;
            let slider = Slider::new(&mut value, start..=end).logarithmic(true);
            assert_eq!(slider.value_at(0.5), (start + end) / 2.0);
        }
    }

    #[test]
    fn stepped_values_stay_in_range_including_partial_final_steps() {
        let mut value = 0.0;
        let slider = Slider::new(&mut value, 0.0..=10.0).step_by(3.0);
        assert_eq!(slider.value_at(0.2), 3.0);
        assert_eq!(slider.value_at(0.0), 0.0);
        assert_eq!(slider.value_at(1.0), 10.0);
        for tick in 0..=100 {
            assert!((0.0..=10.0).contains(&slider.value_at(f64::from(tick) / 100.0)));
        }
    }
    #[test]
    fn reversed_stepped_sliders_reach_both_endpoints() {
        let mut value = 10.0;
        let slider = Slider::new(&mut value, 10.0..=0.0).step_by(3.0);
        assert_eq!(slider.value_at(0.0), 10.0);
        assert_eq!(slider.value_at(1.0), 0.0);
    }
}

#[cfg(test)]
mod dropdown_search_tests {
    use super::DropdownSearch;

    #[test]
    fn prefixes_cycle_expire_and_match_unicode() {
        let items = ["Alpha", "Alpine", "Beta", "Žába"].map(String::from);
        let mut search = DropdownSearch::default();
        assert_eq!(search.find("a", &items, 3, 0.0), Some(0));
        assert_eq!(search.find("A", &items, 0, 0.1), Some(1));
        assert_eq!(search.find("l", &items, 1, 0.2), Some(1));
        assert_eq!(search.find("p", &items, 1, 0.3), Some(1));
        assert_eq!(search.find("Ž", &items, 1, 2.0), Some(3));
        assert_eq!(search.find("b", &items, 3, 1.0), Some(2));
        assert_eq!(search.find("\n", &items, 2, 1.1), None);
        assert_eq!(search.find("", &items, 2, 1.2), None);
        assert_eq!(search.find("a", &[], 0, 1.3), None);
    }
}

#[cfg(test)]
mod choice_navigation_tests {
    use super::{choice_key, ChoiceKey};

    #[test]
    fn windows_keys_distinguish_focus_from_selection() {
        for key in [egui::Key::ArrowRight, egui::Key::ArrowDown] {
            assert_eq!(
                choice_key(key, egui::Modifiers::NONE, true),
                Some(ChoiceKey::Move {
                    forward: true,
                    select: true
                })
            );
            assert_eq!(
                choice_key(key, egui::Modifiers::CTRL, true),
                Some(ChoiceKey::Move {
                    forward: true,
                    select: false
                })
            );
        }
        assert_eq!(
            choice_key(egui::Key::ArrowDown, egui::Modifiers::NONE, false),
            None
        );
        assert_eq!(
            choice_key(egui::Key::Space, egui::Modifiers::NONE, true),
            Some(ChoiceKey::Select)
        );
    }

    #[test]
    fn choice_keys_leave_application_shortcuts_unclaimed() {
        for modifiers in [
            egui::Modifiers::SHIFT,
            egui::Modifiers::ALT,
            egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
            egui::Modifiers::MAC_CMD,
        ] {
            assert_eq!(choice_key(egui::Key::ArrowRight, modifiers, true), None);
        }
        assert_eq!(
            choice_key(egui::Key::Home, egui::Modifiers::CTRL, true),
            None
        );
        assert_eq!(
            choice_key(egui::Key::ArrowRight, egui::Modifiers::CTRL, false),
            None
        );
    }
}
