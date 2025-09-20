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

// Submodules housing larger method/trait impls to keep this file lean
mod update;
mod ops;

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
        // Persist the last directory used for opening files
        pub last_open_dir: Option<PathBuf>,
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
            // Derive a sensible default for last_open_dir if missing
            if app.last_open_dir.is_none() {
                let cand = app
                    .recent_files
                    .last()
                    .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                    .or_else(|| app
                        .session_paths
                        .get(app.session_active.unwrap_or(0).min(app.session_paths.len().saturating_sub(1)))
                        .and_then(|p| p.parent().map(|d| d.to_path_buf())));
                if let Some(dir) = cand.filter(|d| d.is_dir()) { app.last_open_dir = Some(dir); }
            }
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
            // Derive a sensible default for last_open_dir if missing
            if app.last_open_dir.is_none() {
                let cand = app
                    .recent_files
                    .last()
                    .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                    .or_else(|| app
                        .session_paths
                        .get(app.session_active.unwrap_or(0).min(app.session_paths.len().saturating_sub(1)))
                        .and_then(|p| p.parent().map(|d| d.to_path_buf())));
                if let Some(dir) = cand.filter(|d| d.is_dir()) { app.last_open_dir = Some(dir); }
            }
            return app;
        }
        Default::default()
    }

    pub(crate) fn apply_theme(&self, ctx: &egui::Context) {
        crate::style::apply_theme(self, ctx);
    }

    // io helpers moved to crate::io

    // Methods moved to app::ops submodule to keep this file focused on types/new/default.

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
            last_open_dir: None,
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

// Moved eframe::App impl (save/update) into app::update submodule to reduce file size.
