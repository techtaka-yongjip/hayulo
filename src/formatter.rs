#[derive(Clone, Debug)]
pub struct FormatResult {
    pub source: String,
    pub changed: bool,
}

pub fn check_format(source: &str) -> FormatResult {
    let formatted = format_source(source);
    FormatResult {
        changed: formatted != source,
        source: formatted,
    }
}

pub fn format_source(source: &str) -> String {
    let mut lines = Vec::new();
    let mut indent = 0usize;
    let mut previous_blank = false;

    for raw in source.lines() {
        let stripped = raw.trim();
        if stripped.is_empty() {
            if !lines.is_empty() && !previous_blank {
                lines.push(String::new());
            }
            previous_blank = true;
            continue;
        }

        let (opens, closes) = delimiter_counts(stripped);
        let leading = leading_closes(stripped);
        if leading > 0 {
            indent = indent.saturating_sub(leading);
        }
        lines.push(format!("{}{}", "  ".repeat(indent), stripped));
        indent = (indent + opens).saturating_sub(closes.saturating_sub(leading));
        previous_blank = false;
    }

    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }

    format!("{}\n", lines.join("\n"))
}

fn leading_closes(line: &str) -> usize {
    let mut count = 0;
    for ch in line.chars() {
        if ch == '}' || ch == ']' {
            count += 1;
            continue;
        }
        if ch.is_whitespace() {
            continue;
        }
        break;
    }
    count
}

fn delimiter_counts(line: &str) -> (usize, usize) {
    let text = strip_comment(line);
    let mut in_string = false;
    let mut escaped = false;
    let mut opens = 0usize;
    let mut closes = 0usize;
    for ch in text.chars() {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            in_string = !in_string;
        } else if !in_string && (ch == '{' || ch == '[') {
            opens += 1;
        } else if !in_string && (ch == '}' || ch == ']') {
            closes += 1;
        }
    }
    (opens, closes)
}

pub fn strip_comment(line: &str) -> String {
    let mut in_string = false;
    let mut escaped = false;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        let ch = chars[i];
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            in_string = !in_string;
        } else if !in_string && ch == '/' && chars.get(i + 1) == Some(&'/') {
            return chars[..i].iter().collect();
        }
        i += 1;
    }
    line.to_string()
}
