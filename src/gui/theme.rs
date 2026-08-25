//! Colours, fonts and the handful of egui calls that change between versions.
//!
//! Everything version-sensitive in this crate lives in this file on purpose. If
//! a future egui renames something, the compiler will point here and nowhere
//! else. The alternatives for older egui are noted inline.

use eframe::egui::{self, Color32, Stroke};

// Surfaces
pub const BG: Color32 = Color32::from_rgb(0x0D, 0x0D, 0x0E);
pub const SIDEBAR: Color32 = Color32::from_rgb(0x0A, 0x0A, 0x0B);
pub const CARD: Color32 = Color32::from_rgb(0x17, 0x18, 0x1A);
pub const ROW: Color32 = Color32::from_rgb(0x13, 0x14, 0x16);
pub const ROW_HOVER: Color32 = Color32::from_rgb(0x1B, 0x1D, 0x20);
pub const FIELD: Color32 = Color32::from_rgb(0x11, 0x12, 0x14);
pub const BORDER: Color32 = Color32::from_rgb(0x26, 0x28, 0x2B);
pub const BORDER_SOFT: Color32 = Color32::from_rgb(0x1D, 0x1F, 0x22);

// Text
pub const TEXT: Color32 = Color32::from_rgb(0xE9, 0xEA, 0xEB);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0xA0, 0xA4, 0xA9);
pub const TEXT_FAINT: Color32 = Color32::from_rgb(0x6B, 0x70, 0x75);

// Accents
pub const ACCENT: Color32 = Color32::from_rgb(0x3B, 0x82, 0xF6);
pub const ACCENT_DEEP: Color32 = Color32::from_rgb(0x1D, 0x4E, 0xD8);
pub const ACCENT_WASH: Color32 = Color32::from_rgb(0x14, 0x1F, 0x30);
pub const GREEN: Color32 = Color32::from_rgb(0x4A, 0xDE, 0x80);
pub const GREEN_WASH: Color32 = Color32::from_rgb(0x0F, 0x2A, 0x19);
pub const GREEN_EDGE: Color32 = Color32::from_rgb(0x1D, 0x4F, 0x30);
pub const AMBER: Color32 = Color32::from_rgb(0xF5, 0xA5, 0x24);
pub const AMBER_WASH: Color32 = Color32::from_rgb(0x2A, 0x1E, 0x08);
pub const RED: Color32 = Color32::from_rgb(0xF8, 0x71, 0x71);
pub const RED_WASH: Color32 = Color32::from_rgb(0x2A, 0x11, 0x11);
pub const WHITE: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub const INK: Color32 = Color32::from_rgb(0x0B, 0x0B, 0x0C);

/// Corner radius, as u8 because egui 0.31 moved CornerRadius to integer fields.
/// On egui 0.30 and older these need to be f32 (10.0 / 8.0 / 6.0).
pub const R_CARD: u8 = 10;
pub const R_CTRL: u8 = 8;
pub const R_SMALL: u8 = 6;

/// Rarity bands, identical to the thresholds and colours the terminal version used.
pub fn tier(percentage: f32) -> (Color32, &'static str) {
    if percentage <= 1.0 {
        (Color32::from_rgb(0xFF, 0x80, 0x00), "Legendary")
    } else if percentage <= 10.0 {
        (Color32::from_rgb(0xA3, 0x35, 0xEE), "Epic")
    } else if percentage <= 25.0 {
        (Color32::from_rgb(0x00, 0x70, 0xDD), "Rare")
    } else if percentage <= 50.0 {
        (Color32::from_rgb(0x1E, 0xFF, 0x00), "Uncommon")
    } else {
        (Color32::from_rgb(0xFF, 0xFF, 0xFF), "Common")
    }
}

/// A bordered panel. egui 0.31: `corner_radius` + integer `inner_margin`.
/// egui 0.30 and older: `.rounding(10.0)` and `.inner_margin(14.0)`.
pub fn card() -> egui::Frame {
    egui::Frame::default()
        .fill(CARD)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(R_CARD)
        .inner_margin(14)
}

/// The left library column.
pub fn sidebar_frame() -> egui::Frame {
    egui::Frame::default()
        .fill(SIDEBAR)
        .stroke(Stroke::new(1.0, BORDER_SOFT))
        .inner_margin(12)
}

/// The right detail column.
pub fn body_frame() -> egui::Frame {
    egui::Frame::default().fill(BG).inner_margin(18)
}

/// Stand-in for capsule art. Deterministic, so a game keeps its colour.
pub fn game_tint(app_id: u32) -> Color32 {
    const PALETTE: [Color32; 6] = [
        Color32::from_rgb(0x60, 0xA5, 0xFA),
        Color32::from_rgb(0x34, 0xD3, 0x99),
        Color32::from_rgb(0xF8, 0x71, 0x71),
        Color32::from_rgb(0xFB, 0xBF, 0x24),
        Color32::from_rgb(0xA7, 0x8B, 0xFA),
        Color32::from_rgb(0x22, 0xD3, 0xEE),
    ];
    PALETTE[(app_id as usize) % PALETTE.len()]
}

/// Windows-native type when we can get it, egui's bundled font otherwise.
///
/// egui 0.31 stores font data as `Arc<FontData>`. On 0.29 and older drop the
/// `Arc::new(...)` wrapper and insert `FontData::from_owned(bytes)` directly.
fn install_fonts(ctx: &egui::Context) {
    const CANDIDATES: [&str; 4] = [
        r"C:\Windows\Fonts\SegUIVar.ttf",
        r"C:\Windows\Fonts\segoeui.ttf",
        r"C:\Windows\Fonts\selawk.ttf",
        r"C:\Windows\Fonts\tahoma.ttf",
    ];

    let Some(bytes) = CANDIDATES
        .iter()
        .find_map(|path| std::fs::read(path).ok())
    else {
        return;
    };

    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "ui".to_owned(),
        std::sync::Arc::new(egui::FontData::from_owned(bytes)),
    );
    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        family.insert(0, "ui".to_owned());
    }
    ctx.set_fonts(fonts);
}

pub fn install(ctx: &egui::Context) {
    install_fonts(ctx);

    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = BG;
    visuals.faint_bg_color = ROW;
    visuals.extreme_bg_color = FIELD;
    visuals.override_text_color = Some(TEXT);
    visuals.selection.bg_fill = ACCENT.linear_multiply(0.45);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER_SOFT);
    visuals.widgets.inactive.bg_fill = CARD;
    visuals.widgets.inactive.weak_bg_fill = CARD;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.hovered.bg_fill = ROW_HOVER;
    visuals.widgets.hovered.weak_bg_fill = ROW_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.active.bg_fill = ROW_HOVER;
    visuals.widgets.active.weak_bg_fill = ROW_HOVER;
    ctx.set_visuals(visuals);

    ctx.style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(12.0, 7.0);
        style.spacing.scroll.bar_width = 10.0;
    });
}
