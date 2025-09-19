use eframe::egui;
use crate::themes::CodeTheme;
use egui::{RichText, TextureHandle};
use std::fs;
use rfd::FileDialog;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::thread;

const MAX_FILE_SIZE_BYTES: u64 = 10_000_000; // 10MB
const MAX_RECENT_FILES: usize = 10;
const BIG_TEXT_CHAR_THRESHOLD: usize = 500_000; // Disable heavy features beyond this
pub(crate) const HIGHLIGHT_CHAR_THRESHOLD: usize = 200_000; // Disable syntax/mark highlights beyond this

pub enum Content {
    Text(String),
    Image(TextureHandle),
}

#[derive(Clone)]
pub struct TextTab {
    pub path: PathBuf,
    pub text: String,
    pub is_lossy: bool,
    pub line_count: usize,
}

#[derive(Clone)]
pub struct GlobalSearchResult {
    pub tab_index: usize,
    pub path: PathBuf,
    pub line_index: usize,
    pub snippet: String,
    pub match_index_in_tab: usize,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct FileViewerApp {
    #[serde(skip)]
    pub(crate) content: Option<Content>,
    #[serde(skip)]
    pub(crate) current_path: Option<PathBuf>,
    #[serde(skip)]
    pub(crate) error_message: Option<String>,
    pub(crate) dark_mode: bool,
    pub(crate) code_theme: CodeTheme,
    pub(crate) recent_files: Vec<PathBuf>,
    pub(crate) show_line_numbers: bool,
    pub(crate) word_wrap: bool,
    pub(crate) use_syntect: bool,
    pub(crate) drag_and_drop_enabled: bool,
    // Persisted window size (logical points)
    pub(crate) last_window_width: f32,
    pub(crate) last_window_height: f32,
    pub(crate) text_zoom: f32,
    pub(crate) image_zoom: f32,
    #[serde(skip)]
    pub(crate) show_about: bool,
    #[serde(skip)]
    pub(crate) show_settings_window: bool,
    #[serde(skip)]
    pub(crate) show_keybindings: bool,
    pub(crate) image_fit: bool,
    // Derived/runtime-only state for text rendering
    #[serde(skip)]
    pub(crate) text_is_big: bool,
    #[serde(skip)]
    pub(crate) text_line_count: usize,
    #[serde(skip)]
    pub(crate) text_is_lossy: bool,
    // Tabs: keep opened text files for global search and fast switching
    #[serde(skip)]
    pub(crate) open_text_tabs: Vec<TextTab>,
    #[serde(skip)]
    pub(crate) active_text_tab: Option<usize>,
    // Image tabs: store paths only; load on activation
    #[serde(skip)]
    pub(crate) open_image_tabs: Vec<PathBuf>,
    #[serde(skip)]
    pub(crate) active_image_tab: Option<usize>,
    // Simple find state
    #[serde(skip)]
    pub(crate) search_query: String,
    #[serde(skip)]
    pub(crate) search_active: bool,
    #[serde(skip)]
    pub(crate) search_count: usize,
    #[serde(skip)]
    pub(crate) search_current: usize,
    // Optional direct scroll target line for precise jumps
    #[serde(skip)]
    pub(crate) scroll_target_line: Option<usize>,
    // Recent files window toggle
    #[serde(skip)]
    pub(crate) show_recent_window: bool,
    // Global search UI state
    #[serde(skip)]
    pub(crate) show_global_search_window: bool,
    #[serde(skip)]
    pub(crate) global_query: String,
    #[serde(skip)]
    pub(crate) global_case_sensitive: bool,
    #[serde(skip)]
    pub(crate) global_whole_word: bool,
    #[serde(skip)]
    pub(crate) global_regex: bool,
    #[serde(skip)]
    pub(crate) global_results: Vec<GlobalSearchResult>,
    #[serde(skip)]
    pub(crate) global_error: Option<String>,
    // Session restore (persisted)
    pub restore_session: bool,
    pub session_paths: Vec<PathBuf>,
    pub session_active: Option<usize>,
    #[serde(skip)]
    pub(crate) session_restored: bool,
    // Non-blocking file dialog
    #[serde(skip)]
    pub(crate) file_open_rx: Option<Receiver<Option<PathBuf>>>,
    #[serde(skip)]
    pub(crate) file_open_in_flight: bool,
    // Runtime
    #[serde(skip)]
    pub(crate) viewport_initialized: bool,
}

impl FileViewerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage
            && let Some(s) = storage.get_string(eframe::APP_KEY)
            && let Ok(mut app) = serde_json::from_str::<FileViewerApp>(&s)
        {
            // ensure runtime-only fields are initialized
            app.text_is_big = false;
            app.text_line_count = 0;
            app.text_is_lossy = false;
            app.open_text_tabs = Vec::new();
            app.active_text_tab = None;
            app.open_image_tabs = Vec::new();
            app.active_image_tab = None;
            app.search_query = String::new();
            app.search_active = false;
            app.search_count = 0;
            app.scroll_target_line = None;
            app.show_recent_window = false;
            app.show_global_search_window = false;
            app.global_query = String::new();
            app.global_case_sensitive = false;
            app.global_whole_word = false;
            app.global_regex = false;
            app.global_results = Vec::new();
            app.global_error = None;
            // Keep any previously persisted session fields; ensure runtime flags
            app.session_restored = false;
            app.file_open_rx = None;
            app.file_open_in_flight = false;
            app.viewport_initialized = false;
            return app;
        }
        if let Some(mut app) = crate::settings::load_settings_from_disk() {
            app.text_is_big = false;
            app.text_line_count = 0;
            app.text_is_lossy = false;
            app.open_text_tabs = Vec::new();
            app.active_text_tab = None;
            app.open_image_tabs = Vec::new();
            app.active_image_tab = None;
            app.search_query = String::new();
            app.search_active = false;
            app.search_count = 0;
            app.scroll_target_line = None;
            app.show_recent_window = false;
            app.show_global_search_window = false;
            app.global_query = String::new();
            app.global_case_sensitive = false;
            app.global_whole_word = false;
            app.global_regex = false;
            app.global_results = Vec::new();
            app.global_error = None;
            app.session_restored = false;
            app.file_open_rx = None;
            app.file_open_in_flight = false;
            app.viewport_initialized = false;
            return app;
        }
        Default::default()
    }

    pub(crate) fn apply_theme(&self, ctx: &egui::Context) {
        crate::style::apply_theme(self, ctx);
    }

    // io helpers moved to crate::io

    pub fn load_file(&mut self, path: PathBuf, ctx: &egui::Context) {
        self.content = None;
        self.error_message = None;
        self.current_path = None;

        if let Ok(metadata) = fs::metadata(&path)
            && metadata.len() > MAX_FILE_SIZE_BYTES
        {
            self.error_message = Some(format!(
                "File is too large (> {:.1}MB)",
                MAX_FILE_SIZE_BYTES as f64 / 1_000_000.0
            ));
            return;
        }

        let loaded = if crate::io::is_supported_image(&path) {
            match crate::io::load_image(&path) {
                Ok(color_image) => {
                    let texture = ctx.load_texture(
                        path.to_string_lossy(),
                        color_image,
                        egui::TextureOptions::LINEAR,
                    );
                    // Track in image tabs
                    let mut exists = false;
                    for p in &self.open_image_tabs { if p == &path { exists = true; break; } }
                    if !exists { self.open_image_tabs.push(path.clone()); }
                    self.active_image_tab = self.open_image_tabs.iter().position(|p| p == &path);
                    Ok(Content::Image(texture))
                }
                Err(e) => Err(e),
            }
        } else {
            match crate::io::load_text(&path) {
                Ok((text, lossy, lines)) => {
                    self.text_is_big = text.len() >= BIG_TEXT_CHAR_THRESHOLD || lines >= 50_000;
                    self.text_line_count = lines;
                    self.text_is_lossy = lossy;
                    // Update or insert text tab
                    let mut tab_idx_opt = None;
                    for (idx, t) in self.open_text_tabs.iter().enumerate() {
                        if t.path == path { tab_idx_opt = Some(idx); break; }
                    }
                    match tab_idx_opt {
                        Some(idx) => {
                            self.open_text_tabs[idx] = TextTab { path: path.clone(), text: text.clone(), is_lossy: lossy, line_count: lines };
                            self.active_text_tab = Some(idx);
                        }
                        None => {
                            self.open_text_tabs.push(TextTab { path: path.clone(), text: text.clone(), is_lossy: lossy, line_count: lines });
                            self.active_text_tab = Some(self.open_text_tabs.len() - 1);
                        }
                    }
                    Ok(Content::Text(text))
                }
                Err(e) => Err(e),
            }
        };

        match loaded {
            Ok(content) => {
                self.content = Some(content);
                self.current_path = Some(path.clone());
                // Deduplicate and push to recents
                self.recent_files.retain(|p| p != &path);
                self.recent_files.push(path);
                if self.recent_files.len() > MAX_RECENT_FILES {
                    let overflow = self.recent_files.len() - MAX_RECENT_FILES;
                    self.recent_files.drain(0..overflow);
                }
                // Snapshot session and persist
                self.snapshot_session();
                crate::settings::save_settings_to_disk(self);
            }
            Err(e) => self.error_message = Some(e),
        }
    }

    pub(crate) fn start_open_file_dialog(&mut self) {
        if self.file_open_in_flight { return; }
        self.file_open_in_flight = true;
        let (tx, rx) = channel::<Option<PathBuf>>();
        self.file_open_rx = Some(rx);
        thread::spawn(move || {
            let picked = FileDialog::new()
                .add_filter("All Supported", &["txt","rs","py","toml","md","json","js","html","css","png","jpg","jpeg","gif","bmp","webp"])
                .add_filter("Images", &["png","jpg","jpeg","gif","bmp","webp"])
                .add_filter("Text/Source", &["txt","rs","py","toml","md","json","js","html","css"])
                .pick_file();
            let _ = tx.send(picked);
        });
    }

    pub(crate) fn switch_to_text_tab(&mut self, tab_index: usize) {
        if let Some(tab) = self.open_text_tabs.get(tab_index).cloned() {
            self.active_text_tab = Some(tab_index);
            self.current_path = Some(tab.path.clone());
            self.text_is_big = tab.text.len() >= BIG_TEXT_CHAR_THRESHOLD || tab.line_count >= 50_000;
            self.text_line_count = tab.line_count;
            self.text_is_lossy = tab.is_lossy;
            self.content = Some(Content::Text(tab.text));
            // Snapshot session on switch
            self.snapshot_session();
            crate::settings::save_settings_to_disk(self);
        }
    }

    pub(crate) fn recompute_global_search(&mut self) {
        self.global_results.clear();
        self.global_error = None;
        match crate::search::global_search(&self.open_text_tabs, &self.global_query, self.global_case_sensitive, self.global_whole_word, self.global_regex) {
            Ok(res) => self.global_results = res,
            Err(e) => { self.global_error = Some(e); }
        }
    }

    pub(crate) fn snapshot_session(&mut self) {
        // Build session_paths from open text tabs plus current path (for images/non-text)
        let mut paths: Vec<PathBuf> = self.open_text_tabs.iter().map(|t| t.path.clone()).collect();
        if let Some(cur) = self.current_path.clone() {
            let is_text = crate::io::is_supported_text(&cur);
            if !is_text {
                if !paths.contains(&cur) {
                    paths.push(cur);
                }
            }
        }
        // Filter out non-existing files
        paths.retain(|p| p.exists());
        // Active index is current_path in paths if present
        let active = self.current_path.as_ref().and_then(|cur| paths.iter().position(|p| p == cur));
        self.session_paths = paths;
        self.session_active = active;
    }

    // settings helpers moved to crate::settings
}

