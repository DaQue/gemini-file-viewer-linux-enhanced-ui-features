use eframe::egui;
use std::path::PathBuf;

pub(crate) fn handle_input(
    app: &mut crate::app::FileViewerApp,
    ctx: &egui::Context,
    file_to_load: &mut Option<PathBuf>,
) -> bool {
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
                Some(crate::app::Content::Image(_)) => {
                    app.image_fit = false;
                    app.image_zoom = 1.0;
                }
                _ => {}
            }
        }
        if (i.modifiers.command || i.modifiers.alt) && i.key_pressed(egui::Key::Equals) {
            match &app.content {
                Some(crate::app::Content::Text(_)) => {
                    app.text_zoom = (app.text_zoom * 1.05).clamp(0.6, 3.0)
                }
                Some(crate::app::Content::Image(_)) => {
                    app.image_fit = false;
                    app.image_zoom = (app.image_zoom * 1.10).clamp(0.1, 6.0);
                }
                _ => {}
            }
        }
        if (i.modifiers.command || i.modifiers.alt) && i.key_pressed(egui::Key::Minus) {
            match &app.content {
                Some(crate::app::Content::Text(_)) => {
                    app.text_zoom = (app.text_zoom / 1.05).clamp(0.6, 3.0)
                }
                Some(crate::app::Content::Image(_)) => {
                    app.image_fit = false;
                    app.image_zoom = (app.image_zoom / 1.10).clamp(0.1, 6.0);
                }
                _ => {}
            }
        }

        // Navigation with arrow keys for current content type
        let mut queue_neighbor = |forward: bool| {
            if let Some(cur) = app.current_path.as_deref() {
                let next = match app.content.as_ref() {
                    Some(crate::app::Content::Image(_)) => crate::io::neighbor_image(cur, forward),
                    Some(crate::app::Content::Text(_)) => crate::io::neighbor_text(cur, forward),
                    _ => None,
                };
                if let Some(path) = next {
                    *file_to_load = Some(path);
                }
            }
        };

        if i.key_pressed(egui::Key::ArrowRight)
            || (i.modifiers.alt && i.key_pressed(egui::Key::ArrowRight))
        {
            queue_neighbor(true);
        }
        if i.key_pressed(egui::Key::ArrowLeft)
            || (i.modifiers.alt && i.key_pressed(egui::Key::ArrowLeft))
        {
            queue_neighbor(false);
        }
        // Support '<' and '>' typed keys for both images and text
        for ev in &i.events {
            if let egui::Event::Text(t) = ev {
                match t.as_str() {
                    ">" => queue_neighbor(true),
                    "<" => queue_neighbor(false),
                    _ => {}
                }
            }
        }
    });
    toggle_dark
}
