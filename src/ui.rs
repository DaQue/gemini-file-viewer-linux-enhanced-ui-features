use std::path::PathBuf;
use eframe::egui;
use egui::RichText;
use crate::themes::CodeTheme;

pub(crate) fn toolbar(ui: &mut egui::Ui, app: &mut crate::app::FileViewerApp, _ctx: &egui::Context, file_to_load: &mut Option<PathBuf>) {

    // Modern app branding
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.label(RichText::new("📁").size(20.0));
        ui.add_space(8.0);
        ui.label(RichText::new("gfv").heading().strong());
        ui.add_space(8.0);
        ui.label(RichText::new("Pre-beta").weak().small());
    });
    
    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    // All toolbar buttons in single horizontal layout for perfect alignment
    ui.horizontal(|ui| {
        // Open File button
        let mut open_button = egui::Button::new(RichText::new("📂 Open File").strong());
        open_button = open_button.fill(egui::Color32::from_rgb(34, 197, 94)); // Green
        let open_clicked = ui.add_enabled(!app.file_open_in_flight, open_button).clicked();
        if open_clicked {
            // Use blocking dialog here for reliability
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("All Supported", &["txt","rs","py","toml","md","json","js","html","css","png","jpg","jpeg","gif","bmp","webp"])
                .add_filter("Images", &["png","jpg","jpeg","gif","bmp","webp"])
                .add_filter("Text/Source", &["txt","rs","py","toml","md","json","js","html","css"])
                .pick_file() {
                *file_to_load = Some(path);
            }
        }
        if app.file_open_in_flight {
            ui.add_space(8.0);
            ui.add(egui::Spinner::new().size(14.0));
            ui.label(RichText::new("Opening file…").weak());
        }

        // Recent Files window toggle (short label to keep near Open)
        let mut recent_button = egui::Button::new(RichText::new("📋 Recent").strong());
        recent_button = recent_button.fill(egui::Color32::from_rgb(59, 130, 246)); // Blue
        if ui.add_enabled(!app.file_open_in_flight, recent_button).clicked() {
            app.show_recent_window = !app.show_recent_window;
        }

        // Global Search window toggle (restored position)
        let mut global_button = egui::Button::new(RichText::new("🔎 Global Search").strong());
        global_button = global_button.fill(egui::Color32::from_rgb(168, 85, 247)); // Purple
        if ui.add(global_button).clicked() {
            app.show_global_search_window = !app.show_global_search_window;
        }

        // One-shot Reopen Session
        let can_reopen = !app.session_paths.is_empty();
        let mut reopen_button = egui::Button::new(RichText::new("⟳ Reopen Session").strong());
        reopen_button = reopen_button.fill(egui::Color32::from_rgb(107, 114, 128)); // Gray
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

        // Themes button
        ui.menu_button(RichText::new("🎨 Themes").strong(), |ui| {
            ui.set_min_width(300.0);
            
            let prev_theme = app.code_theme;
            
            ui.label(RichText::new("🎨 Code Themes").strong());
            ui.add_space(8.0);
            
            for theme in CodeTheme::all() {
                let is_selected = app.code_theme == *theme;
                let mut button_text = RichText::new(theme.name());
                if is_selected {
                    button_text = button_text.strong();
                }
                
                if ui.selectable_label(is_selected, button_text).clicked() {
                    app.code_theme = *theme;
                    ui.close_menu();
                }
            }
            
            if app.code_theme != prev_theme {
                crate::settings::save_settings_to_disk(app);
            }
        });

        // Settings window toggle (more reliable than a dropdown on some platforms)
        if ui.add(egui::Button::new(RichText::new("⚙️ Settings").strong()).fill(egui::Color32::from_rgb(107, 114, 128))).clicked() {
            app.show_settings_window = true;
        }
        if ui.add(egui::Button::new(RichText::new("ℹ️ About").strong()).fill(egui::Color32::from_rgb(107, 114, 128))).clicked() {
            app.show_about = true;
        }

        // Clear button
        let mut clear_button = egui::Button::new(RichText::new("🗑️ Clear").strong());
        clear_button = clear_button.fill(egui::Color32::from_rgb(239, 68, 68)); // Red
        if ui.add(clear_button).clicked() {
            app.content = None;
            app.current_path = None;
            app.error_message = None;
        }

        // (global search was moved back to earlier position)
    });

    // (image tabs moved into the unified top tab strip)

    // Image controls (zoom and fit)
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
            zoom_out_button = zoom_out_button.fill(egui::Color32::from_rgb(245, 158, 11)); // Orange
            if ui.add(zoom_out_button).clicked() { 
                app.image_fit = false; 
                app.image_zoom = (app.image_zoom / 1.10).clamp(0.1, 6.0); 
            }
            let mut zoom_in_button = egui::Button::new(RichText::new("🔍+").strong());
            zoom_in_button = zoom_in_button.fill(egui::Color32::from_rgb(245, 158, 11)); // Orange
            if ui.add(zoom_in_button).clicked() { 
                app.image_fit = false; 
                app.image_zoom = (app.image_zoom * 1.10).clamp(0.1, 6.0); 
            }
            let mut reset_button = egui::Button::new(RichText::new("100%").strong());
            reset_button = reset_button.fill(egui::Color32::from_rgb(34, 197, 94)); // Green
            if ui.add(reset_button).clicked() { 
                app.image_fit = false; 
                app.image_zoom = 1.0; 
            }
        });
    }
}

