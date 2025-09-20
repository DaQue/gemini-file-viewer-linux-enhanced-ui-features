use super::*;

impl FileViewerApp {
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
                // Remember the directory of the last opened file
                if let Some(parent) = path.parent() { self.last_open_dir = Some(parent.to_path_buf()); }
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
        let last_dir = self.current_path.as_ref().and_then(|p| p.parent()).map(|d| d.to_path_buf()).or_else(|| self.last_open_dir.clone());
        std::thread::spawn(move || {
            let mut dlg = FileDialog::new()
                .add_filter("All Supported", &["txt","rs","py","toml","md","json","js","html","css","png","jpg","jpeg","gif","bmp","webp"])
                .add_filter("Images", &["png","jpg","jpeg","gif","bmp","webp"])
                .add_filter("Text/Source", &["txt","rs","py","toml","md","json","js","html","css"]);
            if let Some(dir) = last_dir { dlg = dlg.set_directory(dir); }
            let picked = dlg.pick_file();
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
}

