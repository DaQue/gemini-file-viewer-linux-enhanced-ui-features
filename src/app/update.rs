use super::*;
use std::path::PathBuf;
use std::sync::mpsc::TryRecvError;

impl eframe::App for FileViewerApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(s) = serde_json::to_string(self) {
            storage.set_string(eframe::APP_KEY, s);
        }
        crate::settings::save_settings_to_disk(self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply visuals each frame
        self.apply_theme(ctx);

        // Ensure window size respects last-run width/height but not below minimums
        if !self.viewport_initialized {
            let min_w = 900.0f32;
            let min_h = 560.0f32;
            let w = self.last_window_width.max(min_w);
            let h = self.last_window_height.max(min_h);
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(w, h)));
            self.viewport_initialized = true;
        }

        // Track current viewport size so we can persist it on exit
        if let Some(rect) = ctx.input(|i| i.viewport().inner_rect) {
            let sz = rect.size();
            // Avoid tiny oscillations
            if (self.last_window_width - sz.x).abs() > 0.5 || (self.last_window_height - sz.y).abs() > 0.5 {
                self.last_window_width = sz.x;
                self.last_window_height = sz.y;
            }
        }

        let mut file_to_load: Option<PathBuf> = None;

        // Keyboard + mouse input (delegated)
        let toggle_dark = crate::input::handle_input(self, ctx, &mut file_to_load);

        // Drag-and-drop files to open
        if self.drag_and_drop_enabled {
            let dropped = ctx.input(|i| i.raw.dropped_files.clone());
            if !dropped.is_empty() {
                // Limit to avoid accidental floods
                let mut opened_first: bool = file_to_load.is_some();
                let mut extra_text_tabs: usize = 0;
                for f in dropped.into_iter().take(20) {
                    if let Some(path) = f.path {
                        if crate::io::is_supported_image(&path) || crate::io::is_supported_text(&path) {
                            if !opened_first {
                                file_to_load = Some(path);
                                opened_first = true;
                            } else if crate::io::is_supported_text(&path) && let Ok((text, lossy, lines)) = crate::io::load_text(&path) {
                                // Add text as background tab without switching
                                if !self.open_text_tabs.iter().any(|t| t.path == path) {
                                    self.open_text_tabs.push(TextTab { path: path.clone(), text, is_lossy: lossy, line_count: lines });
                                    extra_text_tabs += 1;
                                }
                            } else if crate::io::is_supported_image(&path) {
                                // Track image tab without switching
                                if !self.open_image_tabs.iter().any(|p| p == &path) {
                                    self.open_image_tabs.push(path.clone());
                                }
                            }
                        } else {
                            self.error_message = Some("Unsupported file type".to_string());
                        }
                    }
                }
                if extra_text_tabs > 0 && self.active_text_tab.is_none() {
                    // If no active tab yet, activate the last added
                    let last = self.open_text_tabs.len().saturating_sub(1);
                    if self.open_text_tabs.get(last).is_some() { self.active_text_tab = Some(last); }
                }
            }
        }

        // Poll background file open dialog
        if self.file_open_in_flight {
            if let Some(rx) = &self.file_open_rx {
                match rx.try_recv() {
                    Ok(opt) => {
                        self.file_open_in_flight = false;
                        self.file_open_rx = None;
                        if let Some(p) = opt { file_to_load = Some(p); }
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => {
                        self.file_open_in_flight = false;
                        self.file_open_rx = None;
                    }
                }
            } else {
                self.file_open_in_flight = false;
            }
        }

        // Modern About dialog
        if self.show_about {
            egui::Window::new("About gfv")
                .collapsible(false)
                .resizable(false)
                .open(&mut self.show_about)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("📁").size(48.0));
                        ui.add_space(12.0);
                        ui.label(RichText::new(format!("gfv {}", env!("CARGO_PKG_VERSION"))).heading().strong());
                        ui.add_space(16.0);
                        
                        ui.separator();
                        ui.add_space(12.0);
                        
                        ui.label(RichText::new("⌨️ Keyboard Shortcuts").strong());
                        ui.add_space(8.0);
                        
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.monospace(RichText::new("Ctrl+O").strong());
                                ui.monospace(RichText::new("Ctrl+D").strong());
                                ui.monospace(RichText::new("Ctrl+L").strong());
                                ui.monospace(RichText::new("Ctrl+W").strong());
                                ui.monospace(RichText::new("Ctrl+F").strong());
                            });
                            ui.add_space(16.0);
                            ui.vertical(|ui| {
                                ui.label("Open file");
                                ui.label("Toggle dark mode");
                                ui.label("Toggle line numbers");
                                ui.label("Toggle word wrap");
                                ui.label("Find in text");
                            });
                        });

                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.monospace(RichText::new("Ctrl+Wheel").strong());
                                ui.monospace(RichText::new("Ctrl+= / Ctrl+-").strong());
                                ui.monospace(RichText::new("Ctrl+0").strong());
                            });
                            ui.add_space(16.0);
                            ui.vertical(|ui| {
                                ui.label("Zoom text/image");
                                ui.label("Zoom in/out");
                                ui.label("Reset zoom");
                            });
                        });

                        ui.add_space(16.0);
                        ui.separator();
                        ui.add_space(8.0);
                        ui.label(RichText::new("ℹ️ About").strong());
                        ui.add_space(6.0);
                        ui.label(RichText::new("Authors: David Queen, Allison Bayless").small());
                        ui.add_space(6.0);
                        ui.label(RichText::new("Disclaimer: This software is provided ‘as is’ without warranty of any kind, whether express, implied, or statutory, including but not limited to warranties of merchantability, fitness for a particular purpose, and noninfringement. To the maximum extent permitted by law, the authors shall not be liable for any claim, damages, or other liability, whether in contract, tort, or otherwise, arising from or in connection with the software or its use.").small());
                    });
                });
        }
        if self.show_settings_window { crate::ui::settings_window(ctx, self); crate::settings::save_settings_to_disk(self); }
        if toggle_dark {
            self.dark_mode = !self.dark_mode;
            self.apply_theme(ctx);
            crate::settings::save_settings_to_disk(self);
        }

        if self.show_keybindings {
            egui::Window::new("Keybindings")
                .collapsible(false)
                .resizable(true)
                .min_width(520.0)
                .open(&mut self.show_keybindings)
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("General").strong());
                        ui.monospace("Ctrl/Alt+O — Open file");
                        ui.monospace("Ctrl/Alt+D — Toggle dark mode");
                        ui.monospace("Ctrl/Alt+L — Toggle line numbers");
                        ui.monospace("Ctrl/Alt+W — Toggle word wrap");
                        ui.monospace("Ctrl/Alt+, — Open Settings");
                        ui.monospace("F1        — Open About");
                        ui.add_space(8.0);
                        ui.label(RichText::new("Find/Search").strong());
                        ui.monospace("Ctrl/Alt+F — Focus Find");
                        ui.monospace("Enter     — Next match");
                        ui.monospace("Shift+Enter — Previous match");
                        ui.add_space(8.0);
                        ui.label(RichText::new("Navigation").strong());
                        ui.monospace("Left/Right or Alt+Left/Right — Prev/Next file in folder");
                        ui.monospace("< or >    — Previous/Next file in folder");
                        ui.add_space(8.0);
                        ui.label(RichText::new("Zoom").strong());
                        ui.monospace("Ctrl/Alt+=  — Zoom in (text/image)");
                        ui.monospace("Ctrl/Alt+-  — Zoom out (text/image)");
                        ui.monospace("Ctrl/Alt+0  — Reset zoom");
                        ui.monospace("Ctrl/Alt+Wheel — Zoom while hovering content");
                    });
                });
        }

        // Top Toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                crate::ui::toolbar(ui, self, ctx, &mut file_to_load);
            });
        });

        // Tab strip for open text tabs
        crate::ui::tab_strip(ctx, self);

        // Search Bar (for text files and images with navigation)
        if matches!(self.content, Some(Content::Text(_))) || matches!(self.content, Some(Content::Image(_))) {
            egui::TopBottomPanel::top("searchbar").show(ctx, |ui| {
                crate::ui::search_bar(ui, self, &mut file_to_load);
            });
        }

        // Auxiliary windows
        crate::ui::recent_files_window(ctx, self, &mut file_to_load);
        crate::ui::global_search_window(ctx, self);
        // Session restore: once per startup, after UI is initialized
        if self.restore_session && !self.session_restored {
            self.session_restored = true;
            let mut opened_any = false;
            let active_idx = self.session_active.unwrap_or(0);
            for (idx, p) in self.session_paths.clone().into_iter().enumerate() {
                if p.exists() {
                    if idx == active_idx {
                        file_to_load = Some(p.clone());
                        opened_any = true;
                    } else if crate::io::is_supported_text(&p) {
                        if let Ok((text, lossy, lines)) = crate::io::load_text(&p) {
                            let exists = self.open_text_tabs.iter().any(|t| t.path == p);
                            if !exists {
                                self.open_text_tabs.push(TextTab { path: p.clone(), text, is_lossy: lossy, line_count: lines });
                            }
                        }
                    } else if crate::io::is_supported_image(&p) {
                        // Defer actual image load to when activated
                        // Track via current_path if none yet
                        if self.current_path.is_none() && !opened_any {
                            file_to_load = Some(p.clone());
                            opened_any = true;
                        }
                    }
                }
            }
        }

        // Status Bar
        egui::TopBottomPanel::bottom("statusbar").show(ctx, |ui| {
            crate::ui::status_bar(ui, self);
        });

        // Extra status information
        egui::TopBottomPanel::bottom("status-extra").show(ctx, |ui| {
            crate::ui::status_extra(ui, self);
        });

        // Main Content (delegated)
        egui::CentralPanel::default().show(ctx, |ui| {
            crate::central::render_central_panel(ui, self);
        });

        // Deferred file loading to avoid borrow issues
        if let Some(path) = file_to_load {
            self.load_file(path, ctx);
        }
    }
}

