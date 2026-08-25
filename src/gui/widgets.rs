//! Hand-painted pieces of the interface.
//!
//! These are drawn with `Painter` primitives (`rect_filled`, `line_segment`,
//! `galley`) instead of egui's built-in widgets, for two reasons: the built-in
//! widgets do not look like the target design, and these few primitives have had
//! a stable signature across many egui releases.
//!
//! Each list row is allocated as one rectangle and then painted, rather than
//! assembled from nested layouts. That way the geometry is written down in one
//! place and a long description can never push the status badge off the row.

use eframe::egui::{self, Color32, FontId, Pos2, Rect, Response, Sense, Stroke, Vec2, vec2};

use super::theme;

// ---------------------------------------------------------------- primitives

/// A 1px border without `Painter::rect_stroke`, whose signature gained a
/// `StrokeKind` argument in egui 0.31. Two filled rects cannot drift.
fn bordered(painter: &egui::Painter, rect: Rect, radius: u8, border: Color32, fill: Color32) {
    painter.rect_filled(rect, radius, border);
    painter.rect_filled(rect.shrink(1.0), radius.saturating_sub(1), fill);
}

fn galley(
    painter: &egui::Painter,
    text: &str,
    size: f32,
    colour: Color32,
) -> std::sync::Arc<egui::Galley> {
    painter.layout_no_wrap(text.to_owned(), FontId::proportional(size), colour)
}

fn draw_left(painter: &egui::Painter, at: Pos2, text: &str, size: f32, colour: Color32) {
    let g = galley(painter, text, size, colour);
    painter.galley(at, g, colour);
}

fn draw_centre(painter: &egui::Painter, rect: Rect, text: &str, size: f32, colour: Color32) {
    let g = galley(painter, text, size, colour);
    painter.galley(rect.center() - g.size() / 2.0, g, colour);
}

fn draw_right(painter: &egui::Painter, right: Pos2, text: &str, size: f32, colour: Color32) {
    let g = galley(painter, text, size, colour);
    let width = g.size().x;
    painter.galley(Pos2::new(right.x - width, right.y), g, colour);
}

fn width_of(ui: &egui::Ui, text: &str, size: f32) -> f32 {
    galley(ui.painter(), text, size, theme::TEXT).size().x
}

fn hand(ui: &egui::Ui, response: &Response) {
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
}

/// Rough character budget for a width. Belt and braces: the caller also clips,
/// so a bad guess shortens or slightly crowds the text but never overlaps.
fn shorten(text: &str, width: f32, size: f32) -> String {
    let budget = (width / (size * 0.52)).floor().max(4.0) as usize;
    if text.chars().count() <= budget {
        return text.to_owned();
    }
    let mut out: String = text.chars().take(budget.saturating_sub(1)).collect();
    while out.ends_with(' ') || out.ends_with(',') {
        out.pop();
    }
    out.push('…');
    out
}

fn paint_checkbox(painter: &egui::Painter, rect: Rect, checked: bool, warm: bool) {
    if checked {
        painter.rect_filled(rect, 4, theme::ACCENT);
        let stroke = Stroke::new(2.0, Color32::WHITE);
        let left = Pos2::new(rect.left() + 4.0, rect.center().y + 0.5);
        let dip = Pos2::new(rect.center().x - 1.0, rect.bottom() - 5.0);
        let right = Pos2::new(rect.right() - 4.0, rect.top() + 5.5);
        painter.line_segment([left, dip], stroke);
        painter.line_segment([dip, right], stroke);
    } else {
        let border = if warm { theme::TEXT_FAINT } else { theme::BORDER };
        bordered(painter, rect, 4, border, theme::BG);
    }
}

fn paint_swatch(painter: &egui::Painter, rect: Rect, colour: Color32, filled: bool) {
    let fill = colour.linear_multiply(if filled { 0.30 } else { 0.10 });
    let border = colour.linear_multiply(if filled { 0.55 } else { 0.26 });
    bordered(painter, rect, theme::R_SMALL, border, fill);
    if filled {
        painter.rect_filled(
            Rect::from_center_size(rect.center(), vec2(10.0, 10.0)),
            2,
            colour.linear_multiply(0.8),
        );
    }
}

// -------------------------------------------------------------------- badges

/// What a row's right-hand badge says.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Badge {
    Unlocked,
    Locked,
    PendingUnlock,
    PendingReset,
    Failed,
}

