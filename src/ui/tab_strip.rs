use eframe::egui;

pub(crate) fn tab_strip(ctx: &egui::Context, app: &mut crate::app::FileViewerApp) {
    egui::TopBottomPanel::top("tabstrip").show(ctx, |ui| {
        egui::ScrollArea::horizontal().auto_shrink([false, true]).show(ui, |ui| {
            let mut text_to_switch: Option<usize> = None;
            let mut text_to_close: Option<usize> = None;
            let mut img_to_switch: Option<usize> = None;
            let mut img_to_close: Option<usize> = None;
            ui.horizontal(|ui| {
                for (idx, tab) in app.open_text_tabs.iter().enumerate() {
                    let is_active = matches!(app.content, Some(crate::app::Content::Text(_))) && app.active_text_tab == Some(idx);
                    let file_name = tab.path.file_name().and_then(|s| s.to_str()).unwrap_or("(untitled)");
                    let mut frame = egui::Frame::group(ui.style());
                    if is_active { frame = frame.fill(egui::Color32::from_rgb(30, 41, 59)); }
                    frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.selectable_label(is_active, egui::RichText::new(file_name).monospace()).clicked() { text_to_switch = Some(idx); }
                            if ui.small_button("✕").on_hover_text("Close tab").clicked() { text_to_close = Some(idx); }
                        });
                    });
                }
                for (idx, path) in app.open_image_tabs.iter().enumerate() {
                    let is_active = matches!(app.content, Some(crate::app::Content::Image(_))) && app.active_image_tab == Some(idx);
                    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("(image)");
                    let mut frame = egui::Frame::group(ui.style());
                    if is_active { frame = frame.fill(egui::Color32::from_rgb(30, 41, 59)); }
                    frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.selectable_label(is_active, egui::RichText::new(file_name)).clicked() { img_to_switch = Some(idx); }
                            if ui.small_button("✕").on_hover_text("Close image tab").clicked() { img_to_close = Some(idx); }
                        });
                    });
                }
            });
            if let Some(idx) = text_to_switch { app.switch_to_text_tab(idx); }
            if let Some(idx) = text_to_close {
                let was_active = app.active_text_tab == Some(idx);
                if idx < app.open_text_tabs.len() { app.open_text_tabs.remove(idx); }
                if app.open_text_tabs.is_empty() {
                    app.active_text_tab = None;
                    if matches!(app.content, Some(crate::app::Content::Text(_))) { app.content = None; app.current_path = None; }
                } else {
                    if was_active {
                        let new_idx = if idx >= app.open_text_tabs.len() { app.open_text_tabs.len() - 1 } else { idx };
                        app.switch_to_text_tab(new_idx);
                    } else if let Some(a) = app.active_text_tab { if a > idx { app.active_text_tab = Some(a - 1); } }
                }
            }
            if let Some(idx) = img_to_switch { if let Some(_p) = app.open_image_tabs.get(idx).cloned() { app.active_image_tab = Some(idx); ctx.memory_mut(|m| m.request_focus(egui::Id::new("central"))); } }
            if let Some(idx) = img_to_close {
                if idx < app.open_image_tabs.len() { app.open_image_tabs.remove(idx); }
                if app.open_image_tabs.is_empty() && matches!(app.content, Some(crate::app::Content::Image(_))) {
                    app.content = None; app.current_path = None; app.active_image_tab = None;
                } else if let Some(a) = app.active_image_tab { if a > idx { app.active_image_tab = Some(a - 1); } }
            }
        });
    });
}

