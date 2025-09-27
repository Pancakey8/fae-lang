use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    String(String),
    Char(char),
    Number(f32),
    Int(i32),
    LParen,
    RParen,
    LCurly,
    RCurly,
    LSquare,
    RSquare,
    Comma,
    KwIf,
    KwElse,
    KwLoop,
    KwBreak,
    KwWith,
    Bool(bool),
    Symbol(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: Span,
}

fn is_symbolic(ch: char) -> bool {
    (ch.is_alphanumeric() || ch.is_ascii_punctuation()) && !"\"(){}[],".contains(ch)
}

fn kw_match(s: &str) -> Option<TokenKind> {
    match s {
        "if" => Some(TokenKind::KwIf),
        "else" => Some(TokenKind::KwElse),
        "loop" => Some(TokenKind::KwLoop),
        "break" => Some(TokenKind::KwBreak),
        "with" => Some(TokenKind::KwWith),
        "true" => Some(TokenKind::Bool(true)),
        "false" => Some(TokenKind::Bool(false)),
        _ => None,
    }
}

fn escape_map() -> HashMap<&'static str, char> {
    HashMap::from([("\\n", '\n'), ("\\\\", '\\'), ("\\t", '\t'), ("\\\"", '\"')])
}

pub struct Lexer {
    input: String,
    pub tokens: Vec<Token>,
    cursor: usize,
    escapes: HashMap<&'static str, char>,
}

impl Lexer {
    fn is_eof(&self) -> bool {
        self.cursor >= self.input.len()
    }

    fn bump(&mut self) -> Option<char> {
        let mut iter = self.input[self.cursor..].chars();
        let ch = iter.next()?;
        self.cursor += ch.len_utf8();
        Some(ch)
    }

    fn peek(&self) -> Option<char> {
        let mut iter = self.input[self.cursor..].chars();
        iter.next()
    }

    fn matches(&mut self, pat: &str) -> bool {
        let is_m = self.input[self.cursor..].starts_with(pat);
        if is_m {
            self.cursor += pat.len();
        }
        is_m
    }

    fn push_token(&mut self, kind: TokenKind, len: usize) -> bool {
        self.tokens.push(Token {
            kind,
            // ! Unlike parser, we consider cursor positions here, i.e. two positions in between characters to mark a range.
            pos: Span {
                start: self.cursor - len,
                end: self.cursor,
            },
        });
        true
    }

    fn try_whitespace(&mut self) -> bool {
        if self.peek().is_some_and(|c| c.is_whitespace()) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn try_parens(&mut self) -> bool {
        if self.matches("(") {
            self.push_token(TokenKind::LParen, 1)
        } else if self.matches(")") {
            self.push_token(TokenKind::RParen, 1)
        } else if self.matches("{") {
            self.push_token(TokenKind::LCurly, 1)
        } else if self.matches("}") {
            self.push_token(TokenKind::RCurly, 1)
        } else if self.matches("[") {
            self.push_token(TokenKind::LSquare, 1)
        } else if self.matches("]") {
            self.push_token(TokenKind::RSquare, 1)
        } else if self.matches(",") {
            self.push_token(TokenKind::Comma, 1)
        } else {
            false
        }
    }

    fn try_number(&mut self) -> bool {
        let is_pos = self.peek().is_some_and(|c| c != '-');
        let start = self.cursor;
        if !is_pos {
            self.bump();
        }
        while self.peek().is_some_and(|c| c.is_digit(10)) {
            self.bump();
        }
        let len = self.cursor - start;
        if len == 0 || (len == 1 && !is_pos) {
            self.cursor = start;
            return false;
        }
        if self.peek().is_some_and(|c| c == '.') {
            self.bump();
            let start_fp = self.cursor;
            while self.peek().is_some_and(|c| c.is_digit(10)) {
                self.bump();
            }

            if start_fp == self.cursor {
                self.push_token(TokenKind::Error("Dangling floating point".to_string()), 1);
                return true; // Confusing? True just means we avoid fallback here
            }
            self.push_token(
                TokenKind::Number(self.input[start..self.cursor].parse::<f32>().unwrap()),
                self.cursor - start,
            )
        } else {
            self.push_token(
                TokenKind::Int(self.input[start..self.cursor].parse::<i32>().unwrap()),
                self.cursor - start,
            )
        }
    }

    fn try_string(&mut self) -> bool {
        let start = self.cursor;

        if !self.matches("\"") {
            return false;
        }

        let mut is_closed = false;
        let mut contents = String::new();
        while !self.is_eof() {
            if self.matches("\"") {
                is_closed = true;
                break;
            } else {
                let mut any_match = false;
                let keys = self.escapes.keys().copied().collect::<Vec<_>>();
                for k in keys {
                    if self.matches(k) {
                        contents.push(self.escapes[k]);
                        any_match = true;
                        break;
                    }
                }
                if !any_match {
                    contents.push(self.bump().unwrap());
                }
            }
        }

        if !is_closed {
            self.push_token(
                TokenKind::Error("Unclosed string".to_string()),
                self.cursor - start,
            )
        } else {
            self.push_token(TokenKind::String(contents), self.cursor - start)
        }
    }

    fn try_char(&mut self) -> bool {
        let start = self.cursor;
        if !self.matches("'") {
            return false;
        }

        if self.is_eof() {
            return self.push_token(
                TokenKind::Error("Unclosed character quote \"'\"".to_string()),
                1,
            );
        }

        let ch = {
            let keys = self.escapes.keys().copied().collect::<Vec<_>>();
            let mut found: Option<char> = None;
            for k in &keys {
                if self.matches(*k) {
                    found = Some(self.escapes[k]);
                    break;
                }
            }
            match found {
                Some(c) => c,
                None => self.bump().unwrap(),
            }
        };

        if !self.matches("'") {
            return self.push_token(
                TokenKind::Error("Unclosed character quote \"'\"".to_string()),
                self.cursor - start,
            );
        }

        self.push_token(TokenKind::Char(ch), self.cursor - start)
    }

    fn try_symbol(&mut self) -> bool {
        let mut sym = String::new();
        while !self.is_eof() {
            if self.peek().is_some_and(is_symbolic) {
                sym.push(self.bump().unwrap());
            } else {
                break;
            }
        }
        let len = sym.len();
        if len > 0 {
            match kw_match(&sym) {
                Some(tok) => self.push_token(tok, len),
                None => self.push_token(TokenKind::Symbol(sym), len),
            }
        } else {
            false
        }
    }

    fn try_comment(&mut self) -> bool {
        if self.matches("//") {
            while !self.is_eof() && !self.matches("\n") {
                self.bump();
            }
            true
        } else if self.matches("/*") {
            while !self.is_eof() && !self.matches("*/") {
                self.bump();
            }
            true
        } else {
            false
        }
    }

    fn try_one(&mut self) -> bool {
        self.try_whitespace()
            || self.try_comment()
            || self.try_parens()
            || self.try_number()
            || self.try_string()
            || self.try_char()
            || self.try_symbol()
            || false
    }

    pub fn new(input: String) -> Lexer {
        Lexer {
            input,
            tokens: Vec::new(),
            cursor: 0,
            escapes: escape_map(),
        }
    }

    pub fn try_all(&mut self) {
        while !self.is_eof() {
            if !self.try_one() {
                self.bump();
                self.push_token(TokenKind::Error("Unknown token".to_string()), 1);
            }
        }
    }
}
