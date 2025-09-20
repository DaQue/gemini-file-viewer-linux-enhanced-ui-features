use std::path::PathBuf;
use eframe::egui;
use egui::RichText;

pub(crate) fn search_bar(ui: &mut egui::Ui, app: &mut crate::app::FileViewerApp, file_to_load: &mut Option<PathBuf>) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            if let Some(cur) = &app.current_path {
                if let Some(name) = cur.file_name().and_then(|s| s.to_str()) {
                    ui.label(RichText::new(name).strong());
                    ui.add_space(12.0);
                }
            }
            if matches!(app.content, Some(crate::app::Content::Text(_))) {
                ui.label(RichText::new("🔍 Find:").strong());
                ui.add_space(8.0);
                let prev = app.search_query.clone();
                let resp = ui.text_edit_singleline(&mut app.search_query);
                if app.search_active { resp.request_focus(); app.search_active = false; }
                let (enter, shift) = ui.input(|i| (i.key_pressed(egui::Key::Enter), i.modifiers.shift));
                if enter && app.search_count > 0 {
                    if shift { if app.search_current == 0 { app.search_current = app.search_count.saturating_sub(1); } else { app.search_current -= 1; } }
                    else { app.search_current = (app.search_current + 1) % app.search_count; }
                }
                if resp.changed() || (prev.is_empty() && !app.search_query.is_empty()) {
                    app.search_count = 0; app.search_current = 0;
                    if let Some(crate::app::Content::Text(ref text)) = app.content
                        && !app.search_query.is_empty()
                        && text.len() <= crate::app::HIGHLIGHT_CHAR_THRESHOLD {
                        app.search_count = crate::search::recompute_count(&app.search_query, text);
                    }
                }
            }
            if let Some(cur) = app.current_path.clone() {
                ui.add_space(12.0);
                match app.content {
                    Some(crate::app::Content::Image(_)) => {
                        if ui.small_button(RichText::new("⬅️").size(10.0)).on_hover_text("Previous file").clicked()
                            && let Some(prev) = crate::io::neighbor_image(&cur, false) { *file_to_load = Some(prev); }
                        if ui.small_button(RichText::new("➡️").size(10.0)).on_hover_text("Next file").clicked()
                            && let Some(next) = crate::io::neighbor_image(&cur, true) { *file_to_load = Some(next); }
                    }
                    Some(crate::app::Content::Text(_)) => {
                        if ui.small_button(RichText::new("⬅️").size(10.0)).on_hover_text("Previous file").clicked()
                            && let Some(prev) = crate::io::neighbor_text(&cur, false) { *file_to_load = Some(prev); }
                        if ui.small_button(RichText::new("➡️").size(10.0)).on_hover_text("Next file").clicked()
                            && let Some(next) = crate::io::neighbor_text(&cur, true) { *file_to_load = Some(next); }
                    }
                    _ => {}
                }
            }
            if matches!(app.content, Some(crate::app::Content::Text(_))) && !app.search_query.is_empty() {
                ui.add_space(12.0);
                ui.label(RichText::new(format!("{} match(es)", app.search_count)).weak());
                ui.add_space(8.0);
                if ui.small_button(RichText::new("⬅️").size(10.0)).on_hover_text("Previous match").clicked() && app.search_count > 0 {
                    if app.search_current == 0 { app.search_current = app.search_count.saturating_sub(1); } else { app.search_current -= 1; }
                }
                if ui.small_button(RichText::new("➡️").size(10.0)).on_hover_text("Next match").clicked() && app.search_count > 0 {
                    app.search_current = (app.search_current + 1) % app.search_count;
                }
                if app.search_count > 0 { ui.add_space(8.0); ui.label(RichText::new(format!("{}/{}", app.search_current + 1, app.search_count)).strong()); }
            }
        });
    });
}