pub(crate) fn search_bar(ui: &mut egui::Ui, app: &mut crate::app::FileViewerApp, file_to_load: &mut Option<PathBuf>) {
    // Modern search bar with better styling
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            // Search input (only for text files)
            if matches!(app.content, Some(crate::app::Content::Text(_))) {
                ui.label(RichText::new("🔍 Find:").strong());
                ui.add_space(8.0);
                
                let prev = app.search_query.clone();
                let resp = ui.text_edit_singleline(&mut app.search_query);
                if app.search_active {
                    resp.request_focus();
                    app.search_active = false;
                }
                
                // Enter / Shift+Enter navigate matches
                let (enter, shift) = ui.input(|i| (i.key_pressed(egui::Key::Enter), i.modifiers.shift));
                if enter && app.search_count > 0 {
                    if shift {
                        if app.search_current == 0 { app.search_current = app.search_count.saturating_sub(1); } else { app.search_current -= 1; }
                    } else {
                        app.search_current = (app.search_current + 1) % app.search_count;
                    }
                }

                if resp.changed() || (prev.is_empty() && !app.search_query.is_empty()) {
                    app.search_count = 0;
                    app.search_current = 0;
                    if let Some(crate::app::Content::Text(ref text)) = app.content
                        && !app.search_query.is_empty()
                        && text.len() <= crate::app::HIGHLIGHT_CHAR_THRESHOLD {
                        app.search_count = crate::search::recompute_count(&app.search_query, text);
                    }
                }
            }
            
            // File navigation buttons (Prev/Next) - compact with just arrows
            if let Some(cur) = app.current_path.clone() {
                ui.add_space(12.0);
                match app.content {
                    Some(crate::app::Content::Image(_)) => {
                        if ui.small_button(RichText::new("⬅️").size(10.0)).on_hover_text("Previous file").clicked()
                            && let Some(prev) = crate::io::neighbor_image(&cur, false) {
                            *file_to_load = Some(prev);
                        }
                        if ui.small_button(RichText::new("➡️").size(10.0)).on_hover_text("Next file").clicked()
                            && let Some(next) = crate::io::neighbor_image(&cur, true) {
                            *file_to_load = Some(next);
                        }
                    }
                    Some(crate::app::Content::Text(_)) => {
                        if ui.small_button(RichText::new("⬅️").size(10.0)).on_hover_text("Previous file").clicked()
                            && let Some(prev) = crate::io::neighbor_text(&cur, false) {
                            *file_to_load = Some(prev);
                        }
                        if ui.small_button(RichText::new("➡️").size(10.0)).on_hover_text("Next file").clicked()
                            && let Some(next) = crate::io::neighbor_text(&cur, true) {
                            *file_to_load = Some(next);
                        }
                    }
                    _ => {}
                }
            }
            
            // Search result navigation (only for text files with search query)
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
                
                if app.search_count > 0 {
                    ui.add_space(8.0);
                    ui.label(RichText::new(format!("{}/{}", app.search_current + 1, app.search_count)).strong());
                }
            }
        });
    });
}

