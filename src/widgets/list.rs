//! A single-selection list with one keyboard stop and stable item identities.
use egui::{Key, Modifiers, Response, Sense, Ui};

/// A single-selection list. Arrow keys, Home/End and Page Up/Down move selection; Ctrl with
/// these keys moves focus alone. Enter or Space activates the focused item.
/// Disabled and filtered-out items are skipped. Item values must be unique.
///
/// ```
/// # fn example(ui: &mut egui::Ui, selected: &mut usize) {
/// let result = egui_native_look::ListBox::new("files", selected).show(ui, |list| {
///     list.item(10, "Documents");
///     list.item_enabled(false, 20, "Unavailable");
///     list.item(30, "Pictures");
/// });
/// if let Some(item) = result.activated { /* open item */ }
/// # }
/// ```
#[must_use = "call show to display the list"]
pub struct ListBox<'a> {
    id: egui::IdSalt,
    selected: &'a mut usize,
    label: String,
}

/// The result of drawing a list.
pub struct ListBoxOutput<R> {
    /// Value returned by the contents closure.
    pub inner: R,
    /// The list's focus response; changed when selection changes.
    pub response: Response,
    /// Item activated by a click, Enter, Space or accessibility action.
    pub activated: Option<usize>,
}

#[derive(Clone, Default)]
struct ListState {
    visible_focus: bool,
    selected: Option<usize>,
    items: Vec<(usize, bool)>,
    rects: Vec<(usize, egui::Rect)>,
    focus: Option<usize>,
}

