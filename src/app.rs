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
    pub(crate) global_results: Vec<GlobalSearchResult>,
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
            app.search_query = String::new();
            app.search_active = false;
            app.search_count = 0;
            app.scroll_target_line = None;
            app.show_recent_window = false;
            app.show_global_search_window = false;
            app.global_query = String::new();
            app.global_case_sensitive = false;
            app.global_whole_word = false;
            app.global_results = Vec::new();
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
            app.search_query = String::new();
            app.search_active = false;
            app.search_count = 0;
            app.scroll_target_line = None;
            app.show_recent_window = false;
            app.show_global_search_window = false;
            app.global_query = String::new();
            app.global_case_sensitive = false;
            app.global_whole_word = false;
            app.global_results = Vec::new();
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
                // Persist updated recents immediately
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
        }
    }

    pub(crate) fn recompute_global_search(&mut self) {
        self.global_results.clear();
        let q = self.global_query.clone();
        if q.is_empty() { return; }
        for (tab_idx, tab) in self.open_text_tabs.iter().enumerate() {
            let mut match_counter_in_tab: usize = 0;
            for (line_idx, line) in tab.text.lines().enumerate() {
                let mut hay = line.to_string();
                let mut needle = q.clone();
                if !self.global_case_sensitive {
                    hay = hay.to_ascii_lowercase();
                    needle = needle.to_ascii_lowercase();
                }
                let mut offset_in_line: usize = 0;
                let original_line = line;
                while let Some(pos) = hay[offset_in_line..].find(&needle) {
                    let abs_pos = offset_in_line + pos;
                    // Whole-word check
                    if self.global_whole_word {
                        let left_ok = abs_pos == 0 || !original_line.chars().nth(abs_pos - 1).map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false);
                        let right_index = abs_pos + q.len();
                        let right_ok = right_index >= original_line.len() || !original_line.chars().nth(right_index).map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false);
                        if !(left_ok && right_ok) {
                            offset_in_line = abs_pos + needle.len();
                            continue;
                        }
                    }
                    // Create a small snippet around the match
                    let start = abs_pos.saturating_sub(40);
                    let end = (abs_pos + q.len()).saturating_add(40).min(original_line.len());
                    let snippet = original_line[start..end].to_string();
                    self.global_results.push(GlobalSearchResult {
                        tab_index: tab_idx,
                        path: tab.path.clone(),
                        line_index: line_idx,
                        snippet,
                        match_index_in_tab: match_counter_in_tab,
                    });
                    match_counter_in_tab += 1;
                    offset_in_line = abs_pos + needle.len();
                    if offset_in_line >= hay.len() { break; }
                }
            }
        }
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
            global_results: Vec::new(),
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

        let mut file_to_load: Option<PathBuf> = None;

        // Keyboard + mouse input (delegated)
        let toggle_dark = crate::input::handle_input(self, ctx, &mut file_to_load);

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
            egui::Window::new("About Gemini File Viewer")
                .collapsible(false)
                .resizable(false)
                .open(&mut self.show_about)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("📁").size(48.0));
                        ui.add_space(12.0);
                        ui.label(RichText::new("Gemini File Viewer").heading().strong());
                        ui.label(RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION"))).weak());
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
                    });
                });
        }
        if self.show_settings_window {
            let mut dark = self.dark_mode;
            let mut lines = self.show_line_numbers;
            let mut syn = self.use_syntect;
            let mut open = self.show_settings_window;
            egui::Window::new("Settings")
                .collapsible(false)
                .resizable(true)
                .min_width(520.0)
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label(RichText::new("🎨 Display Settings").strong());
                    ui.add_space(8.0);
                    ui.checkbox(&mut dark, RichText::new("🌙 Dark Mode").strong());
                    ui.checkbox(&mut lines, RichText::new("📊 Line Numbers").strong());
                    ui.checkbox(&mut syn, RichText::new("🎨 Syntect Highlighting").strong());
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.label(RichText::new("ℹ️ About").strong());
                    ui.add_space(8.0);
                    ui.label(RichText::new("Gemini File Viewer").weak());
                    ui.label(RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION"))).weak());
                    ui.add_space(4.0);
                    ui.label(RichText::new("Disclaimer: This software is provided “as is” without warranty of any kind, whether express, implied, or statutory, including but not limited to warranties of merchantability, fitness for a particular purpose, and noninfringement. To the maximum extent permitted by law, the authors shall not be liable for any claim, damages, or other liability, whether in contract, tort, or otherwise, arising from or in connection with the software or its use.").small());
                    ui.label(RichText::new("Authors: David Queen, Allison Bayless").small());
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);
                    if ui.button(RichText::new("⌨️ Show Keybindings").strong()).clicked() {
                        self.show_keybindings = true;
                    }
                });
            // Apply after window closure to avoid borrow conflict
            if self.dark_mode != dark { self.dark_mode = dark; self.apply_theme(ctx); }
            if self.show_line_numbers != lines || self.use_syntect != syn || self.dark_mode != dark {
                self.show_line_numbers = lines;
                self.use_syntect = syn;
                crate::settings::save_settings_to_disk(self);
            }
            self.show_settings_window = open;
        }
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