impl Badge {
    /// label, fill, border, text
    fn parts(self) -> (&'static str, Color32, Color32, Color32) {
        match self {
            Badge::Unlocked => ("Unlocked", theme::GREEN_WASH, theme::GREEN_EDGE, theme::GREEN),
            Badge::Locked => ("Locked", theme::BG, theme::BORDER, theme::TEXT_DIM),
            Badge::PendingUnlock => (
                "Pending unlock",
                theme::ACCENT_DEEP,
                theme::ACCENT,
                Color32::WHITE,
            ),
            Badge::PendingReset => (
                "Pending reset",
                theme::AMBER_WASH,
                theme::AMBER,
                theme::AMBER,
            ),
            Badge::Failed => ("Steam refused", theme::RED_WASH, theme::RED, theme::RED),
        }
    }

    fn width(self, ui: &egui::Ui) -> f32 {
        width_of(ui, self.parts().0, 12.0) + 22.0
    }
}

fn paint_badge(painter: &egui::Painter, rect: Rect, kind: Badge) {
    let (label, fill, border, text) = kind.parts();
    bordered(painter, rect, theme::R_SMALL, border, fill);
    draw_centre(painter, rect, label, 12.0, text);
}

// ------------------------------------------------------------------ controls

/// A filter chip. The active chip gets a light fill, the rest an outline.
pub fn chip(ui: &mut egui::Ui, label: &str, active: bool) -> Response {
    let width = width_of(ui, label, 13.0) + 26.0;
    let (rect, response) = ui.allocate_exact_size(vec2(width, 30.0), Sense::click());
    hand(ui, &response);

    let (fill, border, text) = if active {
        (theme::ROW_HOVER, theme::TEXT_FAINT, theme::TEXT)
    } else if response.hovered() {
        (theme::ROW, theme::BORDER, theme::TEXT)
    } else {
        (theme::BG, theme::BORDER, theme::TEXT_DIM)
    };

    let painter = ui.painter();
    bordered(painter, rect, theme::R_CTRL, border, fill);
    draw_centre(painter, rect, label, 13.0, text);
    response
}

/// A solid action button. `primary` is the white "Apply changes" treatment.
pub fn action(ui: &mut egui::Ui, label: &str, primary: bool, enabled: bool) -> Response {
    let width = width_of(ui, label, 13.5) + 34.0;
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(vec2(width, 34.0), sense);
    if enabled {
        hand(ui, &response);
    }

    let (fill, border, text) = match (primary, enabled, response.hovered()) {
        (true, true, false) => (theme::WHITE, theme::WHITE, theme::INK),
        (true, true, true) => (
            Color32::from_rgb(0xE2, 0xE4, 0xE7),
            theme::WHITE,
            theme::INK,
        ),
        (true, false, _) => (theme::CARD, theme::BORDER, theme::TEXT_FAINT),
        (false, true, false) => (theme::CARD, theme::BORDER, theme::TEXT),
        (false, true, true) => (theme::ROW_HOVER, theme::TEXT_FAINT, theme::TEXT),
        (false, false, _) => (theme::BG, theme::BORDER_SOFT, theme::TEXT_FAINT),
    };

    let painter = ui.painter();
    bordered(painter, rect, theme::R_CTRL, border, fill);
    draw_centre(painter, rect, label, 13.5, text);
    response
}

/// One of the three numbers across the top of the detail pane.
pub fn stat(ui: &mut egui::Ui, size: Vec2, label: &str, value: &str, value_colour: Color32) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter();
    bordered(painter, rect, theme::R_CARD, theme::BORDER, theme::CARD);
    draw_left(
        painter,
        Pos2::new(rect.left() + 16.0, rect.top() + 13.0),
        label,
        12.0,
        theme::TEXT_DIM,
    );
    draw_left(
        painter,
        Pos2::new(rect.left() + 16.0, rect.top() + 32.0),
        value,
        27.0,
        value_colour,
    );
}

/// The connection line at the foot of the sidebar.
pub fn status_line(ui: &mut egui::Ui, colour: Color32, label: &str) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 20.0), Sense::hover());
    let painter = ui.painter();
    painter.circle_filled(Pos2::new(rect.left() + 4.0, rect.center().y), 4.0, colour);
    draw_left(
        painter,
        Pos2::new(rect.left() + 16.0, rect.center().y - 8.0),
        label,
        11.5,
        theme::TEXT_FAINT,
    );
}

// ---------------------------------------------------------------------- rows

