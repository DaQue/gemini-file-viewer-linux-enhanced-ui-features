use std::path::PathBuf;
use eframe::egui;

pub(crate) fn handle_input(app: &mut crate::app::FileViewerApp, ctx: &egui::Context, file_to_load: &mut Option<PathBuf>) -> bool {
    let mut toggle_dark = false;
    ctx.input(|i| {
        if (i.modifiers.command || i.modifiers.alt) && i.key_pressed(egui::Key::O) {
            app.start_open_file_dialog();
        }
        if (i.modifiers.command || i.modifiers.alt) && i.key_pressed(egui::Key::D) {
            toggle_dark = true;
        }
        if (i.modifiers.command || i.modifiers.alt) && i.key_pressed(egui::Key::F) {
            app.search_active = true;
        }
        if (i.modifiers.command || i.modifiers.alt) && i.key_pressed(egui::Key::Comma) {
            app.show_settings_window = true;
        }
        if i.key_pressed(egui::Key::F1) {
            app.show_about = true;
        }
        if (i.modifiers.command || i.modifiers.alt) && i.key_pressed(egui::Key::L) {
            app.show_line_numbers = !app.show_line_numbers;
            crate::settings::save_settings_to_disk(app);
        }
        if (i.modifiers.command || i.modifiers.alt) && i.key_pressed(egui::Key::W) {
            app.word_wrap = !app.word_wrap;
            crate::settings::save_settings_to_disk(app);
        }

        // Ctrl + Mouse wheel zoom for content
        if (i.modifiers.command || i.modifiers.alt) && i.raw_scroll_delta.y != 0.0 {
            let dir = i.raw_scroll_delta.y.signum();
            match &app.content {
                Some(crate::app::Content::Text(_)) => {
                    let factor = if dir > 0.0 { 1.05 } else { 1.0 / 1.05 };
                    app.text_zoom = (app.text_zoom * factor).clamp(0.6, 3.0);
                }
                Some(crate::app::Content::Image(_)) => {
                    app.image_fit = false;
                    let factor = if dir > 0.0 { 1.10 } else { 1.0 / 1.10 };
                    app.image_zoom = (app.image_zoom * factor).clamp(0.1, 6.0);
                }
                _ => {}
            }
        }

        // Reset and keyboard zoom shortcuts
        if (i.modifiers.command || i.modifiers.alt) && i.key_pressed(egui::Key::Num0) {
            match &app.content {
                Some(crate::app::Content::Text(_)) => app.text_zoom = 1.0,
                Some(crate::app::Content::Image(_)) => { app.image_fit = false; app.image_zoom = 1.0; },
                _ => {}
            }
        }
        if (i.modifiers.command || i.modifiers.alt) && i.key_pressed(egui::Key::Equals) {
            match &app.content {
                Some(crate::app::Content::Text(_)) => app.text_zoom = (app.text_zoom * 1.05).clamp(0.6, 3.0),
                Some(crate::app::Content::Image(_)) => { app.image_fit = false; app.image_zoom = (app.image_zoom * 1.10).clamp(0.1, 6.0); },
                _ => {}
            }
        }
        if (i.modifiers.command || i.modifiers.alt) && i.key_pressed(egui::Key::Minus) {
            match &app.content {
                Some(crate::app::Content::Text(_)) => app.text_zoom = (app.text_zoom / 1.05).clamp(0.6, 3.0),
                Some(crate::app::Content::Image(_)) => { app.image_fit = false; app.image_zoom = (app.image_zoom / 1.10).clamp(0.1, 6.0); },
                _ => {}
            }
        }

        // Navigation with arrow keys for current content type
        if (i.key_pressed(egui::Key::ArrowRight) || (i.modifiers.alt && i.key_pressed(egui::Key::ArrowRight)))
            && let Some(cur) = app.current_path.clone() {
            match app.content {
                    Some(crate::app::Content::Image(_)) => {
                        if let Some(next) = crate::io::neighbor_image(&cur, true) { *file_to_load = Some(next); }
                    }
                    Some(crate::app::Content::Text(_)) => {
                        if let Some(next) = crate::io::neighbor_text(&cur, true) { *file_to_load = Some(next); }
                    }
                    _ => {}
            }
        }
        if (i.key_pressed(egui::Key::ArrowLeft) || (i.modifiers.alt && i.key_pressed(egui::Key::ArrowLeft)))
            && let Some(cur) = app.current_path.clone() {
            match app.content {
                    Some(crate::app::Content::Image(_)) => {
                        if let Some(prev) = crate::io::neighbor_image(&cur, false) { *file_to_load = Some(prev); }
                    }
                    Some(crate::app::Content::Text(_)) => {
                        if let Some(prev) = crate::io::neighbor_text(&cur, false) { *file_to_load = Some(prev); }
                    }
                    _ => {}
            }
        }
        // Support '<' and '>' typed keys for both images and text
        for ev in &i.events {
            if let egui::Event::Text(t) = ev {
                if t == ">" {
                    if let Some(cur) = app.current_path.clone() {
                        match app.content {
                            Some(crate::app::Content::Image(_)) => { if let Some(next) = crate::io::neighbor_image(&cur, true) { *file_to_load = Some(next); } }
                            Some(crate::app::Content::Text(_)) => { if let Some(next) = crate::io::neighbor_text(&cur, true) { *file_to_load = Some(next); } }
                            _ => {}
                        }
                    }
                } else if t == "<"
                    && let Some(cur) = app.current_path.clone() {
                    match app.content {
                            Some(crate::app::Content::Image(_)) => { if let Some(prev) = crate::io::neighbor_image(&cur, false) { *file_to_load = Some(prev); } }
                            Some(crate::app::Content::Text(_)) => { if let Some(prev) = crate::io::neighbor_text(&cur, false) { *file_to_load = Some(prev); } }
                            _ => {}
                    }
                }
            }
        }
    });
    toggle_dark
}
