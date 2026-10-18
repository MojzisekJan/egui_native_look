//! The controls, each an [`egui::Widget`] built like egui's own: `ui.add(Switch::new(&mut on))`.
//! They take the height of `ui.spacing().interact_size.y`, which [`crate::install`] sets to
//! the theme's control height.

mod basic;
mod choice;
mod containers;
mod entry;
mod extra;
mod form;
mod status;
mod window;

pub use basic::{radio_value, switch_width, Button, Checkbox, Radio, Switch};
pub use choice::{combo_icon, menu_value, Dropdown, PopupButton, Segmented, Slider, Tab, TabBar, TabBarUi, Tabs};
pub use basic::check_mark;
pub use containers::{picked_text, section, submenu, ListRow, MenuItem, Section};
pub use entry::{NumberField, TextField};
pub use extra::{show_toasts, toast, ColorWell, IconButton};
pub use form::{Form, FormUi};
pub use status::{ProgressBar, Spinner};
pub use window::{window_frame, Dialog, TitleBar, ToolWindow, WindowAction};

use crate::{paint, Colors, Look, Metrics};

/// What a control needs to know to draw itself: the look, its colours and its sizes.
pub(crate) struct Ctx {
    pub look: Look,
    pub c: Colors,
    pub m: Metrics,
}

impl Ctx {
    pub fn of(ui: &egui::Ui) -> Ctx {
        let resolved = crate::resolved(ui.ctx());
        Ctx { look: resolved.theme.look, c: resolved.colors(ui.ctx()), m: resolved.theme.metrics.clone() }
    }

    /// The painting view of this: the look, the colours, the rounding.
    pub fn paint(&self) -> paint::Paint {
        paint::Paint { look: self.look, c: self.c, radius: self.m.radius }
    }

    pub fn radius(&self) -> egui::CornerRadius {
        egui::CornerRadius::same(self.m.radius)
    }
}

/// `text` laid out on one line no wider than `width`, ending in an ellipsis when it had to
/// be cut; and whether it was.
pub(crate) fn truncated(ui: &egui::Ui, text: &str, font: egui::FontId, color: egui::Color32, width: f32) -> (std::sync::Arc<egui::Galley>, bool) {
    let mut job = egui::text::LayoutJob::simple(text.to_string(), font, color, width.max(0.0));
    job.wrap.max_rows = 1;
    job.wrap.overflow_character = Some('…');
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    let cut = galley.elided;
    (galley, cut)
}

