//! Palette and type of the original panel, translated to egui.

use std::sync::Arc;

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle,
};

pub const BACKGROUND: Color32 = Color32::from_rgb(0x17, 0x1b, 0x18);
pub const TEXT: Color32 = Color32::from_rgb(0xe7, 0xec, 0xe8);
pub const MUTED: Color32 = Color32::from_rgb(0xa2, 0xaf, 0xa6);
pub const LINE: Color32 = Color32::from_rgb(0x34, 0x3c, 0x36);
pub const GREEN: Color32 = Color32::from_rgb(0x9c, 0xd3, 0xae);
pub const INSET: Color32 = Color32::from_rgb(0x11, 0x16, 0x12);
pub const FIELD: Color32 = Color32::from_rgb(0x20, 0x27, 0x21);
pub const FIELD_LINE: Color32 = Color32::from_rgb(0x3b, 0x47, 0x3e);
pub const LABEL: Color32 = Color32::from_rgb(0xb9, 0xc5, 0xbd);

pub const TAB_SELECTED: Color32 = Color32::from_rgb(0x30, 0x3d, 0x32);
pub const TAB_SELECTED_LINE: Color32 = Color32::from_rgb(0x42, 0x53, 0x47);
pub const MODE_ACTIVE: Color32 = Color32::from_rgb(0x2a, 0x3d, 0x2f);
pub const MODE_ACTIVE_LINE: Color32 = Color32::from_rgb(0x46, 0x65, 0x4f);
pub const MODE_ACTIVE_TEXT: Color32 = Color32::from_rgb(0xd0, 0xea, 0xd8);

pub const HOVER_TINT: Color32 = Color32::from_rgba_premultiplied(11, 11, 11, 11);
pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(0xbc, 0xe8, 0xca);
pub const RUNNING_HOVER: Color32 = Color32::from_rgb(0x36, 0x51, 0x3f);
pub const SECONDARY_HOVER: Color32 = Color32::from_rgb(0x30, 0x40, 0x34);
pub const PRIMARY: Color32 = Color32::from_rgb(0xa8, 0xdb, 0xb8);
pub const PRIMARY_LINE: Color32 = Color32::from_rgb(0xb8, 0xe4, 0xc6);
pub const PRIMARY_TEXT: Color32 = Color32::from_rgb(0x19, 0x33, 0x22);
pub const RUNNING: Color32 = Color32::from_rgb(0x2c, 0x40, 0x32);
pub const RUNNING_LINE: Color32 = Color32::from_rgb(0x50, 0x6f, 0x59);
pub const RUNNING_TEXT: Color32 = Color32::from_rgb(0xc9, 0xe9, 0xd3);

pub const SECONDARY: Color32 = Color32::from_rgb(0x24, 0x2e, 0x26);
pub const SECONDARY_LINE: Color32 = Color32::from_rgb(0x3c, 0x4b, 0x40);
pub const SECONDARY_TEXT: Color32 = Color32::from_rgb(0xd3, 0xdf, 0xd6);

pub const PEER_CONNECTED: Color32 = Color32::from_rgb(0x24, 0x4c, 0x31);
pub const PEER_CONNECTED_LINE: Color32 = Color32::from_rgb(0x68, 0xab, 0x7b);
pub const PEER_CONNECTED_TEXT: Color32 = Color32::from_rgb(0xde, 0xff, 0xe7);

pub const PILL_RUNNING_TEXT: Color32 = Color32::from_rgb(0xb8, 0xdf, 0xc4);
pub const PILL_CONNECTED: Color32 = Color32::from_rgb(0xb4, 0xeb, 0xc2);
pub const PILL_CONNECTED_TEXT: Color32 = Color32::from_rgb(0x15, 0x32, 0x20);
pub const DOT_IDLE: Color32 = Color32::from_rgb(0x67, 0x76, 0x6b);
pub const DOT_CONNECTED: Color32 = Color32::from_rgb(0x20, 0x56, 0x35);

pub const ERROR: Color32 = Color32::from_rgb(0x35, 0x28, 0x21);
pub const ERROR_LINE: Color32 = Color32::from_rgb(0x76, 0x53, 0x49);
pub const ERROR_TEXT: Color32 = Color32::from_rgb(0xf1, 0xc8, 0xb9);

pub const TOAST: Color32 = Color32::from_rgb(0x31, 0x4b, 0x39);
pub const TOAST_LINE: Color32 = Color32::from_rgb(0x56, 0x73, 0x5f);
pub const TOAST_TEXT: Color32 = Color32::from_rgb(0xe1, 0xf0, 0xe6);

pub const SEMIBOLD: &str = "semibold";
const ICONS: &str = "icons";

