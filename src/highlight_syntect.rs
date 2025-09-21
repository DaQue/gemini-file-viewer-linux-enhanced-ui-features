use eframe::egui::{self, Color32, FontId, text::LayoutJob};
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, Style as SynStyle, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};

struct SyntectEngine {
    ss: SyntaxSet,
    ts: ThemeSet,
}

fn engine() -> &'static SyntectEngine {
    static ENG: OnceLock<SyntectEngine> = OnceLock::new();
    ENG.get_or_init(|| SyntectEngine {
        ss: SyntaxSet::load_defaults_newlines(),
        ts: ThemeSet::load_defaults(),
    })
}

fn choose_theme(ts: &ThemeSet, dark_mode: bool) -> Option<&Theme> {
    // Prefer popular defaults; fallback to first available; avoid panics
    let dark_name = "base16-ocean.dark";
    let light_name = "InspiredGitHub";
    if dark_mode {
        ts.themes.get(dark_name).or_else(|| ts.themes.values().next())
    } else {
        ts.themes.get(light_name).or_else(|| ts.themes.values().next())
    }
}

fn fallback_theme() -> &'static Theme {
    static FALLBACK: OnceLock<Theme> = OnceLock::new();
    FALLBACK.get_or_init(|| {
        ThemeSet::load_defaults()
            .themes
            .values()
            .next()
            .cloned()
            .unwrap_or_else(|| {
                let mut theme = Theme::default();
                theme
                    .settings
                    .foreground
                    .get_or_insert(Color { r: 255, g: 255, b: 255, a: 255 });
                theme
                    .settings
                    .background
                    .get_or_insert(Color { r: 0, g: 0, b: 0, a: 255 });
                theme
            })
    })
}

fn syntax_for_ext<'a>(ss: &'a SyntaxSet, ext: &str) -> &'a SyntaxReference {
    if ext.is_empty() {
        return ss.find_syntax_plain_text();
    }
    ss.find_syntax_by_extension(ext)
        .unwrap_or_else(|| ss.find_syntax_plain_text())
}

pub struct SyntectSession<'a> {
    high: HighlightLines<'a>,
}

impl<'a> SyntectSession<'a> {
    pub fn start(ext: &str, dark_mode: bool) -> SyntectSession<'static> {
        let eng = engine();
        let syn = syntax_for_ext(&eng.ss, ext);
        let theme = choose_theme(&eng.ts, dark_mode)
            .unwrap_or_else(|| {
                // Fall back to a simple built-in theme to avoid panics when the ThemeSet is empty
                fallback_theme()
            });
        SyntectSession {
            high: HighlightLines::new(syn, theme),
        }
    }

    pub fn append_line(&mut self, job: &mut LayoutJob, line: &str, font_id: FontId) {
        let eng = engine();
        match self.high.highlight_line(line, &eng.ss) {
            Ok(spans) => {
                for (style, text) in spans {
                    let color = to_egui_color(style);
                    job.append(
                        text,
                        0.0,
                        egui::TextFormat {
                            font_id: font_id.clone(),
                            color,
                            ..Default::default()
                        },
                    );
                }
            }
            Err(_) => {
                // Fallback: append plain text
                job.append(
                    line,
                    0.0,
                    egui::TextFormat {
                        font_id,
                        color: Color32::WHITE,
                        ..Default::default()
                    },
                );
            }
        }
    }
}

fn to_egui_color(style: SynStyle) -> Color32 {
    let c = style.foreground;
    Color32::from_rgb(c.r, c.g, c.b)
}
