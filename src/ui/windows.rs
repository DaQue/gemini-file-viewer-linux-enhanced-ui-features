use std::path::PathBuf;
use eframe::egui;
use egui::RichText;

pub(crate) fn recent_files_window(ctx: &egui::Context, app: &mut crate::app::FileViewerApp, file_to_load: &mut Option<PathBuf>) {
    if !app.show_recent_window { return; }
    let mut open_flag = app.show_recent_window;
    egui::Window::new("Recent Files")
        .open(&mut open_flag)
        .collapsible(false)
        .resizable(true)
        .default_width(520.0)
        .show(ctx, |ui| {
            ui.set_min_width(480.0);
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                if app.recent_files.is_empty() { ui.label(RichText::new("No recent files").weak()); }
                for file in app.recent_files.clone().into_iter().rev() {
                    let display = file.to_string_lossy();
                    if ui.button(egui::RichText::new(display.clone()).monospace()).on_hover_text(display).clicked() {
                        *file_to_load = Some(file);
                        app.show_recent_window = false;
                    }
                }
            });
            ui.separator();
            let can_reopen = !app.session_paths.is_empty();
            if ui.add_enabled(can_reopen, egui::Button::new(RichText::new("⟳ Reopen Last Session").strong())).clicked() {
                let active_idx = app.session_active.unwrap_or(0);
                for (idx, p) in app.session_paths.clone().into_iter().enumerate() {
                    if p.exists() {
                        if idx == active_idx { *file_to_load = Some(p.clone()); }
                        else if crate::io::is_supported_text(&p) {
                            if let Ok((text, lossy, lines)) = crate::io::load_text(&p) {
                                let exists = app.open_text_tabs.iter().any(|t| t.path == p);
                                if !exists { app.open_text_tabs.push(crate::app::TextTab { path: p.clone(), text, is_lossy: lossy, line_count: lines }); }
                            }
                        }
                    }
                }
                app.show_recent_window = false;
            }
            ui.add_space(8.0);
            let clear_button = egui::Button::new(RichText::new("🗑️ Clear Recent Files"));
            let clear_color = egui::Color32::from_rgb(239, 68, 68);
            if ui.add(clear_button.fill(clear_color)).clicked() { app.recent_files.clear(); app.show_recent_window = false; }
        });
    app.show_recent_window = open_flag;
}

pub(crate) fn global_search_window(ctx: &egui::Context, app: &mut crate::app::FileViewerApp) {
    if !app.show_global_search_window { return; }
    let mut open_flag = app.show_global_search_window;
    egui::Window::new("Global Search")
        .open(&mut open_flag)
        .collapsible(false)
        .resizable(true)
        .default_width(720.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    let resp = ui.text_edit_singleline(&mut app.global_query);
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) { app.recompute_global_search(); }
                    if ui.button(RichText::new("Search").strong()).clicked() { app.recompute_global_search(); }
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut app.global_case_sensitive, "Case sensitive");
                    let ww = ui.checkbox(&mut app.global_whole_word, "Whole word");
                    if app.global_regex { app.global_whole_word = false; ww.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Checkbox, false, "Whole word (disabled in regex)")); }
                    ui.checkbox(&mut app.global_regex, "Regex");
                });
                if let Some(err) = &app.global_error { ui.colored_label(egui::Color32::RED, err); }
                ui.separator();
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    for (idx, res) in app.global_results.clone().into_iter().enumerate() {
                        let label = format!("{}:{} — {}", res.path.to_string_lossy(), res.line_index + 1, res.snippet);
                        if ui.selectable_label(false, egui::RichText::new(label).monospace()).clicked() {
                            app.switch_to_text_tab(res.tab_index);
                            app.search_query = app.global_query.clone();
                            if let Some(active) = app.active_text_tab && let Some(tab) = app.open_text_tabs.get(active) {
                                app.search_count = crate::search::recompute_count(&app.search_query, &tab.text);
                            }
                            app.search_current = res.match_index_in_tab.min(app.search_count.saturating_sub(1));
                            app.scroll_target_line = Some(res.line_index);
                            app.show_global_search_window = false;
                        }
                        if idx < app.global_results.len().saturating_sub(1) { ui.separator(); }
                    }
                    if app.global_results.is_empty() && !app.global_query.is_empty() { ui.label(RichText::new("No results").weak()); }
                });
            });
        });
    app.show_global_search_window = open_flag;
}

pub(crate) fn settings_window(ctx: &egui::Context, app: &mut crate::app::FileViewerApp) {
    if !app.show_settings_window { return; }
    let mut open = app.show_settings_window;
    egui::Window::new("Settings")
        .collapsible(false)
        .resizable(true)
        .min_width(520.0)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(RichText::new("🎨 Display Settings").strong());
            ui.add_space(8.0);
            ui.checkbox(&mut app.dark_mode, RichText::new("🌙 Dark Mode").strong());
            ui.checkbox(&mut app.show_line_numbers, RichText::new("📊 Line Numbers").strong());
            ui.checkbox(&mut app.use_syntect, RichText::new("🎨 Syntect Highlighting").strong());
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);
            ui.label(RichText::new("💾 Session").strong());
            ui.checkbox(&mut app.restore_session, "Restore previous session on startup");
        });
    app.show_settings_window = open;
}

