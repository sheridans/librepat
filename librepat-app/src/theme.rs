use std::sync::Arc;

use eframe::egui::{
    Color32, Context, FontData, FontDefinitions, FontFamily, FontId, Stroke, Style, TextStyle,
    Visuals,
};
use librepat_core::ApplianceStatus;

const GO_REGULAR: &[u8] = include_bytes!("../../assets/fonts/Go-Regular.ttf");
const GO_BOLD: &[u8] = include_bytes!("../../assets/fonts/Go-Bold.ttf");

pub(crate) const INK: Color32 = Color32::from_rgb(21, 34, 39);
pub(crate) const PAPER: Color32 = Color32::from_rgb(244, 241, 231);
pub(crate) const TEAL: Color32 = Color32::from_rgb(8, 93, 96);
pub(crate) const TEAL_DARK: Color32 = Color32::from_rgb(5, 65, 68);
pub(crate) const MUTED: Color32 = Color32::from_rgb(91, 105, 106);
pub(crate) const FAIL: Color32 = Color32::from_rgb(173, 49, 39);
pub(crate) const PASS: Color32 = Color32::from_rgb(20, 119, 87);
pub(crate) const TEAL_WASH: Color32 = Color32::from_rgb(218, 229, 224);

pub(crate) const fn status_color(status: ApplianceStatus) -> Color32 {
    match status {
        ApplianceStatus::Pass => PASS,
        ApplianceStatus::Fail => FAIL,
    }
}

pub(crate) fn install(context: &Context) {
    let mut fonts = FontDefinitions::empty();
    fonts.font_data.insert(
        "Go Regular".into(),
        Arc::new(FontData::from_static(GO_REGULAR)),
    );
    fonts
        .font_data
        .insert("Go Bold".into(), Arc::new(FontData::from_static(GO_BOLD)));
    fonts
        .families
        .insert(FontFamily::Proportional, vec!["Go Regular".into()]);
    fonts
        .families
        .insert(FontFamily::Monospace, vec!["Go Regular".into()]);
    let bold_family = FontFamily::Name("Go Bold".into());
    fonts
        .families
        .insert(bold_family.clone(), vec!["Go Bold".into()]);
    context.set_fonts(fonts);

    let mut visuals = Visuals::light();
    visuals.panel_fill = PAPER;
    visuals.window_fill = Color32::from_rgb(250, 248, 241);
    visuals.extreme_bg_color = Color32::from_rgb(255, 253, 247);
    visuals.faint_bg_color = Color32::from_rgb(232, 232, 223);
    visuals.selection.bg_fill = TEAL;
    visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.active.bg_fill = TEAL;
    visuals.widgets.hovered.bg_fill = TEAL_WASH;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, INK);

    let mut style = Style {
        visuals,
        ..Style::default()
    };
    style.spacing.item_spacing = eframe::egui::vec2(10.0, 8.0);
    style.spacing.button_padding = eframe::egui::vec2(12.0, 7.0);
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::new(24.0, bold_family.clone()));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::new(14.0, bold_family));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(14.0, FontFamily::Proportional));
    context.set_theme(eframe::egui::Theme::Light);
    context.set_style_of(eframe::egui::Theme::Light, style);
}
