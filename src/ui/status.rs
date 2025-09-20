use eframe::egui;
use egui::RichText;

pub(crate) fn status_bar(ui: &mut egui::Ui, app: &mut crate::app::FileViewerApp) {
    use std::fs;
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal(|ui| {
            if let Some(path) = &app.current_path {
                ui.label(RichText::new("📄").size(16.0));
                ui.add_space(8.0);
                ui.monospace(RichText::new(path.to_string_lossy()).strong());
                if let Ok(metadata) = fs::metadata(path) {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("({:.1} KB)", metadata.len() as f64 / 1024.0)).weak());
                    });
                }
                ui.add_space(12.0);
                let mut copy_button = egui::Button::new(RichText::new("📋 Copy Path").strong());
                copy_button = copy_button.fill(egui::Color32::from_rgb(34, 197, 94));
                if ui.add(copy_button).on_hover_text("Copy path to clipboard").clicked() {
                    ui.ctx().copy_text(path.to_string_lossy().into());
                }
                let mut folder_button = egui::Button::new(RichText::new("📁 Open Folder").strong());
                folder_button = folder_button.fill(egui::Color32::from_rgb(59, 130, 246));
                if ui.add(folder_button).clicked() {
                    #[cfg(target_os = "windows")]
                    { let _ = std::process::Command::new("explorer").arg(path).spawn(); }
                    #[cfg(target_os = "macos")]
                    { let _ = std::process::Command::new("open").arg("-R").arg(path).spawn(); }
                    #[cfg(all(unix, not(target_os = "macos")))]
                    { if let Some(parent) = path.parent() { let _ = std::process::Command::new("xdg-open").arg(parent).spawn(); } }
                }
            } else { ui.label(RichText::new("📄 No file selected").weak()); }
        });
    });
}

pub(crate) fn status_extra(ui: &mut egui::Ui, app: &mut crate::app::FileViewerApp) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal(|ui| {
            match &app.content {
                Some(crate::app::Content::Image(texture)) => {
                    let size = texture.size();
                    ui.colored_label(egui::Color32::from_rgb(34, 197, 94), RichText::new("🖼️").size(16.0));
                    ui.add_space(8.0);
                    ui.label(RichText::new(format!("{}x{} px", size[0], size[1])).strong());
                    let eff = if app.image_fit { None } else { Some(app.image_zoom) };
                    if let Some(z) = eff { ui.add_space(12.0); ui.colored_label(egui::Color32::from_rgb(245, 158, 11), RichText::new(format!("🔍 {:.0}%", z * 100.0))); }
                    let est = size[0].saturating_mul(size[1]).saturating_mul(4);
                    ui.add_space(12.0);
                    ui.colored_label(egui::Color32::from_rgb(59, 130, 246), RichText::new(format!("💾 ~{:.1} MB", est as f64 / (1024.0 * 1024.0))));
                    if app.image_fit { ui.add_space(12.0); ui.colored_label(egui::Color32::from_rgb(168, 85, 247), RichText::new("📐 Fit: on")); }
                }
                Some(crate::app::Content::Text(_)) => {
                    ui.colored_label(egui::Color32::from_rgb(34, 197, 94), RichText::new("📝").size(16.0));
                    ui.add_space(8.0);
                    ui.label(RichText::new(format!("{} lines", app.text_line_count)).strong());
                    ui.add_space(12.0);
                    ui.colored_label(egui::Color32::from_rgb(245, 158, 11), RichText::new(format!("🔍 {:.0}%", app.text_zoom * 100.0)));
                    if app.text_is_big { ui.add_space(12.0); ui.colored_label(egui::Color32::from_rgb(239, 68, 68), RichText::new("⚠️ Large file: reduced features")); }
                    if app.text_is_lossy { ui.add_space(12.0); ui.colored_label(egui::Color32::from_rgb(239, 68, 68), RichText::new("⚠️ UTF-8 (lossy)")); }
                }
                _ => {}
            }
        });
    });
}

