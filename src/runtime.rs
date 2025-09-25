use std::{collections::HashMap, process::exit};

use crate::{
    ast::{self},
    debug,
    error::{self, FErrorManager},
    faestd,
    lexer::{self as lx},
};

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Int(i32),
    Number(f32),
    Bool(bool),
}

impl Value {
    pub fn type_name(&self) -> &str {
        match self {
            Value::String(_) => "string",
            Value::Int(_) => "int",
            Value::Number(_) => "number",
            Value::Bool(_) => "bool",
        }
    }

    pub fn as_integral(&self) -> Option<f32> {
        match self {
            Value::String(_) => None,
            Value::Int(n) => Some(*n as f32),
            Value::Number(n) => Some(*n),
            Value::Bool(b) => {
                if *b {
                    Some(1.0)
                } else {
                    Some(0.0)
                }
            }
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            Value::String(s) => Some(s.clone()),
            Value::Int(n) => Some(n.to_string()),
            Value::Number(n) => Some(n.to_string()),
            Value::Bool(b) => Some(if *b { "true" } else { "false" }.to_string()),
        }
    }

    pub fn as_int(&self) -> Option<i32> {
        match self {
            Value::String(_) => None,
            Value::Int(n) => Some(*n),
            Value::Number(n) => Some(*n as i32),
            Value::Bool(b) => Some(if *b { 1 } else { 0 }),
        }
    }
}

pub struct Stack {
    pub values: Vec<Value>,
}

impl Stack {
    pub fn new() -> Stack {
        Stack { values: Vec::new() }
    }

    pub fn push(&mut self, val: Value) {
        self.values.push(val);
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.values.pop()
    }

    pub fn peek(&mut self) -> Option<Value> {
        self.values.last().cloned()
    }

    pub fn depth(&mut self) -> usize {
        self.values.len()
    }

    pub fn top_n(&mut self, n: usize) -> Option<Vec<Value>> {
        if n == 0 {
            return Some(Vec::new());
        }
        if self.values.len() < n {
            return None;
        }

        let start = self.values.len() - n;
        let drained: Vec<Value> = self.values.drain(start..).collect();
        Some(drained)
    }
    pub fn is_matching_params(&self, params: &[&str]) -> bool {
        if params.len() > self.values.len() {
            return false;
        }

        for (i, param) in params.iter().rev().enumerate() {
            if *param != self.values[self.values.len() - 1 - i].type_name() {
                return false;
            }
        }

        true
    }
}

pub type InternalFn = fn(&mut Scope, lx::Span);

#[derive(Clone)]
pub enum Function {
    Internal(InternalFn),
    Runtime {
        params: Vec<String>,
        body: Vec<ast::Node>,
    },
}

pub struct Scope {
    pub stack: Stack,
    pub funcs: HashMap<String, Function>,
    pub errs: error::FErrorManager,
}

pub enum ControlState {
    Normal,
    Break,
}

impl Scope {
    pub fn new(errs: error::FErrorManager) -> Scope {
        Scope {
            stack: Stack::new(),
            funcs: faestd::std_get_fns(),
            errs,
        }
    }

    pub fn ensure_params(&self, params: &[&str], at: lx::Span) {
        let stack_len = self.stack.values.len();

        if stack_len < params.len() || !self.stack.is_matching_params(params) {
            let got: String = self
                .stack
                .values
                .iter()
                .rev()
                .take(params.len())
                .map(|v| v.type_name())
                .rev()
                .collect::<Vec<_>>()
                .join(",");
            let wanted = params.join(",");
            self.errs.print_error(
                &at,
                format!("Expected parameters ({wanted}), found ({got})"),
            );
            debug!("STACK:");
            for v in &self.stack.values {
                debug!("- {:?}", v);
            }
            exit(1);
        }
    }

    pub fn run_block(&mut self, block: &Vec<ast::Node>) -> ControlState {
        for node in block {
            match &node.kind {
                ast::NodeKind::Loop { body } => loop {
                    if let ControlState::Break = self.run_block(body) {
                        debug!("BREAK!");
                        break;
                    }
                },
                ast::NodeKind::If { body } => match self.stack.pop() {
                    Some(Value::Bool(true)) => {
                        if let ControlState::Break = self.run_block(body) {
                            return ControlState::Break;
                        }
                    }
                    Some(Value::Bool(false)) => {}
                    _ => {
                        self.errs
                            .print_error(&node.pos, format!("If expects parameters: (bool)"));
                        exit(1);
                    }
                },
                ast::NodeKind::Function { name, params, body } => {
                    if self.funcs.contains_key(name) {
                        self.errs.print_error(
                            &node.pos,
                            format!("Cannot redefine existing function '{name}'"),
                        );
                        exit(1);
                    }
                    self.funcs.insert(
                        name.clone(),
                        Function::Runtime {
                            params: params.to_vec(),
                            body: body.clone(),
                        },
                    );
                }
                ast::NodeKind::Break => return ControlState::Break,
                ast::NodeKind::Instruction(inst) => {
                    let func = self.funcs.get(inst).cloned();
                    match func {
                        Some(Function::Runtime { params, body }) => {
                            let params: Vec<&str> = params.iter().map(|s| s.as_str()).collect();
                            self.ensure_params(&params, node.pos.clone());
                            let errs = std::mem::replace(
                                &mut self.errs,
                                FErrorManager::new(String::new()),
                            );
                            let mut call_scope = Scope::new(errs);
                            call_scope.funcs = self.funcs.clone();
                            call_scope.stack.values = self.stack.top_n(params.len()).unwrap();
                            call_scope.run_block(&body);
                            self.errs = call_scope.errs;
                            self.stack.values.append(&mut call_scope.stack.values);
                        }
                        Some(Function::Internal(f)) => {
                            f(self, node.pos.clone());
                        }
                        None => {
                            self.errs
                                .print_error(&node.pos, format!("Function not found"));
                            exit(1);
                        }
                    }
                }
                ast::NodeKind::String(v) => self.stack.push(Value::String(v.clone())),
                ast::NodeKind::Number(v) => self.stack.push(Value::Number(*v)),
                ast::NodeKind::Int(v) => self.stack.push(Value::Int(*v)),
                ast::NodeKind::Bool(v) => self.stack.push(Value::Bool(*v)),
                ast::NodeKind::Error(_) => unreachable!(),
            }
        }
        ControlState::Normal
    }
}