pub(crate) fn status_bar(ui: &mut egui::Ui, app: &mut crate::app::FileViewerApp) {
    use std::fs;
    
    // Modern status bar with better visual hierarchy
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
                copy_button = copy_button.fill(egui::Color32::from_rgb(34, 197, 94)); // Green
                if ui.add(copy_button).on_hover_text("Copy path to clipboard").clicked() {
                    ui.ctx().copy_text(path.to_string_lossy().into());
                }
                let mut folder_button = egui::Button::new(RichText::new("📁 Open Folder").strong());
                folder_button = folder_button.fill(egui::Color32::from_rgb(59, 130, 246)); // Blue
                if ui.add(folder_button).clicked() {
                    #[cfg(target_os = "windows")]
                    { let _ = std::process::Command::new("explorer").arg(path).spawn(); }
                    #[cfg(target_os = "macos")]
                    { let _ = std::process::Command::new("open").arg("-R").arg(path).spawn(); }
                    #[cfg(all(unix, not(target_os = "macos")))]
                    { if let Some(parent) = path.parent() { let _ = std::process::Command::new("xdg-open").arg(parent).spawn(); } }
                }
            } else {
                ui.label(RichText::new("📄 No file selected").weak());
            }
        });
    });
}

pub(crate) fn status_extra(ui: &mut egui::Ui, app: &mut crate::app::FileViewerApp) {
    // Modern status extra with icons and better formatting
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal(|ui| {
            match &app.content {
                Some(crate::app::Content::Image(texture)) => {
                    let size = texture.size();
                    ui.colored_label(egui::Color32::from_rgb(34, 197, 94), RichText::new("🖼️").size(16.0)); // Green
                    ui.add_space(8.0);
                    ui.label(RichText::new(format!("{}x{} px", size[0], size[1])).strong());
                    
                    let eff = if app.image_fit { None } else { Some(app.image_zoom) };
                    if let Some(z) = eff { 
                        ui.add_space(12.0);
                        ui.colored_label(egui::Color32::from_rgb(245, 158, 11), RichText::new(format!("🔍 {:.0}%", z * 100.0))); // Orange
                    }
                    
                    let est = size[0].saturating_mul(size[1]).saturating_mul(4);
                    ui.add_space(12.0);
                    ui.colored_label(egui::Color32::from_rgb(59, 130, 246), RichText::new(format!("💾 ~{:.1} MB", est as f64 / (1024.0 * 1024.0)))); // Blue
                    
                    if app.image_fit { 
                        ui.add_space(12.0);
                        ui.colored_label(egui::Color32::from_rgb(168, 85, 247), RichText::new("📐 Fit: on")); // Purple
                    }
                }
                Some(crate::app::Content::Text(_)) => {
                    ui.colored_label(egui::Color32::from_rgb(34, 197, 94), RichText::new("📝").size(16.0)); // Green
                    ui.add_space(8.0);
                    ui.label(RichText::new(format!("{} lines", app.text_line_count)).strong());
                    ui.add_space(12.0);
                    ui.colored_label(egui::Color32::from_rgb(245, 158, 11), RichText::new(format!("🔍 {:.0}%", app.text_zoom * 100.0))); // Orange
                    
                    if app.text_is_big { 
                        ui.add_space(12.0);
                        ui.colored_label(egui::Color32::from_rgb(239, 68, 68), RichText::new("⚠️ Large file: reduced features")); // Red
                    }
                    if app.text_is_lossy { 
                        ui.add_space(12.0);
                        ui.colored_label(egui::Color32::from_rgb(239, 68, 68), RichText::new("⚠️ UTF-8 (lossy)")); // Red
                    }
                }
                _ => {}
            }
        });
    });
}

