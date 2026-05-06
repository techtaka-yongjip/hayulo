use crate::diagnostics::{Diagnostic, HayuloError, HayuloResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: String,
    pub value: String,
    pub line: usize,
    pub column: usize,
}

pub fn lex(source: &str, filename: Option<&str>) -> HayuloResult<Vec<Token>> {
    Lexer::new(source, filename).lex()
}

struct Lexer<'a> {
    chars: Vec<char>,
    index: usize,
    line: usize,
    column: usize,
    filename: Option<&'a str>,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    fn new(source: &str, filename: Option<&'a str>) -> Self {
        Self {
            chars: source.chars().collect(),
            index: 0,
            line: 1,
            column: 1,
            filename,
            tokens: Vec::new(),
        }
    }

    fn lex(mut self) -> HayuloResult<Vec<Token>> {
        while !self.is_at_end() {
            let ch = self.peek();
            if matches!(ch, ' ' | '\t' | '\r') {
                self.advance();
                continue;
            }
            if ch == '\n' {
                self.advance_line();
                continue;
            }
            if ch == '/' && self.peek_next() == '/' {
                self.skip_comment();
                continue;
            }
            if ch == '"' {
                self.string()?;
                continue;
            }
            if ch == '?' {
                return Err(HayuloError::new(
                    Diagnostic::new(
                        "syntax.postfix_try_removed",
                        "Hayulo 2.0 uses prefix try instead of postfix ?.",
                    )
                    .file(self.filename)
                    .line(Some(self.line))
                    .column(Some(self.column))
                    .suggestion("Use `try expression` instead of `expression?`."),
                ));
            }
            if ch.is_ascii_digit() {
                self.number();
                continue;
            }
            if ch.is_ascii_alphabetic() || ch == '_' {
                self.identifier();
                continue;
            }
            self.symbol()?;
        }
        self.tokens.push(Token {
            kind: "EOF".to_string(),
            value: String::new(),
            line: self.line,
            column: self.column,
        });
        Ok(self.tokens)
    }

    fn symbol(&mut self) -> HayuloResult<()> {
        let line = self.line;
        let column = self.column;
        let ch = self.advance();

        if ch == '-' && self.matches('>') {
            self.push("ARROW", "->", line, column);
            return Ok(());
        }
        if ch == '=' && self.matches('=') {
            self.push("EQEQ", "==", line, column);
            return Ok(());
        }
        if ch == '=' && self.matches('>') {
            self.push("FAT_ARROW", "=>", line, column);
            return Ok(());
        }
        if ch == '!' && self.matches('=') {
            self.push("BANGEQ", "!=", line, column);
            return Ok(());
        }
        if ch == '<' && self.matches('=') {
            self.push("LTE", "<=", line, column);
            return Ok(());
        }
        if ch == '>' && self.matches('=') {
            self.push("GTE", ">=", line, column);
            return Ok(());
        }

        let kind = match ch {
            '(' => "LPAREN",
            ')' => "RPAREN",
            '{' => "LBRACE",
            '}' => "RBRACE",
            '[' => "LBRACKET",
            ']' => "RBRACKET",
            ',' => "COMMA",
            ':' => "COLON",
            '.' => "DOT",
            '+' => "PLUS",
            '-' => "MINUS",
            '*' => "STAR",
            '/' => "SLASH",
            '%' => "PERCENT",
            '=' => "EQUAL",
            '<' => "LT",
            '>' => "GT",
            _ => {
                return Err(HayuloError::new(
                    Diagnostic::new(
                        "unexpected_character",
                        format!("Unexpected character {ch:?}."),
                    )
                    .file(self.filename)
                    .line(Some(line))
                    .column(Some(column))
                    .suggestion("Remove this character or replace it with valid Hayulo syntax."),
                ));
            }
        };
        self.push(kind, ch.to_string(), line, column);
        Ok(())
    }

    fn string(&mut self) -> HayuloResult<()> {
        let start_line = self.line;
        let start_col = self.column;
        self.advance();
        let mut value = String::new();
        while !self.is_at_end() && self.peek() != '"' {
            let ch = self.advance();
            if ch == '\\' {
                if self.is_at_end() {
                    break;
                }
                let escaped = self.advance();
                value.push(match escaped {
                    'n' => '\n',
                    't' => '\t',
                    '"' => '"',
                    '\\' => '\\',
                    other => other,
                });
            } else if ch == '\n' {
                self.line += 1;
                self.column = 1;
                value.push('\n');
            } else {
                value.push(ch);
            }
        }
        if self.is_at_end() {
            return Err(HayuloError::new(
                Diagnostic::new("unterminated_string", "Unterminated string literal.")
                    .file(self.filename)
                    .line(Some(start_line))
                    .column(Some(start_col))
                    .suggestion("Add a closing double quote."),
            ));
        }
        self.advance();
        self.push("STRING", value, start_line, start_col);
        Ok(())
    }

    fn number(&mut self) {
        let start_line = self.line;
        let start_col = self.column;
        let mut value = String::new();
        value.push(self.advance());
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            value.push(self.advance());
        }
        let mut kind = "INT";
        if !self.is_at_end() && self.peek() == '.' && self.peek_next().is_ascii_digit() {
            kind = "FLOAT";
            value.push(self.advance());
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                value.push(self.advance());
            }
        }
        self.push(kind, value, start_line, start_col);
    }

    fn identifier(&mut self) {
        let start_line = self.line;
        let start_col = self.column;
        let mut value = String::new();
        value.push(self.advance());
        while !self.is_at_end() && (self.peek().is_ascii_alphanumeric() || self.peek() == '_') {
            value.push(self.advance());
        }
        let kind = match value.as_str() {
            "module" | "intent" | "fn" | "pub" | "let" | "set" | "return" | "try" | "match"
            | "if" | "else" | "for" | "in" | "true" | "false" | "and" | "or" | "not" | "test"
            | "expect" => value.to_ascii_uppercase(),
            _ => "IDENT".to_string(),
        };
        self.push(kind, value, start_line, start_col);
    }

    fn skip_comment(&mut self) {
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    fn push(
        &mut self,
        kind: impl Into<String>,
        value: impl Into<String>,
        line: usize,
        column: usize,
    ) {
        self.tokens.push(Token {
            kind: kind.into(),
            value: value.into(),
            line,
            column,
        });
    }

    fn advance(&mut self) -> char {
        let ch = self.chars[self.index];
        self.index += 1;
        self.column += 1;
        ch
    }

    fn advance_line(&mut self) {
        self.index += 1;
        self.line += 1;
        self.column = 1;
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.chars[self.index] != expected {
            return false;
        }
        self.index += 1;
        self.column += 1;
        true
    }

    fn peek(&self) -> char {
        self.chars.get(self.index).copied().unwrap_or('\0')
    }

    fn peek_next(&self) -> char {
        self.chars.get(self.index + 1).copied().unwrap_or('\0')
    }

    fn is_at_end(&self) -> bool {
        self.index >= self.chars.len()
    }
}
