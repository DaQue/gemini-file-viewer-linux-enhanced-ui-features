use eframe::egui::{self, RichText, text::LayoutJob};

pub(crate) fn render_central_panel(ui: &mut egui::Ui, app: &mut crate::app::FileViewerApp) {
    if let Some(err) = &app.error_message {
        ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
    }

    if let Some(content) = &app.content {
        match content {
            crate::app::Content::Text(text) => {
                let mut frame = egui::Frame::group(ui.style());
                frame.fill = app.code_theme.background();
                frame.show(ui, |ui| {
                    ui.style_mut().wrap_mode = Some(if app.word_wrap { egui::TextWrapMode::Wrap } else { egui::TextWrapMode::Extend });
                    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                        let text_style = egui::TextStyle::Monospace;
                        let mut font_id = text_style.resolve(ui.style());
                        font_id.size = (font_id.size * app.text_zoom).clamp(8.0, 48.0);
                        let text_color = app.code_theme.foreground();

                        let do_line_numbers = app.show_line_numbers && !app.text_is_big;
                        let do_highlight = !app.text_is_big && text.len() <= crate::app::HIGHLIGHT_CHAR_THRESHOLD;
                        if do_line_numbers || do_highlight || !app.search_query.is_empty() {
                            let mut bracket_depth: i32 = 0;
                            let mut in_block_comment = false;
                            let ext = app
                                .current_path
                                .as_ref()
                                .and_then(|p| p.extension().and_then(|s| s.to_str()))
                                .unwrap_or("")
                                .to_lowercase();
                            let target_line_from_search = if !app.search_query.is_empty() && app.search_count > 0 {
                                crate::search::find_target_line(text, &app.search_query, app.search_current)
                            } else { None };
                            let mut counter: usize = 0;
                            let mut target_rect: Option<egui::Rect> = None;
                            let mut syntect_session = if app.use_syntect && do_highlight {
                                Some(crate::highlight_syntect::SyntectSession::start(&ext, true))
                            } else { None };
                            for (i, line) in text.lines().enumerate() {
                                let mut line_job = LayoutJob::default();
                                if do_line_numbers {
                                    line_job.append(&format!("{:>4} ", i + 1), 0.0, egui::TextFormat { font_id: font_id.clone(), color: app.code_theme.comment(), ..Default::default() });
                                }
                                if let Some(session) = syntect_session.as_mut() {
                                    session.append_line(&mut line_job, line, font_id.clone());
                                } else {
                                    let mut hctx = crate::highlight::HighlightContext {
                                        ext: &ext,
                                        font_id: font_id.clone(),
                                        base_color: text_color,
                                        do_syntax: do_highlight,
                                        depth: &mut bracket_depth,
                                        current_idx: app.search_current,
                                        counter: &mut counter,
                                        query: &app.search_query,
                                        in_block_comment: &mut in_block_comment,
                                        theme: app.code_theme,
                                    };
                                    crate::highlight::append_highlighted(&mut line_job, line, &mut hctx);
                                }
                                let resp = ui.label(line_job);
                                if target_line_from_search == Some(i) || app.scroll_target_line == Some(i) { target_rect = Some(resp.rect); }
                            }
                            if let Some(rect) = target_rect { ui.scroll_to_rect(rect, Some(egui::Align::Center)); }
                            app.scroll_target_line = None;
                        } else {
                            ui.label(RichText::new(text).monospace().size(font_id.size));
                        }
                    });
                });
            }
            crate::app::Content::Image(texture) => {
                let viewport = ui.available_size();
                egui::ScrollArea::both().show(ui, |ui| {
                    ui.centered_and_justified(|ui| {
                        let size = texture.size();
                        let mut effective_zoom = app.image_zoom;
                        if app.image_fit {
                            let sx = if size[0] > 0 { viewport.x / size[0] as f32 } else { 1.0 };
                            let sy = if size[1] > 0 { viewport.y / size[1] as f32 } else { 1.0 };
                            let fit = sx.min(sy);
                            if fit.is_finite() && fit > 0.0 {
                                effective_zoom = fit.clamp(0.1, 6.0);
                            }
                        }
                        let desired = egui::vec2(size[0] as f32 * effective_zoom, size[1] as f32 * effective_zoom);
                        let image = egui::Image::new(texture).fit_to_exact_size(desired);
                        let resp = ui.add(image);
                        if resp.hovered() {
                            let scroll = ui.input(|i| i.raw_scroll_delta.y);
                            if scroll != 0.0 {
                                app.image_fit = false;
                                let factor = if scroll > 0.0 { 1.10 } else { 1.0 / 1.10 };
                                app.image_zoom = (app.image_zoom * factor).clamp(0.1, 6.0);
                            }
                        }
                    });
                });
            }
        }
    } else if app.error_message.is_none() {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() * 0.2);
            ui.label(RichText::new("📁").size(64.0));
            ui.add_space(16.0);
            ui.label(RichText::new("Gemini File Viewer").heading().strong());
            ui.add_space(8.0);
            ui.label(RichText::new("A modern file viewer for text and images").weak());
            ui.add_space(32.0);
            ui.vertical_centered(|ui| {
                let mut open_button = egui::Button::new(RichText::new("📂 Open File").strong());
                open_button = open_button.fill(egui::Color32::from_rgb(34, 197, 94));
                if ui.add(open_button).clicked() { app.start_open_file_dialog(); }
            });
        });
    }
}