// Segoe Fluent Icons (Windows 11) and Segoe MDL2 Assets (Windows 10) share
// these code points.
pub const HEADPHONES: char = '\u{E7F6}';
pub const MICROPHONE: char = '\u{E720}';
pub const PHONE: char = '\u{E8EA}';
pub const CHECK: char = '\u{E73E}';
pub const REFRESH: char = '\u{E72C}';
pub const COPY: char = '\u{E8C8}';
pub const OPEN: char = '\u{E8A7}';
pub const PLAY: char = '\u{E768}';
pub const STOP: char = '\u{E71A}';

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SEMIBOLD.into()))
}

pub fn icon_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(ICONS.into()))
}

pub fn icon(glyph: char, size: f32, color: Color32) -> egui::RichText {
    egui::RichText::new(glyph.to_string())
        .font(icon_font(size))
        .color(color)
}

pub fn apply(ctx: &egui::Context) {
    ctx.set_fonts(fonts());
    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (TextStyle::Heading, semibold(18.0)),
            (TextStyle::Body, FontId::proportional(13.0)),
            (TextStyle::Button, FontId::proportional(12.0)),
            (TextStyle::Small, FontId::proportional(11.0)),
            (TextStyle::Monospace, FontId::monospace(11.0)),
        ]
        .into();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(8.0, 5.0);
        style.spacing.interact_size.y = 30.0;
        let visuals = &mut style.visuals;
        *visuals = egui::Visuals::dark();
        visuals.panel_fill = BACKGROUND;
        visuals.window_fill = FIELD;
        visuals.extreme_bg_color = FIELD;
        visuals.override_text_color = None;
        visuals.selection.bg_fill = TAB_SELECTED_LINE;
        visuals.selection.stroke = Stroke::new(1.0, TEXT);
        visuals.window_stroke = Stroke::new(1.0, FIELD_LINE);
        visuals.window_corner_radius = CornerRadius::same(8);
        visuals.menu_corner_radius = CornerRadius::same(8);
        let radius = CornerRadius::same(7);
        let widgets = &mut visuals.widgets;
        widgets.noninteractive.bg_fill = BACKGROUND;
        widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
        widgets.noninteractive.bg_stroke = Stroke::new(1.0, LINE);
        widgets.inactive.bg_fill = FIELD;
        widgets.inactive.weak_bg_fill = FIELD;
        widgets.inactive.bg_stroke = Stroke::new(1.0, FIELD_LINE);
        widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(0xe1, 0xe9, 0xe4));
        widgets.inactive.corner_radius = radius;
        widgets.hovered.bg_fill = SECONDARY;
        widgets.hovered.weak_bg_fill = Color32::from_rgb(0x30, 0x40, 0x34);
        widgets.hovered.bg_stroke = Stroke::new(1.0, MODE_ACTIVE_LINE);
        widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        widgets.hovered.corner_radius = radius;
        widgets.active.bg_fill = TAB_SELECTED;
        widgets.active.weak_bg_fill = TAB_SELECTED;
        widgets.active.bg_stroke = Stroke::new(1.0, GREEN);
        widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        widgets.active.corner_radius = radius;
        widgets.open = widgets.active;
        // Keyboard focus ring, like the original 2px green outline.
        visuals.selection.stroke = Stroke::new(2.0, GREEN);
        // Locked controls stay legible while audio runs.
        visuals.disabled_alpha = 0.75;
        // Hand cursor over every clickable control, like the original CSS.
        visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);
    });
}

/// Windows fonts only, nothing bundled: Segoe UI for text and the system icon
/// font for symbols. Both ship with every supported Windows.
fn fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::empty();
    let directory = std::env::var_os("WINDIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "C:\\Windows".into())
        .join("Fonts");
    let mut load = |name: &str, files: &[&str]| {
        let bytes = files
            .iter()
            .find_map(|file| std::fs::read(directory.join(file)).ok())?;
        fonts
            .font_data
            .insert(name.into(), Arc::new(FontData::from_owned(bytes)));
        Some(name.to_owned())
    };
    let text = load("text", &["segoeui.ttf", "arial.ttf", "tahoma.ttf"]);
    let semibold = load(SEMIBOLD, &["seguisb.ttf", "segoeuib.ttf", "arialbd.ttf"]);
    let icons = load(ICONS, &["SegoeIcons.ttf", "segmdl2.ttf"]);
    let mono = load("mono", &["CascadiaMono.ttf", "consola.ttf", "cour.ttf"]);
    // Every family falls back to the regular text font.
    let family = |first: Option<String>| first.into_iter().chain(text.clone()).collect::<Vec<_>>();
    fonts
        .families
        .insert(FontFamily::Proportional, family(None));
    fonts.families.insert(FontFamily::Monospace, family(mono));
    fonts
        .families
        .insert(FontFamily::Name(SEMIBOLD.into()), family(semibold));
    fonts
        .families
        .insert(FontFamily::Name(ICONS.into()), family(icons));
    fonts
}
