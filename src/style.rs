use eframe::egui;

pub(crate) fn apply_theme(app: &crate::app::FileViewerApp, ctx: &egui::Context) {
    let mut visuals = if app.dark_mode {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    if app.dark_mode {
        visuals.window_fill = egui::Color32::from_rgb(15, 15, 20);
        visuals.panel_fill = egui::Color32::from_rgb(22, 22, 28);
        visuals.faint_bg_color = egui::Color32::from_rgb(32, 32, 40);
        visuals.extreme_bg_color = egui::Color32::from_rgb(42, 42, 50);
        visuals.selection.bg_fill = egui::Color32::from_rgb(59, 130, 246);
        visuals.hyperlink_color = egui::Color32::from_rgb(99, 102, 241);
        visuals.button_frame = true;
        visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(55, 65, 81));
    } else {
        visuals.window_fill = egui::Color32::from_rgb(255, 255, 255);
        visuals.panel_fill = egui::Color32::from_rgb(248, 250, 252);
        visuals.faint_bg_color = egui::Color32::from_rgb(241, 245, 249);
        visuals.extreme_bg_color = egui::Color32::from_rgb(226, 232, 240);
        visuals.selection.bg_fill = egui::Color32::from_rgb(59, 130, 246);
        visuals.hyperlink_color = egui::Color32::from_rgb(99, 102, 241);
        visuals.button_frame = true;
        visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(226, 232, 240));
    }

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(12.0, 8.0);
    style.spacing.button_padding = egui::vec2(16.0, 10.0);
    style.spacing.menu_margin = egui::Margin::same(8);
    style.spacing.window_margin = egui::Margin::same(16);
    style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::new(24.0, egui::FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Body, egui::FontId::new(14.0, egui::FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Monospace, egui::FontId::new(13.0, egui::FontFamily::Monospace));
    style.visuals.button_frame = true;
    style.visuals.collapsing_header_frame = true;
    style.visuals = visuals;
    ctx.set_style(style);
}