impl<'a> ListBox<'a> {
    /// Create a list bound to the application's selected item value.
    pub fn new(id: impl egui::AsIdSalt, selected: &'a mut usize) -> Self {
        Self {
            id: egui::IdSalt::new(id),
            selected,
            label: String::new(),
        }
    }
    /// Accessible name of the list.
    pub fn accessible_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
    /// Draw the list. Filtering does not change the application's selected value.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        contents: impl FnOnce(&mut ListBoxUi<'_>) -> R,
    ) -> ListBoxOutput<R> {
        let id = ui.make_persistent_id(self.id);
        let key = id.with("list-state");
        let mut state = ui
            .data(|d| d.get_temp::<ListState>(key))
            .unwrap_or_default();
        let before = *self.selected;
        if state.selected != Some(before) {
            state.focus = Some(before);
        }
        let old_focus = state.focus;
        let page = ui.clip_rect().height() * 0.9;
        ui.input(|i| {
            if i.pointer.any_pressed() {
                state.visible_focus = false;
            } else if i
                .events
                .iter()
                .any(|e| matches!(e, egui::Event::Key { pressed: true, .. }))
            {
                state.visible_focus = true;
            }
        });
        let focused = ui.memory(|m| m.has_focus(id));
        let mut activate = false;
        let mut navigation = Vec::new();
        if focused && ui.is_enabled() {
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(
                    id,
                    egui::EventFilter {
                        vertical_arrows: true,
                        ..Default::default()
                    },
                )
            });
            let enabled: Vec<_> = state
                .items
                .iter()
                .filter(|(_, enabled)| *enabled)
                .map(|(value, _)| *value)
                .collect();
            if !enabled.is_empty() {
                let current = state.focus.or(Some(*self.selected));
                let index = enabled
                    .iter()
                    .position(|value| Some(*value) == current)
                    .unwrap_or(0);
                let mut target = index;
                let mut change_selection = false;
                for modifiers in [Modifiers::NONE, Modifiers::CTRL] {
                    for key in [
                        Key::ArrowUp,
                        Key::ArrowDown,
                        Key::Home,
                        Key::End,
                        Key::PageUp,
                        Key::PageDown,
                    ] {
                        if ui.input_mut(|i| i.consume_key(modifiers, key)) {
                            navigation.push((key, modifiers));
                            target = match key {
                                Key::ArrowUp => target.saturating_sub(1),
                                Key::ArrowDown => (target + 1).min(enabled.len() - 1),
                                Key::Home => 0,
                                Key::End => enabled.len() - 1,
                                _ => page_target(
                                    &enabled,
                                    target,
                                    &state.rects,
                                    page,
                                    key == Key::PageDown,
                                ),
                            };
                            change_selection |= modifiers.is_none();
                        }
                    }
                }
                state.focus = Some(enabled[target]);
                if change_selection {
                    *self.selected = enabled[target];
                }
                activate = ui.input_mut(|i| {
                    i.consume_key(Modifiers::NONE, Key::Enter)
                        | i.consume_key(Modifiers::NONE, Key::Space)
                });
                if activate {
                    *self.selected = enabled[target];
                }
            }
        } else {
            state.focus = Some(*self.selected);
        }
        let reveal =
            focused && (old_focus != state.focus || !ui.memory(|m| m.had_focus_last_frame(id)));
        let mut list = ListBoxUi {
            ui,
            id,
            selected: self.selected,
            focus: state.focus,
            items: Vec::new(),
            rects: Vec::new(),
            activated: None,
            keyboard: focused && state.visible_focus,
            reveal,
            activate,
        };
        let scoped = list.ui.scope_builder(
            egui::UiBuilder::new().id_salt(id).accessibility_parent(id),
            |ui| {
                let mut child = ListBoxUi {
                    ui,
                    id,
                    selected: list.selected,
                    focus: list.focus,
                    items: Vec::new(),
                    rects: Vec::new(),
                    activated: None,
                    keyboard: focused && state.visible_focus,
                    reveal,
                    activate,
                };
                let result = contents(&mut child);
                list.focus = child.focus;
                list.items = child.items;
                list.rects = child.rects;
                list.activated = child.activated;
                result
            },
        );
        // Reconcile against this frame's rows when filtering or enabled state changed.
        if list.items != state.items && (!navigation.is_empty() || activate) {
            let enabled: Vec<_> = list
                .items
                .iter()
                .filter(|(_, on)| *on)
                .map(|(v, _)| *v)
                .collect();
            *list.selected = before;
            list.activated = None;
            if !enabled.is_empty() {
                let mut target = enabled
                    .iter()
                    .position(|v| Some(*v) == old_focus)
                    .unwrap_or(0);
                for (key, modifiers) in navigation {
                    target = match key {
                        Key::ArrowUp => target.saturating_sub(1),
                        Key::ArrowDown => (target + 1).min(enabled.len() - 1),
                        Key::Home => 0,
                        Key::End => enabled.len() - 1,
                        _ => page_target(&enabled, target, &list.rects, page, key == Key::PageDown),
                    };
                    if modifiers.is_none() {
                        *list.selected = enabled[target];
                    }
                }
                list.focus = Some(enabled[target]);
                if activate {
                    *list.selected = enabled[target];
                    list.activated = Some(enabled[target]);
                }
            }
            list.ui.ctx().request_repaint();
        }
        if *list.selected != before
            && !list
                .items
                .iter()
                .any(|(v, enabled)| *v == *list.selected && *enabled)
        {
            *list.selected = before;
            list.activated = None;
        }
        if !list
            .items
            .iter()
            .any(|(v, enabled)| Some(*v) == list.focus && *enabled)
        {
            list.focus = list
                .items
                .iter()
                .find(|(_, enabled)| *enabled)
                .map(|(v, _)| *v);
        }
        let any_enabled = list.items.iter().any(|(_, enabled)| *enabled) && list.ui.is_enabled();
        let mut response = list.ui.interact(
            scoped.response.rect,
            id,
            if any_enabled {
                Sense::focusable_noninteractive()
            } else {
                Sense::hover()
            },
        );
        if !any_enabled {
            response.surrender_focus();
        }
        if *list.selected != before {
            response.mark_changed();
        }
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::ListBox, any_enabled, &self.label)
        });
        list.ui.ctx().accesskit_node_builder(id, |node| {
            if let Some(value) = list
                .focus
                .filter(|value| list.items.iter().any(|(v, enabled)| v == value && *enabled))
            {
                node.set_active_descendant(id.with(value).accesskit_id());
            }
        });
        list.ui.data_mut(|d| {
            d.insert_temp(
                key,
                ListState {
                    visible_focus: state.visible_focus,
                    selected: Some(*list.selected),
                    items: list.items,
                    rects: list.rects,
                    focus: list.focus,
                },
            )
        });
        ListBoxOutput {
            inner: scoped.inner,
            response,
            activated: list.activated,
        }
    }
}

