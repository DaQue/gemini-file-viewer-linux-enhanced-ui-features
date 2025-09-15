use eframe::egui::{self, text::LayoutJob, Color32, FontId};

// Minimal scaffolding for syntect integration (implementation TODO)
pub struct SyntectHighlighter;

impl SyntectHighlighter {
    pub fn new() -> Self { Self }

    pub fn append_line(
        &self,
        job: &mut LayoutJob,
        line: &str,
        _ext: &str,
        font_id: FontId,
        base_color: Color32,
    ) {
        // Placeholder: just append plain text for now; real impl will colorize tokens
        job.append(line, 0.0, egui::TextFormat { font_id, color: base_color, ..Default::default() });
    }
}