impl Default for FileViewerApp {
    fn default() -> Self {
        Self {
            content: None,
            current_path: None,
            error_message: None,
            dark_mode: true,
            code_theme: CodeTheme::default(),
            recent_files: Vec::new(),
            show_line_numbers: true,
            word_wrap: true,
            use_syntect: true,
            drag_and_drop_enabled: true,
            last_window_width: 1000.0,
            last_window_height: 700.0,
            text_zoom: 1.0,
            image_zoom: 1.0,
            show_about: false,
            show_settings_window: false,
            show_keybindings: false,
            image_fit: false,
            text_is_big: false,
            text_line_count: 0,
            text_is_lossy: false,
            open_text_tabs: Vec::new(),
            active_text_tab: None,
            open_image_tabs: Vec::new(),
            active_image_tab: None,
            search_query: String::new(),
            search_active: false,
            search_count: 0,
            search_current: 0,
            scroll_target_line: None,
            show_recent_window: false,
            show_global_search_window: false,
            global_query: String::new(),
            global_case_sensitive: false,
            global_whole_word: false,
            global_regex: false,
            global_results: Vec::new(),
            global_error: None,
            restore_session: false,
            session_paths: Vec::new(),
            session_active: None,
            session_restored: false,
            file_open_rx: None,
            file_open_in_flight: false,
            viewport_initialized: false,
        }
    }
}

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