/// Builder for the items inside a [`ListBox`].
pub struct ListBoxUi<'a> {
    ui: &'a mut Ui,
    id: egui::Id,
    selected: &'a mut usize,
    focus: Option<usize>,
    items: Vec<(usize, bool)>,
    rects: Vec<(usize, egui::Rect)>,
    activated: Option<usize>,
    keyboard: bool,
    reveal: bool,
    activate: bool,
}

impl ListBoxUi<'_> {
    /// Add an enabled item with a stable value.
    pub fn item(&mut self, value: usize, label: impl Into<String>) -> Response {
        self.item_enabled(true, value, label)
    }
    /// Add an item, skipping disabled items during navigation and activation.
    pub fn item_enabled(
        &mut self,
        enabled: bool,
        value: usize,
        label: impl Into<String>,
    ) -> Response {
        let label = label.into();
        let enabled = enabled && self.ui.is_enabled();
        self.items.push((value, enabled));
        let id = self.id.with(value);
        let x = super::Ctx::of(self.ui);
        let (rect, _) = self.ui.allocate_exact_size(
            super::target_size(
                self.ui,
                egui::vec2(self.ui.available_width(), self.ui.spacing().interact_size.y),
            ),
            Sense::hover(),
        );
        self.rects.push((value, rect));
        let mut response = self.ui.interact(
            rect,
            id,
            if enabled {
                Sense::CLICK
            } else {
                Sense::hover()
            },
        );
        response = super::click_response(self.ui, response);
        if response.clicked() && enabled {
            *self.selected = value;
            self.focus = Some(value);
            self.ui.memory_mut(|m| m.request_focus(self.id));
            self.activated = Some(value);
        }
        if self.activate && self.focus == Some(value) && enabled {
            self.activated = Some(value);
        }
        let accessibility_focus = enabled && self.ui.input(|i| i.events.iter().any(|event| matches!(event,
            egui::Event::AccessKitActionRequest(request) if request.target_node == id.accesskit_id() && request.action == egui::accesskit::Action::Focus)));
        if accessibility_focus {
            self.focus = Some(value);
            self.ui.memory_mut(|m| m.request_focus(self.id));
            response.scroll_to_me(None);
        }
        let picked = *self.selected == value;
        if self.ui.memory(|m| m.has_focus(self.id)) {
            super::containers::note_list_focus(self.ui);
        }
        super::containers::paint_list_row(self.ui, &x, rect, &response, picked);
        if self.keyboard && self.focus == Some(value) && enabled {
            crate::paint::ring(
                self.ui.painter(),
                &x.paint(),
                rect,
                egui::CornerRadius::ZERO,
            );
        }
        if self.reveal && self.focus == Some(value) && enabled {
            response.scroll_to_me(None);
        }

        let font = x.m.body;
        let (text, truncated) = super::truncated(
            self.ui,
            &label,
            font.clone(),
            if !enabled {
                x.c.disabled
            } else if picked {
                super::picked_text(self.ui)
            } else {
                x.c.fg
            },
            (rect.width() - 24.0).max(0.0),
        );
        self.ui.painter().galley(
            rect.left_center() + egui::vec2(12.0, -text.size().y * 0.5),
            text,
            x.c.fg,
        );
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::Role::ListBoxOption, enabled, picked, &label)
        });
        if truncated {
            response = response.on_hover_text(label);
        }
        response
    }
}

// Choose a row by its layout position so paging also works across disabled rows.
fn page_target(
    enabled: &[usize],
    current: usize,
    rects: &[(usize, egui::Rect)],
    page: f32,
    down: bool,
) -> usize {
    let top = |value| {
        rects
            .iter()
            .find(|(id, _)| *id == value)
            .map(|(_, r)| r.top())
    };
    let Some(start) = top(enabled[current]) else {
        return current;
    };
    let destination = start + if down { page } else { -page };
    let mut target = current;
    if down {
        while target + 1 < enabled.len()
            && top(enabled[target + 1]).is_some_and(|y| y <= destination)
        {
            target += 1;
        }
        target.max((current + 1).min(enabled.len() - 1))
    } else {
        while target > 0 && top(enabled[target]).is_some_and(|y| y > destination) {
            target -= 1;
        }
        target.min(current.saturating_sub(1))
    }
}
