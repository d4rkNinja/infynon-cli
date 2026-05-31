fn json_highlight_line(line: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Whitespace
        if c == ' ' || c == '\t' {
            let start = i;
            while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t') {
                i += 1;
            }
            spans.push(Span::styled(
                chars[start..i].iter().collect::<String>(),
                Style::default(),
            ));
            continue;
        }

        // Punctuation: { } [ ] : ,
        if c == '{' || c == '}' || c == '[' || c == ']' || c == ':' || c == ',' {
            spans.push(Span::styled(c.to_string(), Style::default().fg(DIM)));
            i += 1;
            continue;
        }

        // String
        if c == '"' {
            let start = i;
            i += 1; // skip opening quote
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' {
                    i += 1; // skip escaped char
                }
                i += 1;
            }
            if i < chars.len() {
                i += 1; // skip closing quote
            }
            let s: String = chars[start..i].iter().collect();
            // Determine if this is a key (followed by ':') or a value
            let mut lookahead = i;
            while lookahead < chars.len() && (chars[lookahead] == ' ' || chars[lookahead] == '\t') {
                lookahead += 1;
            }
            if lookahead < chars.len() && chars[lookahead] == ':' {
                // Key
                spans.push(Span::styled(s, Style::default().fg(CYAN)));
            } else {
                // String value
                spans.push(Span::styled(s, Style::default().fg(GREEN)));
            }
            continue;
        }

        // Number
        if c == '-' || c.is_ascii_digit() {
            let start = i;
            if c == '-' {
                i += 1;
            }
            while i < chars.len()
                && (chars[i].is_ascii_digit()
                    || chars[i] == '.'
                    || chars[i] == 'e'
                    || chars[i] == 'E'
                    || chars[i] == '+'
                    || chars[i] == '-')
            {
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            spans.push(Span::styled(s, Style::default().fg(YELLOW)));
            continue;
        }

        // Boolean / null keywords
        if c == 't' || c == 'f' || c == 'n' {
            let keyword_len = if c == 't' && i + 3 < chars.len() {
                let slice: String = chars[i..i + 4].iter().collect();
                if slice == "true" {
                    Some(4)
                } else {
                    None
                }
            } else if c == 'f' && i + 4 < chars.len() {
                let slice: String = chars[i..i + 5].iter().collect();
                if slice == "false" {
                    Some(5)
                } else {
                    None
                }
            } else if c == 'n' && i + 3 < chars.len() {
                let slice: String = chars[i..i + 4].iter().collect();
                if slice == "null" {
                    Some(4)
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(len) = keyword_len {
                let s: String = chars[i..i + len].iter().collect();
                spans.push(Span::styled(s, Style::default().fg(ORANGE)));
                i += len;
                continue;
            }
        }

        // Fallback: single character
        spans.push(Span::styled(c.to_string(), Style::default().fg(TEXT_DIM)));
        i += 1;
    }

    spans
}

// ── Prompt modal ──────────────────────────────────────────────────────────────
