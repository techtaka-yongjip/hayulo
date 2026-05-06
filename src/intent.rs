use std::collections::BTreeMap;

use regex::Regex;
use serde_json::{Value, json};

use crate::diagnostics::{Diagnostic, HayuloError, HayuloResult};
use crate::formatter::strip_comment;

pub fn parse_top_level_intent(source: &str, filename: Option<&str>) -> HayuloResult<Option<Value>> {
    let lines: Vec<&str> = source.lines().collect();
    let mut intent = BTreeMap::new();
    let mut depth = 0isize;
    let mut i = 0usize;
    let re = Regex::new(r"^\s*intent\s*\{").unwrap();

    while i < lines.len() {
        let line = strip_comment(lines[i]);
        if depth == 0 && re.is_match(&line) {
            let (block, next) = collect_block(&lines, i, filename, "intent")?;
            for (key, value) in parse_fields(&block, filename)? {
                intent.insert(key, value);
            }
            i = next;
            continue;
        }
        depth = (depth + count_braces(&line)).max(0);
        i += 1;
    }

    if intent.is_empty() {
        Ok(None)
    } else {
        Ok(Some(json!(intent)))
    }
}

fn collect_block(
    lines: &[&str],
    start: usize,
    filename: Option<&str>,
    label: &str,
) -> HayuloResult<(Vec<(usize, String)>, usize)> {
    let head = strip_comment(lines[start]);
    if !head.contains('{') {
        return Err(HayuloError::new(
            Diagnostic::new("missing_block", format!("Expected '{{' for {label} block."))
                .file(filename)
                .line(Some(start + 1)),
        ));
    }
    let after_open = head.split_once('{').map(|(_, right)| right).unwrap_or("");
    let mut depth = count_braces(&head);
    let mut block = Vec::new();
    if depth <= 0 {
        let before_close = after_open
            .rsplit_once('}')
            .map(|(left, _)| left)
            .unwrap_or(after_open)
            .trim();
        if !before_close.is_empty() {
            block.push((start + 1, before_close.to_string()));
        }
        return Ok((block, start + 1));
    }
    if !after_open.trim().is_empty() {
        block.push((start + 1, after_open.to_string()));
    }
    let mut i = start + 1;
    while i < lines.len() {
        let delta = count_braces(lines[i]);
        if depth + delta <= 0 {
            let before_close = lines[i]
                .rsplit_once('}')
                .map(|(left, _)| left)
                .unwrap_or(lines[i])
                .trim();
            if !before_close.is_empty() {
                block.push((i + 1, before_close.to_string()));
            }
            return Ok((block, i + 1));
        }
        block.push((i + 1, lines[i].to_string()));
        depth += delta;
        i += 1;
    }
    Err(HayuloError::new(
        Diagnostic::new("unterminated_block", format!("Unterminated {label} block."))
            .file(filename)
            .line(Some(start + 1))
            .suggestion("Add a closing '}'."),
    ))
}

fn parse_fields(
    block: &[(usize, String)],
    filename: Option<&str>,
) -> HayuloResult<Vec<(String, Value)>> {
    let mut values = Vec::new();
    let mut i = 0usize;
    let field_re = Regex::new(r"^([A-Za-z_]\w*)\s*:\s*(.+)$").unwrap();
    while i < block.len() {
        let (line_no, raw) = &block[i];
        let line = clean_line(raw);
        if line.is_empty() {
            i += 1;
            continue;
        }
        let captures = field_re.captures(&line).ok_or_else(|| {
            HayuloError::new(
                Diagnostic::new(
                    "invalid_intent_field",
                    format!("Invalid intent field: {line:?}."),
                )
                .file(filename)
                .line(Some(*line_no))
                .suggestion("Use: key: \"value\" or key: [\"value\"]."),
            )
        })?;
        let key = captures[1].to_string();
        let rhs = captures[2].trim();
        if rhs.starts_with('"') {
            values.push((key, json!(parse_quoted_scalar(rhs, filename, *line_no)?)));
            i += 1;
            continue;
        }
        if rhs.starts_with('[') {
            let mut parts = vec![rhs.to_string()];
            let mut bracket_depth = count_brackets(rhs);
            i += 1;
            let mut last_line = *line_no;
            while bracket_depth > 0 && i < block.len() {
                let (next_line_no, next_raw) = &block[i];
                let next_line = clean_line(next_raw);
                parts.push(next_line.clone());
                bracket_depth += count_brackets(&next_line);
                last_line = *next_line_no;
                i += 1;
            }
            if bracket_depth != 0 {
                return Err(HayuloError::new(
                    Diagnostic::new(
                        "unterminated_intent_list",
                        format!("Unterminated intent list for {key:?}."),
                    )
                    .file(filename)
                    .line(Some(last_line))
                    .suggestion("Add a closing ']'."),
                ));
            }
            values.push((
                key,
                json!(parse_string_list(&parts.join(" "), filename, last_line)?),
            ));
            continue;
        }
        return Err(HayuloError::new(
            Diagnostic::new(
                "invalid_intent_value",
                format!("Invalid intent value for {key:?}."),
            )
            .file(filename)
            .line(Some(*line_no))
            .suggestion("Use a string or list of strings."),
        ));
    }
    Ok(values)
}