/// One game in the left sidebar.
pub fn library_row(
    ui: &mut egui::Ui,
    title: &str,
    subtitle: &str,
    tint: Color32,
    selected: bool,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 52.0), Sense::click());
    hand(ui, &response);

    let painter = ui.painter();
    if selected {
        bordered(
            painter,
            rect,
            theme::R_CTRL,
            theme::ACCENT.linear_multiply(0.55),
            theme::ACCENT_WASH,
        );
    } else if response.hovered() {
        painter.rect_filled(rect, theme::R_CTRL, theme::ROW);
    }

    let tile = Rect::from_min_size(
        Pos2::new(rect.left() + 9.0, rect.top() + 9.0),
        vec2(34.0, 34.0),
    );
    bordered(
        painter,
        tile,
        theme::R_SMALL,
        tint.linear_multiply(0.5),
        tint.linear_multiply(0.22),
    );
    // Two initials stand in for the game's capsule art.
    let initials: String = title
        .split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    draw_centre(painter, tile, &initials, 12.5, tint.linear_multiply(0.9));

    let left = tile.right() + 11.0;
    let text_width = (rect.right() - 10.0 - left).max(20.0);
    let clipped = painter.with_clip_rect(Rect::from_min_max(
        Pos2::new(left, rect.top()),
        Pos2::new(rect.right() - 8.0, rect.bottom()),
    ));
    draw_left(
        &clipped,
        Pos2::new(left, rect.top() + 8.0),
        &shorten(title, text_width, 14.0),
        14.0,
        theme::TEXT,
    );
    draw_left(
        &clipped,
        Pos2::new(left, rect.top() + 28.0),
        &shorten(subtitle, text_width, 11.5),
        11.5,
        theme::TEXT_FAINT,
    );

    response
}

/// Everything one achievement row needs to draw itself.
pub struct RowView<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub percentage: f32,
    pub tier_colour: Color32,
    pub checked: bool,
    pub badge: Badge,
    /// Hover is read from the previous frame, because the background has to be
    /// painted before the response for this frame exists.
    pub warm: bool,
}

pub const ROW_HEIGHT: f32 = 64.0;

pub fn achievement_row(ui: &mut egui::Ui, view: RowView<'_>) -> Response {
    let badge_width = view.badge.width(ui);
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), ROW_HEIGHT), Sense::click());
    hand(ui, &response);

    let painter = ui.painter();
    let pending = matches!(view.badge, Badge::PendingUnlock | Badge::PendingReset);
    let (fill, border) = if pending {
        (theme::ACCENT_WASH, theme::ACCENT.linear_multiply(0.45))
    } else if view.warm {
        (theme::ROW_HOVER, theme::BORDER)
    } else {
        (theme::ROW, theme::BORDER_SOFT)
    };
    bordered(painter, rect, theme::R_CTRL, border, fill);

    let box_rect = Rect::from_center_size(
        Pos2::new(rect.left() + 23.0, rect.center().y),
        vec2(18.0, 18.0),
    );
    paint_checkbox(painter, box_rect, view.checked, view.warm);

    let swatch_rect = Rect::from_center_size(
        Pos2::new(box_rect.right() + 14.0 + 17.0, rect.center().y),
        vec2(34.0, 34.0),
    );
    paint_swatch(
        painter,
        swatch_rect,
        view.tier_colour,
        view.badge == Badge::Unlocked || view.badge == Badge::PendingUnlock,
    );

    let percent_text = format!("{:.1}%", view.percentage);
    let percent_width = width_of(ui, &percent_text, 12.0);
    let badge_rect = Rect::from_min_size(
        Pos2::new(rect.right() - 14.0 - badge_width, rect.center().y - 12.0),
        vec2(badge_width, 24.0),
    );
    let painter = ui.painter();
    paint_badge(painter, badge_rect, view.badge);
    draw_right(
        painter,
        Pos2::new(badge_rect.left() - 14.0, rect.center().y - 8.0),
        &percent_text,
        12.0,
        theme::TEXT_FAINT,
    );

    let left = swatch_rect.right() + 13.0;
    let right = badge_rect.left() - percent_width - 26.0;
    let text_width = (right - left).max(40.0);
    let clipped = painter.with_clip_rect(Rect::from_min_max(
        Pos2::new(left, rect.top()),
        Pos2::new(right.max(left + 40.0), rect.bottom()),
    ));
    draw_left(
        &clipped,
        Pos2::new(left, rect.top() + 13.0),
        &shorten(view.title, text_width, 14.0),
        14.0,
        theme::TEXT,
    );
    if !view.description.is_empty() {
        draw_left(
            &clipped,
            Pos2::new(left, rect.top() + 34.0),
            &shorten(view.description, text_width, 12.5),
            12.5,
            theme::TEXT_DIM,
        );
    }

    response
}
