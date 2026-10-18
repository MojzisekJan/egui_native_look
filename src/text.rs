//! Text centred on its capitals.
//!
//! egui centres a line by its box, room for descenders included, and rounds its parts to
//! whole pixels differently at each size: button labels came out a pixel high and small
//! pills' a pixel low. Centring on the measured capitals puts a label on the middle of
//! its control the same way at every size and in every face.

use egui::{Color32, FontId, Rect};

/// Where the capitals of a line of `font` are centred, measured down from the line's top.
/// It is measured on the "H" of the fonts `ctx` has now; egui keeps that layout in its
/// own cache, which a change of fonts clears, so each context measures its own fonts.
pub fn cap_middle(ctx: &egui::Context, font: &FontId) -> f32 {
    let galley = ctx.fonts_mut(|fonts| fonts.layout_no_wrap("H".into(), font.clone(), Color32::WHITE));
    let middle = galley.rows.first().and_then(|placed| {
        let glyph = placed.row.glyphs.first()?;
        let top = placed.pos.y + glyph.pos.y + glyph.uv_rect.offset.y;
        Some(top + glyph.uv_rect.size.y / 2.0)
    });
    middle.filter(|m| m.is_finite()).unwrap_or(galley.size().y / 2.0)
}

/// The top at which to paint `galley` so that its first line's capitals centre on `y`.
pub fn galley_top(ctx: &egui::Context, galley: &egui::Galley, y: f32) -> f32 {
    match galley.job.sections.first() {
        Some(section) => y - cap_middle(ctx, &section.format.font_id),
        None => y - galley.size().y / 2.0,
    }
}

/// [`egui::Painter::text`], but a vertically centred line centres on its capitals
/// ([`cap_middle`]) rather than on its box.
pub fn text(painter: &egui::Painter, pos: egui::Pos2, anchor: egui::Align2, text: impl ToString, font: FontId, color: Color32) -> Rect {
    let galley = painter.layout_no_wrap(text.to_string(), font, color);
    let mut rect = anchor.anchor_size(pos, galley.size());
    if anchor.y() == egui::Align::Center {
        rect = rect.translate(egui::vec2(0.0, galley_top(painter.ctx(), &galley, pos.y) - rect.top()));
    }
    painter.galley(rect.min, galley, color);
    rect
}

/// The inner margin of a one-line text edit `height` tall, `left` and `right` wide, that
/// puts the capitals of `font` on its middle: egui sets the line at the top margin, so a
/// fixed margin leaves the text above the middle of a taller field.
pub fn entry_margin(ui: &egui::Ui, font: &FontId, height: f32, left: i8, right: i8) -> egui::Margin {
    let line = ui.fonts_mut(|f| f.row_height(font));
    let top = (height / 2.0 - cap_middle(ui.ctx(), font)).round().clamp(0.0, (height - line).max(0.0));
    let bottom = (height - line - top).floor().max(0.0);
    egui::Margin { left, right, top: top as i8, bottom: bottom as i8 }
}