pub(crate) fn tab_strip(ctx: &egui::Context, app: &mut crate::app::FileViewerApp) {
    egui::TopBottomPanel::top("tabstrip").show(ctx, |ui| {
        egui::ScrollArea::horizontal().auto_shrink([false, true]).show(ui, |ui| {
            let mut text_to_switch: Option<usize> = None;
            let mut text_to_close: Option<usize> = None;
            let mut img_to_switch: Option<usize> = None;
            let mut img_to_close: Option<usize> = None;
            ui.horizontal(|ui| {
                // Text tabs first
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
                // Image tabs after
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
                // Remove the tab and update active/content
                let was_active = app.active_text_tab == Some(idx);
                if idx < app.open_text_tabs.len() {
                    app.open_text_tabs.remove(idx);
                }
                if app.open_text_tabs.is_empty() {
                    app.active_text_tab = None;
                    // If we were showing text from that tab, clear content
                    if matches!(app.content, Some(crate::app::Content::Text(_))) {
                        app.content = None;
                        app.current_path = None;
                    }
                } else {
                    // Adjust active index
                    if was_active {
                        let new_idx = if idx >= app.open_text_tabs.len() { app.open_text_tabs.len() - 1 } else { idx };
                        app.switch_to_text_tab(new_idx);
                    } else if let Some(a) = app.active_text_tab {
                        // Shift left if needed
                        if a > idx { app.active_text_tab = Some(a - 1); }
                    }
                }
            }
            if let Some(idx) = img_to_switch {
                if let Some(p) = app.open_image_tabs.get(idx).cloned() { app.active_image_tab = Some(idx); ctx.memory_mut(|m| m.request_focus(egui::Id::new("central"))); }
            }
            if let Some(idx) = img_to_close {
                if idx < app.open_image_tabs.len() { app.open_image_tabs.remove(idx); }
                if app.open_image_tabs.is_empty() && matches!(app.content, Some(crate::app::Content::Image(_))) {
                    app.content = None;
                    app.current_path = None;
                    app.active_image_tab = None;
                } else if let Some(a) = app.active_image_tab { if a > idx { app.active_image_tab = Some(a - 1); } }
            }
        });
    });
}

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
                if app.recent_files.is_empty() {
                    ui.label(RichText::new("No recent files").weak());
                }
                for file in app.recent_files.clone().into_iter().rev() {
                    let display = file.to_string_lossy();
                    if ui.button(egui::RichText::new(display.clone()).monospace()).on_hover_text(display).clicked() {
                        *file_to_load = Some(file);
                        // Auto-close window on selection
                        app.show_recent_window = false;
                    }
                }
            });
            ui.separator();
            // One-shot reopen session
            let can_reopen = !app.session_paths.is_empty();
            if ui.add_enabled(can_reopen, egui::Button::new(RichText::new("⟳ Reopen Last Session").strong())).clicked() {
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
                // Close window after action
                app.show_recent_window = false;
            }
            ui.add_space(8.0);
            let clear_button = egui::Button::new(RichText::new("🗑️ Clear Recent Files"));
            let clear_color = egui::Color32::from_rgb(239, 68, 68);
            if ui.add(clear_button.fill(clear_color)).clicked() {
                app.recent_files.clear();
                app.show_recent_window = false;
            }
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
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        app.recompute_global_search();
                    }
                    if ui.button(RichText::new("Search").strong()).clicked() {
                        app.recompute_global_search();
                    }
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut app.global_case_sensitive, "Case sensitive");
                    let ww = ui.checkbox(&mut app.global_whole_word, "Whole word");
                    if app.global_regex {
                        // Visually disable by graying out when regex mode is on
                        // and force it off to avoid confusion
                        app.global_whole_word = false;
                        ww.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Checkbox, false, "Whole word (disabled in regex)"));
                    }
                    ui.checkbox(&mut app.global_regex, "Regex");
                });
                if let Some(err) = &app.global_error { ui.colored_label(egui::Color32::RED, err); }
                ui.separator();
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    for (idx, res) in app.global_results.clone().into_iter().enumerate() {
                        let label = format!("{}:{} — {}", res.path.to_string_lossy(), res.line_index + 1, res.snippet);
                        if ui.selectable_label(false, egui::RichText::new(label).monospace()).clicked() {
                            // Switch to tab
                            app.switch_to_text_tab(res.tab_index);
                            // Set local search state and jump
                            app.search_query = app.global_query.clone();
                            if let Some(active) = app.active_text_tab
                                && let Some(tab) = app.open_text_tabs.get(active) {
                                app.search_count = crate::search::recompute_count(&app.search_query, &tab.text);
                            }
                            app.search_current = res.match_index_in_tab.min(app.search_count.saturating_sub(1));
                            app.scroll_target_line = Some(res.line_index);
                            // Close window
                            app.show_global_search_window = false;
                        }
                        if idx < app.global_results.len().saturating_sub(1) { ui.separator(); }
                    }
                    if app.global_results.is_empty() && !app.global_query.is_empty() {
                        ui.label(RichText::new("No results").weak());
                    }
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

