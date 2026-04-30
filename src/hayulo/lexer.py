from __future__ import annotations

from dataclasses import dataclass

from .diagnostics import Diagnostic, HayuloSyntaxError


KEYWORDS = {
    "module",
    "intent",
    "fn",
    "pub",
    "return",
    "if",
    "else",
    "for",
    "in",
    "true",
    "false",
    "and",
    "or",
    "not",
    "test",
    "expect",
}

SINGLE_CHAR_TOKENS = {
    "(": "LPAREN",
    ")": "RPAREN",
    "{": "LBRACE",
    "}": "RBRACE",
    "[": "LBRACKET",
    "]": "RBRACKET",
    ",": "COMMA",
    ":": "COLON",
    ".": "DOT",
    "+": "PLUS",
    "-": "MINUS",
    "*": "STAR",
    "/": "SLASH",
    "%": "PERCENT",
    "=": "EQUAL",
    "<": "LT",
    ">": "GT",
}


@dataclass(frozen=True)
class Token:
    kind: str
    value: str
    line: int
    column: int


class Lexer:
    def __init__(self, source: str, filename: str | None = None):
        self.source = source
        self.filename = filename
        self.index = 0
        self.line = 1
        self.column = 1
        self.tokens: list[Token] = []

    def lex(self) -> list[Token]:
        while not self._is_at_end():
            ch = self._peek()

            if ch in " \t\r":
                self._advance()
                continue

            if ch == "\n":
                self._advance_line()
                continue

            if ch == "/" and self._peek_next() == "/":
                self._skip_comment()
                continue

            if ch == '"':
                self._string()
                continue

            if ch.isdigit():
                self._number()
                continue

            if ch.isalpha() or ch == "_":
                self._identifier()
                continue

            self._symbol()

        self.tokens.append(Token("EOF", "", self.line, self.column))
        return self.tokens

    def _symbol(self) -> None:
        line, col = self.line, self.column
        ch = self._advance()

        if ch == "-" and self._match(">"):
            self.tokens.append(Token("ARROW", "->", line, col))
            return

        if ch == "=" and self._match("="):
            self.tokens.append(Token("EQEQ", "==", line, col))
            return

        if ch == "!" and self._match("="):
            self.tokens.append(Token("BANGEQ", "!=", line, col))
            return

        if ch == "<" and self._match("="):
            self.tokens.append(Token("LTE", "<=", line, col))
            return

        if ch == ">" and self._match("="):
            self.tokens.append(Token("GTE", ">=", line, col))
            return

        kind = SINGLE_CHAR_TOKENS.get(ch)
        if kind is None:
            raise HayuloSyntaxError(
                Diagnostic(
                    code="unexpected_character",
                    message=f"Unexpected character {ch!r}.",
                    file=self.filename,
                    line=line,
                    column=col,
                    suggestions=["Remove this character or replace it with valid Hayulo syntax."],
                )
            )
        self.tokens.append(Token(kind, ch, line, col))

    def _string(self) -> None:
        start_line, start_col = self.line, self.column
        self._advance()  # opening quote
        chars: list[str] = []

        while not self._is_at_end() and self._peek() != '"':
            ch = self._advance()
            if ch == "\\":
                if self._is_at_end():
                    break
                escaped = self._advance()
                mapping = {"n": "\n", "t": "\t", '"': '"', "\\": "\\"}
                chars.append(mapping.get(escaped, escaped))
            elif ch == "\n":
                self.line += 1
                self.column = 1
                chars.append("\n")
            else:
                chars.append(ch)

        if self._is_at_end():
            raise HayuloSyntaxError(
                Diagnostic(
                    code="unterminated_string",
                    message="Unterminated string literal.",
                    file=self.filename,
                    line=start_line,
                    column=start_col,
                    suggestions=["Add a closing double quote."],
                )
            )

        self._advance()  # closing quote
        self.tokens.append(Token("STRING", "".join(chars), start_line, start_col))

    def _number(self) -> None:
        start_line, start_col = self.line, self.column
        chars = [self._advance()]
        is_float = False

        while not self._is_at_end() and self._peek().isdigit():
            chars.append(self._advance())

        if not self._is_at_end() and self._peek() == "." and self._peek_next().isdigit():
            is_float = True
            chars.append(self._advance())
            while not self._is_at_end() and self._peek().isdigit():
                chars.append(self._advance())

        kind = "FLOAT" if is_float else "INT"
        self.tokens.append(Token(kind, "".join(chars), start_line, start_col))

    def _identifier(self) -> None:
        start_line, start_col = self.line, self.column
        chars = [self._advance()]
        while not self._is_at_end() and (self._peek().isalnum() or self._peek() in "_"):
            chars.append(self._advance())
        value = "".join(chars)
        kind = value.upper() if value in KEYWORDS else "IDENT"
        self.tokens.append(Token(kind, value, start_line, start_col))

    def _skip_comment(self) -> None:
        while not self._is_at_end() and self._peek() != "\n":
            self._advance()

    def _advance(self) -> str:
        ch = self.source[self.index]
        self.index += 1
        self.column += 1
        return ch

    def _advance_line(self) -> None:
        self.index += 1
        self.line += 1
        self.column = 1

    def _match(self, expected: str) -> bool:
        if self._is_at_end() or self.source[self.index] != expected:
            return False
        self.index += 1
        self.column += 1
        return True

    def _peek(self) -> str:
        if self._is_at_end():
            return "\0"
        return self.source[self.index]

    def _peek_next(self) -> str:
        if self.index + 1 >= len(self.source):
            return "\0"
        return self.source[self.index + 1]

    def _is_at_end(self) -> bool:
        return self.index >= len(self.source)
