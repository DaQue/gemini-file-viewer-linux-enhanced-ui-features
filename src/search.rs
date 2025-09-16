pub(crate) fn recompute_count(query: &str, text: &str) -> usize {
    if query.is_empty() { return 0; }
    let q = query.to_ascii_lowercase();
    text.to_ascii_lowercase().matches(&q).count()
}

pub(crate) fn find_target_line(text: &str, query: &str, target_idx: usize) -> Option<usize> {
    if query.is_empty() { return None; }
    let q = query.to_ascii_lowercase();
    let mut global = 0usize;
    for (i, line) in text.lines().enumerate() {
        let mut rest = line.to_ascii_lowercase();
        while let Some(pos) = rest.find(&q) {
            if global == target_idx { return Some(i); }
            global += 1;
            let next = pos + q.len();
            if next >= rest.len() { break; }
            rest = rest[next..].to_string();
        }
    }
    None
}

pub(crate) fn global_search(
    open_text_tabs: &Vec<crate::app::TextTab>,
    query: &str,
    case_sensitive: bool,
    whole_word: bool,
    regex_mode: bool,
) -> Result<Vec<crate::app::GlobalSearchResult>, String> {
    let mut results: Vec<crate::app::GlobalSearchResult> = Vec::new();
    if query.is_empty() { return Ok(results); }

    let regex_opt = if regex_mode {
        match regex::RegexBuilder::new(query)
            .case_insensitive(!case_sensitive)
            .build() {
            Ok(r) => Some(r),
            Err(e) => return Err(format!("Regex error: {}", e)),
        }
    } else { None };

    for (tab_idx, tab) in open_text_tabs.iter().enumerate() {
        let mut match_counter_in_tab: usize = 0;
        for (line_idx, line) in tab.text.lines().enumerate() {
            if let Some(re) = &regex_opt {
                for m in re.find_iter(line) {
                    let start = m.start().saturating_sub(40);
                    let end = m.end().saturating_add(40).min(line.len());
                    let snippet = line[start..end].to_string();
                    results.push(crate::app::GlobalSearchResult {
                        tab_index: tab_idx,
                        path: tab.path.clone(),
                        line_index: line_idx,
                        snippet,
                        match_index_in_tab: match_counter_in_tab,
                    });
                    match_counter_in_tab += 1;
                }
            } else {
                let mut hay = line.to_string();
                let mut needle = query.to_string();
                if !case_sensitive {
                    hay = hay.to_ascii_lowercase();
                    needle = needle.to_ascii_lowercase();
                }
                let mut offset_in_line: usize = 0;
                let original_line = line;
                while let Some(pos) = hay[offset_in_line..].find(&needle) {
                    let abs_pos = offset_in_line + pos;
                    if whole_word {
                        let left_ok = abs_pos == 0 || !original_line.chars().nth(abs_pos - 1).map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false);
                        let right_index = abs_pos + query.len();
                        let right_ok = right_index >= original_line.len() || !original_line.chars().nth(right_index).map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false);
                        if !(left_ok && right_ok) {
                            offset_in_line = abs_pos + needle.len();
                            continue;
                        }
                    }
                    let start = abs_pos.saturating_sub(40);
                    let end = (abs_pos + query.len()).saturating_add(40).min(original_line.len());
                    let snippet = original_line[start..end].to_string();
                    results.push(crate::app::GlobalSearchResult {
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

    Ok(results)
}

