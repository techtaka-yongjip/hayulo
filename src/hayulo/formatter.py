from __future__ import annotations

from dataclasses import dataclass


INDENT = "  "


@dataclass
class FormatResult:
    source: str
    changed: bool


def format_source(source: str) -> str:
    lines: list[str] = []
    indent = 0
    previous_blank = False

    for raw in source.splitlines():
        stripped = raw.strip()
        if not stripped:
            if lines and not previous_blank:
                lines.append("")
            previous_blank = True
            continue

        opens, closes = _delimiter_counts(stripped)
        leading_closes = _leading_closes(stripped)
        if leading_closes:
            indent = max(0, indent - leading_closes)

        lines.append(f"{INDENT * indent}{stripped}")
        indent = max(0, indent + opens - max(0, closes - leading_closes))
        previous_blank = False

    while lines and lines[-1] == "":
        lines.pop()

    return "\n".join(lines) + "\n"


def check_format(source: str) -> FormatResult:
    formatted = format_source(source)
    return FormatResult(source=formatted, changed=formatted != source)


def _leading_closes(line: str) -> int:
    count = 0
    for ch in line:
        if ch in {"}", "]"}:
            count += 1
            continue
        if ch.isspace():
            continue
        break
    return count


def _delimiter_counts(line: str) -> tuple[int, int]:
    text = _strip_comment(line)
    in_string = False
    escaped = False
    opens = 0
    closes = 0
    for ch in text:
        if escaped:
            escaped = False
        elif ch == "\\":
            escaped = True
        elif ch == '"':
            in_string = not in_string
        elif not in_string and ch in {"{", "["}:
            opens += 1
        elif not in_string and ch in {"}", "]"}:
            closes += 1
    return opens, closes


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
