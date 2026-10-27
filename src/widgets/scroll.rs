//! Scroll containers with look-specific wheel steps and boundary handling.
use egui::{MouseWheelUnit, Ui, Vec2, Vec2b};

/// A scroll area with GTK wheel distances and nested boundary handling.
/// Other looks retain egui's scrolling behavior. Point-based touchpad input keeps
/// its pixel distance; touch dragging and scrollbar interaction remain egui's.
///
/// ```
/// # fn example(ui: &mut egui::Ui) {
/// egui_native_look::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
///     for i in 0..30 { ui.label(format!("Item {i}")); }
/// });
/// # }
/// ```
#[must_use = "call show to display the scroll area"]
pub struct ScrollArea {
    inner: egui::ScrollArea,
    axes: Vec2b,
    id: Option<egui::IdSalt>,
    wheel: bool,
}

impl ScrollArea {
    /// Scroll vertically.
    pub fn vertical() -> Self {
        Self::new(Vec2b::new(false, true))
    }
    /// Scroll horizontally.
    pub fn horizontal() -> Self {
        Self::new(Vec2b::new(true, false))
    }
    /// Scroll in both directions.
    pub fn both() -> Self {
        Self::new(Vec2b::TRUE)
    }
    fn new(axes: Vec2b) -> Self {
        Self {
            inner: egui::ScrollArea::new(axes),
            axes,
            id: None,
            wheel: true,
        }
    }
    /// Stable identity, required when sibling scroll areas change order.
    pub fn id_salt(mut self, id: impl egui::AsIdSalt) -> Self {
        self.id = Some(egui::IdSalt::new(id));
        self
    }
    /// Maximum viewport height. Invalid values leave the limit unchanged.
    pub fn max_height(mut self, height: f32) -> Self {
        if let Some(height) = super::dimension(height) {
            self.inner = self.inner.max_height(height);
        }
        self
    }
    /// Maximum viewport width. Invalid values leave the limit unchanged.
    pub fn max_width(mut self, width: f32) -> Self {
        if let Some(width) = super::dimension(width) {
            self.inner = self.inner.max_width(width);
        }
        self
    }
    /// Whether an axis shrinks when its contents fit.
    pub fn auto_shrink(mut self, shrink: impl Into<Vec2b>) -> Self {
        self.inner = self.inner.auto_shrink(shrink);
        self
    }
    /// Set the vertical offset in points.
    pub fn vertical_scroll_offset(mut self, offset: f32) -> Self {
        if offset.is_finite() {
            self.inner = self.inner.vertical_scroll_offset(offset.max(0.0));
        }
        self
    }
    /// Set the horizontal offset in points.
    pub fn horizontal_scroll_offset(mut self, offset: f32) -> Self {
        if offset.is_finite() {
            self.inner = self.inner.horizontal_scroll_offset(offset.max(0.0));
        }
        self
    }
    /// Whether scrollbars are always visible, hidden, or shown when needed.
    pub fn scroll_bar_visibility(
        mut self,
        visibility: egui::scroll_area::ScrollBarVisibility,
    ) -> Self {
        self.inner = self.inner.scroll_bar_visibility(visibility);
        self
    }
    /// Minimum height reserved when the contents need vertical scrolling.
    pub fn min_scrolled_height(mut self, height: f32) -> Self {
        if let Some(height) = super::dimension(height) {
            self.inner = self.inner.min_scrolled_height(height);
        }
        self
    }
    /// Which input methods may scroll this area.
    pub fn scroll_source(mut self, source: egui::scroll_area::ScrollSource) -> Self {
        self.wheel = source.mouse_wheel;
        self.inner = self.inner.scroll_source(source);
        self
    }
    /// Draw the contents and return their result, viewport and scroll state.
    pub fn show<R>(
        mut self,
        ui: &mut Ui,
        contents: impl FnOnce(&mut Ui) -> R,
    ) -> egui::scroll_area::ScrollAreaOutput<R> {
        let id = self
            .id
            .map_or_else(|| ui.next_auto_id(), |id| ui.make_persistent_id(id));
        self.inner = self.inner.id_salt(id);
        if crate::look(ui.ctx()) != crate::Look::Gtk {
            return self.inner.show(ui, contents);
        }
        let unit_key = id.with("scroll-unit");
        let unit = ui
            .input(|i| {
                i.events.iter().rev().find_map(|e| {
                    if let egui::Event::MouseWheel { unit, .. } = e {
                        Some(*unit)
                    } else {
                        None
                    }
                })
            })
            .or_else(|| ui.data(|d| d.get_temp::<MouseWheelUnit>(unit_key)))
            .unwrap_or(MouseWheelUnit::Point);
        ui.data_mut(|d| d.insert_temp(unit_key, unit));
        // Shift-wheel must not turn back into vertical scrolling in a vertical-only area.
        let saved = ui.style().always_scroll_the_only_direction;
        ui.style_mut().always_scroll_the_only_direction = false;
        let axes = self.axes;
        let wheel = self.wheel;
        let output = self.inner.show_viewport(ui, |child, viewport| {
            let result = contents(child);
            if wheel && child.is_enabled() && child.rect_contains_pointer(child.clip_rect()) {
                let page = viewport.size().max(Vec2::splat(1.0));
                let line = child.ctx().options(|o| o.input_options.line_scroll_speed);
                let factor = match unit {
                    MouseWheelUnit::Line if line > 0.0 => {
                        Vec2::new(page.x.powf(2.0 / 3.0), page.y.powf(2.0 / 3.0)) / line
                    }
                    MouseWheelUnit::Page => page / child.ctx().content_rect().height().max(1.0),
                    _ => Vec2::splat(1.0),
                };
                let content = child.min_size();
                child.input_mut(|input| {
                    for axis in 0..2 {
                        if axes[axis] && content[axis] > page[axis] {
                            input.smooth_scroll_delta[axis] *= factor[axis];
                        }
                    }
                });
            }
            result
        });
        ui.style_mut().always_scroll_the_only_direction = saved;
        if self.wheel && ui.is_enabled() && ui.rect_contains_pointer(output.inner_rect) {
            ui.input_mut(|input| {
                if (0..2).any(|axis| {
                    self.axes[axis] && output.content_size[axis] > output.inner_rect.size()[axis]
                }) {
                    // A scrollable child retains the whole gesture, including other axes.
                    input.smooth_scroll_delta = Vec2::ZERO;
                }
            });
        }
        output
    }
}