fn parse_quoted_scalar(text: &str, filename: Option<&str>, line: usize) -> HayuloResult<String> {
    let (value, index) = consume_quoted(text, 0, filename, line)?;
    let trailing = text[index..].trim();
    if !trailing.is_empty() && trailing != "," {
        return Err(HayuloError::new(
            Diagnostic::new(
                "invalid_intent_value",
                "Intent string values cannot contain trailing tokens.",
            )
            .file(filename)
            .line(Some(line))
            .suggestion("Use a single quoted string value."),
        ));
    }
    Ok(value)
}

fn parse_string_list(text: &str, filename: Option<&str>, line: usize) -> HayuloResult<Vec<String>> {
    let mut text = text.trim().to_string();
    if text.ends_with("],") {
        text.pop();
        text = text.trim_end().to_string();
    }
    if !text.starts_with('[') || !text.ends_with(']') {
        return Err(HayuloError::new(
            Diagnostic::new(
                "invalid_intent_value",
                "Intent list values must start with '[' and end with ']'.",
            )
            .file(filename)
            .line(Some(line))
            .suggestion("Use a list of quoted strings."),
        ));
    }
    let mut values = Vec::new();
    let mut index = 1usize;
    let end = text.len() - 1;
    while index < end {
        while index < end && text.as_bytes()[index].is_ascii_whitespace() {
            index += 1;
        }
        if index < end && text.as_bytes()[index] == b',' {
            index += 1;
            continue;
        }
        if index >= end {
            break;
        }
        if text.as_bytes()[index] != b'"' {
            return Err(HayuloError::new(
                Diagnostic::new(
                    "invalid_intent_value",
                    "Intent lists can only contain strings.",
                )
                .file(filename)
                .line(Some(line))
                .suggestion("Use quoted strings inside intent lists."),
            ));
        }
        let (value, next) = consume_quoted(&text, index, filename, line)?;
        values.push(value);
        index = next;
        while index < end && text.as_bytes()[index].is_ascii_whitespace() {
            index += 1;
        }
        if index < end && text.as_bytes()[index] == b',' {
            index += 1;
            continue;
        }
        if index < end {
            return Err(HayuloError::new(
                Diagnostic::new(
                    "invalid_intent_value",
                    "Intent list items must be separated by commas.",
                )
                .file(filename)
                .line(Some(line))
                .suggestion("Add a comma between list items."),
            ));
        }
    }
    Ok(values)
}

fn consume_quoted(
    text: &str,
    start: usize,
    filename: Option<&str>,
    line: usize,
) -> HayuloResult<(String, usize)> {
    let chars: Vec<char> = text.chars().collect();
    if chars.get(start) != Some(&'"') {
        return Err(HayuloError::new(
            Diagnostic::new("invalid_intent_value", "Expected quoted string.")
                .file(filename)
                .line(Some(line))
                .suggestion("Use double quotes for intent strings."),
        ));
    }
    let mut value = String::new();
    let mut i = start + 1;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '"' {
            let byte_index = chars[..=i].iter().map(|c| c.len_utf8()).sum();
            return Ok((value, byte_index));
        }
        if ch == '\\' {
            i += 1;
            if i >= chars.len() {
                break;
            }
            value.push(match chars[i] {
                'n' => '\n',
                't' => '\t',
                '"' => '"',
                '\\' => '\\',
                other => other,
            });
        } else {
            value.push(ch);
        }
        i += 1;
    }
    Err(HayuloError::new(
        Diagnostic::new("unterminated_string", "Unterminated intent string.")
            .file(filename)
            .line(Some(line))
            .suggestion("Add a closing double quote."),
    ))
}

fn clean_line(line: &str) -> String {
    strip_comment(line).trim().to_string()
}

fn count_braces(line: &str) -> isize {
    count_unquoted(line, '{', '}')
}

fn count_brackets(line: &str) -> isize {
    count_unquoted(line, '[', ']')
}

fn count_unquoted(line: &str, open: char, close: char) -> isize {
    let text = strip_comment(line);
    let mut in_string = false;
    let mut escaped = false;
    let mut count = 0isize;
    for ch in text.chars() {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            in_string = !in_string;
        } else if !in_string && ch == open {
            count += 1;
        } else if !in_string && ch == close {
            count -= 1;
        }
    }
    count
}
