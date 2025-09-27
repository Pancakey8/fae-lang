use crate::{
    debug, lexer as lx,
    runtime::{self as rt},
};

#[derive(Debug, Clone)]
pub enum NodeKind {
    Loop {
        body: Vec<Node>,
    },
    If {
        body: Vec<Node>,
        otherwise: Option<Vec<Node>>,
    },
    Function {
        name: String,
        params: Vec<rt::ValueType>,
        body: Vec<Node>,
    },
    With {
        body: Vec<Node>,
    },
    Break,
    Instruction(String),
    String(String),
    Char(char),
    Number(f32),
    Int(i32),
    Bool(bool),
    Stack(rt::Stack),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub pos: lx::Span,
}

pub struct Parser {
    tokens: Vec<lx::Token>,
    pub nodes: Vec<Node>,
    cursor: usize,
}

impl Parser {
    fn is_eof(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    fn bump(&mut self) -> Option<lx::Token> {
        self.tokens.get(self.cursor).map(|tok| {
            self.cursor += 1;
            tok.clone()
        })
    }

    fn peek(&self) -> Option<lx::Token> {
        self.tokens.get(self.cursor).cloned()
    }

    fn matches(&mut self, kind: lx::TokenKind) -> bool {
        self.tokens.get(self.cursor).map_or(false, |tok| {
            if tok.kind == kind {
                self.cursor += 1;
                true
            } else {
                false
            }
        })
    }

    fn push_node(&mut self, kind: NodeKind, len: usize) -> bool {
        self.nodes.push(Node {
            kind,
            pos: lx::Span {
                // ! Make sure to read actual position by peeking into tokens
                // ! Unlike lexer, we consider token positions here
                start: self.tokens[self.cursor - len].pos.start,
                end: self.tokens[self.cursor - 1].pos.end,
            },
        });
        true
    }

    fn try_literal(&mut self) -> bool {
        self.peek().map_or(false, |tok| match tok.kind {
            lx::TokenKind::Number(n) => {
                self.bump();
                self.push_node(NodeKind::Number(n), 1)
            }
            lx::TokenKind::Int(n) => {
                self.bump();
                self.push_node(NodeKind::Int(n), 1)
            }
            lx::TokenKind::String(s) => {
                self.bump();
                self.push_node(NodeKind::String(s), 1)
            }
            lx::TokenKind::Bool(b) => {
                self.bump();
                self.push_node(NodeKind::Bool(b), 1)
            }
            lx::TokenKind::Char(c) => {
                self.bump();
                self.push_node(NodeKind::Char(c), 1)
            }
            _ => false,
        }) || self.try_stack()
    }

    fn try_stack(&mut self) -> bool {
        if !self.matches(lx::TokenKind::LSquare) {
            return false;
        }
        let start = self.cursor;
        let nodes_start = self.nodes.len();
        while !self.is_eof() {
            if self.matches(lx::TokenKind::RSquare) {
                break;
            }

            if !self.try_literal() {
                self.bump();
                return self.push_node(NodeKind::Error("Expected literal or ']'".to_string()), 1);
            }

            if !self.matches(lx::TokenKind::Comma) {
                if self.matches(lx::TokenKind::RSquare) {
                    break;
                } else {
                    return self.push_node(NodeKind::Error("Expected ',' or ']'".to_string()), 1);
                }
            }
        }
        let nodes = self.nodes.split_off(nodes_start);
        let lits = nodes
            .iter()
            .map(|n| match &n.kind {
                NodeKind::String(s) => rt::Value::String(s.clone()),
                NodeKind::Number(n) => rt::Value::Number(*n),
                NodeKind::Int(n) => rt::Value::Int(*n),
                NodeKind::Bool(b) => rt::Value::Bool(*b),
                NodeKind::Stack(s) => rt::Value::Stack(s.clone()),
                _ => todo!("Not a literal"),
            })
            .collect::<Vec<_>>();
        self.push_node(
            NodeKind::Stack(rt::Stack { values: lits }),
            self.cursor - start,
        )
    }

    fn try_break(&mut self) -> bool {
        if self.matches(lx::TokenKind::KwBreak) {
            self.push_node(NodeKind::Break, 1)
        } else {
            false
        }
    }

    fn parse_body(&mut self) -> Option<Vec<Node>> {
        if !self.matches(lx::TokenKind::LCurly) {
            self.push_node(NodeKind::Error("Expected '{'".to_string()), 1);
            return None;
        }
        if self.matches(lx::TokenKind::RCurly) {
            return Some(Vec::new());
        }
        let start_body = self.nodes.len();
        while !self.is_eof() {
            debug!("{}", self.cursor);
            if self.matches(lx::TokenKind::RCurly) {
                return Some(self.nodes.split_off(start_body));
            }
            if !self.try_one() {
                return None;
            }
        }
        self.push_node(NodeKind::Error("Expected closing '}'".to_string()), 1);
        return None;
    }

    fn try_loop(&mut self) -> bool {
        let start = self.cursor;
        if !self.matches(lx::TokenKind::KwLoop) {
            return false;
        }
        match self.parse_body() {
            Some(body) => self.push_node(NodeKind::Loop { body }, self.cursor - start),
            None => true,
        }
    }

    fn try_if(&mut self) -> bool {
        let start = self.cursor;
        if !self.matches(lx::TokenKind::KwIf) {
            return false;
        }
        let Some(body) = self.parse_body() else {
            return true;
        };
        let mut otherwise = None;
        if self.matches(lx::TokenKind::KwElse) {
            otherwise = self.parse_body();
            if otherwise.is_none() {
                return true;
            }
        }
        self.push_node(NodeKind::If { body, otherwise }, self.cursor - start)
    }

    fn try_function(&mut self) -> bool {
        let start = self.cursor;
        let Some(lx::Token {
            kind: lx::TokenKind::Symbol(name),
            ..
        }) = self.peek()
        else {
            return false;
        };
        self.bump();
        if !self.matches(lx::TokenKind::LParen) {
            self.cursor = start;
            return false;
        }
        let mut params = Vec::new();
        while !self.is_eof() {
            if self.matches(lx::TokenKind::RParen) {
                break;
            }

            if let Some(lx::TokenKind::Symbol(tp)) = self.peek().map(|t| t.kind) {
                self.bump();
                let Some(vtype) = rt::ValueType::from_str(tp.as_str()) else {
                    return self.push_node(NodeKind::Error("Expected valid type".to_string()), 1);
                };
                params.push(vtype);
                if !self.matches(lx::TokenKind::Comma) {
                    if self.matches(lx::TokenKind::RParen) {
                        break;
                    } else {
                        return self
                            .push_node(NodeKind::Error("Expected ',' or ')'".to_string()), 1);
                    }
                }
            } else {
                return self.push_node(NodeKind::Error("Expected type or ')'".to_string()), 1);
            }
        }
        match self.parse_body() {
            Some(body) => self.push_node(
                NodeKind::Function { name, params, body },
                self.cursor - start,
            ),
            None => true,
        }
    }

    fn try_with(&mut self) -> bool {
        let start = self.cursor;
        if !self.matches(lx::TokenKind::KwWith) {
            return false;
        }

        match self.parse_body() {
            Some(body) => self.push_node(NodeKind::With { body }, self.cursor - start),
            None => true,
        }
    }

    fn try_instruction(&mut self) -> bool {
        match self.peek() {
            Some(lx::Token {
                kind: lx::TokenKind::Symbol(inst),
                ..
            }) => {
                self.bump();
                self.push_node(NodeKind::Instruction(inst), 1);
                true
            }
            _ => false,
        }
    }

    fn try_one(&mut self) -> bool {
        self.try_literal()
            || self.try_break()
            || self.try_loop()
            || self.try_if()
            || self.try_function()
            || self.try_with()
            || self.try_instruction()
            || false
    }

    pub fn new(tokens: Vec<lx::Token>) -> Parser {
        Parser {
            tokens,
            nodes: Vec::new(),
            cursor: 0,
        }
    }

    pub fn try_all(&mut self) {
        while !self.is_eof() {
            if !self.try_one() {
                self.bump();
                self.push_node(NodeKind::Error("Unexpected token".to_string()), 1);
            }
        }
    }
}
