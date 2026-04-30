from __future__ import annotations

import re
from typing import Any

from .diagnostics import Diagnostic, HayuloSyntaxError


def parse_top_level_intent(source: str, filename: str | None = None) -> dict[str, Any] | None:
    lines = source.splitlines()
    intent: dict[str, Any] | None = None
    depth = 0
    i = 0

    while i < len(lines):
        line = _strip_comment(lines[i])
        if depth == 0 and re.match(r"\s*intent\s*\{", line):
            block, i = _collect_block(lines, i, filename, "intent")
            values = _parse_fields(block, filename)
            if intent is None:
                intent = {}
            intent.update(values)
            continue

        depth = max(0, depth + _count_braces(line))
        i += 1

    return intent


def _collect_block(lines: list[str], start: int, filename: str | None, label: str) -> tuple[list[tuple[int, str]], int]:
    head = _strip_comment(lines[start])
    if "{" not in head:
        raise HayuloSyntaxError(Diagnostic(code="missing_block", message=f"Expected '{{' for {label} block.", file=filename, line=start + 1))

    after_open = head.split("{", 1)[1]
    depth = _count_braces(head)
    block: list[tuple[int, str]] = []

    if depth <= 0:
        before_close = after_open.rsplit("}", 1)[0].strip()
        if before_close:
            block.append((start + 1, before_close))
        return block, start + 1

    if after_open.strip():
        block.append((start + 1, after_open))

    i = start + 1
    while i < len(lines):
        delta = _count_braces(lines[i])
        if depth + delta <= 0:
            before_close = lines[i].rsplit("}", 1)[0].strip()
            if before_close:
                block.append((i + 1, before_close))
            return block, i + 1
        block.append((i + 1, lines[i]))
        depth += delta
        i += 1

    raise HayuloSyntaxError(Diagnostic(code="unterminated_block", message=f"Unterminated {label} block.", file=filename, line=start + 1, suggestions=["Add a closing '}'."]))


def _parse_fields(block: list[tuple[int, str]], filename: str | None) -> dict[str, Any]:
    values: dict[str, Any] = {}
    i = 0

    while i < len(block):
        line_no, raw = block[i]
        line = _clean_line(raw)
        if not line:
            i += 1
            continue

        match = re.match(r"([A-Za-z_]\w*)\s*:\s*(.+)$", line)
        if not match:
            _error("invalid_intent_field", f"Invalid intent field: {line!r}.", filename, line_no, ["Use: key: \"value\" or key: [\"value\"]."])

        key = match.group(1)
        rhs = match.group(2).strip()

        if rhs.startswith('"'):
            values[key] = _parse_quoted_scalar(rhs, filename, line_no)
            i += 1
            continue

        if rhs.startswith("["):
            parts = [rhs]
            bracket_depth = _count_brackets(rhs)
            i += 1
            while bracket_depth > 0 and i < len(block):
                next_line_no, next_raw = block[i]
                next_line = _clean_line(next_raw)
                parts.append(next_line)
                bracket_depth += _count_brackets(next_line)
                line_no = next_line_no
                i += 1
            if bracket_depth != 0:
                _error("unterminated_intent_list", f"Unterminated intent list for {key!r}.", filename, line_no, ["Add a closing ']'."])
            values[key] = _parse_string_list(" ".join(parts), filename, line_no)
            continue

        _error("invalid_intent_value", f"Invalid intent value for {key!r}.", filename, line_no, ["Use a string or list of strings."])

    return values


def _parse_quoted_scalar(text: str, filename: str | None, line: int) -> str:
    value, index = _consume_quoted(text, 0, filename, line)
    trailing = text[index:].strip()
    if trailing not in {"", ","}:
        _error("invalid_intent_value", "Intent string values cannot contain trailing tokens.", filename, line, ["Use a single quoted string value."])
    return value


def _parse_string_list(text: str, filename: str | None, line: int) -> list[str]:
    text = text.strip()
    if text.endswith("],"):
        text = text[:-1].rstrip()
    if not text.startswith("[") or not text.endswith("]"):
        _error("invalid_intent_value", "Intent list values must start with '[' and end with ']'.", filename, line, ["Use a list of quoted strings."])

    values: list[str] = []
    index = 1
    end = len(text) - 1
    while index < end:
        while index < end and text[index].isspace():
            index += 1
        if index < end and text[index] == ",":
            index += 1
            continue
        if index >= end:
            break
        if text[index] != '"':
            _error("invalid_intent_value", "Intent lists can only contain strings.", filename, line, ["Use quoted strings inside intent lists."])
        value, index = _consume_quoted(text, index, filename, line)
        values.append(value)
        while index < end and text[index].isspace():
            index += 1
        if index < end and text[index] == ",":
            index += 1
            continue
        if index < end:
            _error("invalid_intent_value", "Intent list items must be separated by commas.", filename, line, ["Add a comma between list items."])
    return values


def _consume_quoted(text: str, start: int, filename: str | None, line: int) -> tuple[str, int]:
    if start >= len(text) or text[start] != '"':
        _error("invalid_intent_value", "Expected quoted string.", filename, line, ["Use double quotes for intent strings."])

    chars: list[str] = []
    i = start + 1
    while i < len(text):
        ch = text[i]
        if ch == '"':
            return "".join(chars), i + 1
        if ch == "\\":
            i += 1
            if i >= len(text):
                break
            escaped = text[i]
            chars.append({"n": "\n", "t": "\t", '"': '"', "\\": "\\"}.get(escaped, escaped))
        else:
            chars.append(ch)
        i += 1

    _error("unterminated_string", "Unterminated intent string.", filename, line, ["Add a closing double quote."])


def _clean_line(line: str) -> str:
    return _strip_comment(line).strip()


def _strip_comment(line: str) -> str:
    in_string = False
    escaped = False
    i = 0
    while i < len(line):
        ch = line[i]
        if escaped:
            escaped = False
        elif ch == "\\":
            escaped = True
        elif ch == '"':
            in_string = not in_string
        elif not in_string and line[i : i + 2] == "//":
            return line[:i]
        i += 1
    return line


def _count_braces(line: str) -> int:
    return _count_unquoted(line, "{", "}")


def _count_brackets(line: str) -> int:
    return _count_unquoted(line, "[", "]")


def _count_unquoted(line: str, open_char: str, close_char: str) -> int:
    text = _strip_comment(line)
    in_string = False
    escaped = False
    count = 0
    for ch in text:
        if escaped:
            escaped = False
        elif ch == "\\":
            escaped = True
        elif ch == '"':
            in_string = not in_string
        elif not in_string and ch == open_char:
            count += 1
        elif not in_string and ch == close_char:
            count -= 1
    return count


def _error(code: str, message: str, filename: str | None, line: int, suggestions: list[str]) -> None:
    raise HayuloSyntaxError(Diagnostic(code=code, message=message, file=filename, line=line, suggestions=suggestions))
