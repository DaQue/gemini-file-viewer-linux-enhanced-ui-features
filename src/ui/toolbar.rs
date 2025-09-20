use std::path::PathBuf;
use eframe::egui;
use egui::RichText;
use crate::themes::CodeTheme;

pub(crate) fn toolbar(ui: &mut egui::Ui, app: &mut crate::app::FileViewerApp, _ctx: &egui::Context, file_to_load: &mut Option<PathBuf>) {
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.label(RichText::new("📁").size(20.0));
        ui.add_space(8.0);
        ui.label(RichText::new("gfv").heading().strong());
    });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    let mut open_button = egui::Button::new(RichText::new("📂 Open File").strong());
    open_button = open_button.fill(egui::Color32::from_rgb(34, 197, 94));
    let open_clicked = ui.add_enabled(!app.file_open_in_flight, open_button).clicked();
    if open_clicked { app.start_open_file_dialog(); }
    if app.file_open_in_flight {
        ui.add_space(8.0);
        ui.add(egui::Spinner::new().size(14.0));
        ui.label(RichText::new("Opening file…").weak());
    }

    let mut recent_button = egui::Button::new(RichText::new("📋 Recent").strong());
    recent_button = recent_button.fill(egui::Color32::from_rgb(59, 130, 246));
    if ui.add_enabled(!app.file_open_in_flight, recent_button).clicked() { app.show_recent_window = !app.show_recent_window; }

    let mut global_button = egui::Button::new(RichText::new("🔎 Global Search").strong());
    global_button = global_button.fill(egui::Color32::from_rgb(168, 85, 247));
    if ui.add(global_button).clicked() { app.show_global_search_window = !app.show_global_search_window; }

    let can_reopen = !app.session_paths.is_empty();
    let mut reopen_button = egui::Button::new(RichText::new("⟳ Reopen Session").strong());
    reopen_button = reopen_button.fill(egui::Color32::from_rgb(107, 114, 128));
    if ui.add_enabled(can_reopen, reopen_button).on_hover_text("Open last session once").clicked() {
        let active_idx = app.session_active.unwrap_or(0);
        for (idx, p) in app.session_paths.clone().into_iter().enumerate() {
            if p.exists() {
                if idx == active_idx {
                    *file_to_load = Some(p.clone());
                } else if crate::io::is_supported_text(&p) {
                    if let Ok((text, lossy, lines)) = crate::io::load_text(&p) {
                        let exists = app.open_text_tabs.iter().any(|t| t.path == p);
                        if !exists {
                            app.open_text_tabs.push(crate::app::TextTab { path: p.clone(), text, is_lossy: lossy, line_count: lines });
                        }
                    }
                }
            }
        }
    }

    ui.menu_button(RichText::new("🎨 Themes").strong(), |ui| {
        ui.set_min_width(300.0);
        let prev_theme = app.code_theme;
        ui.label(RichText::new("🎨 Code Themes").strong());
        ui.add_space(8.0);
        for theme in CodeTheme::all() {
            let is_selected = app.code_theme == *theme;
            let mut button_text = RichText::new(theme.name());
            if is_selected { button_text = button_text.strong(); }
            if ui.selectable_label(is_selected, button_text).clicked() {
                app.code_theme = *theme;
                ui.close_menu();
            }
        }
        if app.code_theme != prev_theme { crate::settings::save_settings_to_disk(app); }
    });

    if ui.add(egui::Button::new(RichText::new("⚙️ Settings").strong()).fill(egui::Color32::from_rgb(107, 114, 128))).clicked() {
        app.show_settings_window = true;
    }
    if ui.add(egui::Button::new(RichText::new("ℹ️ About").strong()).fill(egui::Color32::from_rgb(107, 114, 128))).clicked() {
        app.show_about = true;
    }

    let mut clear_button = egui::Button::new(RichText::new("🗑️ Clear").strong());
    clear_button = clear_button.fill(egui::Color32::from_rgb(239, 68, 68));
    if ui.add(clear_button).clicked() {
        app.content = None;
        app.current_path = None;
        app.error_message = None;
    }

    if matches!(app.content, Some(crate::app::Content::Image(_))) {
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);
        let prev_fit = app.image_fit;
        ui.horizontal(|ui| {
            ui.checkbox(&mut app.image_fit, RichText::new("📐 Fit to Window").strong());
            if app.image_fit != prev_fit { crate::settings::save_settings_to_disk(app); }
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let mut zoom_out_button = egui::Button::new(RichText::new("🔍-").strong());
            zoom_out_button = zoom_out_button.fill(egui::Color32::from_rgb(245, 158, 11));
            if ui.add(zoom_out_button).clicked() { app.image_fit = false; app.image_zoom = (app.image_zoom / 1.10).clamp(0.1, 6.0); }
            let mut zoom_in_button = egui::Button::new(RichText::new("🔍+").strong());
            zoom_in_button = zoom_in_button.fill(egui::Color32::from_rgb(245, 158, 11));
            if ui.add(zoom_in_button).clicked() { app.image_fit = false; app.image_zoom = (app.image_zoom * 1.10).clamp(0.1, 6.0); }
            let mut reset_button = egui::Button::new(RichText::new("100%").strong());
            reset_button = reset_button.fill(egui::Color32::from_rgb(34, 197, 94));
            if ui.add(reset_button).clicked() { app.image_fit = false; app.image_zoom = 1.0; }
        });
    }
}

