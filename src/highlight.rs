use eframe::egui::{self, text::LayoutJob, Color32, FontId};
use crate::themes::CodeTheme;

pub struct HighlightContext<'a> {
    pub ext: &'a str,
    pub font_id: FontId,
    pub base_color: Color32,
    pub do_syntax: bool,
    pub depth: &'a mut i32,
    pub current_idx: usize,
    pub counter: &'a mut usize,
    pub query: &'a str,
    pub in_block_comment: &'a mut bool,
    pub theme: CodeTheme,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn append_with_search(
    job: &mut LayoutJob,
    text: &str,
    color: Color32,
    ctx: &mut HighlightContext,
) {
    if ctx.query.is_empty() {
        job.append(text, 0.0, egui::TextFormat { font_id: ctx.font_id.clone(), color, ..Default::default() });
        return;
    }
    let lc_query = ctx.query.to_ascii_lowercase();
    let mut rest = text;
    loop {
        let lc_rest = rest.to_ascii_lowercase();
        if let Some(found_rel) = lc_rest.find(&lc_query) {
            let prefix = &rest[..found_rel];
            if !prefix.is_empty() {
                job.append(prefix, 0.0, egui::TextFormat { font_id: ctx.font_id.clone(), color, ..Default::default() });
            }
            let matched = &rest[found_rel..found_rel + lc_query.len()];
            let mut fmt = egui::TextFormat { font_id: ctx.font_id.clone(), color, ..Default::default() };
            if *ctx.counter == ctx.current_idx {
                fmt.background = ctx.theme.search_current();
            } else {
                fmt.background = ctx.theme.search_highlight();
            }
            job.append(matched, 0.0, fmt);
            *ctx.counter += 1;
            rest = &rest[found_rel + lc_query.len()..];
            if rest.is_empty() { break; }
        } else {
            if !rest.is_empty() {
                job.append(rest, 0.0, egui::TextFormat { font_id: ctx.font_id.clone(), color, ..Default::default() });
            }
            break;
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn token_highlight(
    job: &mut LayoutJob,
    text: &str,
    ctx: &mut HighlightContext,
) {
    if !ctx.do_syntax {
        append_with_search(job, text, ctx.base_color, ctx);
        return;
    }
    let kw_color = ctx.theme.keyword();
    let num_color = ctx.theme.number();
    let bool_color = ctx.theme.keyword();
    let bracket_colors = ctx.theme.bracket_colors();

    let keywords_rs: &[&str] = &[
        "as","async","await","break","const","continue","crate","dyn","else","enum","extern","false","fn","for","if","impl","in","let","loop","match","mod","move","mut","pub","ref","return","self","Self","static","struct","super","trait","true","type","unsafe","use","where","while",
        "union","box","try","yield","macro","macro_rules"
    ];
    let keywords_py: &[&str] = &[
        "False","None","True","and","as","assert","async","await","break","class","continue","def","del","elif","else","except","finally","for","from","global","if","import","in","is","lambda","nonlocal","not","or","pass","raise","return","try","while","with","yield","match","case"
    ];

    let mut buf = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            buf.push(ch);
        } else {
            if !buf.is_empty() {
                let lc = buf.to_ascii_lowercase();
                let (color, _) = if (ctx.ext == "rs" && keywords_rs.contains(&buf.as_str()))
                    || (ctx.ext == "py" && keywords_py.contains(&buf.as_str())) {
                    (kw_color, true)
                } else if lc == "true" || lc == "false" || lc == "null" || lc == "none" {
                    (bool_color, true)
                } else if buf.chars().all(|c| c.is_ascii_digit()) {
                    (num_color, true)
                } else {
                    (ctx.base_color, false)
                };
                append_with_search(job, &buf, color, ctx);
                buf.clear();
            }
            let color = match ch {
                '(' | '[' | '{' => {
                    let idx = ((*ctx.depth).max(0) as usize) % bracket_colors.len();
                    *ctx.depth = ctx.depth.saturating_add(1);
                    Some(bracket_colors[idx])
                }
                ')' | ']' | '}' => {
                    *ctx.depth = ctx.depth.saturating_sub(1);
                    let idx = ((*ctx.depth).max(0) as usize) % bracket_colors.len();
                    Some(bracket_colors[idx])
                }
                _ => None,
            };
            let delim = ch.to_string();
            append_with_search(job, &delim, color.unwrap_or(ctx.base_color), ctx);
        }
    }
    if !buf.is_empty() {
        let lc = buf.to_ascii_lowercase();
        let (color, _) = if (ctx.ext == "rs" && keywords_rs.contains(&buf.as_str()))
            || (ctx.ext == "py" && keywords_py.contains(&buf.as_str())) {
            (kw_color, true)
        } else if lc == "true" || lc == "false" || lc == "null" || lc == "none" {
            (bool_color, true)
        } else if buf.chars().all(|c| c.is_ascii_digit()) {
            (num_color, true)
        } else {
            (ctx.base_color, false)
        };
        append_with_search(job, &buf, color, ctx);
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn append_highlighted(
    job: &mut LayoutJob,
    line: &str,
    ctx: &mut HighlightContext,
) {
    if ctx.do_syntax {
        if ctx.ext == "rs" {
            let mut i = 0usize;
            if *ctx.in_block_comment {
                if let Some(end) = line[i..].find("*/") {
                    let end_abs = i + end + 2;
                    let fmt = egui::TextFormat { font_id: ctx.font_id.clone(), color: ctx.theme.comment(), ..Default::default() };
                    job.append(&line[i..end_abs], 0.0, fmt);
                    *ctx.in_block_comment = false;
                    i = end_abs;
                } else {
                    let fmt = egui::TextFormat { font_id: ctx.font_id.clone(), color: ctx.theme.comment(), ..Default::default() };
                    job.append(&line[i..], 0.0, fmt);
                    return;
                }
            }
            while i < line.len() {
                let rest = &line[i..];
                let pos_sl = rest.find("//");
                let pos_blk = rest.find("/*");
                match (pos_sl, pos_blk) {
                    (Some(psl), Some(pblk)) if psl < pblk => {
                        if psl > 0 {
                            token_highlight(job, &rest[..psl], ctx);
                        }
                        let fmt = egui::TextFormat { font_id: ctx.font_id.clone(), color: ctx.theme.comment(), ..Default::default() };
                        job.append(&rest[psl..], 0.0, fmt);
                        return;
                    }
                    (Some(psl), None) => {
                        if psl > 0 {
                            token_highlight(job, &rest[..psl], ctx);
                        }
                        let fmt = egui::TextFormat { font_id: ctx.font_id.clone(), color: ctx.theme.comment(), ..Default::default() };
                        job.append(&rest[psl..], 0.0, fmt);
                        return;
                    }
                    (None, Some(pblk)) => {
                        if pblk > 0 {
                            token_highlight(job, &rest[..pblk], ctx);
                        }
                        let after = pblk + 2;
                        let tail = &rest[after..];
                        if let Some(end) = tail.find("*/") {
                            let end_abs = i + after + end + 2;
                            let fmt = egui::TextFormat { font_id: ctx.font_id.clone(), color: ctx.theme.comment(), ..Default::default() };
                            job.append(&line[i + pblk..end_abs], 0.0, fmt);
                            i = end_abs;
                            continue;
                        } else {
                            let fmt = egui::TextFormat { font_id: ctx.font_id.clone(), color: ctx.theme.comment(), ..Default::default() };
                            job.append(&rest[pblk..], 0.0, fmt);
                            *ctx.in_block_comment = true;
                            return;
                        }
                    }
                    (None, None) => {
                        token_highlight(job, rest, ctx);
                        return;
                    }
                    (Some(_psl), Some(pblk)) => {
                        if pblk > 0 {
                            token_highlight(job, &rest[..pblk], ctx);
                        }
                        let after = pblk + 2;
                        let tail = &rest[after..];
                        if let Some(end) = tail.find("*/") {
                            let end_abs = i + after + end + 2;
                            let fmt = egui::TextFormat { font_id: ctx.font_id.clone(), color: ctx.theme.comment(), ..Default::default() };
                            job.append(&line[i + pblk..end_abs], 0.0, fmt);
                            i = end_abs;
                            continue;
                        } else {
                            let fmt = egui::TextFormat { font_id: ctx.font_id.clone(), color: ctx.theme.comment(), ..Default::default() };
                            job.append(&rest[pblk..], 0.0, fmt);
                            *ctx.in_block_comment = true;
                            return;
                        }
                    }
                }
            }
            return;
        }
        let comment_prefix = if ctx.ext == "rs" || ctx.ext == "js" { "//" } else if ctx.ext == "toml" { "#" } else { "" };
        let comment_prefix = if ctx.ext == "py" { "#" } else { comment_prefix };
        if !comment_prefix.is_empty() && let Some(pos) = line.find(comment_prefix) {
            append_highlighted(job, &line[..pos], ctx);
            let fmt = egui::TextFormat { font_id: ctx.font_id.clone(), color: ctx.theme.comment(), ..Default::default() };
            job.append(&line[pos..], 0.0, fmt);
            return;
        }
    }

    let mut buf = String::new();

    if ctx.do_syntax {
        let mut chars = line.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '"' {
                if !buf.is_empty() { token_highlight(job, &buf, ctx); buf.clear(); }
                buf.clear();
                let mut s = String::from('"');
                for c2 in chars.by_ref() {
                    s.push(c2);
                    if c2 == '"' { break; }
                }
                append_with_search(job, &s, ctx.theme.string(), ctx);
            } else {
                buf.push(ch);
            }
        }
    } else {
        buf.push_str(line);
    }

    if !buf.is_empty() {
        token_highlight(job, &buf, ctx);
    }
}
